// ish-vm/src/ledger/type_compat.rs — Type compatibility checking.
//
// Determines whether a runtime Value is compatible with a TypeAnnotation.
// Used by the audit bridge to check assignment, parameter, and return types.

use ish_ast::TypeAnnotation;
use crate::value::Value;

/// Check whether a runtime value is compatible with a type annotation.
///
/// This implements the structural compatibility rules from the ish type spec:
/// - Simple types match by name against Value::type_name()
/// - Union types: value matches if compatible with any member
/// - Optional types: inner type or null
/// - Object types: structural matching (required properties present with compatible types)
/// - Function types: param count matching (runtime functions don't carry type info)
/// - List types: element type checking
/// - Tuple types: position-by-position matching against list values
/// - Intersection types: value must satisfy all constituent types
///   (conflicting primitive types → never, i.e., no value can match)
pub fn is_compatible(value: &Value, annotation: &TypeAnnotation) -> bool {
    match annotation {
        TypeAnnotation::Simple(name) => is_simple_compatible(value, name),

        TypeAnnotation::Union(types) => {
            types.iter().any(|t| is_compatible(value, t))
        }

        TypeAnnotation::Optional(inner) => {
            matches!(value, Value::Null) || is_compatible(value, inner)
        }

        TypeAnnotation::List(element_type) => {
            match value {
                Value::List(list_ref) => {
                    let list = list_ref.borrow();
                    list.iter().all(|item| is_compatible(item, element_type))
                }
                _ => false,
            }
        }

        TypeAnnotation::Object(fields) => {
            match value {
                Value::Object(obj_ref) => {
                    let obj = obj_ref.borrow();
                    fields.iter().all(|(field_name, field_type)| {
                        match obj.get(field_name) {
                            Some(field_value) => is_compatible(field_value, field_type),
                            None => false,
                        }
                    })
                }
                _ => false,
            }
        }

        TypeAnnotation::Function { params, ret: _ } => {
            match value {
                Value::Function(func_ref) => {
                    // Runtime functions only carry param names, not types.
                    // Check param count matches.
                    func_ref.params.len() == params.len()
                }
                Value::BuiltinFunction(_) => {
                    // Builtins have no param info at the Value level; accept.
                    true
                }
                _ => false,
            }
        }

        TypeAnnotation::Tuple(types) => {
            // Tuples are represented as lists at runtime.
            match value {
                Value::List(list_ref) => {
                    let list = list_ref.borrow();
                    list.len() == types.len()
                        && list.iter().zip(types.iter()).all(|(v, t)| is_compatible(v, t))
                }
                _ => false,
            }
        }

        TypeAnnotation::Intersection(types) => {
            // Value must be compatible with ALL constituent types.
            // If constituent types are conflicting primitives (e.g., i32 & String),
            // no value can satisfy both → always false.
            types.iter().all(|t| is_compatible(value, t))
        }

        TypeAnnotation::Generic { base, type_args: _ } => {
            // Generic types degrade to base type matching at runtime.
            // Type arguments are erased (checked at analysis time).
            is_simple_compatible(value, base)
        }
    }
}

/// Check whether a value matches a simple (named) type.
fn is_simple_compatible(value: &Value, type_name: &str) -> bool {
    match type_name {
        // Primitive type names — accept both spec names and runtime names.
        "i32" | "i64" | "int" => matches!(value, Value::Int(_)),
        "f32" | "f64" | "float" => matches!(value, Value::Float(_)),
        "bool" | "boolean" => matches!(value, Value::Bool(_)),
        "String" | "string" => matches!(value, Value::String(_)),
        "char" => matches!(value, Value::Char(_)),
        "null" => matches!(value, Value::Null),

        // Structural type categories.
        "object" => matches!(value, Value::Object(_)),
        "list" => matches!(value, Value::List(_)),
        "function" | "fn" => matches!(value, Value::Function(_) | Value::BuiltinFunction(_)),

        // "any" matches everything; "never" matches nothing.
        "any" => true,
        "never" => false,

        // User-defined type names: check if the value is an object with a
        // matching __type__ property (nominal typing via type declarations).
        user_type => {
            match value {
                Value::Object(obj_ref) => {
                    let obj = obj_ref.borrow();
                    match obj.get("__type__") {
                        Some(Value::String(s)) => s.as_str() == user_type,
                        _ => false,
                    }
                }
                _ => false,
            }
        }
    }
}

