use std::collections::HashMap;
use std::rc::Rc;
use std::process::{Command, Stdio};

use ish_ast::*;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::value::*;
use crate::builtins;
use crate::ledger::LedgerState;

/// Signal for control flow: normal completion, return, throw, or break.
enum ControlFlow {
    None,
    Return(Value),
    /// The value produced by the last expression statement.
    ExprValue(Value),
    /// A thrown error value propagating up the call stack.
    Throw(Value),
}

/// The ish virtual machine / interpreter.
pub struct IshVm {
    pub global_env: Environment,
    /// Per-function defer stacks. Each function invocation (and top-level
    /// `run`) pushes a frame; deferred statements execute in LIFO order
    /// when the frame is popped at function exit.
    defer_stack: Vec<Vec<(Statement, Environment)>>,
    /// Assurance ledger runtime state: standard scope stack, entry store,
    /// and built-in registries.
    pub ledger: LedgerState,
}

impl IshVm {
    pub fn new() -> Self {
        let env = Environment::new();
        builtins::register_all(&env);
        crate::reflection::register_ast_builtins(&env);
        IshVm {
            global_env: env,
            defer_stack: Vec::new(),
            ledger: LedgerState::new(),
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

    /// Pop the current defer frame and execute all deferred statements in
    /// LIFO order. Errors from deferred statements are silently ignored
    /// (they do not override in-flight control flow).
    fn pop_and_run_defers(&mut self) {
        if let Some(deferred) = self.defer_stack.pop() {
            for (d, env) in deferred.into_iter().rev() {
                let _ = self.exec_statement(&d, &env);
            }
        }
    }

    /// Execute a full program.
    pub fn run(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last = Value::Null;
        let env = self.global_env.clone();
        self.push_defer_frame();
        for stmt in &program.statements {
            match self.exec_statement(stmt, &env) {
                Ok(ControlFlow::Return(v)) => {
                    last = v;
                    break;
                }
                Ok(ControlFlow::Throw(v)) => {
                    self.pop_and_run_defers();
                    return Err(RuntimeError::system_error(format!(
                        "Unhandled throw: {}", v.to_display_string()
                    ), "E001"));
                }
                Ok(ControlFlow::ExprValue(v)) => last = v,
                Ok(ControlFlow::None) => {}
                Err(e) => {
                    self.pop_and_run_defers();
                    return Err(e);
                }
            }
        }
        self.pop_and_run_defers();
        Ok(last)
    }

    /// Execute a single statement in the given environment.
    fn exec_statement(
        &mut self,
        stmt: &Statement,
        env: &Environment,
    ) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Statement::VariableDecl { name, value, type_annotation, .. } => {
                let val = self.eval_expression(value, env)?;
                self.audit_type_annotation(name, &val, type_annotation.as_ref())?;
                env.define(name.clone(), val);
                Ok(ControlFlow::None)
            }

            Statement::Assignment { target, value } => {
                let val = self.eval_expression(value, env)?;
                match target {
                    AssignTarget::Variable(name) => {
                        env.set(name, val)?;
                    }
                    AssignTarget::Property { object, property } => {
                        let obj = self.eval_expression(object, env)?;
                        if let Value::Object(ref obj_ref) = obj {
                            obj_ref.borrow_mut().insert(property.clone(), val);
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot set property '{}' on {}",
                                property,
                                obj.type_name()
                            ), "E004"));
                        }
                    }
                    AssignTarget::Index { object, index } => {
                        let obj = self.eval_expression(object, env)?;
                        let idx = self.eval_expression(index, env)?;
                        if let Value::List(ref list_ref) = obj {
                            if let Value::Int(i) = idx {
                                let mut list = list_ref.borrow_mut();
                                let len = list.len() as i64;
                                if i < 0 || i >= len {
                                    return Err(RuntimeError::system_error(format!(
                                        "index {} out of bounds (length {})",
                                        i, len
                                    ), "E007"));
                                }
                                list[i as usize] = val;
                            } else {
                                return Err(RuntimeError::system_error("list index must be an integer", "E004"));
                            }
                        } else if let Value::Object(ref obj_ref) = obj {
                            if let Value::String(ref key) = idx {
                                obj_ref.borrow_mut().insert(key.as_ref().clone(), val);
                            } else {
                                return Err(RuntimeError::system_error(
                                    "object index must be a string", "E004"
                                ));
                            }
                        } else {
                            return Err(RuntimeError::system_error(format!(
                                "cannot index into {}",
                                obj.type_name()
                            ), "E004"));
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
                        self.register_defer(*body.clone(), block_env.clone());
                        continue;
                    }
                    match self.exec_statement(s, &block_env)? {
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
                let cond = self.eval_expression(condition, env)?;

                // Type narrowing: when types feature is active, analyze
                // the condition and apply narrowing facts to each branch.
                let features = self.ledger.active_features();
                let types_active = features.contains_key("types");

                if types_active {
                    use crate::ledger::narrowing::{analyze_condition, invert_for_else, NarrowingFact};

                    let facts = analyze_condition(condition);
                    let snapshot = self.ledger.save_entries();

                    if cond.is_truthy() {
                        // Apply facts to true branch.
                        for fact in &facts {
                            match fact {
                                NarrowingFact::IsType { variable, type_name } => {
                                    self.ledger.narrow_type(variable, type_name);
                                }
                                NarrowingFact::NotNull { variable } => {
                                    self.ledger.narrow_exclude_null(variable);
                                }
                                NarrowingFact::IsNull { .. } => {
                                    // IsNull in true branch: no narrowing benefit.
                                }
                            }
                        }
                        let result = self.exec_statement(then_block, env);
                        let then_entries = self.ledger.save_entries();
                        self.ledger.restore_entries(snapshot);
                        self.ledger.merge_entries(then_entries, self.ledger.save_entries());
                        result
                    } else if let Some(eb) = else_block {
                        let else_facts = invert_for_else(&facts);
                        for fact in &else_facts {
                            match fact {
                                NarrowingFact::IsType { variable, type_name } => {
                                    self.ledger.narrow_type(variable, type_name);
                                }
                                NarrowingFact::NotNull { variable } => {
                                    self.ledger.narrow_exclude_null(variable);
                                }
                                NarrowingFact::IsNull { .. } => {}
                            }
                        }
                        let result = self.exec_statement(eb, env);
                        let else_entries = self.ledger.save_entries();
                        self.ledger.restore_entries(snapshot);
                        self.ledger.merge_entries(self.ledger.save_entries(), else_entries);
                        result
                    } else {
                        Ok(ControlFlow::None)
                    }
                } else {
                    // No type narrowing — simple if/else.
                    if cond.is_truthy() {
                        self.exec_statement(then_block, env)
                    } else if let Some(eb) = else_block {
                        self.exec_statement(eb, env)
                    } else {
                        Ok(ControlFlow::None)
                    }
                }
            }

            Statement::While { condition, body } => {
                loop {
                    let cond = self.eval_expression(condition, env)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    match self.exec_statement(body, env)? {
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
            } => {
                let iter_val = self.eval_expression(iterable, env)?;
                if let Value::List(ref list_ref) = iter_val {
                    let items: Vec<Value> = list_ref.borrow().clone();
                    for item in items {
                        let loop_env = env.child();
                        loop_env.define(variable.clone(), item);
                        match self.exec_statement(body, &loop_env)? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Throw(v) => return Ok(ControlFlow::Throw(v)),
                            ControlFlow::None | ControlFlow::ExprValue(_) => {}
                        }
                    }
                } else {
                    return Err(RuntimeError::system_error(format!(
                        "cannot iterate over {}",
                        iter_val.type_name()
                    ), "E004"));
                }
                Ok(ControlFlow::None)
            }

            Statement::Return { value } => {
                let val = if let Some(expr) = value {
                    self.eval_expression(expr, env)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(val))
            }

            Statement::ExpressionStmt(expr) => {
                let val = self.eval_expression(expr, env)?;
                Ok(ControlFlow::ExprValue(val))
            }

            Statement::FunctionDecl {
                name, params, body, return_type, ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                let func = new_function(
                    Some(name.clone()),
                    param_names,
                    param_types,
                    return_type.clone(),
                    *body.clone(),
                    env.clone(),
                );
                env.define(name.clone(), func);
                Ok(ControlFlow::None)
            }

            Statement::Throw { value } => {
                let val = self.eval_expression(value, env)?;
                // Throw audit: when the types feature is active, verify the
                // thrown value qualifies as an error object.
                if self.ledger.active_features().contains_key("types") {
                    if let Value::Object(ref obj_ref) = val {
                        let map = obj_ref.borrow();
                        let has_message = matches!(map.get("message"), Some(Value::String(_)));
                        if !has_message {
                            return Err(RuntimeError::system_error(
                                "throw audit: thrown object must have a 'message: String' property",
                                "E004",
                            ));
                        }
                    }
                }
                Ok(ControlFlow::Throw(val))
            }

            Statement::TryCatch { body, catches, finally } => {
                // Collect thrown value from either ControlFlow::Throw or
                // a RuntimeError with thrown_value (throw that crossed a
                // function boundary via call_function).
                let (result, thrown) = match self.exec_statement(body, env) {
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
                        catch_result = self.exec_statement(&clause.body, &catch_env)?;
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
                    let fin_result = self.exec_statement(fin, env)?;
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
                    match self.eval_expression(expr, &with_env) {
                        Ok(val) => {
                            with_env.define(name.clone(), val.clone());
                            initialized.push((name.clone(), val));
                        }
                        Err(e) => {
                            // Close already-initialized resources in reverse order
                            for (_, res) in initialized.into_iter().rev() {
                                let _ = self.try_close(&res);
                            }
                            return Err(e);
                        }
                    }
                }
                // Execute body
                let result = self.exec_statement(body, &with_env)?;
                // Close resources in reverse order
                let mut close_error: Option<Value> = None;
                for (_, res) in initialized.into_iter().rev() {
                    if let Err(_e) = self.try_close(&res) {
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
                self.register_defer(*body.clone(), env.clone());
                Ok(ControlFlow::None)
            }

            Statement::TypeAlias { .. } => {
                // Type aliases are checked at analysis time, not runtime
                Ok(ControlFlow::None)
            }

            Statement::Use { .. } => {
                // Module imports are resolved at load time, not runtime
                Ok(ControlFlow::None)
            }

            Statement::ModDecl { .. } => {
                // Module declarations are structural, not runtime
                Ok(ControlFlow::None)
            }

            Statement::ShellCommand { command, args, pipes, redirections, background: _ } => {
                self.exec_shell_command(command, args, pipes, redirections, env)
            }

            Statement::Annotated { annotations, inner } => {
                // Process standard annotations: push standards for the scope.
                let mut pushed_standards = Vec::new();
                for ann in annotations {
                    if let Annotation::Standard(name) = ann {
                        self.ledger.push_standard(name.clone());
                        pushed_standards.push(name.clone());
                    }
                }
                let result = self.exec_statement(inner, env);
                // Pop standards in reverse order.
                for _ in pushed_standards.iter().rev() {
                    self.ledger.pop_standard();
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
                self.ledger.standard_registry.register(std);
                Ok(ControlFlow::None)
            }

            Statement::EntryTypeDef { name, .. } => {
                // Register as a simple entry type (no required properties for now).
                use crate::ledger::entry_type::EntryType;
                self.ledger.entry_type_registry.register(EntryType::new(name.clone()));
                Ok(ControlFlow::None)
            }

            Statement::Match { .. } => {
                // Match not yet implemented in interpreter
                Ok(ControlFlow::None)
            }

            Statement::Incomplete { kind } => {
                Err(RuntimeError::system_error(format!("incomplete input: {:?}", kind), "E004"))
            }
        }
    }

    /// Try to call close() on a value, used by WithBlock.
    fn try_close(&mut self, value: &Value) -> Result<(), RuntimeError> {
        if let Value::Object(ref obj_ref) = value {
            let map = obj_ref.borrow();
            if let Some(close_fn) = map.get("close").cloned() {
                drop(map);
                self.call_function(&close_fn, &[])?;
            }
        }
        Ok(())
    }

    /// Evaluate an expression in the given environment.
    pub fn eval_expression(
        &mut self,
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
                let lhs = self.eval_expression(left, env)?;
                // Short-circuit for logical operators
                match op {
                    BinaryOperator::And => {
                        if !lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return self.eval_expression(right, env);
                    }
                    BinaryOperator::Or => {
                        if lhs.is_truthy() {
                            return Ok(lhs);
                        }
                        return self.eval_expression(right, env);
                    }
                    _ => {}
                }
                let rhs = self.eval_expression(right, env)?;
                self.eval_binary_op(op, &lhs, &rhs)
            }

            Expression::UnaryOp { op, operand } => {
                let val = self.eval_expression(operand, env)?;
                match op {
                    UnaryOperator::Not => Ok(Value::Bool(!val.is_truthy())),
                    UnaryOperator::Negate => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(RuntimeError::system_error(format!(
                            "cannot negate {}",
                            val.type_name()
                        ), "E004")),
                    },
                    UnaryOperator::Try => {
                        // ? operator: if value is an error, propagate it; otherwise unwrap
                        // For now, null signals error
                        if val == Value::Null {
                            return Err(RuntimeError::system_error("tried to unwrap null value with ?".to_string(), "E009"));
                        }
                        Ok(val)
                    }
                }
            }

            Expression::FunctionCall { callee, args } => {
                let func_val = self.eval_expression(callee, env)?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(self.eval_expression(arg, env)?);
                }
                self.call_function(&func_val, &arg_vals)
            }

            Expression::ObjectLiteral(pairs) => {
                let mut map = HashMap::new();
                for (key, val_expr) in pairs {
                    let val = self.eval_expression(val_expr, env)?;
                    map.insert(key.clone(), val);
                }
                Ok(new_object(map))
            }

            Expression::ListLiteral(elements) => {
                let mut items = Vec::with_capacity(elements.len());
                for elem in elements {
                    items.push(self.eval_expression(elem, env)?);
                }
                Ok(new_list(items))
            }

            Expression::PropertyAccess { object, property } => {
                let obj = self.eval_expression(object, env)?;
                match obj {
                    Value::Object(ref obj_ref) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(property).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::system_error(format!(
                        "cannot access property '{}' on {}",
                        property,
                        obj.type_name()
                    ), "E004")),
                }
            }

            Expression::IndexAccess { object, index } => {
                let obj = self.eval_expression(object, env)?;
                let idx = self.eval_expression(index, env)?;
                match (&obj, &idx) {
                    (Value::List(list_ref), Value::Int(i)) => {
                        let list = list_ref.borrow();
                        let i = *i;
                        if i < 0 || i >= list.len() as i64 {
                            return Err(RuntimeError::system_error(format!(
                                "index {} out of bounds (length {})",
                                i,
                                list.len()
                            ), "E007"));
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
                    ), "E004")),
                }
            }

