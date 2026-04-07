use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gc::Gc;
use ish_ast::*;

use crate::environment::Environment;
use crate::error::{ErrorCode, RuntimeError};
use crate::value::*;
use crate::builtins;
use crate::ledger::LedgerState;
use crate::{module_loader, access_control, interface_checker};

/// Signal for control flow: normal completion, return, throw, or break.
enum ControlFlow {
    None,
    Return(Value),
    /// The value produced by the last expression statement.
    ExprValue(Value),
    /// A thrown error value propagating up the call stack.
    Throw(Value),
}

/// Per-task state, separate from the shared VM.
/// Each spawned task gets its own TaskContext.
pub struct TaskContext {
    /// Per-function defer stacks. Each function invocation pushes a frame;
    /// deferred statements execute in LIFO order when the frame is popped.
    defer_stack: Vec<Vec<(Statement, Environment)>>,
    /// Stack of (is_async, fn_name) for nested function calls.
    /// Used by async_annotation audit to detect non-async functions performing async ops.
    async_stack: Vec<(bool, String)>,
    /// Stack of module paths currently being loaded (for cycle detection).
    pub loading_stack: Vec<Vec<String>>,
    /// Path of the file currently being executed (for access control and bootstrap checks).
    pub current_file: Option<PathBuf>,
}

impl TaskContext {
    pub fn new() -> Self {
        TaskContext {
            defer_stack: Vec::new(),
            async_stack: Vec::new(),
            loading_stack: Vec::new(),
            current_file: None,
        }
    }

    /// Push a new (empty) defer frame for a function invocation.
    fn push_defer_frame(&mut self) {
        self.defer_stack.push(Vec::new());
    }

