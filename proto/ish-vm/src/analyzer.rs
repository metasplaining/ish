// Code analyzer: classifies functions as yielding or unyielding at declaration time.
//
// The stub analyzer walks a function body looking for yielding AST nodes:
// - Expression::Await, Expression::Spawn, Statement::Yield, Expression::CommandSubstitution
// - Statement::ShellCommand (shell commands require async execution)
// - Expression::FunctionCall where the callee is a known yielding function
//
// It does NOT recurse into nested FunctionDecl or Lambda bodies.

use ish_ast::{Expression, Statement};

use crate::environment::Environment;
use crate::error::{ErrorCode, RuntimeError};
use crate::value::Value;

/// Result of classifying a function's yielding behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldingClassification {
    Yielding,
    Unyielding,
}

/// Classify a function as yielding or unyielding.
///
/// If `is_async` is true, returns `Yielding` immediately without walking the body.
/// Otherwise, walks the body looking for yielding nodes and function calls to
/// known yielding functions in `env`.
///
/// `param_names` are treated as defined-but-unclassified: if the body calls a
/// parameter by name, the call is treated as unyielding (conservative assumption
/// for the stub analyzer — indirect calls are a known limitation).
pub fn classify_function(
    body: &Statement,
    is_async: bool,
    env: &Environment,
    param_names: &[String],
) -> Result<YieldingClassification, RuntimeError> {
    if is_async {
        return Ok(YieldingClassification::Yielding);
    }
    // Create a child environment with parameters defined as Null.
    // This prevents the analyzer from erroring on calls to parameter names.
    let analysis_env = env.child();
    for name in param_names {
        analysis_env.define(name.clone(), Value::Null);
    }
    if contains_yielding_node(body, &analysis_env)? {
        Ok(YieldingClassification::Yielding)
    } else {
        Ok(YieldingClassification::Unyielding)
    }
}

