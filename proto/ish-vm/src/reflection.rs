// AST ↔ Value reflection: convert AST nodes to/from ish Object values.
//
// Every AST node becomes an Object with a "kind" discriminator string
// and node-specific properties. This enables ish programs (the analyzer,
// generator) to manipulate ASTs as plain data.

use std::collections::HashMap;
use std::rc::Rc;

use gc::{Gc, GcCell};
use ish_ast::*;

use crate::error::{ErrorCode, RuntimeError};
use crate::value::*;

// ── AST → Value ─────────────────────────────────────────────────────────────

/// Convert an ish AST Program to an ish Value (Object).
pub fn program_to_value(program: &Program) -> Value {
    let mut map = HashMap::new();
    map.insert("kind".to_string(), Value::String(Rc::new("program".into())));
    let stmts: Vec<Value> = program.statements.iter().map(stmt_to_value).collect();
    map.insert("statements".to_string(), new_list(stmts));
    Value::Object(Gc::new(GcCell::new(map)))
}

/// Convert a Statement to a Value.
pub fn stmt_to_value(stmt: &Statement) -> Value {
    let mut map = HashMap::new();
    match stmt {
        Statement::VariableDecl { name, value, .. } => {
            map.insert("kind".to_string(), str_val("var_decl"));
            map.insert("name".to_string(), str_val(name));
            map.insert("value".to_string(), expr_to_value(value));
        }
        Statement::Assignment { target, value } => {
            map.insert("kind".to_string(), str_val("assignment"));
            map.insert("target".to_string(), assign_target_to_value(target));
            map.insert("value".to_string(), expr_to_value(value));
        }
        Statement::Block { statements } => {
            map.insert("kind".to_string(), str_val("block"));
            let stmts: Vec<Value> = statements.iter().map(stmt_to_value).collect();
            map.insert("statements".to_string(), new_list(stmts));
        }
        Statement::If { condition, then_block, else_block } => {
            map.insert("kind".to_string(), str_val("if"));
            map.insert("condition".to_string(), expr_to_value(condition));
            map.insert("then_block".to_string(), stmt_to_value(then_block));
            if let Some(eb) = else_block {
                map.insert("else_block".to_string(), stmt_to_value(eb));
            } else {
                map.insert("else_block".to_string(), Value::Null);
            }
        }
        Statement::While { condition, body, .. } => {
            map.insert("kind".to_string(), str_val("while"));
            map.insert("condition".to_string(), expr_to_value(condition));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Statement::ForEach { variable, iterable, body, .. } => {
            map.insert("kind".to_string(), str_val("for_each"));
            map.insert("variable".to_string(), str_val(variable));
            map.insert("iterable".to_string(), expr_to_value(iterable));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Statement::Return { value } => {
            map.insert("kind".to_string(), str_val("return"));
            if let Some(expr) = value {
                map.insert("value".to_string(), expr_to_value(expr));
            } else {
                map.insert("value".to_string(), Value::Null);
            }
        }
        Statement::ExpressionStmt(expr) => {
            map.insert("kind".to_string(), str_val("expr_stmt"));
            map.insert("expression".to_string(), expr_to_value(expr));
        }
        Statement::FunctionDecl { name, params, return_type: _, body, .. } => {
            map.insert("kind".to_string(), str_val("function_decl"));
            map.insert("name".to_string(), str_val(name));
            let param_vals: Vec<Value> = params
                .iter()
                .map(|p| {
                    let mut pm = HashMap::new();
                    pm.insert("name".to_string(), str_val(&p.name));
                    Value::Object(Gc::new(GcCell::new(pm)))
                })
                .collect();
            map.insert("params".to_string(), new_list(param_vals));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Statement::Throw { value } => {
            map.insert("kind".to_string(), str_val("throw"));
            map.insert("value".to_string(), expr_to_value(value));
        }
        Statement::TryCatch { body, catches, finally } => {
            map.insert("kind".to_string(), str_val("try_catch"));
            map.insert("body".to_string(), stmt_to_value(body));
            let catch_vals: Vec<Value> = catches
                .iter()
                .map(|c| {
                    let mut cm = HashMap::new();
                    cm.insert("param".to_string(), str_val(&c.param));
                    cm.insert("body".to_string(), stmt_to_value(&c.body));
                    Value::Object(Gc::new(GcCell::new(cm)))
                })
                .collect();
            map.insert("catches".to_string(), new_list(catch_vals));
            if let Some(fin) = finally {
                map.insert("finally".to_string(), stmt_to_value(fin));
            } else {
                map.insert("finally".to_string(), Value::Null);
            }
        }
        Statement::WithBlock { resources, body } => {
            map.insert("kind".to_string(), str_val("with_block"));
            let res_vals: Vec<Value> = resources
                .iter()
                .map(|(name, expr)| {
                    let mut rm = HashMap::new();
                    rm.insert("name".to_string(), str_val(name));
                    rm.insert("value".to_string(), expr_to_value(expr));
                    Value::Object(Gc::new(GcCell::new(rm)))
                })
                .collect();
            map.insert("resources".to_string(), new_list(res_vals));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Statement::Defer { body } => {
            map.insert("kind".to_string(), str_val("defer"));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Statement::TypeAlias { name, definition: _, .. } => {
            map.insert("kind".to_string(), str_val("type_alias"));
            map.insert("name".to_string(), str_val(name));
        }
        Statement::Use { module_path, .. } => {
            map.insert("kind".to_string(), str_val("use"));
            let path_vals: Vec<Value> = module_path.iter().map(|s| str_val(s)).collect();
            map.insert("path".to_string(), new_list(path_vals));
        }
        Statement::DeclareBlock { .. } => {
            map.insert("kind".to_string(), str_val("declare_block"));
        }
        Statement::Bootstrap { .. } => {
            map.insert("kind".to_string(), str_val("bootstrap"));
        }
        Statement::ShellCommand { command, args, background, .. } => {
            map.insert("kind".to_string(), str_val("shell_command"));
            map.insert("command".to_string(), str_val(command));
            let arg_vals: Vec<Value> = args.iter().map(|a| match a {
                ish_ast::ShellArg::Bare(s) | ish_ast::ShellArg::Glob(s)
                | ish_ast::ShellArg::Quoted(s) | ish_ast::ShellArg::EnvVar(s) => str_val(s),
                ish_ast::ShellArg::CommandSub(cmd) => stmt_to_value(cmd),
            }).collect();
            map.insert("args".to_string(), new_list(arg_vals));
            map.insert("background".to_string(), Value::Bool(*background));
        }
        Statement::Annotated { inner, .. } => {
            map.insert("kind".to_string(), str_val("annotated"));
            map.insert("inner".to_string(), stmt_to_value(inner));
        }
        Statement::StandardDef { name, .. } => {
            map.insert("kind".to_string(), str_val("standard_def"));
            map.insert("name".to_string(), str_val(name));
        }
        Statement::EntryTypeDef { name, .. } => {
            map.insert("kind".to_string(), str_val("entry_type_def"));
            map.insert("name".to_string(), str_val(name));
        }
        Statement::Match { .. } => {
            map.insert("kind".to_string(), str_val("match"));
        }
        Statement::Incomplete { kind } => {
            map.insert("kind".to_string(), str_val("incomplete"));
            map.insert("incomplete_kind".to_string(), str_val(&format!("{:?}", kind)));
        }
        Statement::Yield => {
            map.insert("kind".to_string(), str_val("yield"));
        }
    }
    Value::Object(Gc::new(GcCell::new(map)))
}

/// Convert an Expression to a Value.
pub fn expr_to_value(expr: &Expression) -> Value {
    let mut map = HashMap::new();
    match expr {
        Expression::Literal(lit) => {
            map.insert("kind".to_string(), str_val("literal"));
            match lit {
                Literal::Bool(b) => {
                    map.insert("value".to_string(), Value::Bool(*b));
                    map.insert("literal_type".to_string(), str_val("bool"));
                }
                Literal::Int(n) => {
                    map.insert("value".to_string(), Value::Int(*n));
                    map.insert("literal_type".to_string(), str_val("int"));
                }
                Literal::Float(f) => {
                    map.insert("value".to_string(), Value::Float(*f));
                    map.insert("literal_type".to_string(), str_val("float"));
                }
                Literal::String(s) => {
                    map.insert("value".to_string(), str_val(s));
                    map.insert("literal_type".to_string(), str_val("string"));
                }
                Literal::Char(c) => {
                    map.insert("value".to_string(), Value::Char(*c));
                    map.insert("literal_type".to_string(), str_val("char"));
                }
                Literal::Null => {
                    map.insert("value".to_string(), Value::Null);
                    map.insert("literal_type".to_string(), str_val("null"));
                }
            }
        }
        Expression::Identifier(name) => {
            map.insert("kind".to_string(), str_val("identifier"));
            map.insert("name".to_string(), str_val(name));
        }
        Expression::BinaryOp { op, left, right } => {
            map.insert("kind".to_string(), str_val("binary_op"));
            map.insert("op".to_string(), str_val(binop_name(op)));
            map.insert("left".to_string(), expr_to_value(left));
            map.insert("right".to_string(), expr_to_value(right));
        }
        Expression::UnaryOp { op, operand } => {
            map.insert("kind".to_string(), str_val("unary_op"));
            map.insert("op".to_string(), str_val(unop_name(op)));
            map.insert("operand".to_string(), expr_to_value(operand));
        }
        Expression::FunctionCall { callee, args } => {
            map.insert("kind".to_string(), str_val("function_call"));
            map.insert("callee".to_string(), expr_to_value(callee));
            let arg_vals: Vec<Value> = args.iter().map(expr_to_value).collect();
            map.insert("args".to_string(), new_list(arg_vals));
        }
        Expression::ObjectLiteral(pairs) => {
            map.insert("kind".to_string(), str_val("object_literal"));
            let pair_vals: Vec<Value> = pairs
                .iter()
                .map(|(k, v)| {
                    let mut pm = HashMap::new();
                    pm.insert("key".to_string(), str_val(k));
                    pm.insert("value".to_string(), expr_to_value(v));
                    Value::Object(Gc::new(GcCell::new(pm)))
                })
                .collect();
            map.insert("pairs".to_string(), new_list(pair_vals));
        }
        Expression::ListLiteral(elements) => {
            map.insert("kind".to_string(), str_val("list_literal"));
            let elem_vals: Vec<Value> = elements.iter().map(expr_to_value).collect();
            map.insert("elements".to_string(), new_list(elem_vals));
        }
        Expression::PropertyAccess { object, property } => {
            map.insert("kind".to_string(), str_val("property_access"));
            map.insert("object".to_string(), expr_to_value(object));
            map.insert("property".to_string(), str_val(property));
        }
        Expression::IndexAccess { object, index } => {
            map.insert("kind".to_string(), str_val("index_access"));
            map.insert("object".to_string(), expr_to_value(object));
            map.insert("index".to_string(), expr_to_value(index));
        }
        Expression::Lambda { params, body, .. } => {
            map.insert("kind".to_string(), str_val("lambda"));
            let param_vals: Vec<Value> = params
                .iter()
                .map(|p| {
                    let mut pm = HashMap::new();
                    pm.insert("name".to_string(), str_val(&p.name));
                    Value::Object(Gc::new(GcCell::new(pm)))
                })
                .collect();
            map.insert("params".to_string(), new_list(param_vals));
            map.insert("body".to_string(), stmt_to_value(body));
        }
        Expression::StringInterpolation(parts) => {
            map.insert("kind".to_string(), str_val("string_interpolation"));
            let part_vals: Vec<Value> = parts
                .iter()
                .map(|part| match part {
                    ish_ast::StringPart::Text(text) => {
                        let mut pm = HashMap::new();
                        pm.insert("type".to_string(), str_val("text"));
                        pm.insert("value".to_string(), str_val(text));
                        Value::Object(Gc::new(GcCell::new(pm)))
                    }
                    ish_ast::StringPart::Expr(expr) => {
                        let mut pm = HashMap::new();
                        pm.insert("type".to_string(), str_val("expr"));
                        pm.insert("value".to_string(), expr_to_value(expr));
                        Value::Object(Gc::new(GcCell::new(pm)))
                    }
                })
                .collect();
            map.insert("parts".to_string(), new_list(part_vals));
        }
        Expression::CommandSubstitution(cmd) => {
            map.insert("kind".to_string(), str_val("command_substitution"));
            map.insert("command".to_string(), stmt_to_value(cmd));
        }
        Expression::EnvVar(name) => {
            map.insert("kind".to_string(), str_val("env_var"));
            map.insert("name".to_string(), str_val(name));
        }
        Expression::Incomplete { kind } => {
            map.insert("kind".to_string(), str_val("incomplete"));
            map.insert("incomplete_kind".to_string(), str_val(&format!("{:?}", kind)));
        }
        Expression::Await { expr } => {
            map.insert("kind".to_string(), str_val("await"));
            map.insert("expr".to_string(), expr_to_value(expr));
        }
        Expression::Spawn { callee, args } => {
            map.insert("kind".to_string(), str_val("spawn"));
            map.insert("callee".to_string(), expr_to_value(callee));
            map.insert("args".to_string(), Value::List(Gc::new(GcCell::new(
                args.iter().map(|a| expr_to_value(a)).collect(),
            ))));
        }
    }
    Value::Object(Gc::new(GcCell::new(map)))
}

fn assign_target_to_value(target: &AssignTarget) -> Value {
    let mut map = HashMap::new();
    match target {
        AssignTarget::Variable(name) => {
            map.insert("kind".to_string(), str_val("variable"));
            map.insert("name".to_string(), str_val(name));
        }
        AssignTarget::Property { object, property } => {
            map.insert("kind".to_string(), str_val("property"));
            map.insert("object".to_string(), expr_to_value(object));
            map.insert("property".to_string(), str_val(property));
        }
        AssignTarget::Index { object, index } => {
            map.insert("kind".to_string(), str_val("index"));
            map.insert("object".to_string(), expr_to_value(object));
            map.insert("index".to_string(), expr_to_value(index));
        }
    }
    Value::Object(Gc::new(GcCell::new(map)))
}

// ── Value → AST ─────────────────────────────────────────────────────────────

/// Convert a Value (Object) back to an AST Program.
pub fn value_to_program(value: &Value) -> Result<Program, RuntimeError> {
    let kind = get_kind(value)?;
    if kind != "program" {
        return Err(RuntimeError::system_error(format!("expected program, got kind '{}'", kind), ErrorCode::TypeMismatch));
    }
    let stmts_val = get_field(value, "statements")?;
    let stmts = value_to_stmt_list(&stmts_val)?;
    Ok(Program::new(stmts))
}

/// Convert a Value back to a Statement.
pub fn value_to_stmt(value: &Value) -> Result<Statement, RuntimeError> {
    let kind = get_kind(value)?;
    match kind.as_str() {
        "var_decl" => {
            let name = get_string_field(value, "name")?;
            let val_node = get_field(value, "value")?;
            let expr = value_to_expr(&val_node)?;
            Ok(Statement::var_decl(name, expr))
        }
        "assignment" => {
            let target_val = get_field(value, "target")?;
            let val_node = get_field(value, "value")?;
            let target = value_to_assign_target(&target_val)?;
            let expr = value_to_expr(&val_node)?;
            Ok(Statement::Assignment { target, value: expr })
        }
        "block" => {
            let stmts_val = get_field(value, "statements")?;
            let stmts = value_to_stmt_list(&stmts_val)?;
            Ok(Statement::Block { statements: stmts })
        }
        "if" => {
            let cond = value_to_expr(&get_field(value, "condition")?)?;
            let then_block = value_to_stmt(&get_field(value, "then_block")?)?;
            let else_val = get_field(value, "else_block")?;
            let else_block = if matches!(else_val, Value::Null) {
                None
            } else {
                Some(Box::new(value_to_stmt(&else_val)?))
            };
            Ok(Statement::If {
                condition: cond,
                then_block: Box::new(then_block),
                else_block,
            })
        }
        "while" => {
            let cond = value_to_expr(&get_field(value, "condition")?)?;
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Statement::While {
                condition: cond,
                body: Box::new(body),
                yield_every: None,
            })
        }
        "for_each" => {
            let variable = get_string_field(value, "variable")?;
            let iterable = value_to_expr(&get_field(value, "iterable")?)?;
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Statement::ForEach {
                variable,
                iterable,
                body: Box::new(body),
                yield_every: None,
            })
        }
        "return" => {
            let val = get_field(value, "value")?;
            let expr = if matches!(val, Value::Null) {
                None
            } else {
                Some(value_to_expr(&val)?)
            };
            Ok(Statement::Return { value: expr })
        }
        "expr_stmt" => {
            let expr = value_to_expr(&get_field(value, "expression")?)?;
            Ok(Statement::ExpressionStmt(expr))
        }
        "function_decl" => {
            let name = get_string_field(value, "name")?;
            let params_val = get_field(value, "params")?;
            let params = value_to_params(&params_val)?;
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Statement::FunctionDecl {
                name,
                params,
                return_type: None,
                body: Box::new(body),
                visibility: None,
                type_params: vec![],
                is_async: false,
            })
        }
        "throw" => {
            let val_node = get_field(value, "value")?;
            let expr = value_to_expr(&val_node)?;
            Ok(Statement::throw(expr))
        }
        "try_catch" => {
            let body = value_to_stmt(&get_field(value, "body")?)?;
            let catches_val = get_field(value, "catches")?;
            let catches = if let Value::List(ref list_ref) = catches_val {
                let list = list_ref.borrow();
                let mut clauses = Vec::new();
                for c in list.iter() {
                    let param = get_string_field(c, "param")?;
                    let clause_body = value_to_stmt(&get_field(c, "body")?)?;
                    clauses.push(CatchClause::new(param, clause_body));
                }
                clauses
            } else {
                Vec::new()
            };
            let finally_val = get_field(value, "finally")?;
            let finally = if matches!(finally_val, Value::Null) {
                None
            } else {
                Some(value_to_stmt(&finally_val)?)
            };
            Ok(Statement::try_catch(body, catches, finally))
        }
        "with_block" => {
            let resources_val = get_field(value, "resources")?;
            let resources = if let Value::List(ref list_ref) = resources_val {
                let list = list_ref.borrow();
                let mut res = Vec::new();
                for r in list.iter() {
                    let name = get_string_field(r, "name")?;
                    let val_node = get_field(r, "value")?;
                    let expr = value_to_expr(&val_node)?;
                    res.push((name, expr));
                }
                res
            } else {
                Vec::new()
            };
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Statement::WithBlock {
                resources,
                body: Box::new(body),
            })
        }
        "defer" => {
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Statement::Defer {
                body: Box::new(body),
            })
        }
        _ => Err(RuntimeError::system_error(format!("unknown statement kind: '{}'", kind), ErrorCode::TypeMismatch)),
    }
}