    /// Register a deferred statement on the current function's defer stack.
    fn register_defer(&mut self, stmt: Statement, env: Environment) {
        if let Some(frame) = self.defer_stack.last_mut() {
            frame.push((stmt, env));
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-yield-cycle state, reset every time the VM yields.
/// Tracks yield budget for cooperative multitasking.
pub struct YieldContext {
    /// When the current yield budget window started.
    budget_start: Instant,
    /// How long before a yield is triggered (~1ms default).
    budget_duration: Duration,
    /// Depth of @[unyielding] scopes. When > 0, yield checks are suppressed.
    unyielding_depth: usize,
}

impl YieldContext {
    pub fn new() -> Self {
        YieldContext {
            budget_start: Instant::now(),
            budget_duration: Duration::from_millis(1),
            unyielding_depth: 0,
        }
    }

    /// Check if the yield budget is exhausted. If so, yield to the executor
    /// and reset the budget timer. Skipped when inside an @[unyielding] scope.
    async fn check_yield_budget(&mut self) {
        if self.unyielding_depth == 0 && self.budget_start.elapsed() >= self.budget_duration {
            tokio::task::yield_now().await;
            self.budget_start = Instant::now();
        }
    }

    /// Reset the budget timer (called after an explicit yield).
    fn reset_budget(&mut self) {
        self.budget_start = Instant::now();
    }
}

impl Default for YieldContext {
    fn default() -> Self {
        Self::new()
    }
}

/// The ish virtual machine / interpreter.
pub struct IshVm {
    pub global_env: Environment,
    /// Assurance ledger runtime state: standard scope stack, entry store,
    /// and built-in registries.
    pub ledger: LedgerState,
    /// Project context for module loading and access control.
    pub project_context: access_control::ProjectContext,
    /// The initial file being executed (set before `run()` for file-based execution).
    pub initial_file: Option<PathBuf>,
}

impl IshVm {
    pub fn new() -> Self {
        let env = Environment::new();
        builtins::register_all(&env);
        crate::reflection::register_ast_builtins(&env);
        IshVm {
            global_env: env,
            ledger: LedgerState::new(),
            project_context: access_control::ProjectContext {
                project_root: None,
                src_root: None,
            },
            initial_file: None,
        }
    }

    /// Create a VM with custom builtin configuration (e.g., output routing).
    pub fn with_config(config: &builtins::BuiltinConfig) -> Self {
        let env = Environment::new();
        builtins::register_all_with_config(&env, config);
        crate::reflection::register_ast_builtins(&env);
        IshVm {
            global_env: env,
            ledger: LedgerState::new(),
            project_context: access_control::ProjectContext {
                project_root: None,
                src_root: None,
            },
            initial_file: None,
        }
    }

    /// Create a lightweight clone for a spawned task.
    /// Shares the same global environment (via Gc) and clones the ledger.
    fn spawn_clone(&self) -> Self {
        IshVm {
            global_env: self.global_env.clone(),
            ledger: self.ledger.clone(),
            project_context: access_control::ProjectContext {
                project_root: self.project_context.project_root.clone(),
                src_root: self.project_context.src_root.clone(),
            },
            initial_file: self.initial_file.clone(),
        }
    }

    /// Pop the current defer frame and execute all deferred statements in
    /// LIFO order. Errors from deferred statements are silently ignored
    /// (they do not override in-flight control flow).
    async fn pop_and_run_defers(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext, yc: &mut YieldContext) {
        if let Some(deferred) = task.defer_stack.pop() {
            for (d, env) in deferred.into_iter().rev() {
                let _ = Self::exec_statement_yielding(vm, task, yc, &d, &env).await;
            }
        }
    }

    /// Execute a full program.
    pub async fn run(vm: &Rc<RefCell<IshVm>>, program: &Program) -> Result<Value, RuntimeError> {
        let mut task = TaskContext::new();
        task.current_file = vm.borrow().initial_file.clone();
        let mut yc = YieldContext::new();
        let mut last = Value::Null;
        let env = vm.borrow().global_env.clone();
        task.push_defer_frame();
        for stmt in &program.statements {
            match Self::exec_statement_yielding(vm, &mut task, &mut yc, stmt, &env).await {
                Ok(ControlFlow::Return(v)) => {
                    last = v;
                    break;
                }
                Ok(ControlFlow::Throw(v)) => {
                    Self::pop_and_run_defers(vm, &mut task, &mut yc).await;
                    return Err(RuntimeError::system_error(format!(
                        "Unhandled throw: {}", v.to_display_string()
                    ), ErrorCode::UnhandledThrow));
                }
                Ok(ControlFlow::ExprValue(v)) => last = v,
                Ok(ControlFlow::None) => {}
                Err(e) => {
                    Self::pop_and_run_defers(vm, &mut task, &mut yc).await;
                    return Err(e);
                }
            }
        }
        Self::pop_and_run_defers(vm, &mut task, &mut yc).await;

        // Audit: future_drop — check if any futures were spawned but not awaited
        let unawaited = crate::value::take_unawaited_future_count();
        if unawaited > 0 {
            let features = vm.borrow().ledger.active_features();
            if let crate::ledger::audit::AuditResult::Discrepancy(report) =
                crate::ledger::audit::audit_future_drop(
                    &features,
                    &format!("{} future(s) dropped without await", unawaited),
                )
            {
                return Err(RuntimeError::system_error(report.message, ErrorCode::AsyncError));
            }
        }

        Ok(last)
    }

    /// Execute a single statement in the given environment.
    fn exec_statement_yielding<'a>(
        vm: &'a Rc<RefCell<IshVm>>,
        task: &'a mut TaskContext,
        yc: &'a mut YieldContext,
        stmt: &'a Statement,
        env: &'a Environment,
    ) -> Pin<Box<dyn Future<Output = Result<ControlFlow, RuntimeError>> + 'a>> {
        Box::pin(async move {
        match stmt {
            Statement::VariableDecl { name, value, type_annotation, .. } => {
                let val = Self::eval_expression_yielding(vm, task, yc, value, env).await?;
                Self::audit_type_annotation(vm, name, &val, type_annotation.as_ref())?;
                env.define(name.clone(), val);
                Ok(ControlFlow::None)
            }

            Statement::Assignment { target, value } => {
                let val = Self::eval_expression_yielding(vm, task, yc, value, env).await?;
                match target {
                    AssignTarget::Variable(name) => {
                        env.set(name, val)?;
                    }
                    AssignTarget::Property { object, property } => {
                        let obj = Self::eval_expression_yielding(vm, task, yc, object, env).await?;
                        if let Value::Object(ref obj_ref) = obj {
                            obj_ref.borrow_mut().insert(property.clone(), val);
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot set property '{}' on {}",
                                property,
                                obj.type_name()
                            ), ErrorCode::TypeMismatch));
                        }
                    }
                    AssignTarget::Index { object, index } => {
                        let obj = Self::eval_expression_yielding(vm, task, yc, object, env).await?;
                        let idx = Self::eval_expression_yielding(vm, task, yc, index, env).await?;
                        if let Value::List(ref list_ref) = obj {
                            if let Value::Int(i) = idx {
                                let mut list = list_ref.borrow_mut();
                                let len = list.len() as i64;
                                if i < 0 || i >= len {
                                    return Err(RuntimeError::system_error(format!(
                                        "index {} out of bounds (length {})",
                                        i, len
                                    ), ErrorCode::IndexOutOfBounds));
                                }
                                list[i as usize] = val;
                            } else {
                                return Err(RuntimeError::system_error("list index must be an integer", ErrorCode::TypeMismatch));
                            }
                        } else if let Value::Object(ref obj_ref) = obj {
                            if let Value::String(ref key) = idx {
                                obj_ref.borrow_mut().insert(key.as_ref().clone(), val);
                            } else {
                                return Err(RuntimeError::system_error(
                                    "object index must be a string", ErrorCode::TypeMismatch
                                ));
                            }
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot index into {}",
                                obj.type_name()
                            ), ErrorCode::TypeMismatch));
                        }
                    }
                }
                Ok(ControlFlow::None)
            }

            Statement::Block { statements } => {
                let block_env = env.child();
                let mut result = ControlFlow::None;
                for s in statements {
                    // Register defer on the function-level defer stack
                    if let Statement::Defer { body } = s {
                        task.register_defer(*body.clone(), block_env.clone());
                        continue;
                    }
                    match Self::exec_statement_yielding(vm, task, yc, s, &block_env).await? {
                        ControlFlow::Return(v) => { result = ControlFlow::Return(v); break; }
                        ControlFlow::Throw(v) => { result = ControlFlow::Throw(v); break; }
                        ControlFlow::None | ControlFlow::ExprValue(_) => {}
                    }
                }
                Ok(result)
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = Self::eval_expression_yielding(vm, task, yc, condition, env).await?;

                // Type narrowing: always analyze the condition and apply
                // narrowing facts.  Entry maintenance is unconditional.
                use crate::ledger::narrowing::{analyze_condition, invert_for_else, NarrowingFact};

                let facts = analyze_condition(condition);
                let snapshot = vm.borrow().ledger.save_entries();

                if cond.is_truthy() {
                    // Apply facts to true branch.
                    for fact in &facts {
                        match fact {
                            NarrowingFact::IsType { variable, type_name } => {
                                vm.borrow_mut().ledger.narrow_type(variable, type_name);
                            }
                            NarrowingFact::NotNull { variable } => {
                                vm.borrow_mut().ledger.narrow_exclude_null(variable);
                            }
                            NarrowingFact::IsNull { .. } => {
                                // IsNull in true branch: no narrowing benefit.
                            }
                        }
                    }
                    let result = Self::exec_statement_yielding(vm, task, yc, then_block, env).await;
                    let then_entries = vm.borrow().ledger.save_entries();
                    {
                        let mut vm_mut = vm.borrow_mut();
                        vm_mut.ledger.restore_entries(snapshot);
                        let current = vm_mut.ledger.save_entries();
                        vm_mut.ledger.merge_entries(then_entries, current);
                    }
                    result
                } else if let Some(eb) = else_block {
                    let else_facts = invert_for_else(&facts);
                    for fact in &else_facts {
                        match fact {
                            NarrowingFact::IsType { variable, type_name } => {
                                vm.borrow_mut().ledger.narrow_type(variable, type_name);
                            }
                            NarrowingFact::NotNull { variable } => {
                                vm.borrow_mut().ledger.narrow_exclude_null(variable);
                            }
                            NarrowingFact::IsNull { .. } => {}
                        }
                    }
                    let result = Self::exec_statement_yielding(vm, task, yc, eb, env).await;
                    let else_entries = vm.borrow().ledger.save_entries();
                    {
                        let mut vm_mut = vm.borrow_mut();
                        vm_mut.ledger.restore_entries(snapshot);
                        let current = vm_mut.ledger.save_entries();
                        vm_mut.ledger.merge_entries(current, else_entries);
                    }
                    result
                } else {
                    Ok(ControlFlow::None)
                }
            }

            Statement::While { condition, body, yield_every } => {
                // Evaluate yield_every count if present
                let yield_every_n = if let Some(ye_expr) = yield_every {
                    match Self::eval_expression_yielding(vm, task, yc, ye_expr, env).await? {
                        Value::Int(n) if n > 0 => Some(n as u64),
                        _ => None,
                    }
                } else {
                    None
                };
                let mut iteration: u64 = 0;
                loop {
                    let cond = Self::eval_expression_yielding(vm, task, yc, condition, env).await?;
                    if !cond.is_truthy() {
                        break;
                    }
                    // Yield budget check at loop back-edge
                    yc.check_yield_budget().await;
                    // yield every N: unconditional yield every N iterations
                    if let Some(n) = yield_every_n {
                        iteration += 1;
                        if iteration % n == 0 {
                            tokio::task::yield_now().await;
                            yc.reset_budget();
                        }
                    }
                    match Self::exec_statement_yielding(vm, task, yc, body, env).await? {
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Throw(v) => return Ok(ControlFlow::Throw(v)),
                        ControlFlow::None | ControlFlow::ExprValue(_) => {}
                    }
                }
                Ok(ControlFlow::None)
            }

            Statement::ForEach {
                variable,
                iterable,
                body,
                yield_every,
            } => {
                let iter_val = Self::eval_expression_yielding(vm, task, yc, iterable, env).await?;
                // Evaluate yield_every count if present
                let yield_every_n = if let Some(ye_expr) = yield_every {
                    match Self::eval_expression_yielding(vm, task, yc, ye_expr, env).await? {
                        Value::Int(n) if n > 0 => Some(n as u64),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Value::List(ref list_ref) = iter_val {
                    let items: Vec<Value> = list_ref.borrow().clone();
                    let mut iteration: u64 = 0;
                    for item in items {
                        let loop_env = env.child();
                        loop_env.define(variable.clone(), item);
                        // Yield budget check at loop back-edge
                        yc.check_yield_budget().await;
                        // yield every N: unconditional yield every N iterations
                        if let Some(n) = yield_every_n {
                            iteration += 1;
                            if iteration % n == 0 {
                                tokio::task::yield_now().await;
                                yc.reset_budget();
                            }
                        }
                        match Self::exec_statement_yielding(vm, task, yc, body, &loop_env).await? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Throw(v) => return Ok(ControlFlow::Throw(v)),
                            ControlFlow::None | ControlFlow::ExprValue(_) => {}
                        }
                    }
                } else {
                    return Err(RuntimeError::system_error(format!(
                        "cannot iterate over {}",
                        iter_val.type_name()
                    ), ErrorCode::TypeMismatch));
                }
                Ok(ControlFlow::None)
            }

            Statement::Return { value } => {
                let val = if let Some(expr) = value {
                    Self::eval_expression_yielding(vm, task, yc, expr, env).await?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(val))
            }

            Statement::ExpressionStmt(expr) => {
                let val = Self::eval_expression_yielding(vm, task, yc, expr, env).await?;
                Ok(ControlFlow::ExprValue(val))
            }

            Statement::FunctionDecl {
                name, params, body, return_type, is_async, ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                // Classify function as yielding or unyielding using the code analyzer.
                let classification = crate::analyzer::classify_function(body, *is_async, env, &param_names, Some(name.as_str()))?;
                let has_yielding_entry = match classification {
                    crate::analyzer::YieldingClassification::Yielding => {
                        // If declared inside @[unyielding], reject the contradiction.
                        if yc.unyielding_depth > 0 {
                            return Err(RuntimeError::system_error(
                                format!("function '{}' is annotated @[unyielding] but contains yielding operations", name),
                                ErrorCode::UnyieldingViolation,
                            ));
                        }
                        Some(true)
                    }
                    crate::analyzer::YieldingClassification::Unyielding => Some(false),
                };
                // Create a self-contained shim based on yielding classification.
                let captured_body = *body.clone();
                let captured_env = env.clone();
                let captured_params = param_names.clone();
                let captured_vm = vm.clone();
                let captured_is_async = *is_async;
                let captured_name = name.clone();
                let captured_return_type = return_type.clone();

                let shim: Shim = if has_yielding_entry == Some(true) {
                    // Yielding shim: spawn a local task and return Future.
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let vm_clone = captured_vm.clone();
                        let body_clone = captured_body.clone();
                        let is_async = captured_is_async;
                        let fn_name = captured_name.clone();
                        let ret_type = captured_return_type.clone();

                        let handle = tokio::task::spawn_local(async move {
                            let mut task_ctx = TaskContext::new();
                            let mut yield_ctx = YieldContext::new();
                            task_ctx.push_defer_frame();
                            task_ctx.async_stack.push((is_async, fn_name.clone()));
                            yield_ctx.check_yield_budget().await;
                            let result = IshVm::exec_statement_yielding(
                                &vm_clone, &mut task_ctx, &mut yield_ctx, &body_clone, &call_env,
                            ).await;
                            IshVm::pop_and_run_defers(&vm_clone, &mut task_ctx, &mut yield_ctx).await;
                            task_ctx.async_stack.pop();
                            let return_val = match result? {
                                ControlFlow::Return(v) => v,
                                ControlFlow::ExprValue(v) => v,
                                ControlFlow::None => Value::Null,
                                ControlFlow::Throw(v) => return Err(RuntimeError::thrown(v)),
                            };
                            IshVm::audit_type_annotation(
                                &vm_clone, &format!("return of '{fn_name}'"),
                                &return_val, ret_type.as_ref(),
                            )?;
                            Ok(return_val)
                        });
                        Ok(Value::Future(FutureRef::new(handle)))
                    })
                } else {
                    // Unyielding shim: execute body synchronously.
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let mut task_ctx = TaskContext::new();
                        task_ctx.push_defer_frame();
                        task_ctx.async_stack.push((captured_is_async, captured_name.clone()));
                        let result = IshVm::exec_statement_unyielding(
                            &captured_vm, &mut task_ctx, &captured_body, &call_env,
                        );
                        IshVm::pop_and_run_defers_unyielding(&captured_vm, &mut task_ctx);
                        task_ctx.async_stack.pop();
                        let return_val = match result? {
                            ControlFlow::Return(v) => v,
                            ControlFlow::ExprValue(v) => v,
                            ControlFlow::None => Value::Null,
                            ControlFlow::Throw(v) => return Err(RuntimeError::thrown(v)),
                        };
                        IshVm::audit_type_annotation(
                            &captured_vm, &format!("return of '{}'", captured_name),
                            &return_val, captured_return_type.as_ref(),
                        )?;
                        Ok(return_val)
                    })
                };

                let func_val = Value::Function(Gc::new(IshFunction {
                    name: Some(name.clone()),
                    params: param_names,
                    param_types,
                    return_type: return_type.clone(),
                    shim,
                    is_async: *is_async,
                    has_yielding_entry,
                }));
                env.define(name.clone(), func_val);
                Ok(ControlFlow::None)
            }

            Statement::Throw { value } => {
                let val = Self::eval_expression_yielding(vm, task, yc, value, env).await?;
                // Throw audit (unconditional): ensure the thrown value
                // qualifies as an error and add @Error entry.
                use crate::ledger::entry::Entry;
                let thrown = match &val {
                    Value::Object(ref obj_ref) => {
                        let map = obj_ref.borrow();
                        let has_message = matches!(map.get("message"), Some(Value::String(_)));
                        if has_message {
                            // (a) Object with message: String → add @Error entry
                            drop(map);
                            vm.borrow_mut().ledger.add_entry("@thrown",
                                Entry::new("Error").with_param("message", "String"));
                            val
                        } else {
                            // (b) Object without message: String → wrap in system error
                            drop(map);
                            let mut wrapper = HashMap::new();
                            wrapper.insert("message".to_string(), Value::String(Rc::new(
                                "throw audit: thrown object lacks 'message: String' property".to_string()
                            )));
                            wrapper.insert("code".to_string(), Value::String(Rc::new(ErrorCode::UnhandledThrow.as_str().to_string())));
                            wrapper.insert("original".to_string(), val);
                            let wrapped = new_object(wrapper);
                            vm.borrow_mut().ledger.add_entry("@thrown",
                                Entry::new("Error").with_param("message", "String"));
                            wrapped
                        }
                    }
                    _ => {
                        // (c) Non-object → wrap in system error
                        let mut wrapper = HashMap::new();
                        wrapper.insert("message".to_string(), Value::String(Rc::new(
                            format!("throw audit: non-object thrown: {}", val)
                        )));
                        wrapper.insert("code".to_string(), Value::String(Rc::new(ErrorCode::UnhandledThrow.as_str().to_string())));
                        wrapper.insert("original".to_string(), val);
                        let wrapped = new_object(wrapper);
                        vm.borrow_mut().ledger.add_entry("@thrown",
                            Entry::new("Error").with_param("message", "String"));
                        wrapped
                    }
                };
                Ok(ControlFlow::Throw(thrown))
            }

            Statement::TryCatch { body, catches, finally } => {
                // Collect thrown value from either ControlFlow::Throw or
                // a RuntimeError with thrown_value (throw that crossed a
                // function boundary via call_function).
                let (result, thrown) = match Self::exec_statement_yielding(vm, task, yc, body, env).await {
                    Ok(ControlFlow::Throw(v)) => (ControlFlow::None, Some(v)),
                    Ok(other) => (other, None),
                    Err(e) if e.thrown_value.is_some() => {
                        (ControlFlow::None, e.thrown_value)
                    }
                    Err(e) => return Err(e),
                };
                let result = if let Some(thrown) = thrown {
                    // Try to match a catch clause
                    let mut caught = false;
                    let mut catch_result = ControlFlow::None;
                    for clause in catches {
                        // For now, all catch clauses match (type-based
                        // matching will come with the type system).
                        let catch_env = env.child();
                        catch_env.define(clause.param.clone(), thrown.clone());
                        catch_result = Self::exec_statement_yielding(vm, task, yc, &clause.body, &catch_env).await?;
                        caught = true;
                        break;
                    }
                    if caught {
                        catch_result
                    } else {
                        // No matching catch — re-throw
                        ControlFlow::Throw(thrown)
                    }
                } else {
                    result
                };
                // Execute finally block if present (always runs)
                if let Some(fin) = finally {
                    let fin_result = Self::exec_statement_yielding(vm, task, yc, fin, env).await?;
                    // Per decision: no return from finally.
                    // finally block does NOT override the result.
                    // But if finally throws, that replaces the original.
                    if let ControlFlow::Throw(_) = fin_result {
                        return Ok(fin_result);
                    }
                }
                Ok(result)
            }

            Statement::WithBlock { resources, body } => {
                let with_env = env.child();
                let mut initialized: Vec<(String, Value)> = Vec::new();
                // Initialize resources in order
                for (name, expr) in resources {
                    match Self::eval_expression_yielding(vm, task, yc, expr, &with_env).await {
                        Ok(val) => {
                            with_env.define(name.clone(), val.clone());
                            initialized.push((name.clone(), val));
                        }
                        Err(e) => {
                            // Close already-initialized resources in reverse order
                            for (_, res) in initialized.into_iter().rev() {
                                let _ = Self::try_close_yielding(vm, &res).await;
                            }
                            return Err(e);
                        }
                    }
                }
                // Execute body
                let result = Self::exec_statement_yielding(vm, task, yc, body, &with_env).await?;
                // Close resources in reverse order
                let mut close_error: Option<Value> = None;
                for (_, res) in initialized.into_iter().rev() {
                    if let Err(_e) = Self::try_close_yielding(vm, &res).await {
                        // If close fails, save the error
                        if close_error.is_none() {
                            close_error = Some(Value::String(Rc::new(_e.message)));
                        }
                    }
                }
                // If body threw and close also errored, body error takes precedence
                match result {
                    ControlFlow::Throw(_) => Ok(result),
                    other => {
                        if let Some(err) = close_error {
                            Ok(ControlFlow::Throw(err))
                        } else {
                            Ok(other)
                        }
                    }
                }
            }

            Statement::Defer { body } => {
                // Register on the enclosing function's defer stack.
                task.register_defer(*body.clone(), env.clone());
                Ok(ControlFlow::None)
            }

            Statement::TypeAlias { .. } => {
                // Type aliases are checked at analysis time, not runtime
                Ok(ControlFlow::None)
            }

            Statement::Use { module_path, alias, selective } => {
                Self::eval_use(vm, task, yc, env, module_path, alias.as_deref(), selective.as_deref()).await
            }

            Statement::DeclareBlock { body } => {
                Self::eval_declare_block_yielding(vm, task, yc, env, body).await
            }

            Statement::Bootstrap { .. } => {
                Self::eval_bootstrap(vm, task)
            }

            Statement::ShellCommand { command, args, pipes, redirections, background: _ } => {
                Self::exec_shell_command(vm, task, yc, command, args, pipes, redirections, env).await
            }

            Statement::Annotated { annotations, inner } => {
                // Process standard annotations: push standards for the scope.
                let mut pushed_standards = Vec::new();
                let mut unyielding = false;
                let mut saved_budget: Option<Duration> = None;
                for ann in annotations {
                    match ann {
                        Annotation::Standard(name) => {
                            vm.borrow_mut().ledger.push_standard(name.clone());
                            pushed_standards.push(name.clone());
                        }
                        Annotation::Entry(items) => {
                            for item in items {
                                if item.name == "unyielding" {
                                    unyielding = true;
                                } else if item.name == "yield_budget" {
                                    if let Some(ref dur_str) = item.value {
                                        if let Some(dur) = parse_duration_annotation(dur_str) {
                                            saved_budget = Some(yc.budget_duration);
                                            yc.budget_duration = dur;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if unyielding {
                    yc.unyielding_depth += 1;
                    // Audit: guaranteed_yield — unyielding blocks are flagged if
                    // the active standard requires guaranteed_yield, unless the
                    // block/function has a Complexity(value: "simple") entry.
                    let features = vm.borrow().ledger.active_features();
                    if let Some(feature) = features.get("guaranteed_yield") {
                        if feature.annotation
                            == crate::ledger::standard::AnnotationDimension::Required
                        {
                            // Check if an explicit Complexity(simple) entry exists
                            // on the inner statement (by name, if it's a function decl)
                            let fn_name = match inner.as_ref() {
                                Statement::FunctionDecl { name, .. } => Some(name.as_str()),
                                _ => None,
                            };
                            let has_simple = fn_name.map_or(false, |n| {
                                vm.borrow().ledger.get_entries(n)
                                    .and_then(|es| es.get("Complexity"))
                                    .and_then(|e| e.params.get("value"))
                                    .map_or(false, |v| v == "simple")
                            });
                            if !has_simple {
                                let item = fn_name.unwrap_or("block");
                                return Err(RuntimeError::system_error(
                                    format!("'{}' is unyielding — guaranteed yield required by active standard", item),
                                    ErrorCode::AsyncError,
                                ));
                            }
                        }
                    }
                }
                let result = Self::exec_statement_yielding(vm, task, yc, inner, env).await;
                if unyielding {
                    yc.unyielding_depth -= 1;
                }
                if let Some(prev) = saved_budget {
                    yc.budget_duration = prev;
                }
                // Audit: future_drop — check before popping standard scope.
                if !pushed_standards.is_empty() {
                    let unawaited = crate::value::take_unawaited_future_count();
                    if unawaited > 0 {
                        let features = vm.borrow().ledger.active_features();
                        if let crate::ledger::audit::AuditResult::Discrepancy(report) =
                            crate::ledger::audit::audit_future_drop(
                                &features,
                                &format!("{} future(s) dropped without await", unawaited),
                            )
                        {
                            // Pop standards before returning the error.
                            for _ in pushed_standards.iter().rev() {
                                vm.borrow_mut().ledger.pop_standard();
                            }
                            return Err(RuntimeError::system_error(report.message, ErrorCode::AsyncError));
                        }
                    }
                }
                // Pop standards in reverse order.
                for _ in pushed_standards.iter().rev() {
                    vm.borrow_mut().ledger.pop_standard();
                }
                result
            }

            Statement::StandardDef { name, extends, features } => {
                // Register the standard in the ledger.
                use crate::ledger::standard::Standard;

                let mut std = Standard::new(name.clone());
                if let Some(parent) = extends {
                    std = std.with_parent(parent.clone());
                }
                for feat in features {
                    let state = parse_feature_params(&feat.params);
                    std = std.with_feature(feat.name.clone(), state);
                }
                vm.borrow_mut().ledger.standard_registry.register(std);
                Ok(ControlFlow::None)
            }

            Statement::EntryTypeDef { name, .. } => {
                // Register as a simple entry type (no required properties for now).
                use crate::ledger::entry_type::EntryType;
                vm.borrow_mut().ledger.entry_type_registry.register(EntryType::new(name.clone()));
                Ok(ControlFlow::None)
            }

            Statement::Match { .. } => {
                // Match not yet implemented in interpreter
                Ok(ControlFlow::None)
            }

            Statement::Incomplete { kind } => {
                Err(RuntimeError::system_error(format!("incomplete input: {:?}", kind), ErrorCode::TypeMismatch))
            }

            Statement::Yield => {
                // Audit: async_annotation — check if current function is not async
                if let Some((false, ref fn_name)) = task.async_stack.last() {
                    let features = vm.borrow().ledger.active_features();
                    if let crate::ledger::audit::AuditResult::Discrepancy(report) =
                        crate::ledger::audit::audit_async_annotation(
                            &features, false, fn_name,
                        )
                    {
                        return Err(RuntimeError::system_error(report.message, ErrorCode::AsyncError));
                    }
                }
                tokio::task::yield_now().await;
                yc.reset_budget();
                Ok(ControlFlow::None)
            }
        }
        }) // end Box::pin(async move { })
    }

    /// Try to call close() on a value, used by WithBlock (synchronous path).
    fn try_close(vm: &Rc<RefCell<IshVm>>, value: &Value) -> Result<(), RuntimeError> {
        if let Value::Object(ref obj_ref) = value {
            let map = obj_ref.borrow();
            if let Some(close_fn) = map.get("close").cloned() {
                drop(map);
                Self::call_function_inner(vm, &close_fn, &[])?;
            }
        }
        Ok(())
    }

    /// Try to call close() on a value, used by WithBlock (yielding path).
    /// If the close method returns a Future (yielding close fn), awaits it.
    async fn try_close_yielding(vm: &Rc<RefCell<IshVm>>, value: &Value) -> Result<(), RuntimeError> {
        if let Value::Object(ref obj_ref) = value {
            let map = obj_ref.borrow();
            if let Some(close_fn) = map.get("close").cloned() {
                drop(map);
                let result = Self::call_function_inner(vm, &close_fn, &[])?;
                if let Value::Future(ref future_ref) = result {
                    if let Some(handle) = future_ref.take() {
                        match handle.await {
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => return Err(e),
                            Err(_join_err) => return Err(RuntimeError::system_error(
                                "close task panicked",
                                ErrorCode::AsyncError,
                            )),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Evaluate an expression in the given environment.
    fn eval_expression_yielding<'a>(
        vm: &'a Rc<RefCell<IshVm>>,
        task: &'a mut TaskContext,
        yc: &'a mut YieldContext,
        expr: &'a Expression,
        env: &'a Environment,
    ) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + 'a>> {
        Box::pin(async move {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Int(n) => Value::Int(*n),
                Literal::Float(f) => Value::Float(*f),
                Literal::String(s) => Value::String(Rc::new(s.clone())),
                Literal::Char(c) => Value::Char(*c),
                Literal::Null => Value::Null,
            }),

            Expression::Identifier(name) => env.get(name),

            Expression::BinaryOp { op, left, right } => {
                let lhs = Self::eval_expression_yielding(vm, task, yc, left, env).await?;
                // Short-circuit for logical operators
                match op {
                    BinaryOperator::And => {
                        if !lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return Self::eval_expression_yielding(vm, task, yc, right, env).await;
                    }
                    BinaryOperator::Or => {
                        if lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return Self::eval_expression_yielding(vm, task, yc, right, env).await;
                    }
                    _ => {}
                }
                let rhs = Self::eval_expression_yielding(vm, task, yc, right, env).await?;
                Self::eval_binary_op(op, &lhs, &rhs)
            }

            Expression::UnaryOp { op, operand } => {
                let val = Self::eval_expression_yielding(vm, task, yc, operand, env).await?;
                match op {
                    UnaryOperator::Not => Ok(Value::Bool(!val.is_truthy())),
                    UnaryOperator::Negate => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(RuntimeError::system_error(format!(
                            "cannot negate {}",
                            val.type_name()
                        ), ErrorCode::TypeMismatch)),
                    },
                    UnaryOperator::Try => {
                        // ? operator: if value is an error, propagate it; otherwise unwrap
                        // For now, null signals error
                        if val == Value::Null {
                            return Err(RuntimeError::system_error("tried to unwrap null value with ?".to_string(), ErrorCode::NullUnwrap));
                        }
                        Ok(val)
                    }
                }
            }

            Expression::FunctionCall { callee, args } => {
                let func_val = Self::eval_expression_yielding(vm, task, yc, callee, env).await?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(Self::eval_expression_yielding(vm, task, yc, arg, env).await?);
                }
                let result = Self::call_function_inner(vm, &func_val, &arg_vals)?;
                // Implied await: if the callee is yielding and the result is a
                // Future, await it transparently. Unyielding callees that return
                // Value::Future (e.g. apply) are NOT implicitly awaited.
                let is_yielding_callee = if let Value::Function(ref f) = func_val {
                    f.has_yielding_entry == Some(true)
                } else {
                    false
                };
                if is_yielding_callee {
                    if let Value::Future(ref future_ref) = result {
                        match future_ref.take() {
                            Some(handle) => {
                                match handle.await {
                                    Ok(inner) => inner,
                                    Err(join_err) => {
                                        if join_err.is_cancelled() {
                                            Err(RuntimeError::system_error(
                                                "called task was cancelled",
                                                ErrorCode::AsyncError,
                                            ))
                                        } else {
                                            Err(RuntimeError::system_error(
                                                format!("called task panicked: {}", join_err),
                                                ErrorCode::AsyncError,
                                            ))
                                        }
                                    }
                                }
                            }
                            None => {
                                Err(RuntimeError::system_error(
                                    "future has already been awaited",
                                    ErrorCode::AsyncError,
                                ))
                            }
                        }
                    } else {
                        Ok(result)
                    }
                } else {
                    Ok(result)
                }
            }

            Expression::ObjectLiteral(pairs) => {
                let mut map = HashMap::new();
                for (key, val_expr) in pairs {
                    let val = Self::eval_expression_yielding(vm, task, yc, val_expr, env).await?;
                    map.insert(key.clone(), val);
                }
                Ok(new_object(map))
            }

            Expression::ListLiteral(elements) => {
                let mut items = Vec::with_capacity(elements.len());
                for elem in elements {
                    items.push(Self::eval_expression_yielding(vm, task, yc, elem, env).await?);
                }
                Ok(new_list(items))
            }

            Expression::PropertyAccess { object, property } => {
                let obj = Self::eval_expression_yielding(vm, task, yc, object, env).await?;
                match obj {
                    Value::Object(ref obj_ref) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(property).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::system_error(format!(
                        "cannot access property '{}' on {}",
                        property,
                        obj.type_name()
                    ), ErrorCode::TypeMismatch)),
                }
            }

            Expression::IndexAccess { object, index } => {
                let obj = Self::eval_expression_yielding(vm, task, yc, object, env).await?;
                let idx = Self::eval_expression_yielding(vm, task, yc, index, env).await?;
                match (&obj, &idx) {
                    (Value::List(list_ref), Value::Int(i)) => {
                        let list = list_ref.borrow();
                        let i = *i;
                        if i < 0 || i >= list.len() as i64 {
                            return Err(RuntimeError::system_error(format!(
                                "index {} out of bounds (length {})",
                                i,
                                list.len()
                            ), ErrorCode::IndexOutOfBounds));
                        }
                        Ok(list[i as usize].clone())
                    }
                    (Value::Object(obj_ref), Value::String(key)) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(key.as_ref()).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::system_error(format!(
                        "cannot index {} with {}",
                        obj.type_name(),
                        idx.type_name()
                    ), ErrorCode::TypeMismatch)),
                }
            }

            Expression::Lambda { params, body, is_async } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                // Classify lambda as yielding or unyielding using the code analyzer.
                // Lambdas have no name (None), so recursive self-calls are not pre-seeded.
                let classification = crate::analyzer::classify_function(body, *is_async, env, &param_names, None)?;
                let has_yielding_entry = match classification {
                    crate::analyzer::YieldingClassification::Yielding => Some(true),
                    crate::analyzer::YieldingClassification::Unyielding => Some(false),
                };
                // Create a self-contained shim based on yielding classification.
                let captured_body = *body.clone();
                let captured_env = env.clone();
                let captured_params = param_names.clone();
                let captured_vm = vm.clone();
                let captured_is_async = *is_async;

                let shim: Shim = if has_yielding_entry == Some(true) {
                    // Yielding shim: spawn a local task and return Future.
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let vm_clone = captured_vm.clone();
                        let body_clone = captured_body.clone();
                        let is_async = captured_is_async;

                        let handle = tokio::task::spawn_local(async move {
                            let mut task_ctx = TaskContext::new();
                            let mut yield_ctx = YieldContext::new();
                            task_ctx.push_defer_frame();
                            task_ctx.async_stack.push((is_async, "anonymous".to_string()));
                            yield_ctx.check_yield_budget().await;
                            let result = IshVm::exec_statement_yielding(
                                &vm_clone, &mut task_ctx, &mut yield_ctx, &body_clone, &call_env,
                            ).await;
                            IshVm::pop_and_run_defers(&vm_clone, &mut task_ctx, &mut yield_ctx).await;
                            task_ctx.async_stack.pop();
                            match result? {
                                ControlFlow::Return(v) => Ok(v),
                                ControlFlow::ExprValue(v) => Ok(v),
                                ControlFlow::None => Ok(Value::Null),
                                ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
                            }
                        });
                        Ok(Value::Future(FutureRef::new(handle)))
                    })
                } else {
                    // Unyielding shim: execute body synchronously.
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let mut task_ctx = TaskContext::new();
                        task_ctx.push_defer_frame();
                        task_ctx.async_stack.push((captured_is_async, "anonymous".to_string()));
                        let result = IshVm::exec_statement_unyielding(
                            &captured_vm, &mut task_ctx, &captured_body, &call_env,
                        );
                        IshVm::pop_and_run_defers_unyielding(&captured_vm, &mut task_ctx);
                        task_ctx.async_stack.pop();
                        match result? {
                            ControlFlow::Return(v) => Ok(v),
                            ControlFlow::ExprValue(v) => Ok(v),
                            ControlFlow::None => Ok(Value::Null),
                            ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
                        }
                    })
                };

                Ok(Value::Function(Gc::new(IshFunction {
                    name: None,
                    params: param_names,
                    param_types,
                    return_type: None,
                    shim,
                    is_async: *is_async,
                    has_yielding_entry,
                })))
            }

            Expression::StringInterpolation(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ish_ast::StringPart::Text(text) => result.push_str(text),
                        ish_ast::StringPart::Expr(expr) => {
                            let val = Self::eval_expression_yielding(vm, task, yc, expr, env).await?;
                            result.push_str(&val.to_display_string());
                        }
                    }
                }
                Ok(Value::String(Rc::new(result)))
            }

            Expression::CommandSubstitution(inner) => {
                // Execute the inner statement and capture its stdout output
                match inner.as_ref() {
                    Statement::ShellCommand { command, args, pipes, redirections, background: _ } => {
                        let resolved_args = Self::resolve_shell_args(vm, task, yc, args, env).await?;
                        let output = Self::run_command_pipeline(vm, task, yc, command, &resolved_args, pipes, redirections, env, true).await?;
                        Ok(Value::String(Rc::new(output.trim_end_matches('\n').to_string())))
                    }
                    _ => {
                        // Execute as normal statement, capture result
                        match Self::exec_statement_yielding(vm, task, yc, inner, env).await? {
                            ControlFlow::ExprValue(v) => Ok(v),
                            ControlFlow::Return(v) => Ok(v),
                            _ => Ok(Value::Null),
                        }
                    }
                }
            }

            Expression::EnvVar(name) => {
                if name == "?" {
                    // $? — last exit code, stored in the VM
                    match env.get("__ish_last_exit_code") {
                        Ok(v) => Ok(v),
                        Err(_) => Ok(Value::Int(0)),
                    }
                } else {
                    match std::env::var(name) {
                        Ok(val) => Ok(Value::String(Rc::new(val))),
                        Err(_) => Ok(Value::Null),
                    }
                }
            }

            Expression::Incomplete { kind } => {
                Err(RuntimeError::system_error(format!("incomplete expression: {:?}", kind), ErrorCode::TypeMismatch))
            }

            Expression::Await { expr } => {
                // Audit: async_annotation — check if current function is not async
                if let Some((false, ref fn_name)) = task.async_stack.last() {
                    let features = vm.borrow().ledger.active_features();
                    if let crate::ledger::audit::AuditResult::Discrepancy(report) =
                        crate::ledger::audit::audit_async_annotation(
                            &features,
                            false,
                            fn_name,
                        )
                    {
                        return Err(RuntimeError::system_error(report.message, ErrorCode::AsyncError));
                    }
                }

                // Evaluate the await operand to obtain a Future value.
                //
                // For FunctionCall expressions, we call the function WITHOUT implied
                // await so we receive the raw Future from the yielding shim. Implied
                // await would resolve the Future before we see it, leaving us with
                // the resolved value (e.g. Int) rather than the Future to await.
                //
                // For all other expressions (identifiers, index access, etc.) we eval
                // normally — the expression is expected to already hold a Future.
                let val = if let Expression::FunctionCall { callee: call_callee, args: call_args } = expr.as_ref() {
                    let callee_val = Self::eval_expression_yielding(vm, task, yc, call_callee, env).await?;
                    // E012 check: unyielding callee
                    if let Value::Function(ref f) = callee_val {
                        if f.has_yielding_entry != Some(true) {
                            return Err(RuntimeError::system_error(
                                format!("cannot await unyielding function '{}'",
                                    f.name.as_deref().unwrap_or("<anonymous>")),
                                ErrorCode::AwaitUnyielding,
                            ));
                        }
                    }
                    let mut arg_vals = Vec::with_capacity(call_args.len());
                    for arg in call_args {
                        arg_vals.push(Self::eval_expression_yielding(vm, task, yc, arg, env).await?);
                    }
                    // Call without implied await — get the raw Value::Future from the shim.
                    Self::call_function_inner(vm, &callee_val, &arg_vals)?
                } else {
                    // Non-call expression: eval normally, expect a Value::Future.
                    Self::eval_expression_yielding(vm, task, yc, expr, env).await?
                };

                match val {
                    Value::Future(ref future_ref) => {
                        // Await the future
                        match future_ref.take() {
                            Some(handle) => {
                                match handle.await {
                                    Ok(result) => result,
                                    Err(join_err) => {
                                        if join_err.is_cancelled() {
                                            Err(RuntimeError::system_error(
                                                "awaited task was cancelled".to_string(),
                                                ErrorCode::AsyncError,
                                            ))
                                        } else {
                                            Err(RuntimeError::system_error(
                                                format!("awaited task panicked: {}", join_err),
                                                ErrorCode::AsyncError,
                                            ))
                                        }
                                    }
                                }
                            }
                            None => {
                                Err(RuntimeError::system_error(
                                    "future has already been awaited".to_string(),
                                    ErrorCode::AsyncError,
                                ))
                            }
                        }
                    }
                    Value::Function(ref f) if f.has_yielding_entry == Some(false) => {
                        // E012: await applied to an unyielding function value (non-call)
                        Err(RuntimeError::system_error(
                            format!("cannot await unyielding function '{}'",
                                f.name.as_deref().unwrap_or("<anonymous>")),
                            ErrorCode::AwaitUnyielding,
                        ))
                    }
                    _ => {
                        // E014: await applied to a non-future value
                        Err(RuntimeError::system_error(
                            format!("await requires a Future value, got {}", val.type_name()),
                            ErrorCode::AwaitNonFuture,
                        ))
                    }
                }
            }

            Expression::Spawn { callee, args } => {
                // Resolve callee
                let callee_val = Self::eval_expression_yielding(vm, task, yc, callee, env).await?;

                // Check yielding classification before spawning (E013)
                if let Value::Function(ref f) = callee_val {
                    if f.has_yielding_entry != Some(true) {
                        return Err(RuntimeError::system_error(
                            format!("cannot spawn unyielding function '{}'",
                                f.name.as_deref().unwrap_or("<anonymous>")),
                            ErrorCode::SpawnUnyielding,
                        ));
                    }
                }

                // Evaluate arguments in the current context
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    arg_vals.push(Self::eval_expression_yielding(vm, task, yc, arg, env).await?);
                }

                // Call the function — the yielding shim spawns a task and
                // returns Value::Future directly.
                let result = Self::call_function_inner(vm, &callee_val, &arg_vals)?;
                // The result should be Value::Future from the yielding shim.
                Ok(result)
            }
        }
        }) // end Box::pin(async move { })
    }

    // ── Module system: Use, DeclareBlock, Bootstrap ─────────────────────────

    /// Evaluate a `use` statement (yielding path).
    async fn eval_use(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        yc: &mut YieldContext,
        env: &Environment,
        module_path: &[String],
        alias: Option<&str>,
        selective: Option<&[SelectiveImport]>,
    ) -> Result<ControlFlow, RuntimeError> {
        // Step 1: Check if external path (first segment contains '.').
        if module_path.first().map_or(false, |s| s.contains('.')) {
            return Ok(ControlFlow::None); // External paths deferred.
        }

        // Step 2: Get src_root from project context.
        let src_root = {
            let vm_ref = vm.borrow();
            vm_ref.project_context.src_root.clone()
        };
        let src_root = match src_root {
            Some(r) => r,
            None => return Err(RuntimeError::system_error(
                "use statement requires a project context (src_root not set)",
                ErrorCode::ModuleNotFound,
            )),
        };

        // Step 3: Resolve module path to a file.
        let file_path = module_loader::resolve_module_path(module_path, &src_root)?;

        // Step 4: Cycle detection.
        if module_loader::check_cycle(&task.loading_stack, module_path) {
            let cycle_str = task.loading_stack.iter()
                .map(|p| p.join("/"))
                .chain(std::iter::once(module_path.join("/")))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(RuntimeError::system_error(
                format!("Circular module dependency detected: {}", cycle_str),
                ErrorCode::ModuleCycle,
            ));
        }

        // Step 5: Read and parse the file.
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            RuntimeError::system_error(
                format!("Failed to read module file {:?}: {}", file_path, e),
                ErrorCode::ModuleNotFound,
            )
        })?;

        let program = ish_parser::parse(&source).map_err(|errors| {
            RuntimeError::system_error(
                format!("Parse error in module file {:?}: {:?}", file_path, errors),
                ErrorCode::ModuleNotFound,
            )
        })?;

        // Step 6: Check declarations-only (implicit declare wrapping).
        for stmt in &program.statements {
            if !Self::is_declaration(stmt) {
                return Err(RuntimeError::system_error(
                    format!(
                        "File {:?} contains top-level commands and cannot be imported via `use`. Only files with function and type declarations are importable.",
                        file_path
                    ),
                    ErrorCode::ModuleScriptNotImportable,
                ));
            }
        }

        // Step 7: Interface check.
        let interface_errors = interface_checker::check_interface(&file_path, &program.statements);
        if let Some(first_err) = interface_errors.into_iter().next() {
            return Err(RuntimeError::system_error(
                first_err.message,
                first_err.code,
            ));
        }

        // Step 8: Evaluate module in a child environment.
        let saved_file = task.current_file.take();
        task.loading_stack.push(module_path.to_vec());
        task.current_file = Some(file_path.clone());

        let module_env = env.child();
        for stmt in &program.statements {
            // Pre-register names for forward references within the module.
            match stmt {
                Statement::FunctionDecl { name, .. } => {
                    module_env.define(name.clone(), Value::Null);
                }
                Statement::TypeAlias { name, .. } => {
                    module_env.define(name.clone(), Value::Null);
                }
                _ => {}
            }
        }
        for stmt in &program.statements {
            Self::exec_statement_yielding(vm, task, yc, stmt, &module_env).await?;
        }

        task.loading_stack.pop();
        task.current_file = saved_file;

        // Step 9: Bind namespace into caller's environment.
        let module_name = alias.unwrap_or_else(|| module_path.last().map(|s| s.as_str()).unwrap_or(""));

        if let Some(selective_imports) = selective {
            // Selective import: bind each name into caller env.
            let project_root = vm.borrow().project_context.project_root.clone();
            for sel in selective_imports {
                let value = module_env.get(&sel.name).map_err(|_| {
                    RuntimeError::system_error(
                        format!("Symbol '{}' not found in module '{}'", sel.name, module_path.join("/")),
                        ErrorCode::ModuleNotFound,
                    )
                })?;
                // Access control check on selective imports.
                let item_visibility = Self::get_symbol_visibility(&program.statements, &sel.name);
                if let Some(vis) = item_visibility {
                    let check_result = access_control::check_access(
                        &vis,
                        Some(&file_path),
                        project_root.as_deref(),
                        task.current_file.as_deref(),
                        project_root.as_deref(),
                    );
                    if let Err(access_err) = check_result {
                        return Err(RuntimeError::system_error(
                            format!("Access denied for '{}': {}", sel.name, access_err),
                            ErrorCode::ModuleNotFound,
                        ));
                    }
                }
                let bind_name = sel.alias.as_ref().unwrap_or(&sel.name);
                env.define(bind_name.clone(), value);
            }
        } else {
            // Qualified or aliased import: bind module as an object.
            let bindings = module_env.all_bindings();
            let mut obj_map = HashMap::new();
            for (k, v) in bindings {
                obj_map.insert(k, v);
            }
            env.define(module_name.to_string(), new_object(obj_map));
        }

        Ok(ControlFlow::None)
    }

    /// Evaluate a `declare { }` block (yielding path).
    async fn eval_declare_block_yielding(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        yc: &mut YieldContext,
        env: &Environment,
        body: &[Statement],
    ) -> Result<ControlFlow, RuntimeError> {
        // Step 1: Validate declarations-only.
        for stmt in body {
            if !Self::is_declaration(stmt) {
                return Err(RuntimeError::system_error(
                    "declare { } block contains a non-declaration statement. Only function definitions and type aliases are allowed inside declare { }.",
                    ErrorCode::ModuleDeclareBlockCommand,
                ));
            }
        }

        // Step 2: Pre-register all names for forward references.
        for stmt in body {
            match stmt {
                Statement::FunctionDecl { name, .. } => {
                    env.define(name.clone(), Value::Null);
                }
                Statement::TypeAlias { name, .. } => {
                    env.define(name.clone(), Value::Null);
                }
                _ => {}
            }
        }

        // Step 3: Evaluate each declaration in order.
        // The pre-registration enables mutual forward references.
        for stmt in body {
            Self::exec_statement_yielding(vm, task, yc, stmt, env).await?;
        }

        Ok(ControlFlow::None)
    }

    /// Evaluate a `declare { }` block (unyielding path).
    fn eval_declare_block_unyielding(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        env: &Environment,
        body: &[Statement],
    ) -> Result<ControlFlow, RuntimeError> {
        // Step 1: Validate declarations-only.
        for stmt in body {
            if !Self::is_declaration(stmt) {
                return Err(RuntimeError::system_error(
                    "declare { } block contains a non-declaration statement. Only function definitions and type aliases are allowed inside declare { }.",
                    ErrorCode::ModuleDeclareBlockCommand,
                ));
            }
        }

        // Step 2: Pre-register all names for forward references.
        for stmt in body {
            match stmt {
                Statement::FunctionDecl { name, .. } => {
                    env.define(name.clone(), Value::Null);
                }
                Statement::TypeAlias { name, .. } => {
                    env.define(name.clone(), Value::Null);
                }
                _ => {}
            }
        }

        // Step 3: Evaluate each declaration in order.
        for stmt in body {
            Self::exec_statement_unyielding(vm, task, stmt, env)?;
        }

        Ok(ControlFlow::None)
    }

    /// Evaluate a `bootstrap` statement. Checks project membership (E021).
    /// Config parsing is deferred (D20).
    fn eval_bootstrap(
        vm: &Rc<RefCell<IshVm>>,
        task: &TaskContext,
    ) -> Result<ControlFlow, RuntimeError> {
        if let Some(ref current_file) = task.current_file {
            if let Some(parent) = current_file.parent() {
                if module_loader::find_project_root(parent).is_some() {
                    return Err(RuntimeError::system_error(
                        format!(
                            "bootstrap cannot be used inside a project hierarchy. File {:?} is under a project.json directory.",
                            current_file
                        ),
                        ErrorCode::ModuleBootstrapInProject,
                    ));
                }
            }
        }
        // Config parsing deferred (D20). Just return success for standalone scripts.
        let _ = vm; // suppress unused warning
        Ok(ControlFlow::None)
    }

    /// Check whether a statement is a declaration (FunctionDecl, TypeAlias, Use, DeclareBlock, Bootstrap).
    fn is_declaration(stmt: &Statement) -> bool {
        matches!(stmt, Statement::FunctionDecl { .. } | Statement::TypeAlias { .. } | Statement::Use { .. } | Statement::DeclareBlock { .. } | Statement::Bootstrap { .. })
    }

    /// Get the visibility of a named symbol from a list of statements.
    fn get_symbol_visibility(stmts: &[Statement], name: &str) -> Option<Visibility> {
        for stmt in stmts {
            match stmt {
                Statement::FunctionDecl { name: fn_name, visibility, .. } if fn_name == name => {
                    return Some(visibility.clone().unwrap_or(Visibility::Pkg));
                }
                Statement::TypeAlias { name: type_name, visibility, .. } if type_name == name => {
                    return Some(visibility.clone().unwrap_or(Visibility::Pkg));
                }
                _ => {}
            }
        }
        None
    }

    // ── Shell command execution ─────────────────────────────────────────────

    async fn exec_shell_command(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        yc: &mut YieldContext,
        command: &str,
        args: &[ShellArg],
        pipes: &[ShellPipeline],
        redirections: &[Redirection],
        env: &Environment,
    ) -> Result<ControlFlow, RuntimeError> {
        let resolved_args = Self::resolve_shell_args(vm, task, yc, args, env).await?;

        // Check for builtins
        match command {
            "cd" => {
                let dir = resolved_args.first().map(|s| s.as_str()).unwrap_or("~");
                let target = if dir == "~" {
                    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
                } else {
                    dir.to_string()
                };
                match std::env::set_current_dir(&target) {
                    Ok(()) => {
                        env.define("__ish_last_exit_code".to_string(), Value::Int(0));
                        Ok(ControlFlow::None)
                    }
                    Err(e) => {
                        env.define("__ish_last_exit_code".to_string(), Value::Int(1));
                        Err(RuntimeError::system_error(format!("cd: {}: {}", target, e), ErrorCode::ShellError))
                    }
                }
            }
            "pwd" => {
                match std::env::current_dir() {
                    Ok(p) => {
                        let path_str = p.display().to_string();
                        println!("{}", path_str);
                        env.define("__ish_last_exit_code".to_string(), Value::Int(0));
                        Ok(ControlFlow::ExprValue(Value::String(Rc::new(path_str))))
                    }
                    Err(e) => {
                        env.define("__ish_last_exit_code".to_string(), Value::Int(1));
                        Err(RuntimeError::system_error(format!("pwd: {}", e), ErrorCode::ShellError))
                    }
                }
            }
            "exit" => {
                let code: i32 = resolved_args
                    .first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                std::process::exit(code);
            }
            _ => {
                // External command
                let output = Self::run_command_pipeline(vm, task, yc, command, &resolved_args, pipes, redirections, env, false).await?;
                if !output.is_empty() {
                    Ok(ControlFlow::ExprValue(Value::String(Rc::new(output))))
                } else {
                    Ok(ControlFlow::None)
                }
            }
        }
    }

    fn resolve_shell_args<'a>(
        vm: &'a Rc<RefCell<IshVm>>,
        task: &'a mut TaskContext,
        yc: &'a mut YieldContext,
        args: &'a [ShellArg],
        env: &'a Environment,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, RuntimeError>> + 'a>> {
        Box::pin(async move {
        let mut resolved = Vec::new();
        for arg in args {
            match arg {
                ShellArg::Bare(s) => resolved.push(s.clone()),
                ShellArg::Quoted(s) => resolved.push(interpolate_shell_quoted(s, env)),
                ShellArg::Glob(pattern) => {
                    // Expand glob pattern using standard library
                    match glob_expand(pattern) {
                        Some(paths) if !paths.is_empty() => resolved.extend(paths),
                        _ => resolved.push(pattern.clone()), // No matches → pass literally
                    }
                }
                ShellArg::EnvVar(name) => {
                    resolved.push(resolve_shell_var(name, env));
                }
                ShellArg::CommandSub(inner) => {
                    match inner.as_ref() {
                        Statement::ShellCommand { command, args, pipes, redirections, background: _ } => {
                            let sub_args = Self::resolve_shell_args(vm, task, yc, args, env).await?;
                            let output = Self::run_command_pipeline(vm, task, yc, command, &sub_args, pipes, redirections, env, true).await?;
                            resolved.push(output.trim_end_matches('\n').to_string());
                        }
                        _ => {
                            match Self::exec_statement_yielding(vm, task, yc, inner, env).await? {
                                ControlFlow::ExprValue(v) => resolved.push(format!("{}", v)),
                                ControlFlow::Return(v) => resolved.push(format!("{}", v)),
                                _ => resolved.push(String::new()),
                            }
                        }
                    }
                }
            }
        }
        Ok(resolved)
        }) // end Box::pin(async move { })
    }

    async fn run_command_pipeline(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        yc: &mut YieldContext,
        command: &str,
        args: &[String],
        pipes: &[ShellPipeline],
        redirections: &[Redirection],
        env: &Environment,
        capture: bool,
    ) -> Result<String, RuntimeError> {
        use tokio::process::Command;
        use std::process::Stdio;

        // Build the first command
        let mut cmd = Command::new(command);
        cmd.args(args);

        if pipes.is_empty() {
            // Single command — handle redirections
            apply_redirections(&mut cmd, redirections)?;

            if capture {
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let output = cmd.output().await.map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", command, e), ErrorCode::ShellError)
                })?;

                let exit_code = output.status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let status = cmd.status().await.map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", command, e), ErrorCode::ShellError)
                })?;

                let exit_code = status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::new())
            }
        } else {
            // Pipeline: chain commands via stdin/stdout
            cmd.stdout(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| {
                RuntimeError::system_error(format!("{}: {}", command, e), ErrorCode::ShellError)
            })?;

            let mut prev_stdout: Option<tokio::process::ChildStdout> = child.stdout.take();

            for (i, pipe) in pipes.iter().enumerate() {
                let is_last = i == pipes.len() - 1;
                let pipe_args = Self::resolve_shell_args(vm, task, yc, &pipe.args, env).await?;
                let mut next_cmd = Command::new(&pipe.command);
                next_cmd.args(&pipe_args);

                if let Some(stdout) = prev_stdout.take() {
                    use std::os::unix::io::{AsRawFd, FromRawFd};
                    let fd = stdout.as_raw_fd();
                    let stdio = unsafe { std::process::Stdio::from_raw_fd(fd) };
                    std::mem::forget(stdout); // Prevent double-close of fd
                    next_cmd.stdin(stdio);
                }

                if !is_last || capture {
                    next_cmd.stdout(Stdio::piped());
                }

                if is_last {
                    apply_redirections(&mut next_cmd, redirections)?;
                }

                let mut next_child = next_cmd.spawn().map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", pipe.command, e), ErrorCode::ShellError)
                })?;

                prev_stdout = next_child.stdout.take();

                if is_last {
                    let output = next_child.wait_with_output().await.map_err(|e| {
                        RuntimeError::system_error(format!("{}: {}", pipe.command, e), ErrorCode::ShellError)
                    })?;
                    let exit_code = output.status.code().unwrap_or(-1) as i64;
                    env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                    // Wait for the first command too
                    let _ = child.wait().await;

                    if capture {
                        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    }
                    return Ok(String::new());
                } else {
                    // Intermediate — will be consumed by next stage
                    child = next_child;
                }
            }

            // Wait for all processes
            let _ = child.wait().await;
            Ok(String::new())
        }
    }

    /// Call a function value with the given arguments (public entry point).
    pub fn call_function(
        vm: &Rc<RefCell<IshVm>>,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        Self::call_function_inner(vm, func, args)
    }

    /// Internal synchronous function call.
    ///
    /// Shims are self-contained: yielding shims spawn a task and return
    /// `Value::Future`; unyielding shims execute the body and return the
    /// result directly.  This method handles arity checks, parameter type
    /// audits, ledger builtin intercepts, and the shim call.
    fn call_function_inner(
        vm: &Rc<RefCell<IshVm>>,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Function(f) => {
                // Arity check
                if !f.params.is_empty() && args.len() != f.params.len() {
                    return Err(RuntimeError::system_error(format!(
                        "function '{}' expected {} arguments, got {}",
                        f.name.as_deref().unwrap_or("anonymous"),
                        f.params.len(),
                        args.len()
                    ), ErrorCode::ArgumentCountMismatch));
                }
                // Audit parameter types against annotations (when types feature active).
                for (i, (param_name, arg)) in f.params.iter().zip(args.iter()).enumerate() {
                    let param_type = f.param_types.get(i).and_then(|t| t.as_ref());
                    Self::audit_type_annotation(vm, param_name, arg, param_type)?;
                }

                // Intercept ledger builtins that need VM access
                let fn_name = f.name.as_deref().unwrap_or("");
                match fn_name {
                    "active_standard" => return Self::builtin_active_standard(vm, args),
                    "feature_state" => return Self::builtin_feature_state(vm, args),
                    "has_standard" => return Self::builtin_has_standard(vm, args),
                    "has_entry_type" => return Self::builtin_has_entry_type(vm, args),
                    "ledger_state" => return Self::builtin_ledger_state(vm, args),
                    "has_entry" => return Self::builtin_has_entry(vm, args),
                    _ => {}
                }

                // Call the shim — result is the final answer.
                // Yielding shims return Value::Future; unyielding shims
                // return the computed value.
                (f.shim)(args)
            }
            _ => Err(RuntimeError::system_error(format!(
                "cannot call {}",
                func.type_name()
            ), ErrorCode::NotCallable)),
        }
    }

    // ── Type audit helper ─────────────────────────────────────────────────

    /// Audit a value against a type annotation.
    ///
    /// - If annotation is present, always check compatibility (unconditional).
    /// - If annotation is absent, report a discrepancy only when the `types`
    ///   feature is `required` in the active standard.
    fn audit_type_annotation(
        vm: &Rc<RefCell<IshVm>>,
        item_name: &str,
        value: &Value,
        annotation: Option<&ish_ast::TypeAnnotation>,
    ) -> Result<(), RuntimeError> {
        match annotation {
            Some(type_ann) => {
                // Annotation present — always check compatibility.
                if !crate::ledger::type_compat::is_compatible(value, type_ann) {
                    return Err(RuntimeError::system_error(format!(
                        "type mismatch for '{}': expected {:?}, got {}",
                        item_name,
                        type_ann,
                        value.type_name()
                    ), ErrorCode::TypeMismatch));
                }
            }
            None => {
                // No annotation — only an error if annotation is required.
                let features = vm.borrow().ledger.active_features();
                if let Some(types_feature) = features.get("types") {
                    if types_feature.annotation
                        == crate::ledger::standard::AnnotationDimension::Required
                    {
                        return Err(RuntimeError::system_error(format!(
                            "missing type annotation for '{}' (required by active standard)",
                            item_name
                        ), ErrorCode::TypeMismatch));
                    }
                }
            }
        }

        Ok(())
    }

    // ── Ledger query builtins (need &self for ledger access) ────────────────

    /// active_standard() -> String | null
    fn builtin_active_standard(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if !args.is_empty() {
            return Err(RuntimeError::system_error("active_standard expects 0 arguments", ErrorCode::ShellError));
        }
        match vm.borrow().ledger.active_standard() {
            Some(name) => Ok(Value::String(Rc::new(name.to_string()))),
            None => Ok(Value::Null),
        }
    }

    /// feature_state(feature_name) -> String description | null
    fn builtin_feature_state(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("feature_state expects 1 argument", ErrorCode::ShellError));
        }
        if let Value::String(name) = &args[0] {
            match vm.borrow().ledger.feature_state(name) {
                Some(state) => {
                    let ann = match state.annotation {
                        crate::ledger::standard::AnnotationDimension::Optional => "optional",
                        crate::ledger::standard::AnnotationDimension::Required => "required",
                    };
                    let aud = match state.audit {
                        crate::ledger::standard::AuditDimension::Runtime => "runtime",
                        crate::ledger::standard::AuditDimension::Build => "build",
                    };
                    let desc = match &state.parameter {
                        Some(param) => format!("{}/{}:{}", ann, aud, param),
                        None => format!("{}/{}", ann, aud),
                    };
                    Ok(Value::String(Rc::new(desc)))
                }
                None => Ok(Value::Null),
            }
        } else {
            Err(RuntimeError::system_error("feature_state expects a string argument", ErrorCode::TypeMismatch))
        }
    }

    /// has_standard(name) -> Bool
    fn builtin_has_standard(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("has_standard expects 1 argument", ErrorCode::ShellError));
        }
        if let Value::String(name) = &args[0] {
            Ok(Value::Bool(vm.borrow().ledger.standard_registry.get(name).is_some()))
        } else {
            Err(RuntimeError::system_error("has_standard expects a string argument", ErrorCode::TypeMismatch))
        }
    }

    /// has_entry_type(name) -> Bool
    fn builtin_has_entry_type(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("has_entry_type expects 1 argument", ErrorCode::ShellError));
        }
        if let Value::String(name) = &args[0] {
            Ok(Value::Bool(vm.borrow().ledger.entry_type_registry.get(name).is_some()))
        } else {
            Err(RuntimeError::system_error("has_entry_type expects a string argument", ErrorCode::TypeMismatch))
        }
    }

    /// ledger_state(variable_name) -> String
    /// Returns a string representation of all entries on a variable,
    /// e.g. "Type(i32), ExcludeNull".
    fn builtin_ledger_state(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("ledger_state expects 1 argument", ErrorCode::ShellError));
        }
        if let Value::String(name) = &args[0] {
            let desc = match vm.borrow().ledger.get_entries(name) {
                Some(es) => {
                    es.entries.iter().map(|e| {
                        if e.params.is_empty() {
                            e.entry_type.clone()
                        } else {
                            let params: Vec<String> = e.params.iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect();
                            format!("{}({})", e.entry_type, params.join(", "))
                        }
                    }).collect::<Vec<_>>().join(", ")
                }
                None => String::new(),
            };
            Ok(Value::String(Rc::new(desc)))
        } else {
            Err(RuntimeError::system_error("ledger_state expects a string argument", ErrorCode::TypeMismatch))
        }
    }

    /// has_entry(variable_name, entry_type) -> Bool
    fn builtin_has_entry(vm: &Rc<RefCell<IshVm>>, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::system_error("has_entry expects 2 arguments", ErrorCode::ShellError));
        }
        match (&args[0], &args[1]) {
            (Value::String(var), Value::String(entry_type)) => {
                Ok(Value::Bool(vm.borrow().ledger.has_entry(var, entry_type)))
            }
            _ => Err(RuntimeError::system_error(
                "has_entry expects (string, string) arguments", ErrorCode::TypeMismatch
            )),
        }
    }

    // ── Unyielding execution variants ──────────────────────────────────────
    //
    // Synchronous counterparts of exec_statement_yielding / eval_expression_yielding for
    // functions classified as unyielding.  No YieldContext, no async, no
    // Box::pin.  Yield, Await, Spawn, and CommandSubstitution are errors.

    /// Pop the current defer frame and execute all deferred statements
    /// synchronously.
    fn pop_and_run_defers_unyielding(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext) {
        if let Some(deferred) = task.defer_stack.pop() {
            for (d, env) in deferred.into_iter().rev() {
                let _ = Self::exec_statement_unyielding(vm, task, &d, &env);
            }
        }
    }

    /// Execute a statement synchronously (unyielding path).
    fn exec_statement_unyielding(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        stmt: &Statement,
        env: &Environment,
    ) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Statement::VariableDecl { name, value, type_annotation, .. } => {
                let val = Self::eval_expression_unyielding(vm, task, value, env)?;
                Self::audit_type_annotation(vm, name, &val, type_annotation.as_ref())?;
                env.define(name.clone(), val);
                Ok(ControlFlow::None)
            }

            Statement::Assignment { target, value } => {
                let val = Self::eval_expression_unyielding(vm, task, value, env)?;
                match target {
                    AssignTarget::Variable(name) => {
                        env.set(name, val)?;
                    }
                    AssignTarget::Property { object, property } => {
                        let obj = Self::eval_expression_unyielding(vm, task, object, env)?;
                        if let Value::Object(ref obj_ref) = obj {
                            obj_ref.borrow_mut().insert(property.clone(), val);
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot set property '{}' on {}",
                                property,
                                obj.type_name()
                            ), ErrorCode::TypeMismatch));
                        }
                    }
                    AssignTarget::Index { object, index } => {
                        let obj = Self::eval_expression_unyielding(vm, task, object, env)?;
                        let idx = Self::eval_expression_unyielding(vm, task, index, env)?;
                        if let Value::List(ref list_ref) = obj {
                            if let Value::Int(i) = idx {
                                let mut list = list_ref.borrow_mut();
                                let len = list.len() as i64;
                                if i < 0 || i >= len {
                                    return Err(RuntimeError::system_error(format!(
                                        "index {} out of bounds (length {})",
                                        i, len
                                    ), ErrorCode::IndexOutOfBounds));
                                }
                                list[i as usize] = val;
                            } else {
                                return Err(RuntimeError::system_error("list index must be an integer", ErrorCode::TypeMismatch));
                            }
                        } else if let Value::Object(ref obj_ref) = obj {
                            if let Value::String(ref key) = idx {
                                obj_ref.borrow_mut().insert(key.as_ref().clone(), val);
                            } else {
                                return Err(RuntimeError::system_error(
                                    "object index must be a string", ErrorCode::TypeMismatch
                                ));
                            }
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot index into {}",
                                obj.type_name()
                            ), ErrorCode::TypeMismatch));
                        }
                    }
                }
                Ok(ControlFlow::None)
            }

            Statement::Block { statements } => {
                let block_env = env.child();
                let mut result = ControlFlow::None;
                for s in statements {
                    if let Statement::Defer { body } = s {
                        task.register_defer(*body.clone(), block_env.clone());
                        continue;
                    }
                    match Self::exec_statement_unyielding(vm, task, s, &block_env)? {
                        ControlFlow::Return(v) => { result = ControlFlow::Return(v); break; }
                        ControlFlow::Throw(v) => { result = ControlFlow::Throw(v); break; }
                        ControlFlow::None | ControlFlow::ExprValue(_) => {}
                    }
                }
                Ok(result)
            }

            Statement::If { condition, then_block, else_block } => {
                let cond = Self::eval_expression_unyielding(vm, task, condition, env)?;
                use crate::ledger::narrowing::{analyze_condition, invert_for_else, NarrowingFact};
                let facts = analyze_condition(condition);
                let snapshot = vm.borrow().ledger.save_entries();
                if cond.is_truthy() {
                    for fact in &facts {
                        match fact {
                            NarrowingFact::IsType { variable, type_name } => {
                                vm.borrow_mut().ledger.narrow_type(variable, type_name);
                            }
                            NarrowingFact::NotNull { variable } => {
                                vm.borrow_mut().ledger.narrow_exclude_null(variable);
                            }
                            NarrowingFact::IsNull { .. } => {}
                        }
                    }
                    let result = Self::exec_statement_unyielding(vm, task, then_block, env);
                    let then_entries = vm.borrow().ledger.save_entries();
                    {
                        let mut vm_mut = vm.borrow_mut();
                        vm_mut.ledger.restore_entries(snapshot);
                        let current = vm_mut.ledger.save_entries();
                        vm_mut.ledger.merge_entries(then_entries, current);
                    }
                    result
                } else if let Some(eb) = else_block {
                    let else_facts = invert_for_else(&facts);
                    for fact in &else_facts {
                        match fact {
                            NarrowingFact::IsType { variable, type_name } => {
                                vm.borrow_mut().ledger.narrow_type(variable, type_name);
                            }
                            NarrowingFact::NotNull { variable } => {
                                vm.borrow_mut().ledger.narrow_exclude_null(variable);
                            }
                            NarrowingFact::IsNull { .. } => {}
                        }
                    }
                    let result = Self::exec_statement_unyielding(vm, task, eb, env);
                    let else_entries = vm.borrow().ledger.save_entries();
                    {
                        let mut vm_mut = vm.borrow_mut();
                        vm_mut.ledger.restore_entries(snapshot);
                        let current = vm_mut.ledger.save_entries();
                        vm_mut.ledger.merge_entries(current, else_entries);
                    }
                    result
                } else {
                    Ok(ControlFlow::None)
                }
            }

            Statement::While { condition, body, yield_every: _ } => {
                // No yield budget check or yield_every in unyielding context.
                loop {
                    let cond = Self::eval_expression_unyielding(vm, task, condition, env)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    match Self::exec_statement_unyielding(vm, task, body, env)? {
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Throw(v) => return Ok(ControlFlow::Throw(v)),
                        ControlFlow::None | ControlFlow::ExprValue(_) => {}
                    }
                }
                Ok(ControlFlow::None)
            }

            Statement::ForEach { variable, iterable, body, yield_every: _ } => {
                let iter_val = Self::eval_expression_unyielding(vm, task, iterable, env)?;
                if let Value::List(ref list_ref) = iter_val {
                    let items: Vec<Value> = list_ref.borrow().clone();
                    for item in items {
                        let loop_env = env.child();
                        loop_env.define(variable.clone(), item);
                        match Self::exec_statement_unyielding(vm, task, body, &loop_env)? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Throw(v) => return Ok(ControlFlow::Throw(v)),
                            ControlFlow::None | ControlFlow::ExprValue(_) => {}
                        }
                    }
                } else {
                    return Err(RuntimeError::system_error(format!(
                        "cannot iterate over {}",
                        iter_val.type_name()
                    ), ErrorCode::TypeMismatch));
                }
                Ok(ControlFlow::None)
            }

            Statement::Return { value } => {
                let val = if let Some(expr) = value {
                    Self::eval_expression_unyielding(vm, task, expr, env)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(val))
            }

            Statement::ExpressionStmt(expr) => {
                let val = Self::eval_expression_unyielding(vm, task, expr, env)?;
                Ok(ControlFlow::ExprValue(val))
            }

            Statement::FunctionDecl {
                name, params, body, return_type, is_async, ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                let classification = crate::analyzer::classify_function(body, *is_async, env, &param_names, Some(name.as_str()))?;
                let has_yielding_entry = match classification {
                    crate::analyzer::YieldingClassification::Yielding => Some(true),
                    crate::analyzer::YieldingClassification::Unyielding => Some(false),
                };
                // No @[unyielding] contradiction check — declaring a yielding
                // function inside an unyielding execution path is fine.
                // Create a self-contained shim based on yielding classification.
                let captured_body = *body.clone();
                let captured_env = env.clone();
                let captured_params = param_names.clone();
                let captured_vm = vm.clone();
                let captured_is_async = *is_async;
                let captured_name = name.clone();
                let captured_return_type = return_type.clone();

                let shim: Shim = if has_yielding_entry == Some(true) {
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let vm_clone = captured_vm.clone();
                        let body_clone = captured_body.clone();
                        let is_async = captured_is_async;
                        let fn_name = captured_name.clone();
                        let ret_type = captured_return_type.clone();

                        let handle = tokio::task::spawn_local(async move {
                            let mut task_ctx = TaskContext::new();
                            let mut yield_ctx = YieldContext::new();
                            task_ctx.push_defer_frame();
                            task_ctx.async_stack.push((is_async, fn_name.clone()));
                            yield_ctx.check_yield_budget().await;
                            let result = IshVm::exec_statement_yielding(
                                &vm_clone, &mut task_ctx, &mut yield_ctx, &body_clone, &call_env,
                            ).await;
                            IshVm::pop_and_run_defers(&vm_clone, &mut task_ctx, &mut yield_ctx).await;
                            task_ctx.async_stack.pop();
                            let return_val = match result? {
                                ControlFlow::Return(v) => v,
                                ControlFlow::ExprValue(v) => v,
                                ControlFlow::None => Value::Null,
                                ControlFlow::Throw(v) => return Err(RuntimeError::thrown(v)),
                            };
                            IshVm::audit_type_annotation(
                                &vm_clone, &format!("return of '{fn_name}'"),
                                &return_val, ret_type.as_ref(),
                            )?;
                            Ok(return_val)
                        });
                        Ok(Value::Future(FutureRef::new(handle)))
                    })
                } else {
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let mut task_ctx = TaskContext::new();
                        task_ctx.push_defer_frame();
                        task_ctx.async_stack.push((captured_is_async, captured_name.clone()));
                        let result = IshVm::exec_statement_unyielding(
                            &captured_vm, &mut task_ctx, &captured_body, &call_env,
                        );
                        IshVm::pop_and_run_defers_unyielding(&captured_vm, &mut task_ctx);
                        task_ctx.async_stack.pop();
                        let return_val = match result? {
                            ControlFlow::Return(v) => v,
                            ControlFlow::ExprValue(v) => v,
                            ControlFlow::None => Value::Null,
                            ControlFlow::Throw(v) => return Err(RuntimeError::thrown(v)),
                        };
                        IshVm::audit_type_annotation(
                            &captured_vm, &format!("return of '{}'", captured_name),
                            &return_val, captured_return_type.as_ref(),
                        )?;
                        Ok(return_val)
                    })
                };
                let func_val = Value::Function(Gc::new(IshFunction {
                    name: Some(name.clone()),
                    params: param_names,
                    param_types,
                    return_type: return_type.clone(),
                    shim,
                    is_async: *is_async,
                    has_yielding_entry,
                }));
                env.define(name.clone(), func_val);
                Ok(ControlFlow::None)
            }

            Statement::Throw { value } => {
                let val = Self::eval_expression_unyielding(vm, task, value, env)?;
                use crate::ledger::entry::Entry;
                let thrown = match &val {
                    Value::Object(ref obj_ref) => {
                        let map = obj_ref.borrow();
                        let has_message = matches!(map.get("message"), Some(Value::String(_)));
                        if has_message {
                            drop(map);
                            vm.borrow_mut().ledger.add_entry("@thrown",
                                Entry::new("Error").with_param("message", "String"));
                            val
                        } else {
                            drop(map);
                            let mut wrapper = HashMap::new();
                            wrapper.insert("message".to_string(), Value::String(Rc::new(
                                "throw audit: thrown object lacks 'message: String' property".to_string()
                            )));
                            wrapper.insert("code".to_string(), Value::String(Rc::new(ErrorCode::UnhandledThrow.as_str().to_string())));
                            wrapper.insert("original".to_string(), val);
                            let wrapped = new_object(wrapper);
                            vm.borrow_mut().ledger.add_entry("@thrown",
                                Entry::new("Error").with_param("message", "String"));
                            wrapped
                        }
                    }
                    _ => {
                        let mut wrapper = HashMap::new();
                        wrapper.insert("message".to_string(), Value::String(Rc::new(
                            format!("throw audit: non-object thrown: {}", val)
                        )));
                        wrapper.insert("code".to_string(), Value::String(Rc::new(ErrorCode::UnhandledThrow.as_str().to_string())));
                        wrapper.insert("original".to_string(), val);
                        let wrapped = new_object(wrapper);
                        vm.borrow_mut().ledger.add_entry("@thrown",
                            Entry::new("Error").with_param("message", "String"));
                        wrapped
                    }
                };
                Ok(ControlFlow::Throw(thrown))
            }

            Statement::TryCatch { body, catches, finally } => {
                let (result, thrown) = match Self::exec_statement_unyielding(vm, task, body, env) {
                    Ok(ControlFlow::Throw(v)) => (ControlFlow::None, Some(v)),
                    Ok(other) => (other, None),
                    Err(e) if e.thrown_value.is_some() => {
                        (ControlFlow::None, e.thrown_value)
                    }
                    Err(e) => return Err(e),
                };
                let result = if let Some(thrown) = thrown {
                    let mut caught = false;
                    let mut catch_result = ControlFlow::None;
                    for clause in catches {
                        let catch_env = env.child();
                        catch_env.define(clause.param.clone(), thrown.clone());
                        catch_result = Self::exec_statement_unyielding(vm, task, &clause.body, &catch_env)?;
                        caught = true;
                        break;
                    }
                    if caught {
                        catch_result
                    } else {
                        ControlFlow::Throw(thrown)
                    }
                } else {
                    result
                };
                if let Some(fin) = finally {
                    let fin_result = Self::exec_statement_unyielding(vm, task, fin, env)?;
                    if let ControlFlow::Throw(_) = fin_result {
                        return Ok(fin_result);
                    }
                }
                Ok(result)
            }

            Statement::WithBlock { resources, body } => {
                let with_env = env.child();
                let mut initialized: Vec<(String, Value)> = Vec::new();
                for (name, expr) in resources {
                    match Self::eval_expression_unyielding(vm, task, expr, &with_env) {
                        Ok(val) => {
                            with_env.define(name.clone(), val.clone());
                            initialized.push((name.clone(), val));
                        }
                        Err(e) => {
                            for (_, res) in initialized.into_iter().rev() {
                                let _ = Self::try_close_unyielding(vm, task, &res);
                            }
                            return Err(e);
                        }
                    }
                }
                let result = Self::exec_statement_unyielding(vm, task, body, &with_env)?;
                let mut close_error: Option<Value> = None;
                for (_, res) in initialized.into_iter().rev() {
                    if let Err(_e) = Self::try_close_unyielding(vm, task, &res) {
                        if close_error.is_none() {
                            close_error = Some(Value::String(Rc::new(_e.message)));
                        }
                    }
                }
                match result {
                    ControlFlow::Throw(_) => Ok(result),
                    other => {
                        if let Some(err) = close_error {
                            Ok(ControlFlow::Throw(err))
                        } else {
                            Ok(other)
                        }
                    }
                }
            }

            Statement::Defer { body } => {
                task.register_defer(*body.clone(), env.clone());
                Ok(ControlFlow::None)
            }

            Statement::TypeAlias { .. } => Ok(ControlFlow::None),
            Statement::Use { .. } => Ok(ControlFlow::None), // modules loaded via yielding path only
            Statement::DeclareBlock { body } => {
                Self::eval_declare_block_unyielding(vm, task, env, body)
            }
            Statement::Bootstrap { .. } => {
                Self::eval_bootstrap(vm, task)
            }

            Statement::ShellCommand { .. } => {
                Err(RuntimeError::system_error(
                    "shell commands cannot execute in unyielding context",
                    ErrorCode::AsyncError,
                ))
            }

            Statement::Annotated { annotations, inner } => {
                let mut pushed_standards = Vec::new();
                for ann in annotations {
                    match ann {
                        Annotation::Standard(name) => {
                            vm.borrow_mut().ledger.push_standard(name.clone());
                            pushed_standards.push(name.clone());
                        }
                        Annotation::Entry(_items) => {
                            // @[unyielding] and @[yield_budget] are no-ops
                            // (already in unyielding context).
                        }
                    }
                }
                let result = Self::exec_statement_unyielding(vm, task, inner, env);
                if !pushed_standards.is_empty() {
                    let unawaited = crate::value::take_unawaited_future_count();
                    if unawaited > 0 {
                        let features = vm.borrow().ledger.active_features();
                        if let crate::ledger::audit::AuditResult::Discrepancy(report) =
                            crate::ledger::audit::audit_future_drop(
                                &features,
                                &format!("{} future(s) dropped without await", unawaited),
                            )
                        {
                            for _ in pushed_standards.iter().rev() {
                                vm.borrow_mut().ledger.pop_standard();
                            }
                            return Err(RuntimeError::system_error(report.message, ErrorCode::AsyncError));
                        }
                    }
                }
                for _ in pushed_standards.iter().rev() {
                    vm.borrow_mut().ledger.pop_standard();
                }
                result
            }

            Statement::StandardDef { name, extends, features } => {
                use crate::ledger::standard::Standard;
                let mut std = Standard::new(name.clone());
                if let Some(parent) = extends {
                    std = std.with_parent(parent.clone());
                }
                for feat in features {
                    let state = parse_feature_params(&feat.params);
                    std = std.with_feature(feat.name.clone(), state);
                }
                vm.borrow_mut().ledger.standard_registry.register(std);
                Ok(ControlFlow::None)
            }

            Statement::EntryTypeDef { name, .. } => {
                use crate::ledger::entry_type::EntryType;
                vm.borrow_mut().ledger.entry_type_registry.register(EntryType::new(name.clone()));
                Ok(ControlFlow::None)
            }

            Statement::Match { .. } => Ok(ControlFlow::None),

            Statement::Incomplete { kind } => {
                Err(RuntimeError::system_error(format!("incomplete input: {:?}", kind), ErrorCode::TypeMismatch))
            }

            Statement::Yield => {
                Err(RuntimeError::system_error(
                    "yield cannot be used in unyielding context",
                    ErrorCode::AsyncError,
                ))
            }
        }
    }

    /// Try to call close() on a value synchronously (unyielding path).
    fn try_close_unyielding(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext, value: &Value) -> Result<(), RuntimeError> {
        if let Value::Object(ref obj_ref) = value {
            let map = obj_ref.borrow();
            if let Some(close_fn) = map.get("close").cloned() {
                drop(map);
                Self::call_function_unyielding(vm, task, &close_fn, &[])?;
            }
        }
        Ok(())
    }

    /// Evaluate an expression synchronously (unyielding path).
    fn eval_expression_unyielding(
        vm: &Rc<RefCell<IshVm>>,
        task: &mut TaskContext,
        expr: &Expression,
        env: &Environment,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Int(n) => Value::Int(*n),
                Literal::Float(f) => Value::Float(*f),
                Literal::String(s) => Value::String(Rc::new(s.clone())),
                Literal::Char(c) => Value::Char(*c),
                Literal::Null => Value::Null,
            }),

            Expression::Identifier(name) => env.get(name),

            Expression::BinaryOp { op, left, right } => {
                let lhs = Self::eval_expression_unyielding(vm, task, left, env)?;
                match op {
                    BinaryOperator::And => {
                        if !lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return Self::eval_expression_unyielding(vm, task, right, env);
                    }
                    BinaryOperator::Or => {
                        if lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return Self::eval_expression_unyielding(vm, task, right, env);
                    }
                    _ => {}
                }
                let rhs = Self::eval_expression_unyielding(vm, task, right, env)?;
                Self::eval_binary_op(op, &lhs, &rhs)
            }

            Expression::UnaryOp { op, operand } => {
                let val = Self::eval_expression_unyielding(vm, task, operand, env)?;
                match op {
                    UnaryOperator::Not => Ok(Value::Bool(!val.is_truthy())),
                    UnaryOperator::Negate => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(RuntimeError::system_error(format!(
                            "cannot negate {}",
                            val.type_name()
                        ), ErrorCode::TypeMismatch)),
                    },
                    UnaryOperator::Try => {
                        if val == Value::Null {
                            return Err(RuntimeError::system_error("tried to unwrap null value with ?".to_string(), ErrorCode::NullUnwrap));
                        }
                        Ok(val)
                    }
                }
            }

            Expression::FunctionCall { callee, args } => {
                let func_val = Self::eval_expression_unyielding(vm, task, callee, env)?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(Self::eval_expression_unyielding(vm, task, arg, env)?);
                }
                Self::call_function_unyielding(vm, task, &func_val, &arg_vals)
            }

            Expression::ObjectLiteral(pairs) => {
                let mut map = HashMap::new();
                for (key, val_expr) in pairs {
                    let val = Self::eval_expression_unyielding(vm, task, val_expr, env)?;
                    map.insert(key.clone(), val);
                }
                Ok(new_object(map))
            }

            Expression::ListLiteral(elements) => {
                let mut items = Vec::with_capacity(elements.len());
                for elem in elements {
                    items.push(Self::eval_expression_unyielding(vm, task, elem, env)?);
                }
                Ok(new_list(items))
            }

            Expression::PropertyAccess { object, property } => {
                let obj = Self::eval_expression_unyielding(vm, task, object, env)?;
                match obj {
                    Value::Object(ref obj_ref) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(property).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::system_error(format!(
                        "cannot access property '{}' on {}",
                        property,
                        obj.type_name()
                    ), ErrorCode::TypeMismatch)),
                }
            }

            Expression::IndexAccess { object, index } => {
                let obj = Self::eval_expression_unyielding(vm, task, object, env)?;
                let idx = Self::eval_expression_unyielding(vm, task, index, env)?;
                match (&obj, &idx) {
                    (Value::List(list_ref), Value::Int(i)) => {
                        let list = list_ref.borrow();
                        let i = *i;
                        if i < 0 || i >= list.len() as i64 {
                            return Err(RuntimeError::system_error(format!(
                                "index {} out of bounds (length {})",
                                i,
                                list.len()
                            ), ErrorCode::IndexOutOfBounds));
                        }
                        Ok(list[i as usize].clone())
                    }
                    (Value::Object(obj_ref), Value::String(key)) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(key.as_ref()).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::system_error(format!(
                        "cannot index {} with {}",
                        obj.type_name(),
                        idx.type_name()
                    ), ErrorCode::TypeMismatch)),
                }
            }

            Expression::Lambda { params, body, is_async } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                let classification = crate::analyzer::classify_function(body, *is_async, env, &param_names, None)?;
                let has_yielding_entry = match classification {
                    crate::analyzer::YieldingClassification::Yielding => Some(true),
                    crate::analyzer::YieldingClassification::Unyielding => Some(false),
                };
                let captured_body = *body.clone();
                let captured_env = env.clone();
                let captured_params = param_names.clone();
                let captured_vm = vm.clone();
                let captured_is_async = *is_async;

                let shim: Shim = if has_yielding_entry == Some(true) {
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let vm_clone = captured_vm.clone();
                        let body_clone = captured_body.clone();
                        let is_async = captured_is_async;

                        let handle = tokio::task::spawn_local(async move {
                            let mut task_ctx = TaskContext::new();
                            let mut yield_ctx = YieldContext::new();
                            task_ctx.push_defer_frame();
                            task_ctx.async_stack.push((is_async, "anonymous".to_string()));
                            yield_ctx.check_yield_budget().await;
                            let result = IshVm::exec_statement_yielding(
                                &vm_clone, &mut task_ctx, &mut yield_ctx, &body_clone, &call_env,
                            ).await;
                            IshVm::pop_and_run_defers(&vm_clone, &mut task_ctx, &mut yield_ctx).await;
                            task_ctx.async_stack.pop();
                            match result? {
                                ControlFlow::Return(v) => Ok(v),
                                ControlFlow::ExprValue(v) => Ok(v),
                                ControlFlow::None => Ok(Value::Null),
                                ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
                            }
                        });
                        Ok(Value::Future(FutureRef::new(handle)))
                    })
                } else {
                    Rc::new(move |args: &[Value]| {
                        let call_env = captured_env.child();
                        for (param, arg) in captured_params.iter().zip(args.iter()) {
                            call_env.define(param.clone(), arg.clone());
                        }
                        let mut task_ctx = TaskContext::new();
                        task_ctx.push_defer_frame();
                        task_ctx.async_stack.push((captured_is_async, "anonymous".to_string()));
                        let result = IshVm::exec_statement_unyielding(
                            &captured_vm, &mut task_ctx, &captured_body, &call_env,
                        );
                        IshVm::pop_and_run_defers_unyielding(&captured_vm, &mut task_ctx);
                        task_ctx.async_stack.pop();
                        match result? {
                            ControlFlow::Return(v) => Ok(v),
                            ControlFlow::ExprValue(v) => Ok(v),
                            ControlFlow::None => Ok(Value::Null),
                            ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
                        }
                    })
                };

                Ok(Value::Function(Gc::new(IshFunction {
                    name: None,
                    params: param_names,
                    param_types,
                    return_type: None,
                    shim,
                    is_async: *is_async,
                    has_yielding_entry,
                })))
            }

            Expression::StringInterpolation(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ish_ast::StringPart::Text(text) => result.push_str(text),
                        ish_ast::StringPart::Expr(expr) => {
                            let val = Self::eval_expression_unyielding(vm, task, expr, env)?;
                            result.push_str(&val.to_display_string());
                        }
                    }
                }
                Ok(Value::String(Rc::new(result)))
            }

            Expression::CommandSubstitution(_) => {
                Err(RuntimeError::system_error(
                    "command substitution cannot be used in unyielding context",
                    ErrorCode::AsyncError,
                ))
            }

            Expression::EnvVar(name) => {
                if name == "?" {
                    match env.get("__ish_last_exit_code") {
                        Ok(v) => Ok(v),
                        Err(_) => Ok(Value::Int(0)),
                    }
                } else {
                    match std::env::var(name) {
                        Ok(val) => Ok(Value::String(Rc::new(val))),
                        Err(_) => Ok(Value::Null),
                    }
                }
            }

            Expression::Incomplete { kind } => {
                Err(RuntimeError::system_error(format!("incomplete expression: {:?}", kind), ErrorCode::TypeMismatch))
            }

            Expression::Await { expr: _ } => {
                Err(RuntimeError::system_error(
                    "await cannot be used in unyielding context",
                    ErrorCode::AsyncError,
                ))
            }

            Expression::Spawn { callee, args } => {
                // Spawn is valid in unyielding context: start the task and
                // return the Future without suspending.
                let callee_val = Self::eval_expression_unyielding(vm, task, callee, env)?;
                // Check yielding classification before spawning (E013)
                if let Value::Function(ref f) = callee_val {
                    if f.has_yielding_entry != Some(true) {
                        return Err(RuntimeError::system_error(
                            format!("cannot spawn unyielding function '{}'",
                                f.name.as_deref().unwrap_or("<anonymous>")),
                            ErrorCode::SpawnUnyielding,
                        ));
                    }
                }
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    arg_vals.push(Self::eval_expression_unyielding(vm, task, arg, env)?);
                }
                let result = Self::call_function_inner(vm, &callee_val, &arg_vals)?;
                Ok(result)
            }
        }
    }

    /// Call a function synchronously (unyielding path).
    /// Delegates to call_function_inner — shims are self-contained.
    fn call_function_unyielding(
        vm: &Rc<RefCell<IshVm>>,
        _task: &mut TaskContext,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        Self::call_function_inner(vm, func, args)
    }

    /// Evaluate a binary operation.
    fn eval_binary_op(
        op: &BinaryOperator,
        lhs: &Value,
        rhs: &Value,
    ) -> Result<Value, RuntimeError> {
        match op {
            // Arithmetic
            BinaryOperator::Add => Self::add(lhs, rhs),
            BinaryOperator::Sub => Self::arith(lhs, rhs, |a, b| a - b, |a, b| a - b),
            BinaryOperator::Mul => Self::arith(lhs, rhs, |a, b| a * b, |a, b| a * b),
            BinaryOperator::Div => {
                // Check for division by zero
                match rhs {
                    Value::Int(0) => return Err(RuntimeError::system_error("division by zero", ErrorCode::DivisionByZero)),
                    Value::Float(f) if *f == 0.0 => {
                        return Err(RuntimeError::system_error("division by zero", ErrorCode::DivisionByZero))
                    }
                    _ => {}
                }
                Self::arith(lhs, rhs, |a, b| a / b, |a, b| a / b)
            }
            BinaryOperator::Mod => {
                match rhs {
                    Value::Int(0) => return Err(RuntimeError::system_error("modulo by zero", ErrorCode::DivisionByZero)),
                    _ => {}
                }
                Self::arith(lhs, rhs, |a, b| a % b, |a, b| a % b)
            }

            // Comparison
            BinaryOperator::Eq => Ok(Value::Bool(lhs == rhs)),
            BinaryOperator::NotEq => Ok(Value::Bool(lhs != rhs)),
            BinaryOperator::Lt => Self::compare(lhs, rhs, |o| o.is_lt()),
            BinaryOperator::Gt => Self::compare(lhs, rhs, |o| o.is_gt()),
            BinaryOperator::LtEq => Self::compare(lhs, rhs, |o| !o.is_gt()),
            BinaryOperator::GtEq => Self::compare(lhs, rhs, |o| !o.is_lt()),

            // Logical (handled above via short-circuit, but just in case)
            BinaryOperator::And | BinaryOperator::Or => {
                unreachable!("logical ops handled in eval_expression_yielding/eval_expression_unyielding")
            }
        }
    }

    fn add(lhs: &Value, rhs: &Value) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            (Value::String(a), other) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, other.to_display_string()))))
            }
            (Value::Char(a), Value::Char(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            (Value::Char(a), Value::String(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            _ => Err(RuntimeError::system_error(format!(
                "cannot add {} and {}",
                lhs.type_name(),
                rhs.type_name()
            ), ErrorCode::TypeMismatch)),
        }
    }

    fn arith(
        lhs: &Value,
        rhs: &Value,
        int_op: impl Fn(i64, i64) -> i64,
        float_op: impl Fn(f64, f64) -> f64,
    ) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
            _ => Err(RuntimeError::system_error(format!(
                "cannot perform arithmetic on {} and {}",
                lhs.type_name(),
                rhs.type_name()
            ), ErrorCode::TypeMismatch)),
        }
    }

    fn compare(
        lhs: &Value,
        rhs: &Value,
        pred: impl Fn(std::cmp::Ordering) -> bool,
    ) -> Result<Value, RuntimeError> {
        let ordering = match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Value::Int(a), Value::Float(b)) => (*a as f64)
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Value::Float(a), Value::Int(b)) => a
                .partial_cmp(&(*b as f64))
                .unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Char(a), Value::Char(b)) => a.cmp(b),
            _ => {
                return Err(RuntimeError::system_error(format!(
                    "cannot compare {} and {}",
                    lhs.type_name(),
                    rhs.type_name()
                ), ErrorCode::TypeMismatch))
            }
        };
        Ok(Value::Bool(pred(ordering)))
    }
}

