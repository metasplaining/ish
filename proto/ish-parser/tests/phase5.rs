use ish_parser::parse;
use ish_ast::*;

#[test]
fn parse_priv_fn() {
    let prog = parse("priv fn f() {\n  return 0\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "f");
            assert_eq!(visibility, &Some(Visibility::Priv));
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_pkg_fn() {
    let prog = parse("pkg fn f() {\n  return 0\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "f");
            assert_eq!(visibility, &Some(Visibility::Pkg));
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_fn() {
    let prog = parse("pub fn f() {\n  return 0\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "f");
            assert_eq!(visibility, &Some(Visibility::Pub));
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_no_visibility_fn() {
    let prog = parse("fn f() {\n  return 0\n}").unwrap();
    match &prog.statements[0] {
        Statement::FunctionDecl { name, visibility, .. } => {
            assert_eq!(name, "f");
            assert_eq!(visibility, &None);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_priv_let() {
    let prog = parse("priv let x = 1").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { name, visibility, .. } => {
            assert_eq!(name, "x");
            assert_eq!(visibility, &Some(Visibility::Priv));
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_type() {
    let prog = parse("pub type Id = int").unwrap();
    match &prog.statements[0] {
        Statement::TypeAlias { name, visibility, .. } => {
            assert_eq!(name, "Id");
            assert_eq!(visibility, &Some(Visibility::Pub));
        }
        other => panic!("expected TypeAlias, got {:?}", other),
    }
}

#[test]
fn parse_use_plain() {
    let prog = parse("use foo/bar").unwrap();
    match &prog.statements[0] {
        Statement::Use { module_path, alias, selective } => {
            assert_eq!(module_path, &vec!["foo".to_string(), "bar".to_string()]);
            assert_eq!(alias, &None);
            assert_eq!(selective, &None);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_use_aliased() {
    let prog = parse("use foo/bar as b").unwrap();
    match &prog.statements[0] {
        Statement::Use { module_path, alias, selective } => {
            assert_eq!(module_path, &vec!["foo".to_string(), "bar".to_string()]);
            assert_eq!(alias, &Some("b".to_string()));
            assert_eq!(selective, &None);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_use_selective() {
    let prog = parse("use foo/bar { Type }").unwrap();
    match &prog.statements[0] {
        Statement::Use { module_path, alias, selective } => {
            assert_eq!(module_path, &vec!["foo".to_string(), "bar".to_string()]);
            assert_eq!(alias, &None);
            let sel = selective.as_ref().unwrap();
            assert_eq!(sel.len(), 1);
            assert_eq!(sel[0].name, "Type");
            assert_eq!(sel[0].alias, None);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_use_selective_rename() {
    let prog = parse("use foo/bar { Type as T }").unwrap();
    match &prog.statements[0] {
        Statement::Use { module_path, alias, selective } => {
            assert_eq!(module_path, &vec!["foo".to_string(), "bar".to_string()]);
            assert_eq!(alias, &None);
            let sel = selective.as_ref().unwrap();
            assert_eq!(sel.len(), 1);
            assert_eq!(sel[0].name, "Type");
            assert_eq!(sel[0].alias, Some("T".to_string()));
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_use_external() {
    let prog = parse("use example.com/foo/bar").unwrap();
    match &prog.statements[0] {
        Statement::Use { module_path, alias, selective } => {
            assert_eq!(module_path, &vec!["example.com".to_string(), "foo".to_string(), "bar".to_string()]);
            assert_eq!(alias, &None);
            assert_eq!(selective, &None);
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn parse_declare_block() {
    let prog = parse("declare {\n  fn a() {\n    return 1\n  }\n  fn b() {\n    return 2\n  }\n}").unwrap();
    match &prog.statements[0] {
        Statement::DeclareBlock { body } => {
            assert_eq!(body.len(), 2);
            match &body[0] {
                Statement::FunctionDecl { name, .. } => assert_eq!(name, "a"),
                other => panic!("expected FunctionDecl, got {:?}", other),
            }
            match &body[1] {
                Statement::FunctionDecl { name, .. } => assert_eq!(name, "b"),
                other => panic!("expected FunctionDecl, got {:?}", other),
            }
        }
        other => panic!("expected DeclareBlock, got {:?}", other),
    }
}

#[test]
fn parse_bootstrap_path() {
    let prog = parse("bootstrap 'path/to/cfg.json'").unwrap();
    match &prog.statements[0] {
        Statement::Bootstrap { source } => {
            match source {
                BootstrapSource::Path(p) => assert_eq!(p, "path/to/cfg.json"),
                other => panic!("expected Path, got {:?}", other),
            }
        }
        other => panic!("expected Bootstrap, got {:?}", other),
    }
}

#[test]
fn parse_bootstrap_url() {
    let prog = parse("bootstrap 'https://example.com/cfg.json'").unwrap();
    match &prog.statements[0] {
        Statement::Bootstrap { source } => {
            match source {
                BootstrapSource::Url(u) => assert_eq!(u, "https://example.com/cfg.json"),
                other => panic!("expected Url, got {:?}", other),
            }
        }
        other => panic!("expected Bootstrap, got {:?}", other),
    }
}

#[test]
fn parse_bootstrap_inline() {
    let prog = parse(r#"bootstrap { "ish": ">=1.0" }"#).unwrap();
    match &prog.statements[0] {
        Statement::Bootstrap { source } => {
            match source {
                BootstrapSource::Inline(json) => {
                    assert!(json.contains("ish"));
                }
                other => panic!("expected Inline, got {:?}", other),
            }
        }
        other => panic!("expected Bootstrap, got {:?}", other),
    }
}

#[test]
fn parse_incomplete_declare() {
    let prog = parse("declare {").unwrap();
    assert!(prog.has_incomplete_continuable());
    match &prog.statements[0] {
        Statement::Incomplete { kind } => {
            assert_eq!(kind, &IncompleteKind::DeclareBlock);
        }
        other => panic!("expected Incomplete, got {:?}", other),
    }
}