/// Check whether two type annotations are structurally compatible.
/// Returns true if a value matching `source` would also match `target`.
///
/// This is used for checking assignment compatibility where both sides
/// have type annotations (e.g., `let x: T = expr` where expr has inferred type).
pub fn types_compatible(source: &TypeAnnotation, target: &TypeAnnotation) -> bool {
    // Handle "any" early — matches everything in both positions.
    if matches!(source, TypeAnnotation::Simple(n) if n == "any")
        || matches!(target, TypeAnnotation::Simple(n) if n == "any")
    {
        return true;
    }

    match (source, target) {
        // Same simple type.
        (TypeAnnotation::Simple(a), TypeAnnotation::Simple(b)) => {
            normalize_type_name(a) == normalize_type_name(b)
        }

        // Source union: every member must match target.
        // (Must come before the blanket Union target arm.)
        (TypeAnnotation::Union(source_types), _) => {
            source_types.iter().all(|s| types_compatible(s, target))
        }

        // Target union: source matches if it fits any member.
        (_, TypeAnnotation::Union(target_types)) => {
            target_types.iter().any(|t| types_compatible(source, t))
        }

        // Optional source: both inner and null must match target.
        (TypeAnnotation::Optional(inner), _) => {
            types_compatible(inner, target)
                && types_compatible(&TypeAnnotation::Simple("null".into()), target)
        }

        // Optional target: source matches inner type or is null.
        (_, TypeAnnotation::Optional(inner)) => {
            types_compatible(source, inner)
                || types_compatible(source, &TypeAnnotation::Simple("null".into()))
        }

        // List types: element types must be compatible.
        (TypeAnnotation::List(a), TypeAnnotation::List(b)) => {
            types_compatible(a, b)
        }

        // Object types: target fields must all be present in source with compatible types.
        (TypeAnnotation::Object(source_fields), TypeAnnotation::Object(target_fields)) => {
            target_fields.iter().all(|(name, target_type)| {
                source_fields.iter().any(|(sn, st)| sn == name && types_compatible(st, target_type))
            })
        }

        // Function types: param count matches, param types contra-variant, return covariant.
        (
            TypeAnnotation::Function { params: sp, ret: sr },
            TypeAnnotation::Function { params: tp, ret: tr },
        ) => {
            sp.len() == tp.len()
                && sp.iter().zip(tp.iter()).all(|(s, t)| types_compatible(t, s)) // contravariant
                && types_compatible(sr, tr) // covariant
        }

        // Tuple types: same length, position-by-position.
        (TypeAnnotation::Tuple(a), TypeAnnotation::Tuple(b)) => {
            a.len() == b.len()
                && a.iter().zip(b.iter()).all(|(sa, sb)| types_compatible(sa, sb))
        }

        // Intersection source: at least one member must be compatible with target.
        (TypeAnnotation::Intersection(source_types), _) => {
            source_types.iter().any(|s| types_compatible(s, target))
        }

        // Intersection target: source must be compatible with all members.
        (_, TypeAnnotation::Intersection(target_types)) => {
            target_types.iter().all(|t| types_compatible(source, t))
        }

        // Generic types: base name match, erase type args.
        (TypeAnnotation::Generic { base: a, .. }, TypeAnnotation::Generic { base: b, .. }) => {
            normalize_type_name(a) == normalize_type_name(b)
        }

        // Fallback: not structurally compatible.
        _ => false,
    }
}