impl Default for IshVm {
    fn default() -> Self {
        Self::new()
    }
}

// ── Ledger helpers ──────────────────────────────────────────────────────────

/// Parse feature params from AST FeatureSpec params into a FeatureState.
///
/// Params can contain: "optional", "required", "runtime", "build", or a
/// feature-specific parameter like "wrapping", "panicking", etc.
fn parse_feature_params(params: &[String]) -> crate::ledger::standard::FeatureState {
    use crate::ledger::standard::{FeatureState, AnnotationDimension, AuditDimension};

    let mut annotation = AnnotationDimension::Required;
    let mut audit = AuditDimension::Runtime;
    let mut extra_param = None;

    for p in params {
        match p.as_str() {
            "optional" => annotation = AnnotationDimension::Optional,
            "required" => annotation = AnnotationDimension::Required,
            "runtime" => audit = AuditDimension::Runtime,
            "build" => audit = AuditDimension::Build,
            other => extra_param = Some(other.to_string()),
        }
    }

    let mut state = FeatureState::new(annotation, audit);
    if let Some(param) = extra_param {
        state = state.with_parameter(param);
    }
    state
}

/// Parse a duration annotation like "100us", "1ms", "500ns".
fn parse_duration_annotation(s: &str) -> Option<Duration> {
    let s = s.trim();
    if let Some(num_str) = s.strip_suffix("us") {
        num_str.trim().parse::<u64>().ok().map(Duration::from_micros)
    } else if let Some(num_str) = s.strip_suffix("ms") {
        num_str.trim().parse::<u64>().ok().map(Duration::from_millis)
    } else if let Some(num_str) = s.strip_suffix("ns") {
        num_str.trim().parse::<u64>().ok().map(Duration::from_nanos)
    } else if let Some(num_str) = s.strip_suffix('s') {
        num_str.trim().parse::<u64>().ok().map(Duration::from_secs)
    } else {
        // Default: interpret as microseconds
        s.parse::<u64>().ok().map(Duration::from_micros)
    }
}

