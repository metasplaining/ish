use ish_parser::parse;
use ish_ast::*;

#[test]
fn parse_use_simple() {
    let prog = parse("use math").unwrap();
    assert_eq!(prog.statements.len(), 1);
    match &prog.statements[0] {
        Statement::Use { path } => {
            assert_eq!(path, &vec!["math".to_string()]);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_use_nested_path() {
    let prog = parse("use std::collections::list").unwrap();
    match &prog.statements[0] {
        Statement::Use { path } => {
            assert_eq!(path, &vec![
                "std".to_string(),
                "collections".to_string(),
                "list".to_string(),
            ]);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_mod_file_module() {
    let prog = parse("mod utils").unwrap();
    match &prog.statements[0] {
        Statement::ModDecl { name, body, visibility } => {
            assert_eq!(name, "utils");
            assert!(body.is_none());
            assert!(visibility.is_none());
        }
        other => panic!("expected ModDecl, got {:?}", other),
    }
}

#[test]
fn parse_mod_inline_block() {
    let prog = parse("mod helpers {\n  fn add(a, b) {\n    return a + b\n  }\n}").unwrap();
    match &prog.statements[0] {
        Statement::ModDecl { name, body, visibility } => {
            assert_eq!(name, "helpers");
            assert!(body.is_some());
            assert!(visibility.is_none());
        }
        other => panic!("expected ModDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_fn() {
    let prog = parse("pub fn greet(name) {\n  return name\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(visibility, &Some(Visibility::Public));
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_scope_fn() {
    let prog = parse("pub(super) fn internal() {\n  return null\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "internal");
            assert_eq!(visibility, &Some(Visibility::PubScope("super".to_string())));
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_let() {
    let prog = parse("pub let x = 42").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { name, visibility, .. } => {
            assert_eq!(name, "x");
            assert_eq!(visibility, &Some(Visibility::Public));
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_type_alias() {
    let prog = parse("pub type Id = int").unwrap();
    match &prog.statements[0] {
        Statement::TypeAlias { name, visibility, .. } => {
            assert_eq!(name, "Id");
            assert_eq!(visibility, &Some(Visibility::Public));
        }
        other => panic!("expected TypeAlias, got {:?}", other),
    }
}

#[test]
fn parse_pub_mod() {
    let prog = parse("pub mod api {\n  fn handler() {\n    return null\n  }\n}").unwrap();
    match &prog.statements[0] {
        Statement::ModDecl { name, visibility, .. } => {
            assert_eq!(name, "api");
            assert_eq!(visibility, &Some(Visibility::Public));
        }
        other => panic!("expected ModDecl, got {:?}", other),
    }
}

#[test]
fn parse_private_fn_no_modifier() {
    let prog = parse("fn secret() {\n  return 0\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "secret");
            assert_eq!(visibility, &None);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}
