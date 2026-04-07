use std::path::Path;
use std::collections::HashMap;
use ish_ast::{Statement, Visibility};
use ish_runtime::error::ErrorCode;

#[derive(Debug)]
pub struct InterfaceError {
    pub code: ErrorCode,
    pub symbol: String,
    pub message: String,
}

/// Check interface file consistency for a module.
///
/// `module_file` is the path to the `.ish` file.
/// `pub_declarations` is the list of top-level statements from the module.
///
/// Returns an empty `Vec` if no sibling `.ishi` file exists.
pub fn check_interface(
    module_file: &Path,
    pub_declarations: &[Statement],
) -> Vec<InterfaceError> {
    // Compute sibling .ishi path.
    let ishi_path = module_file.with_extension("ishi");
    if !ishi_path.exists() {
        return vec![];
    }

    // Read and parse the .ishi file.
    let source = match std::fs::read_to_string(&ishi_path) {
        Ok(s) => s,
        Err(e) => {
            return vec![InterfaceError {
                code: ErrorCode::InterfaceSymbolNotInImplementation,
                symbol: String::new(),
                message: format!("Failed to read interface file {:?}: {}", ishi_path, e),
            }];
        }
    };

    let ishi_program = match ish_parser::parse(&source) {
        Ok(p) => p,
        Err(_) => {
            return vec![InterfaceError {
                code: ErrorCode::InterfaceSymbolNotInImplementation,
                symbol: String::new(),
                message: format!("Failed to parse interface file {:?}", ishi_path),
            }];
        }
    };

    // Extract pub symbols from the .ishi file.
    let ishi_symbols = extract_pub_symbols(&ishi_program.statements);

    // Extract pub symbols from the implementation.
    let impl_symbols = extract_pub_symbols(pub_declarations);

    let mut errors = Vec::new();

    // E022: symbol in .ishi but absent from implementation.
    for (name, ishi_stmt) in &ishi_symbols {
        match impl_symbols.get(name.as_str()) {
            None => {
                errors.push(InterfaceError {
                    code: ErrorCode::InterfaceSymbolNotInImplementation,
                    symbol: name.clone(),
                    message: format!(
                        "Interface file declares '{}' but it is absent from the implementation",
                        name
                    ),
                });
            }
            Some(impl_stmt) => {
                // E024: symbol present in both but signatures differ.
                if !signatures_match(ishi_stmt, impl_stmt) {
                    errors.push(InterfaceError {
                        code: ErrorCode::InterfaceSymbolMismatch,
                        symbol: name.clone(),
                        message: format!(
                            "Interface and implementation signatures for '{}' do not match",
                            name
                        ),
                    });
                }
            }
        }
    }

    // E023: pub symbol in implementation but absent from .ishi.
    for name in impl_symbols.keys() {
        if !ishi_symbols.contains_key(name) {
            errors.push(InterfaceError {
                code: ErrorCode::InterfaceSymbolNotInInterface,
                symbol: name.to_string(),
                message: format!(
                    "Implementation has pub symbol '{}' not declared in the interface file",
                    name
                ),
            });
        }
    }

    errors
}

/// Extract a mapping of symbol name -> Statement for all pub declarations.
fn extract_pub_symbols(stmts: &[Statement]) -> HashMap<String, &Statement> {
    let mut symbols = HashMap::new();
    for stmt in stmts {
        match stmt {
            Statement::FunctionDecl { name, visibility, .. } => {
                if matches!(visibility, Some(Visibility::Pub)) {
                    symbols.insert(name.clone(), stmt);
                }
            }
            Statement::TypeAlias { name, visibility, .. } => {
                if matches!(visibility, Some(Visibility::Pub)) {
                    symbols.insert(name.clone(), stmt);
                }
            }
            _ => {}
        }
    }
    symbols
}