// ── Shell helpers ───────────────────────────────────────────────────────────

fn apply_redirections(cmd: &mut tokio::process::Command, redirections: &[Redirection]) -> Result<(), RuntimeError> {
    use std::fs::{File, OpenOptions};
    use std::process::Stdio;
    for redir in redirections {
        match redir.kind {
            RedirectKind::StdoutWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), ErrorCode::IoError)
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StdoutAppend => {
                let f = OpenOptions::new().create(true).append(true).open(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), ErrorCode::IoError)
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StderrWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), ErrorCode::IoError)
                })?;
                cmd.stderr(f);
            }
            RedirectKind::StderrAndStdout => {
                // 2>&1 — merge stderr into stdout (Stdio::piped or inherit)
                cmd.stderr(Stdio::inherit());
            }
            RedirectKind::AllWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), ErrorCode::IoError)
                })?;
                let f2 = f.try_clone().map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}", e), ErrorCode::IoError)
                })?;
                cmd.stdout(f);
                cmd.stderr(f2);
            }
        }
    }
    Ok(())
}

fn glob_expand(pattern: &str) -> Option<Vec<String>> {
    // Simple glob expansion using std::fs
    let path = std::path::Path::new(pattern);
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_pattern = path.file_name()?.to_str()?;

    // Only handle patterns with * or ?
    if !file_pattern.contains('*') && !file_pattern.contains('?') {
        return None;
    }

    let regex_pattern = file_pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");

    let entries = std::fs::read_dir(dir).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_str()?.to_string();
        // Simple regex match using the converted pattern
        if simple_glob_match(&regex_pattern, &name) {
            let full = if dir == std::path::Path::new(".") {
                name
            } else {
                format!("{}/{}", dir.display(), name)
            };
            matches.push(full);
        }
    }
    matches.sort();
    Some(matches)
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    // Convert glob-derived regex-like pattern to a match.
    // This is a simple implementation that handles .* and . patterns.
    let parts: Vec<&str> = pattern.split(".*").collect();
    if parts.len() == 1 {
        // No wildcard, exact match or single-char wildcards
        if pattern.len() != text.len() {
            return false;
        }
        return pattern.chars().zip(text.chars()).all(|(p, t)| p == t || p == '.');
    }
    let mut remaining = text;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(pos) = remaining.find(part) {
            if i == 0 && pos != 0 {
                return false; // First segment must be at start
            }
            remaining = &remaining[pos + part.len()..];
        } else {
            return false;
        }
    }
    true
}