/// Convert a Value back to an Expression.
pub fn value_to_expr(value: &Value) -> Result<Expression, RuntimeError> {
    let kind = get_kind(value)?;
    match kind.as_str() {
        "literal" => {
            let lit_type = get_string_field(value, "literal_type")?;
            let val = get_field(value, "value")?;
            match lit_type.as_str() {
                "bool" => {
                    if let Value::Bool(b) = val {
                        Ok(Expression::bool(b))
                    } else {
                        Err(RuntimeError::system_error("literal bool has non-bool value", ErrorCode::TypeMismatch))
                    }
                }
                "int" => {
                    if let Value::Int(n) = val {
                        Ok(Expression::int(n))
                    } else {
                        Err(RuntimeError::system_error("literal int has non-int value", ErrorCode::TypeMismatch))
                    }
                }
                "float" => {
                    if let Value::Float(f) = val {
                        Ok(Expression::float(f))
                    } else {
                        Err(RuntimeError::system_error("literal float has non-float value", ErrorCode::TypeMismatch))
                    }
                }
                "string" => {
                    if let Value::String(ref s) = val {
                        Ok(Expression::string(s.as_ref().clone()))
                    } else {
                        Err(RuntimeError::system_error("literal string has non-string value", ErrorCode::TypeMismatch))
                    }
                }
                "char" => {
                    if let Value::Char(c) = val {
                        Ok(Expression::char_lit(c))
                    } else {
                        Err(RuntimeError::system_error("literal char has non-char value", ErrorCode::TypeMismatch))
                    }
                }
                "null" => Ok(Expression::null()),
                _ => Err(RuntimeError::system_error(format!("unknown literal_type: '{}'", lit_type), ErrorCode::TypeMismatch)),
            }
        }
        "identifier" => {
            let name = get_string_field(value, "name")?;
            Ok(Expression::ident(name))
        }
        "binary_op" => {
            let op_str = get_string_field(value, "op")?;
            let op = parse_binop(&op_str)?;
            let left = value_to_expr(&get_field(value, "left")?)?;
            let right = value_to_expr(&get_field(value, "right")?)?;
            Ok(Expression::binary(op, left, right))
        }
        "unary_op" => {
            let op_str = get_string_field(value, "op")?;
            let op = parse_unop(&op_str)?;
            let operand = value_to_expr(&get_field(value, "operand")?)?;
            Ok(Expression::unary(op, operand))
        }
        "function_call" => {
            let callee = value_to_expr(&get_field(value, "callee")?)?;
            let args_val = get_field(value, "args")?;
            let args = value_to_expr_list(&args_val)?;
            Ok(Expression::call(callee, args))
        }
        "object_literal" => {
            let pairs_val = get_field(value, "pairs")?;
            if let Value::List(ref list_ref) = pairs_val {
                let list = list_ref.borrow();
                let mut pairs = Vec::new();
                for pair in list.iter() {
                    let key = get_string_field(pair, "key")?;
                    let val = value_to_expr(&get_field(pair, "value")?)?;
                    pairs.push((key, val));
                }
                Ok(Expression::ObjectLiteral(pairs))
            } else {
                Err(RuntimeError::system_error("object_literal pairs must be a list", ErrorCode::TypeMismatch))
            }
        }
        "list_literal" => {
            let elems_val = get_field(value, "elements")?;
            let elements = value_to_expr_list(&elems_val)?;
            Ok(Expression::ListLiteral(elements))
        }
        "property_access" => {
            let object = value_to_expr(&get_field(value, "object")?)?;
            let property = get_string_field(value, "property")?;
            Ok(Expression::property(object, property))
        }
        "index_access" => {
            let object = value_to_expr(&get_field(value, "object")?)?;
            let index = value_to_expr(&get_field(value, "index")?)?;
            Ok(Expression::index(object, index))
        }
        "lambda" => {
            let params_val = get_field(value, "params")?;
            let params = value_to_params(&params_val)?;
            let body = value_to_stmt(&get_field(value, "body")?)?;
            Ok(Expression::lambda(params, body))
        }
        _ => Err(RuntimeError::system_error(format!("unknown expression kind: '{}'", kind), ErrorCode::TypeMismatch)),
    }
}

