// Built-in functions for the ish interpreter.
//
// Organized by category: strings, lists, objects, I/O, types, conversion.

use std::rc::Rc;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::value::*;

/// Register all built-in functions into the given environment.
pub fn register_all(env: &Environment) {
    register_io(env);
    register_strings(env);
    register_lists(env);
    register_objects(env);
    register_types(env);
    register_conversion(env);
    register_errors(env);
    register_ledger(env);
}

// ── Ledger query stubs ──────────────────────────────────────────────────────
// These builtins need VM access so their actual logic is in interpreter.rs
// (call_function intercepts them by name). The stubs here just make the
// names callable; reaching the closure body means the intercept is missing.

fn register_ledger(env: &Environment) {
    for name in &[
        "active_standard",
        "feature_state",
        "has_standard",
        "has_entry_type",
        "ledger_state",
        "has_entry",
    ] {
        let n = (*name).to_string();
        let n2 = n.clone();
        env.define(
            n.clone(),
            new_builtin(&n, move |_args| {
                Err(RuntimeError::system_error(format!(
                    "{} must be intercepted by the interpreter",
                    n2
                ), "E004"))
            }),
        );
    }
}

// ── I/O ─────────────────────────────────────────────────────────────────────

fn register_io(env: &Environment) {
    env.define(
        "print".into(),
        new_builtin("print", |args| {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{}", arg.to_display_string());
            }
            Ok(Value::Null)
        }),
    );

    env.define(
        "println".into(),
        new_builtin("println", |args| {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{}", arg.to_display_string());
            }
            println!();
            Ok(Value::Null)
        }),
    );

    env.define(
        "read_file".into(),
        new_builtin("read_file", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("read_file expects 1 argument", "E003"));
            }
            if let Value::String(path) = &args[0] {
                match std::fs::read_to_string(path.as_ref()) {
                    Ok(content) => Ok(Value::String(Rc::new(content))),
                    Err(e) => Err(RuntimeError::system_error(format!("read_file error: {}", e), "E008")),
                }
            } else {
                Err(RuntimeError::system_error("read_file expects a string path", "E004"))
            }
        }),
    );

    env.define(
        "write_file".into(),
        new_builtin("write_file", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("write_file expects 2 arguments", "E003"));
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::write(path.as_ref(), content.as_ref()) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(RuntimeError::system_error(format!("write_file error: {}", e), "E008")),
                }
            } else {
                Err(RuntimeError::system_error(
                    "write_file expects (string path, string content)", "E004"))
            }
        }),
    );
}

// ── String operations ───────────────────────────────────────────────────────

