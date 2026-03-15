#[cfg(test)]
mod tests {
    use ish_parser::parse;
    use ish_ast::*;

    // ── Object Literals ─────────────────────────────────────────────────

    #[test]
    fn test_empty_object() {
        let prog = parse("let x = {}").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 0);
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    #[test]
    fn test_single_pair_object() {
        let prog = parse("let x = { name: \"Alice\" }").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "name");
                assert_eq!(pairs[0].1, Expression::Literal(Literal::String("Alice".into())));
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    #[test]
    fn test_multi_pair_object() {
        let prog = parse("let x = { a: 1, b: 2, c: 3 }").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 3);
                assert_eq!(pairs[0].0, "a");
                assert_eq!(pairs[1].0, "b");
                assert_eq!(pairs[2].0, "c");
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    #[test]
    fn test_nested_object() {
        let prog = parse("let x = { inner: { val: 42 } }").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "inner");
                match &pairs[0].1 {
                    Expression::ObjectLiteral(inner_pairs) => {
                        assert_eq!(inner_pairs.len(), 1);
                        assert_eq!(inner_pairs[0].0, "val");
                    }
                    other => panic!("expected nested object, got {:?}", other),
                }
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    #[test]
    fn test_object_trailing_comma() {
        let prog = parse("let x = { a: 1, b: 2, }").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 2);
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    #[test]
    fn test_object_string_keys() {
        let prog = parse("let x = { \"my-key\": 1 }").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ObjectLiteral(pairs), .. } => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "my-key");
            }
            other => panic!("expected object literal, got {:?}", other),
        }
    }

    // ── List Literals ───────────────────────────────────────────────────

    #[test]
    fn test_empty_list() {
        let prog = parse("let x = []").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 0);
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    #[test]
    fn test_single_element_list() {
        let prog = parse("let x = [42]").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 1);
                assert_eq!(elems[0], Expression::Literal(Literal::Int(42)));
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    #[test]
    fn test_multi_element_list() {
        let prog = parse("let x = [1, 2, 3]").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 3);
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    #[test]
    fn test_nested_list() {
        let prog = parse("let x = [[1, 2], [3, 4]]").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 2);
                match &elems[0] {
                    Expression::ListLiteral(inner) => assert_eq!(inner.len(), 2),
                    other => panic!("expected inner list, got {:?}", other),
                }
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    #[test]
    fn test_list_trailing_comma() {
        let prog = parse("let x = [1, 2,]").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 2);
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    #[test]
    fn test_list_with_expressions() {
        let prog = parse("let x = [1 + 2, a, f(3)]").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::ListLiteral(elems), .. } => {
                assert_eq!(elems.len(), 3);
                match &elems[0] {
                    Expression::BinaryOp { op: BinaryOperator::Add, .. } => {}
                    other => panic!("expected binary op, got {:?}", other),
                }
                match &elems[1] {
                    Expression::Identifier(name) => assert_eq!(name, "a"),
                    other => panic!("expected identifier, got {:?}", other),
                }
                match &elems[2] {
                    Expression::FunctionCall { .. } => {}
                    other => panic!("expected function call, got {:?}", other),
                }
            }
            other => panic!("expected list literal, got {:?}", other),
        }
    }

    // ── Lambdas ─────────────────────────────────────────────────────────

    #[test]
    fn test_lambda_expression_body() {
        let prog = parse("let double = (x) => x + x").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::Lambda { params, body }, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "x");
                // Expression body should be wrapped in Block { [Return { Some(expr) }] }
                match body.as_ref() {
                    Statement::Block { statements } => {
                        assert_eq!(statements.len(), 1);
                        match &statements[0] {
                            Statement::Return { value: Some(Expression::BinaryOp { op: BinaryOperator::Add, .. }) } => {}
                            other => panic!("expected return with add, got {:?}", other),
                        }
                    }
                    other => panic!("expected block body, got {:?}", other),
                }
            }
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn test_lambda_block_body() {
        let prog = parse("let inc = (x) => {\n    return x + 1\n}").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::Lambda { params, body }, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "x");
                match body.as_ref() {
                    Statement::Block { statements } => {
                        assert_eq!(statements.len(), 1);
                        match &statements[0] {
                            Statement::Return { value: Some(_) } => {}
                            other => panic!("expected return, got {:?}", other),
                        }
                    }
                    other => panic!("expected block body, got {:?}", other),
                }
            }
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn test_lambda_no_params() {
        let prog = parse("let f = () => 42").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::Lambda { params, .. }, .. } => {
                assert_eq!(params.len(), 0);
            }
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn test_lambda_multiple_params() {
        let prog = parse("let add = (a, b) => a + b").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::Lambda { params, .. }, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
            }
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn test_lambda_typed_params() {
        let prog = parse("let f = (x: int, y: string) => x").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value: Expression::Lambda { params, .. }, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].type_annotation, Some(TypeAnnotation::Simple("int".into())));
                assert_eq!(params[1].type_annotation, Some(TypeAnnotation::Simple("string".into())));
            }
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn test_lambda_as_argument() {
        let prog = parse("map(list, (x) => x * 2)").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::FunctionCall { callee, args }) => {
                assert_eq!(**callee, Expression::Identifier("map".into()));
                assert_eq!(args.len(), 2);
                match &args[1] {
                    Expression::Lambda { params, .. } => {
                        assert_eq!(params.len(), 1);
                    }
                    other => panic!("expected lambda arg, got {:?}", other),
                }
            }
            other => panic!("expected function call, got {:?}", other),
        }
    }

    // ── F-strings (String Interpolation) ────────────────────────────────

    #[test]
    fn test_f_string_no_interpolation() {
        let prog = parse("f\"hello world\"").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    StringPart::Text(s) => assert_eq!(s, "hello world"),
                    other => panic!("expected text part, got {:?}", other),
                }
            }
            other => panic!("expected string interpolation, got {:?}", other),
        }
    }

    #[test]
    fn test_f_string_simple_interpolation() {
        let prog = parse("f\"hello {name}\"").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    StringPart::Text(s) => assert_eq!(s, "hello "),
                    other => panic!("expected text, got {:?}", other),
                }
                match &parts[1] {
                    StringPart::Expr(Expression::Identifier(name)) => assert_eq!(name, "name"),
                    other => panic!("expected identifier expr, got {:?}", other),
                }
            }
            other => panic!("expected string interpolation, got {:?}", other),
        }
    }

    #[test]
    fn test_f_string_multiple_interpolations() {
        let prog = parse("f\"{a} and {b}\"").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    StringPart::Expr(Expression::Identifier(name)) => assert_eq!(name, "a"),
                    other => panic!("expected expr, got {:?}", other),
                }
                match &parts[1] {
                    StringPart::Text(s) => assert_eq!(s, " and "),
                    other => panic!("expected text, got {:?}", other),
                }
                match &parts[2] {
                    StringPart::Expr(Expression::Identifier(name)) => assert_eq!(name, "b"),
                    other => panic!("expected expr, got {:?}", other),
                }
            }
            other => panic!("expected string interpolation, got {:?}", other),
        }
    }

    #[test]
    fn test_f_string_with_expression() {
        let prog = parse("f\"result: {1 + 2}\"").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
                assert_eq!(parts.len(), 2);
                match &parts[1] {
                    StringPart::Expr(Expression::BinaryOp { op: BinaryOperator::Add, .. }) => {}
                    other => panic!("expected binary op, got {:?}", other),
                }
            }
            other => panic!("expected string interpolation, got {:?}", other),
        }
    }

    // ── Property access chains and index on literals ────────────────────

    #[test]
    fn test_property_access_chain() {
        let prog = parse("a.b.c").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::PropertyAccess { object, property }) => {
                assert_eq!(property, "c");
                match object.as_ref() {
                    Expression::PropertyAccess { object: inner, property: p } => {
                        assert_eq!(p, "b");
                        assert_eq!(**inner, Expression::Identifier("a".into()));
                    }
                    other => panic!("expected nested property access, got {:?}", other),
                }
            }
            other => panic!("expected property access, got {:?}", other),
        }
    }

    #[test]
    fn test_index_on_list_literal() {
        let prog = parse("[1, 2, 3][0]").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::IndexAccess { object, index }) => {
                match object.as_ref() {
                    Expression::ListLiteral(elems) => assert_eq!(elems.len(), 3),
                    other => panic!("expected list literal, got {:?}", other),
                }
                assert_eq!(**index, Expression::Literal(Literal::Int(0)));
            }
            other => panic!("expected index access, got {:?}", other),
        }
    }

    #[test]
    fn test_method_call_on_object() {
        let prog = parse("obj.method(1, 2)").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::FunctionCall { callee, args }) => {
                match callee.as_ref() {
                    Expression::PropertyAccess { object, property } => {
                        assert_eq!(**object, Expression::Identifier("obj".into()));
                        assert_eq!(property, "method");
                    }
                    other => panic!("expected property access as callee, got {:?}", other),
                }
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected function call, got {:?}", other),
        }
    }
}