/// Normalize a type name for comparison.
fn normalize_type_name(name: &str) -> &str {
    match name {
        "i32" | "i64" | "int" => "int",
        "f32" | "f64" | "float" => "float",
        "Bool" | "bool" | "boolean" => "bool",
        "String" | "string" => "string",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::rc::Rc;
    use gc::{Gc, GcCell};

    // ── is_compatible: simple types ──────────────────────────────────────

    #[test]
    fn int_matches_int() {
        let val = Value::Int(42);
        assert!(is_compatible(&val, &TypeAnnotation::Simple("i32".into())));
        assert!(is_compatible(&val, &TypeAnnotation::Simple("int".into())));
        assert!(is_compatible(&val, &TypeAnnotation::Simple("i64".into())));
    }

    #[test]
    fn float_matches_float() {
        let val = Value::Float(3.14);
        assert!(is_compatible(&val, &TypeAnnotation::Simple("f64".into())));
        assert!(is_compatible(&val, &TypeAnnotation::Simple("float".into())));
    }

    #[test]
    fn bool_matches_bool() {
        assert!(is_compatible(&Value::Bool(true), &TypeAnnotation::Simple("bool".into())));
    }

    #[test]
    fn string_matches_string() {
        let val = Value::String(Rc::new("hello".into()));
        assert!(is_compatible(&val, &TypeAnnotation::Simple("String".into())));
        assert!(is_compatible(&val, &TypeAnnotation::Simple("string".into())));
    }

    #[test]
    fn null_matches_null() {
        assert!(is_compatible(&Value::Null, &TypeAnnotation::Simple("null".into())));
    }

    #[test]
    fn any_matches_everything() {
        assert!(is_compatible(&Value::Int(1), &TypeAnnotation::Simple("any".into())));
        assert!(is_compatible(&Value::Null, &TypeAnnotation::Simple("any".into())));
    }

    #[test]
    fn never_matches_nothing() {
        assert!(!is_compatible(&Value::Int(1), &TypeAnnotation::Simple("never".into())));
    }

    #[test]
    fn type_mismatch() {
        assert!(!is_compatible(&Value::Int(1), &TypeAnnotation::Simple("String".into())));
        assert!(!is_compatible(&Value::Bool(true), &TypeAnnotation::Simple("int".into())));
    }

    // ── is_compatible: union types ───────────────────────────────────────

    #[test]
    fn union_matches_any_member() {
        let union = TypeAnnotation::Union(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        assert!(is_compatible(&Value::Int(42), &union));
        assert!(is_compatible(&Value::String(Rc::new("hi".into())), &union));
        assert!(!is_compatible(&Value::Bool(true), &union));
    }

    // ── is_compatible: optional types ────────────────────────────────────

    #[test]
    fn optional_matches_inner_or_null() {
        let opt = TypeAnnotation::Optional(Box::new(TypeAnnotation::Simple("i32".into())));
        assert!(is_compatible(&Value::Int(1), &opt));
        assert!(is_compatible(&Value::Null, &opt));
        assert!(!is_compatible(&Value::Bool(true), &opt));
    }

    // ── is_compatible: list types ────────────────────────────────────────

    #[test]
    fn list_matches_element_type() {
        let list_type = TypeAnnotation::List(Box::new(TypeAnnotation::Simple("i32".into())));
        let list = Value::List(Gc::new(GcCell::new(vec![Value::Int(1), Value::Int(2)])));
        assert!(is_compatible(&list, &list_type));
    }

    #[test]
    fn list_rejects_wrong_element() {
        let list_type = TypeAnnotation::List(Box::new(TypeAnnotation::Simple("i32".into())));
        let list = Value::List(Gc::new(GcCell::new(vec![Value::Int(1), Value::Bool(true)])));
        assert!(!is_compatible(&list, &list_type));
    }

    #[test]
    fn empty_list_matches_any_element_type() {
        let list_type = TypeAnnotation::List(Box::new(TypeAnnotation::Simple("i32".into())));
        let list = Value::List(Gc::new(GcCell::new(vec![])));
        assert!(is_compatible(&list, &list_type));
    }

    // ── is_compatible: object structural matching ────────────────────────

    #[test]
    fn object_matches_structurally() {
        let obj_type = TypeAnnotation::Object(vec![
            ("name".into(), TypeAnnotation::Simple("String".into())),
            ("age".into(), TypeAnnotation::Simple("i32".into())),
        ]);
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String(Rc::new("Alice".into())));
        map.insert("age".into(), Value::Int(30));
        map.insert("extra".into(), Value::Bool(true)); // extra field OK
        let obj = Value::Object(Gc::new(GcCell::new(map)));
        assert!(is_compatible(&obj, &obj_type));
    }

    #[test]
    fn object_rejects_missing_field() {
        let obj_type = TypeAnnotation::Object(vec![
            ("name".into(), TypeAnnotation::Simple("String".into())),
            ("age".into(), TypeAnnotation::Simple("i32".into())),
        ]);
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String(Rc::new("Alice".into())));
        let obj = Value::Object(Gc::new(GcCell::new(map)));
        assert!(!is_compatible(&obj, &obj_type));
    }

    #[test]
    fn object_rejects_wrong_field_type() {
        let obj_type = TypeAnnotation::Object(vec![
            ("name".into(), TypeAnnotation::Simple("String".into())),
        ]);
        let mut map = HashMap::new();
        map.insert("name".into(), Value::Int(42));
        let obj = Value::Object(Gc::new(GcCell::new(map)));
        assert!(!is_compatible(&obj, &obj_type));
    }

    // ── is_compatible: tuple types ───────────────────────────────────────

    #[test]
    fn tuple_matches_position_by_position() {
        let tuple_type = TypeAnnotation::Tuple(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        let list = Value::List(Gc::new(GcCell::new(vec![
            Value::Int(1),
            Value::String(Rc::new("hello".into())),
        ])));
        assert!(is_compatible(&list, &tuple_type));
    }

    #[test]
    fn tuple_rejects_wrong_length() {
        let tuple_type = TypeAnnotation::Tuple(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        let list = Value::List(Gc::new(GcCell::new(vec![Value::Int(1)])));
        assert!(!is_compatible(&list, &tuple_type));
    }

    // ── is_compatible: intersection types ────────────────────────────────

    #[test]
    fn intersection_of_objects_matches() {
        // { name: String } & { age: i32 } — object must have both fields.
        let intersection = TypeAnnotation::Intersection(vec![
            TypeAnnotation::Object(vec![("name".into(), TypeAnnotation::Simple("String".into()))]),
            TypeAnnotation::Object(vec![("age".into(), TypeAnnotation::Simple("i32".into()))]),
        ]);
        let mut map = HashMap::new();
        map.insert("name".into(), Value::String(Rc::new("Alice".into())));
        map.insert("age".into(), Value::Int(30));
        let obj = Value::Object(Gc::new(GcCell::new(map)));
        assert!(is_compatible(&obj, &intersection));
    }

    #[test]
    fn intersection_of_conflicting_primitives_matches_nothing() {
        // i32 & String — no value can be both.
        let intersection = TypeAnnotation::Intersection(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        assert!(!is_compatible(&Value::Int(42), &intersection));
        assert!(!is_compatible(&Value::String(Rc::new("hi".into())), &intersection));
    }

    // ── is_compatible: function types ────────────────────────────────────

    #[test]
    fn function_matches_param_count() {
        let fn_type = TypeAnnotation::Function {
            params: vec![TypeAnnotation::Simple("i32".into()), TypeAnnotation::Simple("String".into())],
            ret: Box::new(TypeAnnotation::Simple("bool".into())),
        };
        // Create a function value with 2 params.
        let func = Value::Function(Gc::new(crate::value::IshFunction {
            name: Some("test".into()),
            params: vec!["a".into(), "b".into()],
            param_types: vec![None, None],
            return_type: None,
            body: ish_ast::Statement::Block { statements: vec![] },
            closure_env: crate::environment::Environment::new(),
        }));
        assert!(is_compatible(&func, &fn_type));
    }

    #[test]
    fn function_rejects_wrong_param_count() {
        let fn_type = TypeAnnotation::Function {
            params: vec![TypeAnnotation::Simple("i32".into())],
            ret: Box::new(TypeAnnotation::Simple("bool".into())),
        };
        let func = Value::Function(Gc::new(crate::value::IshFunction {
            name: Some("test".into()),
            params: vec!["a".into(), "b".into()],
            param_types: vec![None, None],
            return_type: None,
            body: ish_ast::Statement::Block { statements: vec![] },
            closure_env: crate::environment::Environment::new(),
        }));
        assert!(!is_compatible(&func, &fn_type));
    }

    // ── types_compatible ─────────────────────────────────────────────────

    #[test]
    fn same_simple_types_compatible() {
        let a = TypeAnnotation::Simple("i32".into());
        let b = TypeAnnotation::Simple("int".into());
        assert!(types_compatible(&a, &b));
    }

    #[test]
    fn different_simple_types_incompatible() {
        let a = TypeAnnotation::Simple("i32".into());
        let b = TypeAnnotation::Simple("String".into());
        assert!(!types_compatible(&a, &b));
    }

    #[test]
    fn source_fits_union_target() {
        let source = TypeAnnotation::Simple("i32".into());
        let target = TypeAnnotation::Union(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        assert!(types_compatible(&source, &target));
    }

    #[test]
    fn union_source_must_fit_all_members() {
        let source = TypeAnnotation::Union(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
        ]);
        let target = TypeAnnotation::Union(vec![
            TypeAnnotation::Simple("i32".into()),
            TypeAnnotation::Simple("String".into()),
            TypeAnnotation::Simple("bool".into()),
        ]);
        assert!(types_compatible(&source, &target));
    }

    #[test]
    fn object_structural_compat() {
        let source = TypeAnnotation::Object(vec![
            ("name".into(), TypeAnnotation::Simple("String".into())),
            ("age".into(), TypeAnnotation::Simple("i32".into())),
        ]);
        let target = TypeAnnotation::Object(vec![
            ("name".into(), TypeAnnotation::Simple("String".into())),
        ]);
        assert!(types_compatible(&source, &target));
    }

    #[test]
    fn function_contravariant_params_covariant_return() {
        let source = TypeAnnotation::Function {
            params: vec![TypeAnnotation::Simple("any".into())],
            ret: Box::new(TypeAnnotation::Simple("i32".into())),
        };
        let target = TypeAnnotation::Function {
            params: vec![TypeAnnotation::Simple("i32".into())],
            ret: Box::new(TypeAnnotation::Simple("int".into())),
        };
        assert!(types_compatible(&source, &target));
    }
}
