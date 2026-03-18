use std::collections::HashMap;
use std::rc::Rc;
use std::process::{Command, Stdio};

use ish_ast::*;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::value::*;
use crate::builtins;

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
}

impl IshVm {
    pub fn new() -> Self {
        let env = Environment::new();
        builtins::register_all(&env);
        crate::reflection::register_ast_builtins(&env);
        IshVm {
            global_env: env,
            defer_stack: Vec::new(),
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
                    return Err(RuntimeError::new(format!(
                        "Unhandled throw: {}", v.to_display_string()
                    )));
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
            Statement::VariableDecl { name, value, .. } => {
                let val = self.eval_expression(value, env)?;
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
                            return Err(RuntimeError::new(format!(
                                "cannot set property '{}' on {}",
                                property,
                                obj.type_name()
                            )));
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
                                    return Err(RuntimeError::new(format!(
                                        "index {} out of bounds (length {})",
                                        i, len
                                    )));
                                }
                                list[i as usize] = val;
                            } else {
                                return Err(RuntimeError::new("list index must be an integer"));
                            }
                        } else if let Value::Object(ref obj_ref) = obj {
                            if let Value::String(ref key) = idx {
                                obj_ref.borrow_mut().insert(key.as_ref().clone(), val);
                            } else {
                                return Err(RuntimeError::new(
                                    "object index must be a string",
                                ));
                            }
                        } else {
                            return Err(RuntimeError::new(format!(
                                "cannot index into {}",
                                obj.type_name()
                            )));
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
                if cond.is_truthy() {
                    self.exec_statement(then_block, env)
                } else if let Some(eb) = else_block {
                    self.exec_statement(eb, env)
                } else {
                    Ok(ControlFlow::None)
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
                    return Err(RuntimeError::new(format!(
                        "cannot iterate over {}",
                        iter_val.type_name()
                    )));
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
                name, params, body, ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let func = new_function(
                    Some(name.clone()),
                    param_names,
                    *body.clone(),
                    env.clone(),
                );
                env.define(name.clone(), func);
                Ok(ControlFlow::None)
            }

            Statement::Throw { value } => {
                let val = self.eval_expression(value, env)?;
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

            Statement::Annotated { inner, .. } => {
                // Execute the inner statement; annotations are metadata
                self.exec_statement(inner, env)
            }

            Statement::StandardDef { .. } | Statement::EntryTypeDef { .. } => {
                // Standard and entry type definitions are declarative metadata
                Ok(ControlFlow::None)
            }

            Statement::Match { .. } => {
                // Match not yet implemented in interpreter
                Ok(ControlFlow::None)
            }

            Statement::Incomplete { kind } => {
                Err(RuntimeError::new(format!("incomplete input: {:?}", kind)))
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
                        _ => Err(RuntimeError::new(format!(
                            "cannot negate {}",
                            val.type_name()
                        ))),
                    },
                    UnaryOperator::Try => {
                        // ? operator: if value is an error, propagate it; otherwise unwrap
                        // For now, null signals error
                        if val == Value::Null {
                            return Err(RuntimeError::new("tried to unwrap null value with ?".to_string()));
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
                    _ => Err(RuntimeError::new(format!(
                        "cannot access property '{}' on {}",
                        property,
                        obj.type_name()
                    ))),
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
                            return Err(RuntimeError::new(format!(
                                "index {} out of bounds (length {})",
                                i,
                                list.len()
                            )));
                        }
                        Ok(list[i as usize].clone())
                    }
                    (Value::Object(obj_ref), Value::String(key)) => {
                        let map = obj_ref.borrow();
                        Ok(map.get(key.as_ref()).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::new(format!(
                        "cannot index {} with {}",
                        obj.type_name(),
                        idx.type_name()
                    ))),
                }
            }

            Expression::Lambda { params, body } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                Ok(new_function(None, param_names, *body.clone(), env.clone()))
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
                Err(RuntimeError::new(format!("incomplete expression: {:?}", kind)))
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
                        Err(RuntimeError::new(format!("cd: {}: {}", target, e)))
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
                        Err(RuntimeError::new(format!("pwd: {}", e)))
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
                    RuntimeError::new(format!("{}: {}", command, e))
                })?;

                let exit_code = output.status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let status = cmd.status().map_err(|e| {
                    RuntimeError::new(format!("{}: {}", command, e))
                })?;

                let exit_code = status.code().unwrap_or(-1) as i64;
                env.define("__ish_last_exit_code".to_string(), Value::Int(exit_code));

                Ok(String::new())
            }
        } else {
            // Pipeline: chain commands via stdin/stdout
            cmd.stdout(Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| {
                RuntimeError::new(format!("{}: {}", command, e))
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
                    RuntimeError::new(format!("{}: {}", pipe.command, e))
                })?;

                prev_stdout = next_child.stdout.take();

                if is_last {
                    let output = next_child.wait_with_output().map_err(|e| {
                        RuntimeError::new(format!("{}: {}", pipe.command, e))
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
                    return Err(RuntimeError::new(format!(
                        "function '{}' expected {} arguments, got {}",
                        f.name.as_deref().unwrap_or("anonymous"),
                        f.params.len(),
                        args.len()
                    )));
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
                match result? {
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::ExprValue(v) => Ok(v),
                    ControlFlow::None => Ok(Value::Null),
                    // Per proposal: throw does not cross function boundaries.
                    // A thrown value that escapes a function body is re-thrown
                    // by the default return handler (streamlined mode).
                    ControlFlow::Throw(v) => {
                        Err(RuntimeError::thrown(v))
                    }
                }
            }
            Value::BuiltinFunction(b) => (b.func)(args),
            _ => Err(RuntimeError::new(format!(
                "cannot call {}",
                func.type_name()
            ))),
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
                    Value::Int(0) => return Err(RuntimeError::new("division by zero")),
                    Value::Float(f) if *f == 0.0 => {
                        return Err(RuntimeError::new("division by zero"))
                    }
                    _ => {}
                }
                self.arith(lhs, rhs, |a, b| a / b, |a, b| a / b)
            }
            BinaryOperator::Mod => {
                match rhs {
                    Value::Int(0) => return Err(RuntimeError::new("modulo by zero")),
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
            _ => Err(RuntimeError::new(format!(
                "cannot add {} and {}",
                lhs.type_name(),
                rhs.type_name()
            ))),
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
            _ => Err(RuntimeError::new(format!(
                "cannot perform arithmetic on {} and {}",
                lhs.type_name(),
                rhs.type_name()
            ))),
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
                return Err(RuntimeError::new(format!(
                    "cannot compare {} and {}",
                    lhs.type_name(),
                    rhs.type_name()
                )))
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

// ── Shell helpers ───────────────────────────────────────────────────────────

fn apply_redirections(cmd: &mut Command, redirections: &[Redirection]) -> Result<(), RuntimeError> {
    use std::fs::{File, OpenOptions};
    for redir in redirections {
        match redir.kind {
            RedirectKind::StdoutWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::new(format!("redirect: {}: {}", redir.target, e))
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StdoutAppend => {
                let f = OpenOptions::new().create(true).append(true).open(&redir.target).map_err(|e| {
                    RuntimeError::new(format!("redirect: {}: {}", redir.target, e))
                })?;
                cmd.stdout(f);
            }
            RedirectKind::StderrWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::new(format!("redirect: {}: {}", redir.target, e))
                })?;
                cmd.stderr(f);
            }
            RedirectKind::StderrAndStdout => {
                // 2>&1 — merge stderr into stdout (Stdio::piped or inherit)
                cmd.stderr(Stdio::inherit());
            }
            RedirectKind::AllWrite => {
                let f = File::create(&redir.target).map_err(|e| {
                    RuntimeError::new(format!("redirect: {}: {}", redir.target, e))
                })?;
                let f2 = f.try_clone().map_err(|e| {
                    RuntimeError::new(format!("redirect: {}", e))
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
    fn test_new_error_builtin() {
        // is_error(new_error("test"))  -> true
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("is_error"),
                vec![Expression::call(
                    Expression::ident("new_error"),
                    vec![Expression::string("test message")],
                )],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_throw_error_caught_with_message() {
        // fn test() {
        //   try { throw new_error("boom"); }
        //   catch (e) { return error_message(e); }
        // }
        // test()  -> "boom"
        let program = ProgramBuilder::new()
            .function("test", &[], |b| {
                b.try_catch(
                    |b| b.throw(Expression::call(
                        Expression::ident("new_error"),
                        vec![Expression::string("boom")],
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
}
