/// Phase 8: Advanced Features — match statements and generics
use ish_parser::parse;
use ish_ast::*;

// ── Match Statements ──────────────────────────────────────────────

#[test]
fn match_on_integer_literals() {
    let prog = parse("match x {\n  1 => print(\"one\")\n  2 => print(\"two\")\n  _ => print(\"other\")\n}").unwrap();
    assert_eq!(prog.statements.len(), 1);
    match &prog.statements[0] {
        Statement::Match { subject, arms } => {
            assert!(matches!(subject, Expression::Identifier(n) if n == "x"));
            assert_eq!(arms.len(), 3);
            assert!(matches!(&arms[0].pattern, MatchPattern::Literal(Literal::Int(1))));
            assert!(matches!(&arms[1].pattern, MatchPattern::Literal(Literal::Int(2))));
            assert!(matches!(&arms[2].pattern, MatchPattern::Wildcard));
        }
        _ => panic!("expected Match"),
    }
}

#[test]
fn match_on_string_literals() {
    let prog = parse("match cmd {\n  \"start\" => run()\n  \"stop\" => halt()\n}").unwrap();
    match &prog.statements[0] {
        Statement::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            match &arms[0].pattern {
                MatchPattern::Literal(Literal::String(s)) => assert_eq!(s, "start"),
                _ => panic!("expected string literal pattern"),
            }
        }
        _ => panic!("expected Match"),
    }
}

#[test]
fn match_with_variable_binding() {
    let prog = parse("match value {\n  x => print(x)\n}").unwrap();
    match &prog.statements[0] {
        Statement::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert!(matches!(&arms[0].pattern, MatchPattern::Identifier(n) if n == "x"));
        }
        _ => panic!("expected Match"),
    }
}

#[test]
fn match_with_block_body() {
    let prog = parse("match n {\n  1 => {\n    let x = 10\n    print(x)\n  }\n  _ => print(0)\n}").unwrap();
    match &prog.statements[0] {
        Statement::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert!(matches!(&arms[0].body, Statement::Block { .. }));
        }
        _ => panic!("expected Match"),
    }
}

#[test]
fn match_with_boolean_and_null_patterns() {
    let prog = parse("match flag {\n  true => yes()\n  false => no()\n  null => unknown()\n}").unwrap();
    match &prog.statements[0] {
        Statement::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            assert!(matches!(&arms[0].pattern, MatchPattern::Literal(Literal::Bool(true))));
            assert!(matches!(&arms[1].pattern, MatchPattern::Literal(Literal::Bool(false))));
            assert!(matches!(&arms[2].pattern, MatchPattern::Literal(Literal::Null)));
        }
        _ => panic!("expected Match"),
    }
}

// ── Generic Functions ─────────────────────────────────────────────

#[test]
fn generic_function_single_param() {
    let prog = parse("fn identity<T>(x: T) -> T {\n  return x\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, type_params, params, return_type, .. } => {
            assert_eq!(name, "identity");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(params.len(), 1);
            assert!(matches!(&params[0].type_annotation, Some(TypeAnnotation::Simple(t)) if t == "T"));
            assert!(matches!(return_type, Some(TypeAnnotation::Simple(t)) if t == "T"));
        }
        _ => panic!("expected FunctionDecl"),
    }
}

#[test]
fn generic_function_multiple_params() {
    let prog = parse("fn map<T, U>(items: List[T], f: fn(T) -> U) -> List[U] {\n  return items\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, type_params, .. } => {
            assert_eq!(name, "map");
            assert_eq!(type_params, &vec!["T".to_string(), "U".to_string()]);
        }
        _ => panic!("expected FunctionDecl"),
    }
}

// ── Generic Type Annotations ──────────────────────────────────────

#[test]
fn generic_type_single_arg() {
    let prog = parse("let items: Result<int> = ok(42)").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { type_annotation: Some(ta), .. } => {
            match ta {
                TypeAnnotation::Generic { base, type_args } => {
                    assert_eq!(base, "Result");
                    assert_eq!(type_args.len(), 1);
                    assert!(matches!(&type_args[0], TypeAnnotation::Simple(t) if t == "int"));
                }
                _ => panic!("expected Generic type, got {:?}", ta),
            }
        }
        _ => panic!("expected VariableDecl"),
    }
}

#[test]
fn generic_type_multiple_args() {
    let prog = parse("let m: Map<string, int> = create()").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { type_annotation: Some(ta), .. } => {
            match ta {
                TypeAnnotation::Generic { base, type_args } => {
                    assert_eq!(base, "Map");
                    assert_eq!(type_args.len(), 2);
                    assert!(matches!(&type_args[0], TypeAnnotation::Simple(t) if t == "string"));
                    assert!(matches!(&type_args[1], TypeAnnotation::Simple(t) if t == "int"));
                }
                _ => panic!("expected Generic type, got {:?}", ta),
            }
        }
        _ => panic!("expected VariableDecl"),
    }
}