fn value_to_assign_target(value: &Value) -> Result<AssignTarget, RuntimeError> {
    let kind = get_kind(value)?;
    match kind.as_str() {
        "variable" => {
            let name = get_string_field(value, "name")?;
            Ok(AssignTarget::Variable(name))
        }
        "property" => {
            let object = value_to_expr(&get_field(value, "object")?)?;
            let property = get_string_field(value, "property")?;
            Ok(AssignTarget::Property {
                object: Box::new(object),
                property,
            })
        }
        "index" => {
            let object = value_to_expr(&get_field(value, "object")?)?;
            let index = value_to_expr(&get_field(value, "index")?)?;
            Ok(AssignTarget::Index {
                object: Box::new(object),
                index: Box::new(index),
            })
        }
        _ => Err(RuntimeError::system_error(format!("unknown assign target kind: '{}'", kind), ErrorCode::TypeMismatch)),
    }
}

fn value_to_params(value: &Value) -> Result<Vec<Parameter>, RuntimeError> {
    if let Value::List(ref list_ref) = value {
        let list = list_ref.borrow();
        let mut params = Vec::new();
        for p in list.iter() {
            let name = get_string_field(p, "name")?;
            params.push(Parameter::new(name));
        }
        Ok(params)
    } else {
        Err(RuntimeError::system_error("params must be a list", ErrorCode::TypeMismatch))
    }
}

