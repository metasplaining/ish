use std::collections::HashMap;
use std::rc::Rc;

use ish_ast::*;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::value::*;
use crate::builtins;

/// Signal for control flow: normal completion, return, or break.
enum ControlFlow {
    None,
    Return(Value),
    /// The value produced by the last expression statement.
    ExprValue(Value),
}

/// The ish virtual machine / interpreter.
pub struct IshVm {
    pub global_env: Environment,
}

impl IshVm {
    pub fn new() -> Self {
        let env = Environment::new();
        builtins::register_all(&env);
        crate::reflection::register_ast_builtins(&env);
        IshVm { global_env: env }
    }

    /// Execute a full program.
    pub fn run(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last = Value::Null;
        let env = self.global_env.clone();
        for stmt in &program.statements {
            match self.exec_statement(stmt, &env)? {
                ControlFlow::Return(v) => return Ok(v),
                ControlFlow::ExprValue(v) => last = v,
                ControlFlow::None => {}
            }
        }
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
                for s in statements {
                    match self.exec_statement(s, &block_env)? {
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::None | ControlFlow::ExprValue(_) => {}
                    }
                }
                Ok(ControlFlow::None)
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
        }
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
                match self.exec_statement(&f.body, &call_env)? {
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::ExprValue(v) => Ok(v),
                    ControlFlow::None => Ok(Value::Null),
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::builder::ProgramBuilder;

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
}