            Expression::Lambda { params, body } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let param_types: Vec<Option<ish_ast::TypeAnnotation>> =
                    params.iter().map(|p| p.type_annotation.clone()).collect();
                Ok(new_function(None, param_names, param_types, None, *body.clone(), env.clone()))
            }

            Expression::StringInterpolation(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ish_ast::StringPart::Text(text) => result.push_str(text),
                        ish_ast::StringPart::Expr(expr) => {
                            let val = self.eval_expression(expr, env)?;
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
                        let resolved_args = self.resolve_shell_args(args, env)?;
                        let output = self.run_command_pipeline(command, &resolved_args, pipes, redirections, env, true)?;
                        Ok(Value::String(Rc::new(output.trim_end_matches('\n').to_string())))
                    }
                    _ => {
                        // Execute as normal statement, capture result
                        match self.exec_statement(inner, env)? {
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
                Err(RuntimeError::system_error(format!("incomplete expression: {:?}", kind), "E004"))
            }
        }
    }

    // ── Shell command execution ─────────────────────────────────────────────

    fn exec_shell_command(
        &mut self,
        command: &str,
        args: &[ShellArg],
        pipes: &[ShellPipeline],
        redirections: &[Redirection],
        env: &Environment,
    ) -> Result<ControlFlow, RuntimeError> {
        let resolved_args = self.resolve_shell_args(args, env)?;

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
                        Err(RuntimeError::system_error(format!("cd: {}: {}", target, e), "E010"))
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
                        Err(RuntimeError::system_error(format!("pwd: {}", e), "E010"))
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
                let output = self.run_command_pipeline(command, &resolved_args, pipes, redirections, env, false)?;
                if !output.is_empty() {
                    Ok(ControlFlow::ExprValue(Value::String(Rc::new(output))))
                } else {
                    Ok(ControlFlow::None)
                }
            }
        }
    }

    fn resolve_shell_args(
        &mut self,
        args: &[ShellArg],
        env: &Environment,
    ) -> Result<Vec<String>, RuntimeError> {
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
                            let sub_args = self.resolve_shell_args(args, env)?;
                            let output = self.run_command_pipeline(command, &sub_args, pipes, redirections, env, true)?;
                            resolved.push(output.trim_end_matches('\n').to_string());
                        }
                        _ => {
                            match self.exec_statement(inner, env)? {
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
    }

    fn run_command_pipeline(
        &mut self,
        command: &str,
        args: &[String],
        pipes: &[ShellPipeline],
        redirections: &[Redirection],
        env: &Environment,
        capture: bool,
    ) -> Result<String, RuntimeError> {
        // Build the first command
        let mut cmd = Command::new(command);
        cmd.args(args);

        if pipes.is_empty() {
            // Single command — handle redirections
            apply_redirections(&mut cmd, redirections)?;

            if capture {
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let output = cmd.output().map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", command, e), "E010")
                })?;

                let exit_code = output.status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let status = cmd.status().map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", command, e), "E010")
                })?;

                let exit_code = status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::new())
            }
        } else {
            // Pipeline: chain commands via stdin/stdout
            cmd.stdout(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| {
                RuntimeError::system_error(format!("{}: {}", command, e), "E010")
            })?;

            let mut prev_stdout = child.stdout.take();

            for (i, pipe) in pipes.iter().enumerate() {
                let is_last = i == pipes.len() - 1;
                let pipe_args = self.resolve_shell_args(&pipe.args, env)?;
                let mut next_cmd = Command::new(&pipe.command);
                next_cmd.args(&pipe_args);

                if let Some(stdout) = prev_stdout.take() {
                    next_cmd.stdin(stdout);
                }

                if !is_last || capture {
                    next_cmd.stdout(Stdio::piped());
                }

                if is_last {
                    apply_redirections(&mut next_cmd, redirections)?;
                }

                let mut next_child = next_cmd.spawn().map_err(|e| {
                    RuntimeError::system_error(format!("{}: {}", pipe.command, e), "E010")
                })?;

                prev_stdout = next_child.stdout.take();

                if is_last {
                    let output = next_child.wait_with_output().map_err(|e| {
                        RuntimeError::system_error(format!("{}: {}", pipe.command, e), "E010")
                    })?;
                    let exit_code = output.status.code().unwrap_or(-1) as i64;
                    env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                    // Wait for the first command too
                    let _ = child.wait();

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
            let _ = child.wait();
            Ok(String::new())
        }
    }

    /// Call a function value with the given arguments.
    pub fn call_function(
        &mut self,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Function(f) => {
                if args.len() != f.params.len() {
                    return Err(RuntimeError::system_error(format!(
                        "function '{}' expected {} arguments, got {}",
                        f.name.as_deref().unwrap_or("anonymous"),
                        f.params.len(),
                        args.len()
                    ), "E003"));
                }
                // Audit parameter types against annotations (when types feature active).
                for (i, (param_name, arg)) in f.params.iter().zip(args.iter()).enumerate() {
                    let param_type = f.param_types.get(i).and_then(|t| t.as_ref());
                    self.audit_type_annotation(param_name, arg, param_type)?;
                }
                // Create a new scope from the closure environment
                let call_env = f.closure_env.child();
                for (param, arg) in f.params.iter().zip(args.iter()) {
                    call_env.define(param.clone(), arg.clone());
                }
                // Function-scoped defer: push a defer frame, run body,
                // then execute all deferred statements before returning.
                self.push_defer_frame();
                let result = self.exec_statement(&f.body, &call_env);
                self.pop_and_run_defers();
                let return_val = match result? {
                    ControlFlow::Return(v) => v,
                    ControlFlow::ExprValue(v) => v,
                    ControlFlow::None => Value::Null,
                    // Per proposal: throw does not cross function boundaries.
                    // A thrown value that escapes a function body is re-thrown
                    // by the default return handler (streamlined mode).
                    ControlFlow::Throw(v) => {
                        return Err(RuntimeError::thrown(v));
                    }
                };
                // Audit return type.
                let fn_name = f.name.as_deref().unwrap_or("anonymous");
                self.audit_type_annotation(
                    &format!("return of '{fn_name}'"),
                    &return_val,
                    f.return_type.as_ref(),
                )?;
                Ok(return_val)
            }
            Value::BuiltinFunction(b) => match b.name.as_str() {
                "active_standard" => self.builtin_active_standard(args),
                "feature_state" => self.builtin_feature_state(args),
                "has_standard" => self.builtin_has_standard(args),
                "has_entry_type" => self.builtin_has_entry_type(args),
                _ => (b.func)(args),
            },
            _ => Err(RuntimeError::system_error(format!(
                "cannot call {}",
                func.type_name()
            ), "E006")),
        }
    }

    // ── Type audit helper ─────────────────────────────────────────────────

    /// Audit a value against a type annotation, respecting the active standard.
    ///
    /// - If no standard is active (streamlined), no checking is performed.
    /// - If types feature is `required` and annotation is missing → error.
    /// - If annotation is present and value doesn't match → error.
    fn audit_type_annotation(
        &self,
        item_name: &str,
        value: &Value,
        annotation: Option<&ish_ast::TypeAnnotation>,
    ) -> Result<(), RuntimeError> {
        let features = self.ledger.active_features();
        let types_feature = match features.get("types") {
            Some(fs) => fs,
            None => return Ok(()), // no types feature active → no checking
        };

        match annotation {
            Some(type_ann) => {
                // Annotation present — check compatibility.
                if !crate::ledger::type_compat::is_compatible(value, type_ann) {
                    return Err(RuntimeError::system_error(format!(
                        "type mismatch for '{}': expected {:?}, got {}",
                        item_name,
                        type_ann,
                        value.type_name()
                    ), "E004"));
                }
            }
            None => {
                // No annotation — only an error if annotation is required.
                if types_feature.annotation
                    == crate::ledger::standard::AnnotationDimension::Required
                {
                    return Err(RuntimeError::system_error(format!(
                        "missing type annotation for '{}' (required by active standard)",
                        item_name
                    ), "E004"));
                }
            }
        }

        Ok(())
    }

    // ── Ledger query builtins (need &self for ledger access) ────────────────

    /// active_standard() -> String | null
    fn builtin_active_standard(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if !args.is_empty() {
            return Err(RuntimeError::system_error("active_standard expects 0 arguments", "E010"));
        }
        match self.ledger.active_standard() {
            Some(name) => Ok(Value::String(Rc::new(name.to_string()))),
            None => Ok(Value::Null),
        }
    }

    /// feature_state(feature_name) -> String description | null
    fn builtin_feature_state(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("feature_state expects 1 argument", "E010"));
        }
        if let Value::String(name) = &args[0] {
            match self.ledger.feature_state(name) {
                Some(state) => {
                    let ann = match state.annotation {
                        crate::ledger::standard::AnnotationDimension::Optional => "optional",
                        crate::ledger::standard::AnnotationDimension::Required => "required",
                    };
                    let aud = match state.audit {
                        crate::ledger::standard::AuditDimension::Runtime => "runtime",
                        crate::ledger::standard::AuditDimension::Build => "build",
                    };
                    let desc = format!("{}/{}", ann, aud);
                    Ok(Value::String(Rc::new(desc)))
                }
                None => Ok(Value::Null),
            }
        } else {
            Err(RuntimeError::system_error("feature_state expects a string argument", "E004"))
        }
    }

    /// has_standard(name) -> Bool
    fn builtin_has_standard(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("has_standard expects 1 argument", "E010"));
        }
        if let Value::String(name) = &args[0] {
            Ok(Value::Bool(self.ledger.standard_registry.get(name).is_some()))
        } else {
            Err(RuntimeError::system_error("has_standard expects a string argument", "E004"))
        }
    }

    /// has_entry_type(name) -> Bool
    fn builtin_has_entry_type(&self, args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("has_entry_type expects 1 argument", "E010"));
        }
        if let Value::String(name) = &args[0] {
            Ok(Value::Bool(self.ledger.entry_type_registry.get(name).is_some()))
        } else {
            Err(RuntimeError::system_error("has_entry_type expects a string argument", "E004"))
        }
    }

    /// Evaluate a binary operation.
    fn eval_binary_op(
        &self,
        op: &BinaryOperator,
        lhs: &Value,
        rhs: &Value,
    ) -> Result<Value, RuntimeError> {
        match op {
            // Arithmetic
            BinaryOperator::Add => self.add(lhs, rhs),
            BinaryOperator::Sub => self.arith(lhs, rhs, |a, b| a - b, |a, b| a - b),
            BinaryOperator::Mul => self.arith(lhs, rhs, |a, b| a * b, |a, b| a * b),
            BinaryOperator::Div => {
                // Check for division by zero
                match rhs {
                    Value::Int(0) => return Err(RuntimeError::system_error("division by zero", "E002")),
                    Value::Float(f) if *f == 0.0 => {
                        return Err(RuntimeError::system_error("division by zero", "E002"))
                    }
                    _ => {}
                }
                self.arith(lhs, rhs, |a, b| a / b, |a, b| a / b)
            }
            BinaryOperator::Mod => {
                match rhs {
                    Value::Int(0) => return Err(RuntimeError::system_error("modulo by zero", "E002")),
                    _ => {}
                }
                self.arith(lhs, rhs, |a, b| a % b, |a, b| a % b)
            }

            // Comparison
            BinaryOperator::Eq => Ok(Value::Bool(lhs == rhs)),
            BinaryOperator::NotEq => Ok(Value::Bool(lhs != rhs)),
            BinaryOperator::Lt => self.compare(lhs, rhs, |o| o.is_lt()),
            BinaryOperator::Gt => self.compare(lhs, rhs, |o| o.is_gt()),
            BinaryOperator::LtEq => self.compare(lhs, rhs, |o| !o.is_gt()),
            BinaryOperator::GtEq => self.compare(lhs, rhs, |o| !o.is_lt()),

            // Logical (handled above via short-circuit, but just in case)
            BinaryOperator::And | BinaryOperator::Or => {
                unreachable!("logical ops handled in eval_expression")
            }
        }
    }

    fn add(&self, lhs: &Value, rhs: &Value) -> Result<Value, RuntimeError> {
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
            ), "E004")),
        }
    }

    fn arith(
        &self,
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
            ), "E004")),
        }
    }

    fn compare(
        &self,
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
                ), "E004"))
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