fn register_strings(env: &Environment) {
    env.define(
        "str_concat".into(),
        new_builtin("str_concat", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_concat expects 2 arguments", "E003"));
            }
            let a = args[0].to_display_string();
            let b = args[1].to_display_string();
            Ok(Value::String(Rc::new(format!("{}{}", a, b))))
        }),
    );

    env.define(
        "str_length".into(),
        new_builtin("str_length", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_length expects 1 argument", "E003"));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::Int(s.len() as i64))
            } else {
                Err(RuntimeError::system_error("str_length expects a string", "E004"))
            }
        }),
    );

    env.define(
        "str_slice".into(),
        new_builtin("str_slice", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("str_slice expects 3 arguments", "E003"));
            }
            if let (Value::String(s), Value::Int(start), Value::Int(end)) =
                (&args[0], &args[1], &args[2])
            {
                let start = *start as usize;
                let end = (*end as usize).min(s.len());
                if start > end || start > s.len() {
                    return Ok(Value::String(Rc::new(String::new())));
                }
                Ok(Value::String(Rc::new(s[start..end].to_string())))
            } else {
                Err(RuntimeError::system_error("str_slice expects (string, int, int)", "E004"))
            }
        }),
    );

    env.define(
        "str_contains".into(),
        new_builtin("str_contains", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_contains expects 2 arguments", "E003"));
            }
            if let (Value::String(s), Value::String(substr)) = (&args[0], &args[1]) {
                Ok(Value::Bool(s.contains(substr.as_ref())))
            } else {
                Err(RuntimeError::system_error("str_contains expects (string, string)", "E004"))
            }
        }),
    );

    env.define(
        "str_starts_with".into(),
        new_builtin("str_starts_with", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_starts_with expects 2 arguments", "E003"));
            }
            if let (Value::String(s), Value::String(prefix)) = (&args[0], &args[1]) {
                Ok(Value::Bool(s.starts_with(prefix.as_ref())))
            } else {
                Err(RuntimeError::system_error(
                    "str_starts_with expects (string, string)", "E004"))
            }
        }),
    );

    env.define(
        "str_replace".into(),
        new_builtin("str_replace", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("str_replace expects 3 arguments", "E003"));
            }
            if let (Value::String(s), Value::String(from), Value::String(to)) =
                (&args[0], &args[1], &args[2])
            {
                Ok(Value::String(Rc::new(
                    s.replace(from.as_ref(), to.as_ref()),
                )))
            } else {
                Err(RuntimeError::system_error(
                    "str_replace expects (string, string, string)", "E004"))
            }
        }),
    );

    env.define(
        "str_split".into(),
        new_builtin("str_split", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_split expects 2 arguments", "E003"));
            }
            if let (Value::String(s), Value::String(delim)) = (&args[0], &args[1]) {
                let parts: Vec<Value> = s
                    .split(delim.as_ref())
                    .map(|p| Value::String(Rc::new(p.to_string())))
                    .collect();
                Ok(new_list(parts))
            } else {
                Err(RuntimeError::system_error("str_split expects (string, string)", "E004"))
            }
        }),
    );

    env.define(
        "str_to_upper".into(),
        new_builtin("str_to_upper", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_to_upper expects 1 argument", "E003"));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.to_uppercase())))
            } else {
                Err(RuntimeError::system_error("str_to_upper expects a string", "E004"))
            }
        }),
    );

    env.define(
        "str_to_lower".into(),
        new_builtin("str_to_lower", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_to_lower expects 1 argument", "E003"));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.to_lowercase())))
            } else {
                Err(RuntimeError::system_error("str_to_lower expects a string", "E004"))
            }
        }),
    );

    env.define(
        "str_char_at".into(),
        new_builtin("str_char_at", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_char_at expects 2 arguments", "E003"));
            }
            if let (Value::String(s), Value::Int(i)) = (&args[0], &args[1]) {
                let i = *i as usize;
                if let Some(ch) = s.chars().nth(i) {
                    Ok(Value::String(Rc::new(ch.to_string())))
                } else {
                    Ok(Value::Null)
                }
            } else {
                Err(RuntimeError::system_error("str_char_at expects (string, int)", "E004"))
            }
        }),
    );

    env.define(
        "str_trim".into(),
        new_builtin("str_trim", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_trim expects 1 argument", "E003"));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.trim().to_string())))
            } else {
                Err(RuntimeError::system_error("str_trim expects a string", "E004"))
            }
        }),
    );
}

// ── List operations ─────────────────────────────────────────────────────────

fn register_lists(env: &Environment) {
    env.define(
        "list_push".into(),
        new_builtin("list_push", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_push expects 2 arguments", "E003"));
            }
            if let Value::List(list_ref) = &args[0] {
                list_ref.borrow_mut().push(args[1].clone());
                Ok(Value::Null)
            } else {
                Err(RuntimeError::system_error("list_push expects a list as first argument", "E003"))
            }
        }),
    );

    env.define(
        "list_pop".into(),
        new_builtin("list_pop", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("list_pop expects 1 argument", "E003"));
            }
            if let Value::List(list_ref) = &args[0] {
                Ok(list_ref.borrow_mut().pop().unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("list_pop expects a list", "E004"))
            }
        }),
    );

    env.define(
        "list_length".into(),
        new_builtin("list_length", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("list_length expects 1 argument", "E003"));
            }
            if let Value::List(list_ref) = &args[0] {
                Ok(Value::Int(list_ref.borrow().len() as i64))
            } else {
                Err(RuntimeError::system_error("list_length expects a list", "E004"))
            }
        }),
    );

    env.define(
        "list_get".into(),
        new_builtin("list_get", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_get expects 2 arguments", "E003"));
            }
            if let (Value::List(list_ref), Value::Int(i)) = (&args[0], &args[1]) {
                let list = list_ref.borrow();
                let i = *i as usize;
                if i < list.len() {
                    Ok(list[i].clone())
                } else {
                    Err(RuntimeError::system_error(format!(
                        "index {} out of bounds (length {})",
                        i,
                        list.len()
                    ), "E004"))
                }
            } else {
                Err(RuntimeError::system_error("list_get expects (list, int)", "E004"))
            }
        }),
    );

    env.define(
        "list_set".into(),
        new_builtin("list_set", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("list_set expects 3 arguments", "E003"));
            }
            if let (Value::List(list_ref), Value::Int(i)) = (&args[0], &args[1]) {
                let mut list = list_ref.borrow_mut();
                let i = *i as usize;
                if i < list.len() {
                    list[i] = args[2].clone();
                    Ok(Value::Null)
                } else {
                    Err(RuntimeError::system_error(format!(
                        "index {} out of bounds (length {})",
                        i,
                        list.len()
                    ), "E004"))
                }
            } else {
                Err(RuntimeError::system_error("list_set expects (list, int, value)", "E004"))
            }
        }),
    );

    env.define(
        "list_slice".into(),
        new_builtin("list_slice", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("list_slice expects 3 arguments", "E003"));
            }
            if let (Value::List(list_ref), Value::Int(start), Value::Int(end)) =
                (&args[0], &args[1], &args[2])
            {
                let list = list_ref.borrow();
                let start = *start as usize;
                let end = (*end as usize).min(list.len());
                if start > end {
                    return Ok(new_list(vec![]));
                }
                Ok(new_list(list[start..end].to_vec()))
            } else {
                Err(RuntimeError::system_error("list_slice expects (list, int, int)", "E004"))
            }
        }),
    );

    env.define(
        "list_join".into(),
        new_builtin("list_join", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_join expects 2 arguments", "E003"));
            }
            if let (Value::List(list_ref), Value::String(sep)) = (&args[0], &args[1]) {
                let list = list_ref.borrow();
                let strs: Vec<String> = list.iter().map(|v| v.to_display_string()).collect();
                Ok(Value::String(Rc::new(strs.join(sep.as_ref()))))
            } else {
                Err(RuntimeError::system_error("list_join expects (list, string)", "E004"))
            }
        }),
    );
}

