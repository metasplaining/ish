use ish_parser::parse;
use ish_ast::*;

#[test]
fn parse_standard_annotation_on_fn() {
    let prog = parse("@standard[cautious]\nfn safe_add(a, b) {\n  return a + b\n}").unwrap();
    assert_eq!(prog.statements.len(), 1);
    match &prog.statements[0] {
        Statement::Annotated { annotations, inner } => {
            assert_eq!(annotations.len(), 1);
            assert!(matches!(&annotations[0], Annotation::Standard(name) if name == "cautious"));
            assert!(matches!(inner.as_ref(), Statement::FunctionDecl { .. }));
        }
        other => panic!("expected Annotated, got {:?}", other),
    }
}

#[test]
fn parse_entry_annotation_boolean() {
    let prog = parse("@[mutable]\nlet x = 5").unwrap();
    match &prog.statements[0] {
        Statement::Annotated { annotations, inner } => {
            assert_eq!(annotations.len(), 1);
            match &annotations[0] {
                Annotation::Entry(items) => {
                    assert_eq!(items.len(), 1);
                    assert_eq!(items[0].name, "mutable");
                    assert!(items[0].value.is_none());
                }
                other => panic!("expected Entry, got {:?}", other),
            }
            assert!(matches!(inner.as_ref(), Statement::VariableDecl { .. }));
        }
        other => panic!("expected Annotated, got {:?}", other),
    }
}

#[test]
fn parse_entry_annotation_with_value() {
    let prog = parse("@[type(i32)]\nlet x = 5").unwrap();
    match &prog.statements[0] {
        Statement::Annotated { annotations, inner } => {
            match &annotations[0] {
                Annotation::Entry(items) => {
                    assert_eq!(items[0].name, "type");
                    assert_eq!(items[0].value, Some("i32".to_string()));
                }
                other => panic!("expected Entry, got {:?}", other),
            }
            assert!(matches!(inner.as_ref(), Statement::VariableDecl { .. }));
        }
        other => panic!("expected Annotated, got {:?}", other),
    }
}

#[test]
fn parse_multiple_entry_items() {
    let prog = parse("@[mutable, nullable]\nlet x = 5").unwrap();
    match &prog.statements[0] {
        Statement::Annotated { annotations, .. } => {
            match &annotations[0] {
                Annotation::Entry(items) => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].name, "mutable");
                    assert_eq!(items[1].name, "nullable");
                }
                other => panic!("expected Entry, got {:?}", other),
            }
        }
        other => panic!("expected Annotated, got {:?}", other),
    }
}

#[test]
fn parse_multiple_annotations() {
    let prog = parse("@standard[cautious]\n@[pure]\nfn add(a, b) {\n  return a + b\n}").unwrap();
    match &prog.statements[0] {
        Statement::Annotated { annotations, inner } => {
            assert_eq!(annotations.len(), 2);
            assert!(matches!(&annotations[0], Annotation::Standard(name) if name == "cautious"));
            assert!(matches!(&annotations[1], Annotation::Entry(items) if items[0].name == "pure"));
            assert!(matches!(inner.as_ref(), Statement::FunctionDecl { .. }));
        }
        other => panic!("expected Annotated, got {:?}", other),
    }
}

#[test]
fn parse_standard_def_simple() {
    let prog = parse("standard streamlined []").unwrap();
    match &prog.statements[0] {
        Statement::StandardDef { name, extends, features } => {
            assert_eq!(name, "streamlined");
            assert!(extends.is_none());
            assert!(features.is_empty());
        }
        other => panic!("expected StandardDef, got {:?}", other),
    }
}

#[test]
fn parse_standard_def_with_features() {
    let prog = parse("standard cautious [\n  types(live),\n  null_safety(live)\n]").unwrap();
    match &prog.statements[0] {
        Statement::StandardDef { name, extends, features } => {
            assert_eq!(name, "cautious");
            assert!(extends.is_none());
            assert_eq!(features.len(), 2);
            assert_eq!(features[0].name, "types");
            assert_eq!(features[0].params, vec!["live".to_string()]);
            assert_eq!(features[1].name, "null_safety");
        }
        other => panic!("expected StandardDef, got {:?}", other),
    }
}

#[test]
fn parse_standard_def_extends() {
    let prog = parse("standard api_safety extends cautious [\n  checked_exceptions(pre)\n]").unwrap();
    match &prog.statements[0] {
        Statement::StandardDef { name, extends, features } => {
            assert_eq!(name, "api_safety");
            assert_eq!(extends, &Some("cautious".to_string()));
            assert_eq!(features.len(), 1);
        }
        other => panic!("expected StandardDef, got {:?}", other),
    }
}

#[test]
fn parse_entry_type_def() {
    let prog = parse("entry type validated {\n  applies_to: [\"variable\", \"property\"]\n}").unwrap();
    match &prog.statements[0] {
        Statement::EntryTypeDef { name, fields } => {
            assert_eq!(name, "validated");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "applies_to");
        }
        other => panic!("expected EntryTypeDef, got {:?}", other),
    }
}