// ── Shell helpers ───────────────────────────────────────────────────────────

fn apply_redirections(cmd: &mut Command, redirections: &[Redirection]) -> Result<(), RuntimeError> {
    use std::fs::{File, OpenOptions};
    for redir in redirections {
        match redir.kind {
            RedirectKind::StdoutWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), "E008")
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StdoutAppend => {
                let f = OpenOptions::new().create(true).append(true).open(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), "E008")
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StderrWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), "E008")
                })?;
                cmd.stderr(f);
            }
            RedirectKind::StderrAndStdout => {
                // 2>&1 — merge stderr into stdout (Stdio::piped or inherit)
                cmd.stderr(Stdio::inherit());
            }
            RedirectKind::AllWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}: {}", redir.target, e), "E008")
                })?;
                let f2 = f.try_clone().map_err(|e| {
                    RuntimeError::system_error(format!("redirect: {}", e), "E008")
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

    fn run_source(source: &str) -> Value {
        let program = parse(source).unwrap_or_else(|errs| {
            panic!("parse failed: {:?}", errs)
        });

        let mut vm = IshVm::new();
        vm.run(&program).unwrap()
    }

    #[test]
    fn test_variable_decl_and_lookup() {
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(42))
            .expr_stmt(Expression::ident("x"))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_arithmetic() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::int(10),
                Expression::int(32),
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_function_declaration_and_call() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_factorial_recursive() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(3628800));
    }

    #[test]
    fn test_closures() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_objects_and_lists() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(13));
    }

    #[test]
    fn test_while_loop() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(55));
    }

    #[test]
    fn test_nested_scoping() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_string_concatenation() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::string("hello "),
                Expression::string("world"),
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("hello world".into())));
    }

    #[test]
    fn test_double_quoted_string_interpolates_expression_and_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_DQ", "alpha");

        let result = run_source(
            r#"
            let x = 40;
            "value {x + 2} ${ISH_VM_TEST_INTERP_DQ}"
            "#,
        );

        assert_eq!(result, Value::String(Rc::new("value 42 alpha".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_DQ");
    }

    #[test]
    fn test_double_quoted_string_interpolates_expression_and_bare_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_DQ_BARE", "beta");

        let result = run_source(
            r#"
            let x = 6;
            "sum {x * 7} $ISH_VM_TEST_INTERP_DQ_BARE"
            "#,
        );

        assert_eq!(result, Value::String(Rc::new("sum 42 beta".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_DQ_BARE");
    }

    #[test]
    fn test_triple_double_string_interpolates_expression_and_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_TDQ", "gamma");

        let result = run_source(
            r#"
            let x = 41;
            """triple {x + 1} ${ISH_VM_TEST_INTERP_TDQ}"""
            "#,
        );

        assert_eq!(result, Value::String(Rc::new("triple 42 gamma".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_TDQ");
    }

    #[test]
    fn test_triple_double_string_interpolates_expression_and_bare_env_var() {
        std::env::set_var("ISH_VM_TEST_INTERP_TDQ_BARE", "delta");

        let result = run_source(
            r#"
            let x = 21;
            """triple {x * 2} $ISH_VM_TEST_INTERP_TDQ_BARE"""
            "#,
        );

        assert_eq!(result, Value::String(Rc::new("triple 42 delta".into())));
        std::env::remove_var("ISH_VM_TEST_INTERP_TDQ_BARE");
    }

    // ── Error handling tests ────────────────────────────────────────────

    #[test]
    fn test_throw_unhandled_becomes_error() {
        // throw "boom"; -> should produce a RuntimeError
        let program = ProgramBuilder::new()
            .stmt(Statement::throw(Expression::string("boom")))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("boom"));
    }

    #[test]
    fn test_try_catch_basic() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        // The result of the program is Null because try_catch itself
        // returns ControlFlow::None at the top level. The caught value
        // doesn't propagate as the program result.
        // Let's verify it doesn't error:
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_try_catch_returns_caught_value() {
        // fn test() { try { throw 42; } catch(e) { return e; } }
        // test()  -> 42
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.throw(Expression::int(42)),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::ident("e"))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("test"), vec![]))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_try_finally_runs_on_normal() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(11));
    }

    #[test]
    fn test_try_finally_runs_on_throw() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(11));
    }

    #[test]
    fn test_throw_does_not_cross_function_boundary() {
        // fn bad() { throw "error"; }
        // bad()  -> should produce RuntimeError
        let program = ProgramBuilder::new()
            .function("bad", &[], |b| {
                b.throw(Expression::string("error"))
            })
            .expr_stmt(Expression::call(Expression::ident("bad"), vec![]))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_throw_from_function_caught_by_caller() {
        // fn bad() { throw 99; }
        // fn wrapper() {
        //   try { bad(); } catch(e) { return e; }
        // }
        // wrapper()  -> 99
        let program = ProgramBuilder::new()
            .function("bad", &[], |b| {
                b.throw(Expression::int(99))
            })
            .function("wrapper", &[], |b| {
                b.try_catch(
                    |b| b.expr_stmt(Expression::call(Expression::ident("bad"), vec![])),
                    vec![CatchClause::new("e", Statement::block(vec![
                        Statement::ret(Some(Expression::ident("e"))),
                    ]))],
                    None::<fn(BlockBuilder) -> BlockBuilder>,
                )
            })
            .expr_stmt(Expression::call(Expression::ident("wrapper"), vec![]))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_with_block_calls_close() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_with_block_calls_close_on_throw() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_defer_executes_at_function_exit() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String(Rc::new("body".into())));
            assert_eq!(list[1], Value::String(Rc::new("deferred".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[test]
    fn test_defer_lifo_order() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        if let Value::List(ref list_ref) = result {
            let list = list_ref.borrow();
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String(Rc::new("second".into())));
            assert_eq!(list[1], Value::String(Rc::new("first".into())));
        } else {
            panic!("expected list, got {:?}", result);
        }
    }

    #[test]
    fn test_defer_function_scoped() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
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

    #[test]
    fn test_defer_loop_accumulates() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
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

    #[test]
    fn test_defer_lambda_boundary() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
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

    #[test]
    fn test_is_error_builtin() {
        // is_error({ message: "test" })  -> true
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::string("test message")),
                ])],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_throw_error_caught_with_message() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("boom".into())));
    }

    #[test]
    fn test_try_catch_no_throw_runs_normally() {
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

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    // ── Char literal / Value::Char tests ────────────────────────────────

    #[test]
    fn test_char_literal() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::char_lit('A'))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[test]
    fn test_char_equality() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Eq,
                Expression::char_lit('A'),
                Expression::char_lit('A'),
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_comparison() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Lt,
                Expression::char_lit('A'),
                Expression::char_lit('B'),
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_concatenation() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::binary(
                BinaryOperator::Add,
                Expression::char_lit('H'),
                Expression::char_lit('i'),
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("Hi".into())));
    }

    #[test]
    fn test_char_builtin_from_string() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("char"),
                vec![Expression::string("A")],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[test]
    fn test_char_builtin_from_int() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("char"),
                vec![Expression::int(65)],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Char('A'));
    }

    #[test]
    fn test_char_display() {
        let val = Value::Char('X');
        assert_eq!(val.to_display_string(), "X");
    }

    #[test]
    fn test_char_type_name() {
        let val = Value::Char('A');
        assert_eq!(val.type_name(), "char");
    }

    // ── Type audit tests ──────────────────────────────────────

    fn run_source_err(source: &str) -> String {
        let program = parse(source).unwrap_or_else(|errs| {
            panic!("parse failed: {:?}", errs)
        });
        let mut vm = IshVm::new();
        match vm.run(&program) {
            Err(e) => format!("{}", e),
            Ok(v) => panic!("expected error, got: {:?}", v),
        }
    }

    #[test]
    fn type_audit_correct_annotation_passes() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 = 42