// ── Object operations ───────────────────────────────────────────────────────

fn register_objects(env: &Environment) {
    env.define(
        "obj_get".into(),
        new_builtin("obj_get", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_get expects 2 arguments", "E003"));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                Ok(obj_ref
                    .borrow()
                    .get(key.as_ref())
                    .cloned()
                    .unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("obj_get expects (object, string)", "E004"))
            }
        }),
    );

    env.define(
        "obj_set".into(),
        new_builtin("obj_set", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("obj_set expects 3 arguments", "E003"));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                obj_ref
                    .borrow_mut()
                    .insert(key.as_ref().clone(), args[2].clone());
                Ok(Value::Null)
            } else {
                Err(RuntimeError::system_error("obj_set expects (object, string, value)", "E004"))
            }
        }),
    );

    env.define(
        "obj_has".into(),
        new_builtin("obj_has", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_has expects 2 arguments", "E003"));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                Ok(Value::Bool(obj_ref.borrow().contains_key(key.as_ref())))
            } else {
                Err(RuntimeError::system_error("obj_has expects (object, string)", "E004"))
            }
        }),
    );

    env.define(
        "obj_keys".into(),
        new_builtin("obj_keys", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("obj_keys expects 1 argument", "E003"));
            }
            if let Value::Object(obj_ref) = &args[0] {
                let keys: Vec<Value> = obj_ref
                    .borrow()
                    .keys()
                    .map(|k| Value::String(Rc::new(k.clone())))
                    .collect();
                Ok(new_list(keys))
            } else {
                Err(RuntimeError::system_error("obj_keys expects an object", "E004"))
            }
        }),
    );

    env.define(
        "obj_values".into(),
        new_builtin("obj_values", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("obj_values expects 1 argument", "E003"));
            }
            if let Value::Object(obj_ref) = &args[0] {
                let values: Vec<Value> = obj_ref.borrow().values().cloned().collect();
                Ok(new_list(values))
            } else {
                Err(RuntimeError::system_error("obj_values expects an object", "E004"))
            }
        }),
    );

    env.define(
        "obj_remove".into(),
        new_builtin("obj_remove", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_remove expects 2 arguments", "E003"));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                let removed = obj_ref
                    .borrow_mut()
                    .remove(key.as_ref())
                    .unwrap_or(Value::Null);
                Ok(removed)
            } else {
                Err(RuntimeError::system_error("obj_remove expects (object, string)", "E004"))
            }
        }),
    );
}

// ── Type checking ───────────────────────────────────────────────────────────

fn register_types(env: &Environment) {
    env.define(
        "type_of".into(),
        new_builtin("type_of", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("type_of expects 1 argument", "E003"));
            }
            Ok(Value::String(Rc::new(args[0].type_name().to_string())))
        }),
    );

    env.define(
        "is_type".into(),
        new_builtin("is_type", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("is_type expects 2 arguments", "E003"));
            }
            if let Value::String(type_name) = &args[1] {
                Ok(Value::Bool(args[0].type_name() == type_name.as_ref()))
            } else {
                Err(RuntimeError::system_error("is_type expects (value, string)", "E004"))
            }
        }),
    );
}

// ── Conversion ──────────────────────────────────────────────────────────────

