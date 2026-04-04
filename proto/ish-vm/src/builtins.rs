// Built-in functions for the ish interpreter.
//
// Organized by category: strings, lists, objects, I/O, types, conversion.

use std::rc::Rc;

use crate::environment::Environment;
use crate::error::{ErrorCode, RuntimeError};
use crate::value::{self, *};

/// Configuration for builtin output routing.
#[derive(Clone, Default)]
pub struct BuiltinConfig {
    /// When set, println/print send output through this channel
    /// instead of writing directly to stdout.
    pub output_sender: Option<crossbeam::channel::Sender<String>>,
}

/// Register all built-in functions into the given environment.
pub fn register_all(env: &Environment) {
    register_all_with_config(env, &BuiltinConfig::default());
}

/// Register all built-in functions with the given configuration.
pub fn register_all_with_config(env: &Environment, config: &BuiltinConfig) {
    register_io(env, config);
    register_strings(env);
    register_lists(env);
    register_objects(env);
    register_types(env);
    register_conversion(env);
    register_errors(env);
    register_ledger(env);
    register_apply(env);
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
            new_compiled_function(&n, vec![], vec![], None, move |_args| {
                Err(RuntimeError::system_error(format!(
                    "{} must be intercepted by the interpreter",
                    n2
                ), ErrorCode::TypeMismatch))
            }, Some(false)),
        );
    }
}

// ── Cross-boundary function call ────────────────────────────────────────────
// apply(fn, args_list) calls fn with elements of args_list as arguments.
// Since shims are self-contained, apply just calls the target function's shim
// directly, returning either Value::Future (yielding) or the result (unyielding).

fn register_apply(env: &Environment) {
    env.define(
        "apply".into(),
        new_compiled_function("apply", vec!["fn".into(), "args".into()], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error(
                    "apply expects 2 arguments: a function and a list of arguments",
                    ErrorCode::ArgumentCountMismatch,
                ));
            }
            let func = &args[0];
            let arg_list = match &args[1] {
                Value::List(list) => {
                    let items: Vec<Value> = list.borrow().iter().cloned().collect();
                    items
                }
                _ => {
                    return Err(RuntimeError::system_error(
                        "apply expects a list as the second argument",
                        ErrorCode::TypeMismatch,
                    ));
                }
            };
            match func {
                Value::Function(f) => (f.shim)(&arg_list),
                _ => Err(RuntimeError::system_error(
                    format!("apply first argument must be a function, got {}", func.type_name()),
                    ErrorCode::TypeMismatch,
                )),
            }
        }, Some(false)),
    );
}

// ── I/O ─────────────────────────────────────────────────────────────────────