x
"#);
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn type_audit_wrong_annotation_fails() {
        let err = run_source_err(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: String = 42
"#);
        assert!(err.contains("type"), "expected type error, got: {}", err);
    }

    #[test]
    fn type_audit_required_missing_annotation_fails() {
        let err = run_source_err(r#"
standard strict_std [
    types(required, runtime)
]
@standard[strict_std]
let x = 42
"#);
        assert!(err.contains("annotation") || err.contains("type"),
                "expected missing annotation error, got: {}", err);
    }

    #[test]
    fn type_audit_function_param_type_check() {
        let err = run_source_err(r#"
standard typed_std [
    types(optional, runtime)
]
fn greet(name: String) {
    name
}
@standard[typed_std]
let r = greet(42)
"#);
        assert!(err.contains("type"), "expected type error, got: {}", err);
    }

    #[test]
    fn type_audit_function_param_correct() {
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
"#);
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn type_audit_return_type_check() {
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
"#);
        assert!(err.contains("type"), "expected return type error, got: {}", err);
    }

    #[test]
    fn type_audit_return_type_correct() {
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
"#);
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn type_audit_nullable_annotation_accepts_null() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
x
"#);
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn type_audit_union_annotation() {
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | String = "hello"
x
"#);
        assert_eq!(result, Value::String(Rc::new("hello".into())));
    }

    // ── Narrowing tests ───────────────────────────────────────

    #[test]
    fn narrowing_null_check_does_not_crash() {
        // Smoke test: narrowing wiring runs without panicking under types feature.
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
let x = 42
@standard[typed_std]
let r: i32 = x
r
"#);
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn narrowing_if_true_branch() {
        // When condition is true, true branch executes.
        let result = run_source(r#"
standard typed_std [
    types(optional, runtime)
]
let x = 10
if x != null {
    println("not null")
}
x
"#);
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn narrowing_if_else_branch() {
        // When condition is false, else branch executes.
        let result = run_source(r#"
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
"#);
        assert_eq!(result, Value::Null);
    }

    // ── Error model tests (TODO 42) ────────────────────────────────────

    #[test]
    fn system_error_has_message_and_code() {
        // RuntimeError::system_error creates an object with message and code
        let err = RuntimeError::system_error("test message", "E001");
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

    #[test]
    fn is_error_true_for_object_with_message() {
        // is_error({ message: "x" }) -> true
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::string("x")),
                ])],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Bool(true));
    }

    #[test]
    fn is_error_false_for_plain_object() {
        // is_error({ name: "x" }) -> false (no message property)
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("name", Expression::string("x")),
                ])],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Bool(false));
    }

    #[test]
    fn is_error_false_for_non_string_message() {
        // is_error({ message: 42 }) -> false (message must be String)
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::object(vec![
                    ("message", Expression::int(42)),
                ])],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Bool(false));
    }

    #[test]
    fn is_error_false_for_non_object() {
        // is_error("hello") -> false
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::string("hello")],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Bool(false));
    }

    #[test]
    fn error_message_extracts_message() {
        // error_message({ message: "oops" }) -> "oops"
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_message"),
                vec![Expression::object(vec![
                    ("message", Expression::string("oops")),
                ])],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::String(Rc::new("oops".into())));
    }

    #[test]
    fn error_code_extracts_code() {
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
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::String(Rc::new("E002".into())));
    }

    #[test]
    fn error_code_returns_null_for_uncoded_error() {
        // error_code({ message: "x" }) -> null
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_code"),
                vec![Expression::object(vec![
                    ("message", Expression::string("x")),
                ])],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Null);
    }

    #[test]
    fn error_code_returns_null_for_non_object() {
        // error_code(42) -> null
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("error_code"),
                vec![Expression::int(42)],
            ))
            .build();
        let mut vm = IshVm::new();
        assert_eq!(vm.run(&program).unwrap(), Value::Null);
    }

    #[test]
    fn throw_audit_accepts_object_with_message_under_types() {
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
"#);
        assert_eq!(result, Value::String(Rc::new("throw ok".into())));
    }

    #[test]
    fn throw_audit_rejects_object_without_message_under_types() {
        // Under a standard with types feature, throwing { name: "bad" } should fail.
        let program = ProgramBuilder::new()
            .stmt(Statement::throw(Expression::object(vec![
                ("name", Expression::string("bad")),
            ])))
            .build();
        let mut vm = IshVm::new();
        // Use optional annotations so throw audit runs without type annotation errors
        vm.ledger.standard_registry.register(
            super::super::ledger::standard::Standard::new("audit_types")
                .with_feature("types", super::super::ledger::standard::FeatureState::new(
                    super::super::ledger::standard::AnnotationDimension::Optional,
                    super::super::ledger::standard::AuditDimension::Runtime,
                ))
        );
        vm.ledger.push_standard("audit_types".to_string());
        let result = vm.run(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("throw audit"));
    }

    #[test]
    fn throw_audit_skipped_without_types_feature() {
        // Without a standard, throwing any value is allowed (no audit).
        // throw "plain string" -> RuntimeError but no audit complaint
        let program = ProgramBuilder::new()
            .stmt(Statement::throw(Expression::string("plain")))
            .build();
        let mut vm = IshVm::new();
        let result = vm.run(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // The error is an unhandled throw, not an audit failure
        assert!(err.message.contains("plain"));
        assert!(!err.message.contains("throw audit"));
    }

    #[test]
    fn caught_system_error_has_code() {
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
        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("E002".into())));
    }

    #[test]
    fn caught_system_error_is_error_true() {
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
        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn caught_system_error_has_message() {
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
        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        // Division by zero message
        if let Value::String(ref s) = result {
            assert!(s.contains("zero") || s.contains("division"),
                "expected division-by-zero message, got: {}", s);
        } else {
            panic!("expected string, got {:?}", result);
        }
    }
}