fn register_conversion(env: &Environment) {
    env.define(
        "to_string".into(),
        new_builtin("to_string", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_string expects 1 argument", "E003"));
            }
            Ok(Value::String(Rc::new(args[0].to_display_string())))
        }),
    );

    env.define(
        "to_int".into(),
        new_builtin("to_int", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_int expects 1 argument", "E003"));
            }
            match &args[0] {
                Value::Int(n) => Ok(Value::Int(*n)),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                Value::String(s) => match s.parse::<i64>() {
                    Ok(n) => Ok(Value::Int(n)),
                    Err(_) => Err(RuntimeError::system_error(format!(
                        "cannot convert '{}' to int",
                        s
                    ), "E004")),
                },
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to int",
                    args[0].type_name()
                ), "E004")),
            }
        }),
    );

    env.define(
        "to_float".into(),
        new_builtin("to_float", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_float expects 1 argument", "E003"));
            }
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::Int(n) => Ok(Value::Float(*n as f64)),
                Value::String(s) => match s.parse::<f64>() {
                    Ok(f) => Ok(Value::Float(f)),
                    Err(_) => Err(RuntimeError::system_error(format!(
                        "cannot convert '{}' to float",
                        s
                    ), "E004")),
                },
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to float",
                    args[0].type_name()
                ), "E004")),
            }
        }),
    );

    env.define(
        "char".into(),
        new_builtin("char", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("char expects 1 argument", "E003"));
            }
            match &args[0] {
                Value::String(s) => {
                    let mut chars = s.chars();
                    if let Some(c) = chars.next() {
                        if chars.next().is_none() {
                            Ok(Value::Char(c))
                        } else {
                            Err(RuntimeError::system_error("char expects a single-character string", "E004"))
                        }
                    } else {
                        Err(RuntimeError::system_error("char expects a non-empty string", "E004"))
                    }
                }
                Value::Int(n) => {
                    let code = *n as u32;
                    match char::from_u32(code) {
                        Some(c) => Ok(Value::Char(c)),
                        None => Err(RuntimeError::system_error(format!(
                            "invalid Unicode code point: {}",
                            n
                        ), "E004")),
                    }
                }
                Value::Char(c) => Ok(Value::Char(*c)),
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to char",
                    args[0].type_name()
                ), "E004")),
            }
        }),
    );
}

// ── Error Handling ──────────────────────────────────────────────────────────

fn register_errors(env: &Environment) {
    // is_error(value) -> checks if a value is an Error object
    // An error object is any object with a "message" String property.
    env.define(
        "is_error".into(),
        new_builtin("is_error", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("is_error expects 1 argument", "E003"));
            }
            let result = if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                matches!(map.get("message"), Some(Value::String(_)))
            } else {
                false
            };
            Ok(Value::Bool(result))
        }),
    );

    // error_message(error) -> extracts the message from an Error object
    env.define(
        "error_message".into(),
        new_builtin("error_message", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("error_message expects 1 argument", "E003"));
            }
            if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                Ok(map.get("message").cloned().unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("error_message expects an error object", "E004"))
            }
        }),
    );

    // error_code(error) -> extracts the code from a CodedError object (null if not CodedError)
    env.define(
        "error_code".into(),
        new_builtin("error_code", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("error_code expects 1 argument", "E003"));
            }
            if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                Ok(map.get("code").cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }),
    );
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::*;
    use ish_ast::builder::ProgramBuilder;
    use crate::interpreter::IshVm;

    #[test]
    fn test_type_of() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("type_of"),
                vec![Expression::int(42)],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("int".into())));
    }

    #[test]
    fn test_str_length() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("str_length"),
                vec![Expression::string("hello")],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_list_operations() {
        // let lst = [1, 2, 3];
        // list_push(lst, 4);
        // list_length(lst) -> 4
        let program = ProgramBuilder::new()
            .var_decl(
                "lst",
                Expression::list(vec![
                    Expression::int(1),
                    Expression::int(2),
                    Expression::int(3),
                ]),
            )
            .expr_stmt(Expression::call(
                Expression::ident("list_push"),
                vec![Expression::ident("lst"), Expression::int(4)],
            ))
            .expr_stmt(Expression::call(
                Expression::ident("list_length"),
                vec![Expression::ident("lst")],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_obj_operations() {
        // let obj = { x: 10 };
        // obj_set(obj, "y", 20);
        // obj_get(obj, "y") -> 20
        let program = ProgramBuilder::new()
            .var_decl(
                "obj",
                Expression::object(vec![("x", Expression::int(10))]),
            )
            .expr_stmt(Expression::call(
                Expression::ident("obj_set"),
                vec![
                    Expression::ident("obj"),
                    Expression::string("y"),
                    Expression::int(20),
                ],
            ))
            .expr_stmt(Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("obj"), Expression::string("y")],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_string_manipulation() {
        // str_concat("hello", " world")
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("str_concat"),
                vec![Expression::string("hello"), Expression::string(" world")],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("hello world".into())));
    }

    #[test]
    fn test_to_string_conversion() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("to_string"),
                vec![Expression::int(42)],
            ))
            .build();

        let mut vm = IshVm::new();
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Value::String(Rc::new("42".into())));
    }
}
