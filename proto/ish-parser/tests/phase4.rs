#[cfg(test)]
mod tests {
    use ish_parser::parse;
    use ish_ast::*;

    // ── Simple Type Annotations ─────────────────────────────────────────

    #[test]
    fn test_simple_type() {
        let prog = parse("let x: int = 5").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Simple(t)), .. } => {
                assert_eq!(t, "int");
            }
            other => panic!("expected simple type, got {:?}", other),
        }
    }

    // ── Union Types ─────────────────────────────────────────────────────

    #[test]
    fn test_union_type() {
        let prog = parse("let x: int | string = 5").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Union(types)), .. } => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], TypeAnnotation::Simple("int".into()));
                assert_eq!(types[1], TypeAnnotation::Simple("string".into()));
            }
            other => panic!("expected union type, got {:?}", other),
        }
    }

    #[test]
    fn test_triple_union_type() {
        let prog = parse("let x: int | string | bool = 5").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Union(types)), .. } => {
                assert_eq!(types.len(), 3);
            }
            other => panic!("expected union type, got {:?}", other),
        }
    }

    // ── Function Types ──────────────────────────────────────────────────

    #[test]
    fn test_function_type_annotation() {
        let prog = parse("let f: fn(int, string) -> bool = g").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Function { params, ret }), .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], TypeAnnotation::Simple("int".into()));
                assert_eq!(params[1], TypeAnnotation::Simple("string".into()));
                assert_eq!(**ret, TypeAnnotation::Simple("bool".into()));
            }
            other => panic!("expected function type, got {:?}", other),
        }
    }

    #[test]
    fn test_function_type_no_params() {
        let prog = parse("let f: fn() -> int = g").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Function { params, ret }), .. } => {
                assert_eq!(params.len(), 0);
                assert_eq!(**ret, TypeAnnotation::Simple("int".into()));
            }
            other => panic!("expected function type, got {:?}", other),
        }
    }

    // ── Tuple Types ─────────────────────────────────────────────────────

    #[test]
    fn test_tuple_type() {
        let prog = parse("let p: (int, string) = pair").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Tuple(types)), .. } => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], TypeAnnotation::Simple("int".into()));
                assert_eq!(types[1], TypeAnnotation::Simple("string".into()));
            }
            other => panic!("expected tuple type, got {:?}", other),
        }
    }

    #[test]
    fn test_triple_tuple() {
        let prog = parse("let t: (int, string, bool) = val").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { type_annotation: Some(TypeAnnotation::Tuple(types)), .. } => {
                assert_eq!(types.len(), 3);
            }
            other => panic!("expected tuple type, got {:?}", other),
        }
    }

    // ── Type Aliases ────────────────────────────────────────────────────

    #[test]
    fn test_type_alias_simple() {
        let prog = parse("type UserId = int").unwrap();
        match &prog.statements[0] {
            Statement::TypeAlias { name, definition, .. } => {
                assert_eq!(name, "UserId");
                assert_eq!(*definition, TypeAnnotation::Simple("int".into()));
            }
            other => panic!("expected type alias, got {:?}", other),
        }
    }

    #[test]
    fn test_type_alias_union() {
        let prog = parse("type Result = Success | Error").unwrap();
        match &prog.statements[0] {
            Statement::TypeAlias { name, definition, .. } => {
                assert_eq!(name, "Result");
                match definition {
                    TypeAnnotation::Union(types) => {
                        assert_eq!(types.len(), 2);
                    }
                    other => panic!("expected union type, got {:?}", other),
                }
            }
            other => panic!("expected type alias, got {:?}", other),
        }
    }

    // ── Function Return Types ───────────────────────────────────────────

    #[test]
    fn test_fn_with_union_return_type() {
        let prog = parse("fn find(x: int) -> int | null {\n    return null\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { return_type: Some(TypeAnnotation::Union(types)), .. } => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], TypeAnnotation::Simple("int".into()));
                assert_eq!(types[1], TypeAnnotation::Simple("null".into()));
            }
            other => panic!("expected fn with union return, got {:?}", other),
        }
    }
}