/// Compare two declarations for interface signature match.
///
/// FunctionDecl: params count + type annotations + return type must match.
/// TypeAlias: definition must match.
fn signatures_match(ishi_stmt: &Statement, impl_stmt: &Statement) -> bool {
    match (ishi_stmt, impl_stmt) {
        (
            Statement::FunctionDecl {
                params: ishi_params,
                return_type: ishi_ret,
                type_params: ishi_tp,
                ..
            },
            Statement::FunctionDecl {
                params: impl_params,
                return_type: impl_ret,
                type_params: impl_tp,
                ..
            },
        ) => {
            if ishi_params.len() != impl_params.len() {
                return false;
            }
            for (ip, pp) in ishi_params.iter().zip(impl_params.iter()) {
                if ip.type_annotation != pp.type_annotation {
                    return false;
                }
            }
            ishi_ret == impl_ret && ishi_tp == impl_tp
        }
        (
            Statement::TypeAlias { definition: ishi_def, .. },
            Statement::TypeAlias { definition: impl_def, .. },
        ) => ishi_def == impl_def,
        _ => false, // Mismatched declaration kinds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use ish_ast::{Parameter, Statement, Visibility, TypeAnnotation};
    use ish_runtime::error::ErrorCode;

    fn make_pub_fn(name: &str, params: Vec<Parameter>, return_type: Option<TypeAnnotation>) -> Statement {
        Statement::FunctionDecl {
            name: name.to_string(),
            params,
            return_type,
            body: Box::new(Statement::Block { statements: vec![] }),
            visibility: Some(Visibility::Pub),
            type_params: vec![],
            is_async: false,
        }
    }

    fn make_param(name: &str, ty: Option<TypeAnnotation>) -> Parameter {
        Parameter {
            name: name.to_string(),
            type_annotation: ty,
            default_value: None,
        }
    }

    #[test]
    fn no_ishi_file() {
        let dir = std::env::temp_dir().join("ish_test_ic_no_ishi");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let module_file = dir.join("test.ish");
        fs::write(&module_file, "fn foo() {}").unwrap();
        // No .ishi file — should return empty vec
        let result = check_interface(&module_file, &[]);
        assert!(result.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn symbol_not_in_implementation() {
        let dir = std::env::temp_dir().join("ish_test_ic_not_impl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let module_file = dir.join("test.ish");
        fs::write(&module_file, "").unwrap();
        // .ishi declares pub fn foo()
        fs::write(dir.join("test.ishi"), "pub fn foo() {}").unwrap();
        // But implementation has nothing
        let result = check_interface(&module_file, &[]);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].code, ErrorCode::InterfaceSymbolNotInImplementation));
        assert_eq!(result[0].symbol, "foo");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn symbol_not_in_interface() {
        let dir = std::env::temp_dir().join("ish_test_ic_not_iface");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let module_file = dir.join("test.ish");
        fs::write(&module_file, "pub fn bar() {}").unwrap();
        // .ishi is empty
        fs::write(dir.join("test.ishi"), "").unwrap();
        // Implementation has pub fn bar
        let bar = make_pub_fn("bar", vec![], None);
        let result = check_interface(&module_file, &[bar]);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].code, ErrorCode::InterfaceSymbolNotInInterface));
        assert_eq!(result[0].symbol, "bar");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn symbol_mismatch() {
        let dir = std::env::temp_dir().join("ish_test_ic_mismatch");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let module_file = dir.join("test.ish");
        fs::write(&module_file, "pub fn foo(x: String) {}").unwrap();
        // .ishi declares pub fn foo(x: Int)
        fs::write(dir.join("test.ishi"), "pub fn foo(x: Int) {}").unwrap();
        // Implementation has pub fn foo(x: String)
        let foo = make_pub_fn(
            "foo",
            vec![make_param("x", Some(TypeAnnotation::Simple("String".to_string())))],
            None,
        );
        let result = check_interface(&module_file, &[foo]);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].code, ErrorCode::InterfaceSymbolMismatch));
        assert_eq!(result[0].symbol, "foo");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn all_match() {
        let dir = std::env::temp_dir().join("ish_test_ic_match");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let module_file = dir.join("test.ish");
        fs::write(&module_file, "pub fn foo(x: Int) {}").unwrap();
        // .ishi declares pub fn foo(x: Int)
        fs::write(dir.join("test.ishi"), "pub fn foo(x: Int) {}").unwrap();
        // Implementation matches
        let foo = make_pub_fn(
            "foo",
            vec![make_param("x", Some(TypeAnnotation::Simple("Int".to_string())))],
            None,
        );
        let result = check_interface(&module_file, &[foo]);
        assert!(result.is_empty(), "Expected no errors but got: {:?}", result);
        let _ = fs::remove_dir_all(&dir);
    }
}