fn value_to_stmt_list(value: &Value) -> Result<Vec<Statement>, RuntimeError> {
    if let Value::List(ref list_ref) = value {
        let list = list_ref.borrow();
        list.iter().map(value_to_stmt).collect()
    } else {
        Err(RuntimeError::system_error("expected a list of statements", ErrorCode::TypeMismatch))
    }
}

fn value_to_expr_list(value: &Value) -> Result<Vec<Expression>, RuntimeError> {
    if let Value::List(ref list_ref) = value {
        let list = list_ref.borrow();
        list.iter().map(value_to_expr).collect()
    } else {
        Err(RuntimeError::system_error("expected a list of expressions", ErrorCode::TypeMismatch))
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn str_val(s: &str) -> Value {
    Value::String(Rc::new(s.to_string()))
}

fn get_kind(value: &Value) -> Result<String, RuntimeError> {
    get_string_field(value, "kind")
}

fn get_field(value: &Value, field: &str) -> Result<Value, RuntimeError> {
    if let Value::Object(ref obj_ref) = value {
        let map = obj_ref.borrow();
        Ok(map.get(field).cloned().unwrap_or(Value::Null))
    } else {
        Err(RuntimeError::system_error(format!(
            "expected object when accessing field '{}', got {}",
            field,
            value.type_name()
        ), ErrorCode::TypeMismatch))
    }
}

fn get_string_field(value: &Value, field: &str) -> Result<String, RuntimeError> {
    let val = get_field(value, field)?;
    if let Value::String(ref s) = val {
        Ok(s.as_ref().clone())
    } else {
        Err(RuntimeError::system_error(format!(
            "expected string for field '{}', got {}",
            field,
            val.type_name()
        ), ErrorCode::TypeMismatch))
    }
}

fn binop_name(op: &BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Add => "add",
        BinaryOperator::Sub => "sub",
        BinaryOperator::Mul => "mul",
        BinaryOperator::Div => "div",
        BinaryOperator::Mod => "mod",
        BinaryOperator::Eq => "eq",
        BinaryOperator::NotEq => "neq",
        BinaryOperator::Lt => "lt",
        BinaryOperator::Gt => "gt",
        BinaryOperator::LtEq => "lteq",
        BinaryOperator::GtEq => "gteq",
        BinaryOperator::And => "and",
        BinaryOperator::Or => "or",
    }
}

fn parse_binop(s: &str) -> Result<BinaryOperator, RuntimeError> {
    match s {
        "add" => Ok(BinaryOperator::Add),
        "sub" => Ok(BinaryOperator::Sub),
        "mul" => Ok(BinaryOperator::Mul),
        "div" => Ok(BinaryOperator::Div),
        "mod" => Ok(BinaryOperator::Mod),
        "eq" => Ok(BinaryOperator::Eq),
        "neq" => Ok(BinaryOperator::NotEq),
        "lt" => Ok(BinaryOperator::Lt),
        "gt" => Ok(BinaryOperator::Gt),
        "lteq" => Ok(BinaryOperator::LtEq),
        "gteq" => Ok(BinaryOperator::GtEq),
        "and" => Ok(BinaryOperator::And),
        "or" => Ok(BinaryOperator::Or),
        _ => Err(RuntimeError::system_error(format!("unknown binary operator: '{}'", s), ErrorCode::TypeMismatch)),
    }
}

fn unop_name(op: &UnaryOperator) -> &'static str {
    match op {
        UnaryOperator::Not => "not",
        UnaryOperator::Negate => "negate",
        UnaryOperator::Try => "try",
    }
}