fn register_io(env: &Environment, config: &BuiltinConfig) {
    let print_sender = config.output_sender.clone();
    env.define(
        "print".into(),
        new_compiled_function("print", vec![], vec![], None, move |args| {
            let mut output = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    output.push(' ');
                }
                output.push_str(&arg.to_display_string());
            }
            if let Some(ref sender) = print_sender {
                let _ = sender.send(output);
            } else {
                print!("{}", output);
            }
            Ok(Value::Null)
        }, Some(false)),
    );

    let println_sender = config.output_sender.clone();
    env.define(
        "println".into(),
        new_compiled_function("println", vec![], vec![], None, move |args| {
            let mut output = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    output.push(' ');
                }
                output.push_str(&arg.to_display_string());
            }
            if let Some(ref sender) = println_sender {
                let _ = sender.send(output);
            } else {
                println!("{}", output);
            }
            Ok(Value::Null)
        }, Some(false)),
    );

    env.define(
        "read_file".into(),
        new_compiled_function("read_file", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("read_file expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(path) = &args[0] {
                match std::fs::read_to_string(path.as_ref()) {
                    Ok(content) => Ok(Value::String(Rc::new(content))),
                    Err(e) => Err(RuntimeError::system_error(format!("read_file error: {}", e), ErrorCode::IoError)),
                }
            } else {
                Err(RuntimeError::system_error("read_file expects a string path", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "write_file".into(),
        new_compiled_function("write_file", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("write_file expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::write(path.as_ref(), content.as_ref()) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(RuntimeError::system_error(format!("write_file error: {}", e), ErrorCode::IoError)),
                }
            } else {
                Err(RuntimeError::system_error(
                    "write_file expects (string path, string content)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );
}

// ── String operations ───────────────────────────────────────────────────────

fn register_strings(env: &Environment) {
    env.define(
        "str_concat".into(),
        new_compiled_function("str_concat", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_concat expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            let a = args[0].to_display_string();
            let b = args[1].to_display_string();
            Ok(Value::String(Rc::new(format!("{}{}", a, b))))
        }, Some(false)),
    );

    env.define(
        "str_length".into(),
        new_compiled_function("str_length", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_length expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::Int(s.len() as i64))
            } else {
                Err(RuntimeError::system_error("str_length expects a string", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_slice".into(),
        new_compiled_function("str_slice", vec![], vec![], None, |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("str_slice expects 3 arguments", ErrorCode::ArgumentCountMismatch));
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
                Err(RuntimeError::system_error("str_slice expects (string, int, int)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_contains".into(),
        new_compiled_function("str_contains", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_contains expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(s), Value::String(substr)) = (&args[0], &args[1]) {
                Ok(Value::Bool(s.contains(substr.as_ref())))
            } else {
                Err(RuntimeError::system_error("str_contains expects (string, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_starts_with".into(),
        new_compiled_function("str_starts_with", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_starts_with expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(s), Value::String(prefix)) = (&args[0], &args[1]) {
                Ok(Value::Bool(s.starts_with(prefix.as_ref())))
            } else {
                Err(RuntimeError::system_error(
                    "str_starts_with expects (string, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_replace".into(),
        new_compiled_function("str_replace", vec![], vec![], None, |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("str_replace expects 3 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(s), Value::String(from), Value::String(to)) =
                (&args[0], &args[1], &args[2])
            {
                Ok(Value::String(Rc::new(
                    s.replace(from.as_ref(), to.as_ref()),
                )))
            } else {
                Err(RuntimeError::system_error(
                    "str_replace expects (string, string, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_split".into(),
        new_compiled_function("str_split", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_split expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(s), Value::String(delim)) = (&args[0], &args[1]) {
                let parts: Vec<Value> = s
                    .split(delim.as_ref())
                    .map(|p| Value::String(Rc::new(p.to_string())))
                    .collect();
                Ok(new_list(parts))
            } else {
                Err(RuntimeError::system_error("str_split expects (string, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_to_upper".into(),
        new_compiled_function("str_to_upper", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_to_upper expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.to_uppercase())))
            } else {
                Err(RuntimeError::system_error("str_to_upper expects a string", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_to_lower".into(),
        new_compiled_function("str_to_lower", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_to_lower expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.to_lowercase())))
            } else {
                Err(RuntimeError::system_error("str_to_lower expects a string", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_char_at".into(),
        new_compiled_function("str_char_at", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("str_char_at expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::String(s), Value::Int(i)) = (&args[0], &args[1]) {
                let i = *i as usize;
                if let Some(ch) = s.chars().nth(i) {
                    Ok(Value::String(Rc::new(ch.to_string())))
                } else {
                    Ok(Value::Null)
                }
            } else {
                Err(RuntimeError::system_error("str_char_at expects (string, int)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "str_trim".into(),
        new_compiled_function("str_trim", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("str_trim expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(s) = &args[0] {
                Ok(Value::String(Rc::new(s.trim().to_string())))
            } else {
                Err(RuntimeError::system_error("str_trim expects a string", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );
}

// ── List operations ─────────────────────────────────────────────────────────

fn register_lists(env: &Environment) {
    env.define(
        "list_push".into(),
        new_compiled_function("list_push", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_push expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::List(list_ref) = &args[0] {
                list_ref.borrow_mut().push(args[1].clone());
                Ok(Value::Null)
            } else {
                Err(RuntimeError::system_error("list_push expects a list as first argument", ErrorCode::ArgumentCountMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_pop".into(),
        new_compiled_function("list_pop", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("list_pop expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::List(list_ref) = &args[0] {
                Ok(list_ref.borrow_mut().pop().unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("list_pop expects a list", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_length".into(),
        new_compiled_function("list_length", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("list_length expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::List(list_ref) = &args[0] {
                Ok(Value::Int(list_ref.borrow().len() as i64))
            } else {
                Err(RuntimeError::system_error("list_length expects a list", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_get".into(),
        new_compiled_function("list_get", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_get expects 2 arguments", ErrorCode::ArgumentCountMismatch));
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
                    ), ErrorCode::TypeMismatch))
                }
            } else {
                Err(RuntimeError::system_error("list_get expects (list, int)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_set".into(),
        new_compiled_function("list_set", vec![], vec![], None, |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("list_set expects 3 arguments", ErrorCode::ArgumentCountMismatch));
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
                    ), ErrorCode::TypeMismatch))
                }
            } else {
                Err(RuntimeError::system_error("list_set expects (list, int, value)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_slice".into(),
        new_compiled_function("list_slice", vec![], vec![], None, |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("list_slice expects 3 arguments", ErrorCode::ArgumentCountMismatch));
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
                Err(RuntimeError::system_error("list_slice expects (list, int, int)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "list_join".into(),
        new_compiled_function("list_join", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("list_join expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::List(list_ref), Value::String(sep)) = (&args[0], &args[1]) {
                let list = list_ref.borrow();
                let strs: Vec<String> = list.iter().map(|v| v.to_display_string()).collect();
                Ok(Value::String(Rc::new(strs.join(sep.as_ref()))))
            } else {
                Err(RuntimeError::system_error("list_join expects (list, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );
}

// ── Object operations ───────────────────────────────────────────────────────

fn register_objects(env: &Environment) {
    env.define(
        "obj_get".into(),
        new_compiled_function("obj_get", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_get expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                Ok(obj_ref
                    .borrow()
                    .get(key.as_ref())
                    .cloned()
                    .unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("obj_get expects (object, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "obj_set".into(),
        new_compiled_function("obj_set", vec![], vec![], None, |args| {
            if args.len() != 3 {
                return Err(RuntimeError::system_error("obj_set expects 3 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                obj_ref
                    .borrow_mut()
                    .insert(key.as_ref().clone(), args[2].clone());
                Ok(Value::Null)
            } else {
                Err(RuntimeError::system_error("obj_set expects (object, string, value)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "obj_has".into(),
        new_compiled_function("obj_has", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_has expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                Ok(Value::Bool(obj_ref.borrow().contains_key(key.as_ref())))
            } else {
                Err(RuntimeError::system_error("obj_has expects (object, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "obj_keys".into(),
        new_compiled_function("obj_keys", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("obj_keys expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::Object(obj_ref) = &args[0] {
                let keys: Vec<Value> = obj_ref
                    .borrow()
                    .keys()
                    .map(|k| Value::String(Rc::new(k.clone())))
                    .collect();
                Ok(new_list(keys))
            } else {
                Err(RuntimeError::system_error("obj_keys expects an object", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "obj_values".into(),
        new_compiled_function("obj_values", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("obj_values expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::Object(obj_ref) = &args[0] {
                let values: Vec<Value> = obj_ref.borrow().values().cloned().collect();
                Ok(new_list(values))
            } else {
                Err(RuntimeError::system_error("obj_values expects an object", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    env.define(
        "obj_remove".into(),
        new_compiled_function("obj_remove", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("obj_remove expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let (Value::Object(obj_ref), Value::String(key)) = (&args[0], &args[1]) {
                let removed = obj_ref
                    .borrow_mut()
                    .remove(key.as_ref())
                    .unwrap_or(Value::Null);
                Ok(removed)
            } else {
                Err(RuntimeError::system_error("obj_remove expects (object, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );
}

// ── Type checking ───────────────────────────────────────────────────────────

fn register_types(env: &Environment) {
    env.define(
        "type_of".into(),
        new_compiled_function("type_of", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("type_of expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            Ok(Value::String(Rc::new(args[0].type_name().to_string())))
        }, Some(false)),
    );

    env.define(
        "is_type".into(),
        new_compiled_function("is_type", vec![], vec![], None, |args| {
            if args.len() != 2 {
                return Err(RuntimeError::system_error("is_type expects 2 arguments", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::String(type_name) = &args[1] {
                Ok(Value::Bool(args[0].type_name() == type_name.as_ref()))
            } else {
                Err(RuntimeError::system_error("is_type expects (value, string)", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );
}

// ── Conversion ──────────────────────────────────────────────────────────────

fn register_conversion(env: &Environment) {
    env.define(
        "to_string".into(),
        new_compiled_function("to_string", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_string expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            Ok(Value::String(Rc::new(args[0].to_display_string())))
        }, Some(false)),
    );

    env.define(
        "to_int".into(),
        new_compiled_function("to_int", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_int expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            match &args[0] {
                Value::Int(n) => Ok(Value::Int(*n)),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                Value::String(s) => match s.parse::<i64>() {
                    Ok(n) => Ok(Value::Int(n)),
                    Err(_) => Err(RuntimeError::system_error(format!(
                        "cannot convert '{}' to int",
                        s
                    ), ErrorCode::TypeMismatch)),
                },
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to int",
                    args[0].type_name()
                ), ErrorCode::TypeMismatch)),
            }
        }, Some(false)),
    );

    env.define(
        "to_float".into(),
        new_compiled_function("to_float", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("to_float expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::Int(n) => Ok(Value::Float(*n as f64)),
                Value::String(s) => match s.parse::<f64>() {
                    Ok(f) => Ok(Value::Float(f)),
                    Err(_) => Err(RuntimeError::system_error(format!(
                        "cannot convert '{}' to float",
                        s
                    ), ErrorCode::TypeMismatch)),
                },
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to float",
                    args[0].type_name()
                ), ErrorCode::TypeMismatch)),
            }
        }, Some(false)),
    );

    env.define(
        "char".into(),
        new_compiled_function("char", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("char expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            match &args[0] {
                Value::String(s) => {
                    let mut chars = s.chars();
                    if let Some(c) = chars.next() {
                        if chars.next().is_none() {
                            Ok(Value::Char(c))
                        } else {
                            Err(RuntimeError::system_error("char expects a single-character string", ErrorCode::TypeMismatch))
                        }
                    } else {
                        Err(RuntimeError::system_error("char expects a non-empty string", ErrorCode::TypeMismatch))
                    }
                }
                Value::Int(n) => {
                    let code = *n as u32;
                    match char::from_u32(code) {
                        Some(c) => Ok(Value::Char(c)),
                        None => Err(RuntimeError::system_error(format!(
                            "invalid Unicode code point: {}",
                            n
                        ), ErrorCode::TypeMismatch)),
                    }
                }
                Value::Char(c) => Ok(Value::Char(*c)),
                _ => Err(RuntimeError::system_error(format!(
                    "cannot convert {} to char",
                    args[0].type_name()
                ), ErrorCode::TypeMismatch)),
            }
        }, Some(false)),
    );
}

// ── Error Handling ──────────────────────────────────────────────────────────

fn register_errors(env: &Environment) {
    // is_error(value) -> checks if a value is an Error object
    // An error object is any object with a "message" String property.
    env.define(
        "is_error".into(),
        new_compiled_function("is_error", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("is_error expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            let result = if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                matches!(map.get("message"), Some(Value::String(_)))
            } else {
                false
            };
            Ok(Value::Bool(result))
        }, Some(false)),
    );

    // error_message(error) -> extracts the message from an Error object
    env.define(
        "error_message".into(),
        new_compiled_function("error_message", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("error_message expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                Ok(map.get("message").cloned().unwrap_or(Value::Null))
            } else {
                Err(RuntimeError::system_error("error_message expects an error object", ErrorCode::TypeMismatch))
            }
        }, Some(false)),
    );

    // error_code(error) -> extracts the code from a CodedError object (null if not CodedError)
    env.define(
        "error_code".into(),
        new_compiled_function("error_code", vec![], vec![], None, |args| {
            if args.len() != 1 {
                return Err(RuntimeError::system_error("error_code expects 1 argument", ErrorCode::ArgumentCountMismatch));
            }
            if let Value::Object(ref obj_ref) = args[0] {
                let map = obj_ref.borrow();
                Ok(map.get("code").cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }, Some(false)),
    );
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use ish_ast::*;
    use ish_ast::builder::ProgramBuilder;
    use crate::interpreter::IshVm;

    #[tokio::test]
    async fn test_type_of() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("type_of"),
                vec![Expression::int(42)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("int".into())));
    }

    #[tokio::test]
    async fn test_str_length() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("str_length"),
                vec![Expression::string("hello")],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[tokio::test]
    async fn test_list_operations() {
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

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(4));
    }

    #[tokio::test]
    async fn test_obj_operations() {
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

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[tokio::test]
    async fn test_string_manipulation() {
        // str_concat("hello", " world")
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("str_concat"),
                vec![Expression::string("hello"), Expression::string(" world")],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("hello world".into())));
    }

    #[tokio::test]
    async fn test_to_string_conversion() {
        let program = ProgramBuilder::new()
            .expr_stmt(Expression::call(
                Expression::ident("to_string"),
                vec![Expression::int(42)],
            ))
            .build();

        let vm = Rc::new(RefCell::new(IshVm::new()));
        let result = IshVm::run(&vm, &program).await.unwrap();
        assert_eq!(result, Value::String(Rc::new("42".into())));
    }
}