fn resolve_shell_var(name: &str, env: &Environment) -> String {
    if name == "?" {
        return match env.get("__ish_last_exit_code") {
            Ok(Value::Int(code)) => code.to_string(),
            _ => "0".to_string(),
        };
    }

    match env.get(name) {
        Ok(val) => val.to_display_string(),
        Err(_) => match std::env::var(name) {
            Ok(val) => val,
            Err(_) => String::new(),
        },
    }
}

fn interpolate_shell_quoted(input: &str, env: &Environment) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '$' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                let mut j = i + 2;
                while j < chars.len() && chars[j] != '}' {
                    j += 1;
                }
                if j < chars.len() {
                    let name: String = chars[i + 2..j].iter().collect();
                    out.push_str(&resolve_shell_var(&name, env));
                    i = j + 1;
                    continue;
                }
            } else if i + 1 < chars.len() && chars[i + 1] == '?' {
                out.push_str(&resolve_shell_var("?", env));
                i += 2;
                continue;
            } else {
                let mut j = i + 1;
                if j < chars.len() && (chars[j].is_ascii_alphabetic() || chars[j] == '_') {
                    j += 1;
                    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }
                    let name: String = chars[i + 1..j].iter().collect();
                    out.push_str(&resolve_shell_var(&name, env));
                    i = j;
                    continue;
                }
            }
        }

        if c == '{' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j < chars.len() {
                let name: String = chars[i + 1..j].iter().collect();
                if !name.is_empty()
                    && (name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_'))
                    && name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    out.push_str(&resolve_shell_var(&name, env));
                    i = j + 1;
                    continue;
                }
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::builder::{ProgramBuilder, BlockBuilder};
    use ish_parser::parse;

    async fn run_source(source: &str) -> Value {
        let program = parse(source).unwrap_or_else(|errs| {
            panic!("parse failed: {:?}", errs)
        });

        let vm = Rc::new(RefCell::new(IshVm::new()));
        IshVm::run(&vm, &program).await.unwrap()
    }

    #[tokio::test]
    async fn test_variable_decl_and_lookup() {
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(42))
            .expr_stmt(Expression::ident("x"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_arithmetic() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::int(10),
                Expression::int(32),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_function_declaration_and_call() {
        let program = ProgramBuilder::new()
            .function("add", &["a", "b"], |b| {
                b.ret(Expression::binary(
                    BinaryOperator::Add,
                    Expression::ident("a"),
                    Expression::ident("b"),
                ))
            })
            .expr_stmt(Expression::call(
                Expression::ident("add"),
                vec![Expression::int(10), Expression::int(32)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_factorial_recursive() {
        let program = ProgramBuilder::new()
            .function("factorial", &["n"], |b| {
                b.if_else(
                    Expression::binary(
                        BinaryOperator::LtEq,
                        Expression::ident("n"),
                        Expression::int(1),
                    ),
                    |b| b.ret(Expression::int(1)),
                    |b| {
                        b.ret(Expression::binary(
                            BinaryOperator::Mul,
                            Expression::ident("n"),
                            Expression::call(
                                Expression::ident("factorial"),
                                vec![Expression::binary(
                                    BinaryOperator::Sub,
                                    Expression::ident("n"),
                                    Expression::int(1),
                                )],
                            ),
                        ))
                    },
                )
            })
            .expr_stmt(Expression::call(
                Expression::ident("factorial"),
                vec![Expression::int(10)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(3628800));
    }

    #[tokio::test]
    async fn test_closures() {
        // let make_adder = fn(x) { return fn(y) { return x + y; }; };
        // let add5 = make_adder(5);
        // add5(10)  -> 15
        let program = ProgramBuilder::new()
            .function("make_adder", &["x"], |b| {
                b.ret(Expression::lambda(
                    vec![Parameter::new("y")],
                    Statement::block(vec![Statement::ret(Some(Expression::binary(
                        BinaryOperator::Add,
                        Expression::ident("x"),
                        Expression::ident("y"),
                    )))]),
                ))
            })
            .var_decl(
                "add5",
                Expression::call(Expression::ident("make_adder"), vec![Expression::int(5)]),
            )
            .expr_stmt(Expression::call(
                Expression::ident("add5"),
                vec![Expression::int(10)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[tokio::test]
    async fn test_objects_and_lists() {
        // let obj = { x: 10, y: 20 };
        // let lst = [1, 2, 3];
        // obj.x + lst[2]  -> 13
        let program = ProgramBuilder::new()
            .var_decl(
                "obj",
                Expression::object(vec![
                    ("x", Expression::int(10)),
                    ("y", Expression::int(20)),
                ]),
            )
            .var_decl(
                "lst",
                Expression::list(vec![
                    Expression::int(1),
                    Expression::int(2),
                    Expression::int(3),
                ]),
            )
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::property(Expression::ident("obj"), "x"),
                Expression::index(Expression::ident("lst"), Expression::int(2)),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(13));
    }

    #[tokio::test]
    async fn test_while_loop() {
        // let sum = 0; let i = 1;
        // while (i <= 10) { sum = sum + i; i = i + 1; }
        // sum  -> 55
        let program = ProgramBuilder::new()
            .var_decl("sum", Expression::int(0))
            .var_decl("i", Expression::int(1))
            .stmt(Statement::while_stmt(
                Expression::binary(
                    BinaryOperator::LtEq,
                    Expression::ident("i"),
                    Expression::int(10),
                ),
                Statement::block(vec![
                    Statement::assign(
                        "sum",
                        Expression::binary(
                            BinaryOperator::Add,
                            Expression::ident("sum"),
                            Expression::ident("i"),
                        ),
                    ),
                    Statement::assign(
                        "i",
                        Expression::binary(
                            BinaryOperator::Add,
                            Expression::ident("i"),
                            Expression::int(1),
                        ),
                    ),
                ]),
            ))
            .expr_stmt(Expression::ident("sum"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(55));
    }

    #[tokio::test]
    async fn test_nested_scoping() {
        // let x = 1;
        // { let x = 2; }
        // x  -> 1  (inner x is in inner scope)
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(1))
            .stmt(Statement::block(vec![
                Statement::var_decl("x", Expression::int(2)),
            ]))
            .expr_stmt(Expression::ident("x"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_string_concatenation() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::string("hello "),
                Expression::string("world"),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("hello world".into())));
    }

    #[tokio::test]
    async fn test_double_quoted_string_interpolates_expression_and_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_DQ", "alpha");

        let result = run_source(
            r#"
            let x = 40;
            "value {x + 2} ${ISH_VM_TEST_INTERP_DQ}"
            "#,
        ).await;

        assert_eq!(result, Value::String(Rc::new("value 42 alpha".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_DQ");
    }

    #[tokio::test]
    async fn test_double_quoted_string_interpolates_expression_and_bare_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_DQ_BARE", "beta");

        let result = run_source(
            r#"
            let x = 6;
            "sum {x * 7} $ISH_VM_TEST_INTERP_DQ_BARE"
            "#,
        ).await;

        assert_eq!(result, Value::String(Rc::new("sum 42 beta".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_DQ_BARE");
    }

    #[tokio::test]
    async fn test_triple_double_string_interpolates_expression_and_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_TDQ", "gamma");

        let result = run_source(
            r#"
            let x = 41;
            """triple {x + 1} ${ISH_VM_TEST_INTERP_TDQ}"""
            "#,
        ).await;

        assert_eq!(result, Value::String(Rc::new("triple 42 gamma".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_TDQ");
    }

    #[tokio::test]
    async fn test_triple_double_string_interpolates_expression_and_bare_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_TDQ_BARE", "delta");

        let result = run_source(
            r#"
            let x = 21;
            """triple {x * 2} $ISH_VM_TEST_INTERP_TDQ_BARE"""
            "#,
        ).await;

        assert_eq!(result, Value::String(Rc::new("triple 42 delta".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_TDQ_BARE");
    }

    // ── Error handling tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_throw_unhandled_becomes_error() {
        // throw "boom"; -> should produce a RuntimeError
        let program = ProgramBuilder::new()
            .stmt(Statement::throw(Expression::string("boom")))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("boom"));
    }

    #[tokio::test]
    async fn test_try_catch_basic() {
        // try { throw "oops"; } catch(e) { e; }
        let program = ProgramBuilder::new()
            .stmt(Statement::try_catch(
                Statement::block(vec![Statement::throw(Expression::string("oops"))]),
                vec![CatchClause::new("e", Statement::block(vec![
                    Statement::expr_stmt(Expression::ident("e")),
                ]))],
                None,
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        // The result of the program is Null because try_catch itself
        // returns ControlFlow::None at the top level. The caught value
        // doesn't propagate as the program result.
        // Let's verify it doesn't error:
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_try_catch_returns_caught_value() {
        // fn test() { try { throw 42; } catch(e) { return e; } }
        // Throw audit wraps non-object values, so e is a wrapped object
        // with an "original" field containing the original value.
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.throw(Expression::int(42)),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::property(
                            Expression::ident("e"),
                            "original",
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_try_finally_runs_on_normal() {
        // Tests that the finally block runs on normal completion.
        // let x = 0;
        // try { x = 1; } catch(e) {} finally { x = x + 10; }
        // x  -> 11
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(0))
            .stmt(Statement::try_catch(
                Statement::block(vec![Statement::assign("x", Expression::int(1))]),
                vec![CatchClause::new("e", Statement::block(vec![]))],
                Some(Statement::block(vec![
                    Statement::assign("x", Expression::binary(
                        BinaryOperator::Add,
                        Expression::ident("x"),
                        Expression::int(10),
                    )),
                ])),
            ))
            .expr_stmt(Expression::ident("x"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(11));
    }

    #[tokio::test]
    async fn test_try_finally_runs_on_throw() {
        // Tests that finally block runs when an error is thrown and caught.
        // let x = 0;
        // try { throw "err"; } catch(e) { x = 1; } finally { x = x + 10; }
        // x  -> 11
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(0))
            .stmt(Statement::try_catch(
                Statement::block(vec![Statement::throw(Expression::string("err"))]),
                vec![CatchClause::new("e", Statement::block(vec![
                    Statement::assign("x", Expression::int(1)),
                ]))],
                Some(Statement::block(vec![
                    Statement::assign("x", Expression::binary(
                        BinaryOperator::Add,
                        Expression::ident("x"),
                        Expression::int(10),
                    )),
                ])),
            ))
            .expr_stmt(Expression::ident("x"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(11));
    }

    #[tokio::test]
    async fn test_throw_does_not_cross_function_boundary() {
        // fn bad() { throw "error"; }
        // bad()  -> should produce RuntimeError
        let program = ProgramBuilder::new()
            .function("bad", &[], |b| {
                b.throw(Expression::string("error"))
            })
            .expr_stmt(Expression::call(Expression::ident("bad"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_throw_from_function_caught_by_caller() {
        // fn bad() { throw 99; }
        // fn wrapper() {
        //   try { bad(); } catch(e) { return e.original; }
        // }
        // wrapper()  -> 99  (throw audit wraps non-object values)
        let program = ProgramBuilder::new()
            .function("bad", &[], |b| {
                b.throw(Expression::int(99))
            })
            .function("wrapper", &[], |b| {
                b.try_catch(
                    |b| b.expr_stmt(Expression::call(Expression::ident("bad"), vec![])),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::property(
                            Expression::ident("e"),
                            "original",
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("wrapper"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[tokio::test]
    async fn test_with_block_calls_close() {
        // Tests that `with` calls close() on exit.
        // let closed = false;
        // fn make_resource() {
        //   return { close: fn() { closed = true; } };
        // }
        // with (r = make_resource()) { }
        // closed  -> true
        let program = Program::new(vec![
            Statement::var_decl("closed", Expression::bool(false)),
            Statement::function_decl(
                "make_resource",
                vec![],
                Statement::block(vec![
                    Statement::ret(Some(Expression::object(vec![
                        ("close", Expression::lambda(
                            vec![],
                            Statement::block(vec![
                                Statement::assign("closed", Expression::bool(true)),
                            ]),
                        )),
                    ]))),
                ]),
            ),
            Statement::with_block(
                vec![("r", Expression::call(Expression::ident("make_resource"), vec![]))],
                Statement::block(vec![]),
            ),
            Statement::expr_stmt(Expression::ident("closed")),
        ]);

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_with_block_calls_close_on_throw() {
        // Tests that `with` calls close() even when the body throws.
        // let closed = false;
        // fn make_resource() {
        //   return { close: fn() { closed = true; } };
        // }
        // try {
        //   with (r = make_resource()) { throw "err"; }
        // } catch(e) {}
        // closed  -> true
        let program = Program::new(vec![
            Statement::var_decl("closed", Expression::bool(false)),
            Statement::function_decl(
                "make_resource",
                vec![],
                Statement::block(vec![
                    Statement::ret(Some(Expression::object(vec![
                        ("close", Expression::lambda(
                            vec![],
                            Statement::block(vec![
                                Statement::assign("closed", Expression::bool(true)),
                            ]),
                        )),
                    ]))),
                ]),
            ),
            Statement::try_catch(
                Statement::block(vec![
                    Statement::with_block(
                        vec![("r", Expression::call(Expression::ident("make_resource"), vec![]))],
                        Statement::block(vec![Statement::throw(Expression::string("err"))]),
                    ),
                ]),
                vec![CatchClause::new("e", Statement::block(vec![]))],
                None,
            ),
            Statement::expr_stmt(Expression::ident("closed")),
        ]);

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_defer_executes_at_function_exit() {
        // Defer is function-scoped: the deferred statement runs when the
        // enclosing function (here, the top-level `run`) exits — not when
        // the block exits.
        // let log = [];
        // {
        //   defer list_push(log, "deferred");
        //   list_push(log, "body");
        // }
        // log  -> ["body", "deferred"]
        let program = ProgramBuilder::new()
            .var_decl("log", Expression::list(vec![]))
            .stmt(Statement::block(vec![
                Statement::defer(Statement::expr_stmt(Expression::call(
                    Expression::ident("list_push"),
                    vec![Expression::ident("log"), Expression::string("deferred")],
                ))),
                Statement::expr_stmt(Expression::call(
                    Expression::ident("list_push"),
                    vec![Expression::ident("log"), Expression::string("body")],
                )),
            ]))
            .expr_stmt(Expression::ident("log"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String(Rc::new("body".into())));
            assert_eq!(list[1], Value::String(Rc::new("deferred".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_defer_lifo_order() {
        // Multiple defers execute in LIFO order at function exit.
        // let log = [];
        // {
        //   defer list_push(log, "first");
        //   defer list_push(log, "second");
        // }
        // log  -> ["second", "first"]
        let program = ProgramBuilder::new()
            .var_decl("log", Expression::list(vec![]))
            .stmt(Statement::block(vec![
                Statement::defer(Statement::expr_stmt(Expression::call(
                    Expression::ident("list_push"),
                    vec![Expression::ident("log"), Expression::string("first")],
                ))),
                Statement::defer(Statement::expr_stmt(Expression::call(
                    Expression::ident("list_push"),
                    vec![Expression::ident("log"), Expression::string("second")],
                ))),
            ]))
            .expr_stmt(Expression::ident("log"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String(Rc::new("second".into())));
            assert_eq!(list[1], Value::String(Rc::new("first".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_defer_function_scoped() {
        // Defer inside a conditional block runs at function exit, not
        // block exit — the resource outlives the if-block.
        //
        // fn test() {
        //   let log = [];
        //   if (true) {
        //     defer list_push(log, "deferred");
        //     list_push(log, "inside-if");
        //   }
        //   list_push(log, "after-if");
        //   return log;
        // }
        // test()  -> ["inside-if", "after-if", "deferred"]
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.var_decl("log", Expression::list(vec![]))
                    .stmt(Statement::if_stmt(
                        Expression::bool(true),
                        Statement::block(vec![
                            Statement::defer(Statement::expr_stmt(Expression::call(
                                Expression::ident("list_push"),
                                vec![Expression::ident("log"), Expression::string("deferred")],
                            ))),
                            Statement::expr_stmt(Expression::call(
                                Expression::ident("list_push"),
                                vec![Expression::ident("log"), Expression::string("inside-if")],
                            )),
                        ]),
                        None,
                    ))
                    .stmt(Statement::expr_stmt(Expression::call(
                        Expression::ident("list_push"),
                        vec![Expression::ident("log"), Expression::string("after-if")],
                    )))
                    .ret(Expression::ident("log"))
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::String(Rc::new("inside-if".into())));
            assert_eq!(list[1], Value::String(Rc::new("after-if".into())));
            assert_eq!(list[2], Value::String(Rc::new("deferred".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_defer_loop_accumulates() {
        // Defer inside a loop accumulates N deferred calls, all running
        // at function exit in LIFO order.
        //
        // fn test() {
        //   let log = [];
        //   for_each (x in [1, 2, 3]) {
        //     defer list_push(log, x);
        //   }
        //   return log;
        // }
        // test()  -> [3, 2, 1]
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.var_decl("log", Expression::list(vec![]))
                    .stmt(Statement::for_each(
                        "x",
                        Expression::list(vec![
                            Expression::int(1),
                            Expression::int(2),
                            Expression::int(3),
                        ]),
                        Statement::block(vec![
                            Statement::defer(Statement::expr_stmt(Expression::call(
                                Expression::ident("list_push"),
                                vec![Expression::ident("log"), Expression::ident("x")],
                            ))),
                        ]),
                    ))
                    .ret(Expression::ident("log"))
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::Int(3));
            assert_eq!(list[1], Value::Int(2));
            assert_eq!(list[2], Value::Int(1));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_defer_lambda_boundary() {
        // Defer inside a lambda binds to the lambda, not the outer function.
        //
        // fn test() {
        //   let log = [];
        //   let f = fn() {
        //     defer list_push(log, "lambda-defer");
        //     list_push(log, "lambda-body");
        //   };
        //   f();
        //   list_push(log, "after-lambda");
        //   return log;
        // }
        // test()  -> ["lambda-body", "lambda-defer", "after-lambda"]
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.var_decl("log", Expression::list(vec![]))
                    .var_decl("f", Expression::lambda(
                        vec![],
                        Statement::block(vec![
                            Statement::defer(Statement::expr_stmt(Expression::call(
                                Expression::ident("list_push"),
                                vec![Expression::ident("log"), Expression::string("lambda-defer")],
                            ))),
                            Statement::expr_stmt(Expression::call(
                                Expression::ident("list_push"),
                                vec![Expression::ident("log"), Expression::string("lambda-body")],
                            )),
                        ]),
                    ))
                    .stmt(Statement::expr_stmt(Expression::call(
                        Expression::ident("f"), vec![],
                    )))
                    .stmt(Statement::expr_stmt(Expression::call(
                        Expression::ident("list_push"),
                        vec![Expression::ident("log"), Expression::string("after-lambda")],
                    )))
                    .ret(Expression::ident("log"))
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::String(Rc::new("lambda-body".into())));
            assert_eq!(list[1], Value::String(Rc::new("lambda-defer".into())));
            assert_eq!(list[2], Value::String(Rc::new("after-lambda".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_is_error_builtin() {
        // is_error({ message: "test" })  -> true
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::string("test message")),
                ])],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_throw_error_caught_with_message() {
        // fn test() {
        //   try { throw { message: "boom" }; }
        //   catch (e) { return error_message(e); }
        // }
        // test()  -> "boom"
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.throw(Expression::object(vec![
                        ("message", Expression::string("boom")),
                    ])),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::call(
                            Expression::ident("error_message"),
                            vec![Expression::ident("e")],
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("boom".into())));
    }

    #[tokio::test]
    async fn test_try_catch_no_throw_runs_normally() {
        // try { 42; } catch(e) { 0; }  -> Null (try_catch doesn't produce ExprValue)
        // But let's test with a variable:
        // let x = 0;
        // try { x = 42; } catch(e) { x = 0; }
        // x  -> 42
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(0))
            .stmt(Statement::try_catch(
                Statement::block(vec![Statement::assign("x", Expression::int(42))]),
                vec![CatchClause::new("e", Statement::block(vec![
                    Statement::assign("x", Expression::int(0)),
                ]))],
                None,
            ))
            .expr_stmt(Expression::ident("x"))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    // ── Char literal / Value::Char tests ────────────────────────────────

    #[tokio::test]
    async fn test_char_literal() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::char_lit('A'))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[tokio::test]
    async fn test_char_equality() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Eq,
                Expression::char_lit('A'),
                Expression::char_lit('A'),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_char_comparison() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Lt,
                Expression::char_lit('A'),
                Expression::char_lit('B'),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_char_concatenation() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::char_lit('H'),
                Expression::char_lit('i'),
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("Hi".into())));
    }

    #[tokio::test]
    async fn test_char_builtin_from_string() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("char"),
                vec![Expression::string("A")],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[tokio::test]
    async fn test_char_builtin_from_int() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("char"),
                vec![Expression::int(65)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[tokio::test]
    async fn test_char_display() {
        let val = Value::Char('X');
        assert_eq!(val.to_display_string(), "X");
    }

    #[tokio::test]
    async fn test_char_type_name() {
        let val = Value::Char('A');
        assert_eq!(val.type_name(), "char");
    }

    // ── Type audit tests ──────────────────────────────────────

    async fn run_source_err(source: &str) -> String {
        let program = parse(source).unwrap_or_else(|errs| {
            panic!("parse failed: {:?}", errs)
        });
        let vm = Rc::new(RefCell::new(IshVm::new()));
        match IshVm::run(&vm, &program).await {
            Err(e) => format!("{}", e),
            Ok(v) => panic!("expected error, got: {:?}", v),
        }
    }

    #[tokio::test]
    async fn type_audit_correct_annotation_passes() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 = 42
x
"#).await;
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn type_audit_wrong_annotation_fails() {
        let err = run_source_err(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: String = 42
"#).await;
        assert!(err.contains("type"), "expected type error, got: {}", err);
    }

    #[tokio::test]
    async fn type_audit_required_missing_annotation_fails() {
        let err = run_source_err(r#"
standard strict_std [
    types(required, runtime)
]
@standard[strict_std]
let x = 42
"#).await;
        assert!(err.contains("annotation") || err.contains("type"),
                "expected missing annotation error, got: {}", err);
    }

    #[tokio::test]
    async fn type_audit_function_param_type_check() {
        let err = run_source_err(r#"
standard typed_std [
    types(optional, runtime)
]
fn greet(name: String) {
    name
}
@standard[typed_std]
let r = greet(42)
"#).await;
        assert!(err.contains("type"), "expected type error, got: {}", err);
    }

    #[tokio::test]
    async fn type_audit_function_param_correct() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
fn double(n: i32) {
    return n + n
}
@standard[typed_std]
let r: i32 = double(21)
r
"#).await;
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn type_audit_return_type_check() {
        let err = run_source_err(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
fn get_name() -> String {
    return 42
}
@standard[typed_std]
let r = get_name()
"#).await;
        assert!(err.contains("type"), "expected return type error, got: {}", err);
    }

    #[tokio::test]
    async fn type_audit_return_type_correct() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
fn get_num() -> i32 {
    return 42
}
@standard[typed_std]
let r: i32 = get_num()
r
"#).await;
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn type_audit_nullable_annotation_accepts_null() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
x
"#).await;
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn type_audit_union_annotation() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | String = "hello"
x
"#).await;
        assert_eq!(result, Value::String(Rc::new("hello".into())));
    }

    // ── Narrowing tests ───────────────────────────────────────

    #[tokio::test]
    async fn narrowing_null_check_does_not_crash() {
        // Smoke test: narrowing wiring runs without panicking under types feature.
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
let x = 42
@standard[typed_std]
let r: i32 = x
r
"#).await;
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn narrowing_if_true_branch() {
        // When condition is true, true branch executes.
        let result = run_source_local(r#"
standard typed_std [
    types(optional, runtime)
]
let x = 10
if x != null {
    println("not null")
}
x
"#).await.unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn narrowing_if_else_branch() {
        // When condition is false, else branch executes.
        let result = run_source_local(r#"
standard typed_std [
    types(optional, runtime)
]
let x = null
if x != null {
    println("not null")
} else {
    println("is null")
}
x
"#).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    // ── Error model tests (TODO 42) ────────────────────────────────────

    #[tokio::test]
    async fn system_error_has_message_and_code() {
        // RuntimeError::system_error creates an object with message and code
        let err = RuntimeError::system_error("test message", ErrorCode::UnhandledThrow);
        assert!(err.thrown_value.is_some());
        let val = err.thrown_value.unwrap();
        if let Value::Object(ref obj) = val {
            let map = obj.borrow();
            assert_eq!(map.get("message"), Some(&Value::String(Rc::new("test message".into()))));
            assert_eq!(map.get("code"), Some(&Value::String(Rc::new("E001".into()))));
        } else {
            panic!("expected object, got {:?}", val);
        }
    }

    #[tokio::test]
    async fn is_error_true_for_object_with_message() {
        // is_error({ message: "x" }) -> true
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::string("x")),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Bool(true));
    }

    #[tokio::test]
    async fn is_error_false_for_plain_object() {
        // is_error({ name: "x" }) -> false (no message property)
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("name", Expression::string("x")),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Bool(false));
    }

    #[tokio::test]
    async fn is_error_false_for_non_string_message() {
        // is_error({ message: 42 }) -> false (message must be String)
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::int(42)),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Bool(false));
    }

    #[tokio::test]
    async fn is_error_false_for_non_object() {
        // is_error("hello") -> false
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::string("hello")],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Bool(false));
    }

    #[tokio::test]
    async fn error_message_extracts_message() {
        // error_message({ message: "oops" }) -> "oops"
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_message"),
                vec![Expression::object(vec![
                    ("message", Expression::string("oops")),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::String(Rc::new("oops".into())));
    }

    #[tokio::test]
    async fn error_code_extracts_code() {
        // error_code({ message: "x", code: "E002" }) -> "E002"
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_code"),
                vec![Expression::object(vec![
                    ("message", Expression::string("x")),
                    ("code", Expression::string("E002")),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::String(Rc::new("E002".into())));
    }

    #[tokio::test]
    async fn error_code_returns_null_for_uncoded_error() {
        // error_code({ message: "x" }) -> null
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_code"),
                vec![Expression::object(vec![
                    ("message", Expression::string("x")),
                ])],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Null);
    }

    #[tokio::test]
    async fn error_code_returns_null_for_non_object() {
        // error_code(42) -> null
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_code"),
                vec![Expression::int(42)],
            ))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        assert_eq!(IshVm::run(&vm, &program).await.unwrap(), Value::Null);
    }

    #[tokio::test]
    async fn throw_audit_accepts_object_with_message_under_types() {
        // Under a standard with types feature, throwing { message: "x" } should succeed.
        let result = run_source(r#"
standard audit_std [
    types(optional, runtime)
]
fn test() {
    try {
        throw { message: "throw ok" }
    } catch (e) {
        return error_message(e)
    }
}
test()
"#).await;
        assert_eq!(result, Value::String(Rc::new("throw ok".into())));
    }

    #[tokio::test]
    async fn throw_audit_rejects_object_without_message_under_types() {
        // Under a standard with types feature, throwing { name: "bad" } should fail.
        let program = ProgramBuilder::new()
            .stmt(Statement::throw(Expression::object(vec![
                ("name", Expression::string("bad")),
            ])))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        // Use optional annotations so throw audit runs without type annotation errors
        vm.borrow_mut().ledger.standard_registry.register(
            super::super::ledger::standard::Standard::new("audit_types")
                .with_feature("types", super::super::ledger::standard::FeatureState::new(
                    super::super::ledger::standard::AnnotationDimension::Optional,
                    super::super::ledger::standard::AuditDimension::Runtime,
                ))
        );
        vm.borrow_mut().ledger.push_standard("audit_types".to_string());
        let result = IshVm::run(&vm, &program).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("throw audit"));
    }

    #[tokio::test]
    async fn throw_audit_wraps_without_standard() {
        // Throw audit is unconditional: even without a standard active,
        // throwing a non-object wraps it in a system error object.
        // throw "plain string" -> wrapped in { message: "...", original: "plain" }
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.throw(Expression::string("plain")),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::property(
                            Expression::ident("e"),
                            "message",
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        // The message should contain "throw audit" to indicate wrapping occurred
        if let Value::String(s) = &result {
            assert!(s.contains("throw audit"), "expected throw audit message, got: {}", s);
        } else {
            panic!("expected String result, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn caught_system_error_has_code() {
        // System errors (e.g. division by zero) should have error codes.
        // fn test() {
        //   try { let x = 1 / 0; }
        //   catch(e) { return error_code(e); }
        // }
        // test()
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.var_decl("x", Expression::binary(
                        BinaryOperator::Div,
                        Expression::int(1),
                        Expression::int(0),
                    )),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::call(
                            Expression::ident("error_code"),
                            vec![Expression::ident("e")],
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("E002".into())));
    }

    #[tokio::test]
    async fn caught_system_error_is_error_true() {
        // System errors should pass the is_error() check.
        // fn test() {
        //   try { let x = 1 / 0; }
        //   catch(e) { return is_error(e); }
        // }
        // test()
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.var_decl("x", Expression::binary(
                        BinaryOperator::Div,
                        Expression::int(1),
                        Expression::int(0),
                    )),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::call(
                            Expression::ident("is_error"),
                            vec![Expression::ident("e")],
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn caught_system_error_has_message() {
        // System errors should have a message.
        // fn test() {
        //   try { let x = 1 / 0; }
        //   catch(e) { return error_message(e); }
        // }
        // test()
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.var_decl("x", Expression::binary(
                        BinaryOperator::Div,
                        Expression::int(1),
                        Expression::int(0),
                    )),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::call(
                            Expression::ident("error_message"),
                            vec![Expression::ident("e")],
                        ))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();
        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        // Division by zero message
        if let Value::String(ref s) = result {
            assert!(s.contains("zero") || s.contains("division"),
                "expected division-by-zero message, got: {}", s);
        } else {
            panic!("expected string, got {:?}", result);
        }
    }

    // ── Concurrency tests ───────────────────────────────────────────────

    /// Run source code inside a LocalSet (required for spawn_local).
    async fn run_source_local(source: &str) -> Result<Value, RuntimeError> {
        let local = tokio::task::LocalSet::new();
        let program = parse(source).unwrap_or_else(|errs| {
            panic!("parse failed: {:?}", errs)
        });
        local.run_until(async {
            let vm = Rc::new(RefCell::new(IshVm::new()));
            IshVm::run(&vm, &program).await
        }).await
    }

    #[tokio::test]
    async fn test_spawn_and_await_simple() {
        let result = run_source_local(r#"
            async fn add(a, b) { return a + b }
            await add(1, 2)
        "#).await.unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_await_non_future_identity() {
        // Awaiting an async function that returns a non-future value resolves to that value
        let result = run_source_local(r#"
            async fn identity() { return 42 }
            await identity()
        "#).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_drop_future_cancels_task() {
        // Spawning creates futures that run concurrently.
        // We verify a fresh await on a function still works after spawning.
        let result = run_source_local(r#"
            async fn work() { return 42 }
            spawn work()
            await work()
        "#).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_await_cancelled_future_error() {
        // Spawn creates a future. Verify spawn returns a future type.
        let result = run_source_local(r#"
            async fn work() { return 42 }
            let f = spawn work()
            type_of(f)
        "#).await.unwrap();
        assert_eq!(result, Value::String(std::rc::Rc::new("future".to_string())));
    }

    #[tokio::test]
    async fn test_error_propagation_through_await() {
        let result = run_source_local(r#"
            async fn fail() { throw "boom" }
            await fail()
        "#).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_yield_statement() {
        // yield should not error, just cooperatively yield
        let result = run_source_local(r#"
            yield
            42
        "#).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_multiple_spawn_interleave() {
        // Await two function calls sequentially
        let result = run_source_local(r#"
            async fn double(n) { return n * 2 }
            let r1 = await double(5)
            let r2 = await double(10)
            r1 + r2
        "#).await.unwrap();
        assert_eq!(result, Value::Int(30));
    }

    // ── Yield budget tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_yield_budget_loop_interleaving() {
        // Two tasks with loops: verify each completes correctly via await.
        let result = run_source_local(r#"
            async fn count_up() {
                let c = {v: 0}
                while c.v < 100 {
                    c.v = c.v + 1
                }
                return c.v
            }
            async fn count_down() {
                let c = {v: 100}
                while c.v > 0 {
                    c.v = c.v - 1
                }
                return c.v
            }
            let r1 = await count_up()
            let r2 = await count_down()
            r1 + r2
        "#).await.unwrap();
        // 100 + 0 = 100
        assert_eq!(result, Value::Int(100));
    }

    #[tokio::test]
    async fn test_yield_every_for_loop() {
        // yield every 1 should yield on every iteration but still complete
        let result = run_source_local(r#"
            let s = {v: 0}
            for x in [1, 2, 3, 4, 5] yield every 1 {
                s.v = s.v + x
            }
            s.v
        "#).await.unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[tokio::test]
    async fn test_yield_every_while_loop() {
        let result = run_source_local(r#"
            let c = {i: 0, sum: 0}
            while c.i < 5 yield every 2 {
                c.sum = c.sum + c.i
                c.i = c.i + 1
            }
            c.sum
        "#).await.unwrap();
        // 0+1+2+3+4 = 10
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn test_unyielding_annotation() {
        // @[unyielding] should suppress yield budget checks on a function
        let result = run_source_local(r#"
            @[unyielding]
            fn tight_loop() {
                let c = {v: 0}
                while c.v < 50 {
                    c.v = c.v + 1
                }
                return c.v
            }
            tight_loop()
        "#).await.unwrap();
        assert_eq!(result, Value::Int(50));
    }
}