fn parse_unop(s: &str) -> Result<UnaryOperator, RuntimeError> {
    match s {
        "not" => Ok(UnaryOperator::Not),
        "negate" => Ok(UnaryOperator::Negate),
        "try" => Ok(UnaryOperator::Try),
        _ => Err(RuntimeError::system_error(format!("unknown unary operator: '{}'", s), ErrorCode::TypeMismatch)),
    }
}

// ── AST Factory Builtins ────────────────────────────────────────────────────

fn simple_ast_builtin(
    name: &'static str,
    arity: usize,
    fields: &'static [&'static str],
) -> Value {
    use crate::value::new_compiled_function;
    new_compiled_function(name, vec![], vec![], None, move |args| {
        if args.len() != arity {
            return Err(RuntimeError::system_error(
                format!("{} expects {} argument(s)", name, arity),
                ErrorCode::TypeMismatch,
            ));
        }
        let mut map = HashMap::new();
        map.insert("kind".to_string(), str_val(
            name.strip_prefix("ast_").unwrap_or(name)
        ));
        for (field, arg) in fields.iter().zip(args.iter()) {
            map.insert(field.to_string(), arg.clone());
        }
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false))
}

/// Register AST factory built-in functions (callable from ish programs).
pub fn register_ast_builtins(env: &crate::environment::Environment) {
    use crate::value::new_compiled_function;

    // Simple builtins — all follow the same pattern
    env.define("ast_program".into(), simple_ast_builtin("ast_program", 1, &["statements"]));
    env.define("ast_identifier".into(), simple_ast_builtin("ast_identifier", 1, &["name"]));
    env.define("ast_binary_op".into(), simple_ast_builtin("ast_binary_op", 3, &["op", "left", "right"]));
    env.define("ast_unary_op".into(), simple_ast_builtin("ast_unary_op", 2, &["op", "operand"]));
    env.define("ast_function_call".into(), simple_ast_builtin("ast_function_call", 2, &["callee", "args"]));
    env.define("ast_block".into(), simple_ast_builtin("ast_block", 1, &["statements"]));
    env.define("ast_return".into(), simple_ast_builtin("ast_return", 1, &["value"]));
    env.define("ast_var_decl".into(), simple_ast_builtin("ast_var_decl", 2, &["name", "value"]));
    env.define("ast_if".into(), simple_ast_builtin("ast_if", 3, &["condition", "then_block", "else_block"]));
    env.define("ast_while".into(), simple_ast_builtin("ast_while", 2, &["condition", "body"]));
    env.define("ast_function_decl".into(), simple_ast_builtin("ast_function_decl", 3, &["name", "params", "body"]));
    env.define("ast_expr_stmt".into(), simple_ast_builtin("ast_expr_stmt", 1, &["expression"]));
    env.define("ast_lambda".into(), simple_ast_builtin("ast_lambda", 2, &["params", "body"]));
    env.define("ast_property_access".into(), simple_ast_builtin("ast_property_access", 2, &["object", "property"]));
    env.define("ast_index_access".into(), simple_ast_builtin("ast_index_access", 2, &["object", "index"]));
    env.define("ast_object_literal".into(), simple_ast_builtin("ast_object_literal", 1, &["pairs"]));
    env.define("ast_list_literal".into(), simple_ast_builtin("ast_list_literal", 1, &["elements"]));
    env.define("ast_assignment".into(), simple_ast_builtin("ast_assignment", 2, &["target", "value"]));
    env.define("ast_for_each".into(), simple_ast_builtin("ast_for_each", 3, &["variable", "iterable", "body"]));
    env.define("ast_throw".into(), simple_ast_builtin("ast_throw", 1, &["value"]));
    env.define("ast_try_catch".into(), simple_ast_builtin("ast_try_catch", 3, &["body", "catches", "finally"]));
    env.define("ast_defer".into(), simple_ast_builtin("ast_defer", 1, &["body"]));

    // Exceptions — must remain as inline closures

    // ast_literal: extra `literal_type` field populated by inspecting args[0]'s type
    env.define(
        "ast_literal".into(),
        new_compiled_function("ast_literal", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("ast_literal expects 1 argument", ErrorCode::TypeMismatch));
            }
            let mut map = HashMap::new();
            map.insert("kind".to_string(), str_val("literal"));
            map.insert("value".to_string(), args[0].clone());
            let lit_type = match &args[0] {
                Value::Bool(_) => "bool",
                Value::Int(_) => "int",
                Value::Float(_) => "float",
                Value::String(_) => "string",
                Value::Char(_) => "char",
                Value::Null => "null",
                _ => "unknown",
            };
            map.insert("literal_type".to_string(), str_val(lit_type));
            Ok(Value::Object(Gc::new(GcCell::new(map))))
        }, Some(false)),
    );

    // ast_param: no `kind` field at all
    env.define(
        "ast_param".into(),
        new_compiled_function("ast_param", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("ast_param expects 1 argument", ErrorCode::TypeMismatch));
            }
            let mut map = HashMap::new();
            map.insert("name".to_string(), args[0].clone());
            Ok(Value::Object(Gc::new(GcCell::new(map))))
        }, Some(false)),
    );

    // ast_assign_target_var: kind is "variable", not derived from name
    env.define(
        "ast_assign_target_var".into(),
        new_compiled_function("ast_assign_target_var", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("ast_assign_target_var expects 1 argument", ErrorCode::TypeMismatch));
            }
            let mut map = HashMap::new();
            map.insert("kind".to_string(), str_val("variable"));
            map.insert("name".to_string(), args[0].clone());
            Ok(Value::Object(Gc::new(GcCell::new(map))))
        }, Some(false)),
    );
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::builder::ProgramBuilder;

    #[test]
    fn test_roundtrip_simple_program() {
        let program = ProgramBuilder::new()
            .var_decl("x", Expression::int(42))
            .function("double", &["n"], |b| {
                b.ret(Expression::binary(
                    BinaryOperator::Mul,
                    Expression::ident("n"),
                    Expression::int(2),
                ))
            })
            .build();

        let value = program_to_value(&program);
        let restored = value_to_program(&value).unwrap();
        assert_eq!(program, restored);
    }

    #[test]
    fn test_roundtrip_complex_program() {
        let program = ProgramBuilder::new()
            .function("test_fn", &["a", "b"], |b| {
                b.var_decl("neg", Expression::unary(UnaryOperator::Negate, Expression::ident("a")))
                    .var_decl("obj", Expression::object(vec![("key", Expression::string("value"))]))
                    .var_decl("lst", Expression::list(vec![Expression::int(1), Expression::int(2)]))
                    .var_decl("prop", Expression::property(Expression::ident("obj"), "key"))
                    .var_decl("elem", Expression::index(Expression::ident("lst"), Expression::int(0)))
                    .if_else(
                        Expression::binary(BinaryOperator::Gt, Expression::ident("a"), Expression::int(0)),
                        |b| b.ret(Expression::ident("a")),
                        |b| b.ret(Expression::unary(UnaryOperator::Negate, Expression::ident("a"))),
                    )
                    .while_loop(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("a"), Expression::int(100)),
                        |b| b.assign("a", Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::int(1))),
                    )
                    .ret(Expression::null())
            })
            .build();

        let value = program_to_value(&program);
        let restored = value_to_program(&value).unwrap();
        assert_eq!(program, restored);
    }

    #[test]
    fn test_roundtrip_lambda() {
        let program = ProgramBuilder::new()
            .var_decl(
                "f",
                Expression::lambda(
                    vec![Parameter::new("x")],
                    Statement::block(vec![
                        Statement::ret(Some(Expression::binary(
                            BinaryOperator::Mul,
                            Expression::ident("x"),
                            Expression::int(2),
                        ))),
                    ]),
                ),
            )
            .build();

        let value = program_to_value(&program);
        let restored = value_to_program(&value).unwrap();
        assert_eq!(program, restored);
    }

    #[tokio::test]
    async fn test_execute_ast_built_from_values() {
        // Build an AST as values, convert to AST, execute it
        use crate::interpreter::IshVm;

        // Build: fn add(a, b) { return a + b; }
        // Call: add(10, 32)
        let program = Program::new(vec![
            Statement::function_decl(
                "add",
                vec![Parameter::new("a"), Parameter::new("b")],
                Statement::block(vec![Statement::ret(Some(Expression::binary(
                    BinaryOperator::Add,
                    Expression::ident("a"),
                    Expression::ident("b"),
                )))]),
            ),
            Statement::expr_stmt(Expression::call(
                Expression::ident("add"),
                vec![Expression::int(10), Expression::int(32)],
            )),
        ]);

        // Convert to values and back
        let value = program_to_value(&program);
        let restored = value_to_program(&value).unwrap();

        let vm = std::rc::Rc::new(std::cell::RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &restored).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }
}