/// Walk a statement looking for yielding nodes. Does not recurse into
/// nested FunctionDecl or Lambda bodies.
fn contains_yielding_node(stmt: &Statement, env: &Environment) -> Result<bool, RuntimeError> {
    match stmt {
        Statement::Yield => Ok(true),

        Statement::ExpressionStmt(expr) => expr_contains_yielding(expr, env),

        Statement::VariableDecl { value, .. } => expr_contains_yielding(value, env),

        Statement::Assignment { value, .. } => expr_contains_yielding(value, env),

        Statement::Block { statements } => {
            for s in statements {
                if contains_yielding_node(s, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            if expr_contains_yielding(condition, env)? {
                return Ok(true);
            }
            if contains_yielding_node(then_block, env)? {
                return Ok(true);
            }
            if let Some(eb) = else_block {
                if contains_yielding_node(eb, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Statement::While {
            condition, body, ..
        } => {
            if expr_contains_yielding(condition, env)? {
                return Ok(true);
            }
            contains_yielding_node(body, env)
        }

        Statement::ForEach {
            iterable, body, ..
        } => {
            if expr_contains_yielding(iterable, env)? {
                return Ok(true);
            }
            contains_yielding_node(body, env)
        }

        Statement::Return { value } => {
            if let Some(expr) = value {
                expr_contains_yielding(expr, env)
            } else {
                Ok(false)
            }
        }

        Statement::Throw { value } => expr_contains_yielding(value, env),

        Statement::TryCatch {
            body,
            catches,
            finally,
        } => {
            if contains_yielding_node(body, env)? {
                return Ok(true);
            }
            for c in catches {
                if contains_yielding_node(&c.body, env)? {
                    return Ok(true);
                }
            }
            if let Some(f) = finally {
                if contains_yielding_node(f, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Statement::WithBlock { resources, body, .. } => {
            for (_, expr) in resources {
                if expr_contains_yielding(expr, env)? {
                    return Ok(true);
                }
            }
            contains_yielding_node(body, env)
        }

        Statement::Defer { body } => contains_yielding_node(body, env),

        Statement::Annotated { inner, .. } => contains_yielding_node(inner, env),

        Statement::Match { subject, arms } => {
            if expr_contains_yielding(subject, env)? {
                return Ok(true);
            }
            for arm in arms {
                if contains_yielding_node(&arm.body, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        // Do NOT recurse into nested function declarations.
        Statement::FunctionDecl { .. } => Ok(false),

        // Shell commands require async execution — they are yielding.
        Statement::ShellCommand { .. } => Ok(true),

        // Non-yielding statements.
        Statement::TypeAlias { .. }
        | Statement::Use { .. }
        | Statement::ModDecl { .. }
        | Statement::StandardDef { .. }
        | Statement::EntryTypeDef { .. }
        | Statement::Incomplete { .. } => Ok(false),
    }
}

/// Walk an expression looking for yielding nodes.
fn expr_contains_yielding(expr: &Expression, env: &Environment) -> Result<bool, RuntimeError> {
    match expr {
        // Direct yielding nodes.
        Expression::Await { .. } => Ok(true),
        Expression::Spawn { .. } => Ok(true),
        Expression::CommandSubstitution(_) => Ok(true),

        // Function call: check if callee is a known yielding function.
        Expression::FunctionCall { callee, args } => {
            // Check if callee itself contains yielding (e.g. method chain)
            if expr_contains_yielding(callee, env)? {
                return Ok(true);
            }
            // Check arguments
            for arg in args {
                if expr_contains_yielding(arg, env)? {
                    return Ok(true);
                }
            }
            // If callee is a direct identifier, look up in env to check yielding status
            if let Expression::Identifier(name) = callee.as_ref() {
                match env.get(name) {
                    Ok(Value::Function(ref f)) => {
                        if f.has_yielding_entry == Some(true) {
                            return Ok(true);
                        }
                    }
                    Ok(_) => {
                        // Non-function value — not yielding (will error at call time)
                    }
                    Err(_) => {
                        // Undefined variable — treat as unyielding (conservative).
                        // This handles forward references and variables not yet in scope.
                        // Known limitation: forward references to yielding functions
                        // may be misclassified as unyielding.
                    }
                }
            }
            Ok(false)
        }

        // Recursive cases.
        Expression::BinaryOp { left, right, .. } => {
            if expr_contains_yielding(left, env)? {
                return Ok(true);
            }
            expr_contains_yielding(right, env)
        }

        Expression::UnaryOp { operand, .. } => expr_contains_yielding(operand, env),

        Expression::ObjectLiteral(pairs) => {
            for (_, expr) in pairs {
                if expr_contains_yielding(expr, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Expression::ListLiteral(items) => {
            for item in items {
                if expr_contains_yielding(item, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        Expression::PropertyAccess { object, .. } => expr_contains_yielding(object, env),

        Expression::IndexAccess { object, index } => {
            if expr_contains_yielding(object, env)? {
                return Ok(true);
            }
            expr_contains_yielding(index, env)
        }

        Expression::StringInterpolation(parts) => {
            for part in parts {
                if let ish_ast::StringPart::Expr(e) = part {
                    if expr_contains_yielding(e, env)? {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }

        // Do NOT recurse into lambda bodies.
        Expression::Lambda { .. } => Ok(false),

        // Leaf expressions — not yielding.
        Expression::Literal(_)
        | Expression::Identifier(_)
        | Expression::EnvVar(_)
        | Expression::Incomplete { .. } => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::*;

    fn empty_env() -> Environment {
        Environment::new()
    }

    fn env_with_yielding_fn(name: &str) -> Environment {
        let env = Environment::new();
        let f = crate::value::new_compiled_function(
            name.to_string(),
            vec![],
            vec![],
            None,
            |_: &[Value]| Ok(Value::Null),
            Some(true),
        );
        env.define(name.to_string(), f);
        env
    }

    fn env_with_unyielding_fn(name: &str) -> Environment {
        let env = Environment::new();
        let f = crate::value::new_compiled_function(
            name.to_string(),
            vec![],
            vec![],
            None,
            |_: &[Value]| Ok(Value::Null),
            Some(false),
        );
        env.define(name.to_string(), f);
        env
    }

    #[test]
    fn async_fn_is_yielding() {
        let body = Statement::Block { statements: vec![] };
        let result = classify_function(&body, true, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn plain_fn_is_unyielding() {
        let body = Statement::Block {
            statements: vec![Statement::Return {
                value: Some(Expression::Literal(Literal::Int(42))),
            }],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Unyielding);
    }

    #[test]
    fn fn_with_await_is_yielding() {
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::Await {
                callee: Box::new(Expression::Identifier("work".into())),
                args: vec![],
            })],
        };
        // The callee "work" must be defined for the analyzer not to error
        let env = env_with_yielding_fn("work");
        let result = classify_function(&body, false, &env, &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn fn_with_spawn_is_yielding() {
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::Spawn {
                callee: Box::new(Expression::Identifier("work".into())),
                args: vec![],
            })],
        };
        let env = env_with_yielding_fn("work");
        let result = classify_function(&body, false, &env, &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn fn_with_yield_is_yielding() {
        let body = Statement::Block {
            statements: vec![Statement::Yield],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn fn_with_command_substitution_is_yielding() {
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(
                Expression::CommandSubstitution(Box::new(Statement::ShellCommand {
                    command: "echo".into(),
                    args: vec![],
                    pipes: vec![],
                    redirections: vec![],
                    background: false,
                })),
            )],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn nested_fn_decl_not_recursed() {
        // Parent body contains only a nested function decl with await inside.
        // The parent should still be unyielding.
        let inner_body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::Await {
                callee: Box::new(Expression::Identifier("work".into())),
                args: vec![],
            })],
        };
        let body = Statement::Block {
            statements: vec![Statement::FunctionDecl {
                name: "inner".into(),
                params: vec![],
                return_type: None,
                body: Box::new(inner_body),
                visibility: None,
                type_params: vec![],
                is_async: false,
            }],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Unyielding);
    }

    #[test]
    fn call_to_yielding_fn_propagates() {
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::FunctionCall {
                callee: Box::new(Expression::Identifier("async_fn".into())),
                args: vec![],
            })],
        };
        let env = env_with_yielding_fn("async_fn");
        let result = classify_function(&body, false, &env, &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }

    #[test]
    fn call_to_unyielding_fn_stays_unyielding() {
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::FunctionCall {
                callee: Box::new(Expression::Identifier("pure_fn".into())),
                args: vec![],
            })],
        };
        let env = env_with_unyielding_fn("pure_fn");
        let result = classify_function(&body, false, &env, &[]).unwrap();
        assert_eq!(result, YieldingClassification::Unyielding);
    }

    #[test]
    fn call_to_undefined_fn_is_unyielding() {
        // Undefined functions are treated as unyielding (conservative).
        // No error — forward references are a known limitation.
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::FunctionCall {
                callee: Box::new(Expression::Identifier("nonexistent".into())),
                args: vec![],
            })],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Unyielding);
    }

    #[test]
    fn call_to_parameter_name_is_unyielding() {
        // Calling a parameter by name should not error and should be
        // treated as unyielding (conservative assumption).
        let body = Statement::Block {
            statements: vec![Statement::ExpressionStmt(Expression::FunctionCall {
                callee: Box::new(Expression::Identifier("callback".into())),
                args: vec![],
            })],
        };
        let result = classify_function(
            &body,
            false,
            &empty_env(),
            &["callback".to_string()],
        )
        .unwrap();
        assert_eq!(result, YieldingClassification::Unyielding);
    }

    #[test]
    fn fn_with_shell_command_is_yielding() {
        let body = Statement::Block {
            statements: vec![Statement::ShellCommand {
                command: "echo".into(),
                args: vec![],
                pipes: vec![],
                redirections: vec![],
                background: false,
            }],
        };
        let result = classify_function(&body, false, &empty_env(), &[]).unwrap();
        assert_eq!(result, YieldingClassification::Yielding);
    }
}
