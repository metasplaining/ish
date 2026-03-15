#[cfg(test)]
mod tests {
    use ish_parser::parse;
    use ish_ast::*;

    // ── Throw ───────────────────────────────────────────────────────────

    #[test]
    fn test_throw_expression() {
        let prog = parse("throw \"error\"").unwrap();
        match &prog.statements[0] {
            Statement::Throw { value: Expression::Literal(Literal::String(s)) } => {
                assert_eq!(s, "error");
            }
            other => panic!("expected throw, got {:?}", other),
        }
    }

    #[test]
    fn test_throw_variable() {
        let prog = parse("throw err").unwrap();
        match &prog.statements[0] {
            Statement::Throw { value: Expression::Identifier(name) } => {
                assert_eq!(name, "err");
            }
            other => panic!("expected throw, got {:?}", other),
        }
    }

    // ── Try/Catch ───────────────────────────────────────────────────────

    #[test]
    fn test_try_catch_basic() {
        let prog = parse("try {\n    risky()\n} catch (e) {\n    handle(e)\n}").unwrap();
        match &prog.statements[0] {
            Statement::TryCatch { body, catches, finally } => {
                match body.as_ref() {
                    Statement::Block { statements } => assert_eq!(statements.len(), 1),
                    other => panic!("expected block body, got {:?}", other),
                }
                assert_eq!(catches.len(), 1);
                assert_eq!(catches[0].param, "e");
                assert_eq!(catches[0].type_annotation, None);
                assert!(finally.is_none());
            }
            other => panic!("expected try/catch, got {:?}", other),
        }
    }

    #[test]
    fn test_try_catch_typed() {
        let prog = parse("try {\n    x()\n} catch (e: Error) {\n    log(e)\n}").unwrap();
        match &prog.statements[0] {
            Statement::TryCatch { catches, .. } => {
                assert_eq!(catches[0].param, "e");
                assert_eq!(catches[0].type_annotation, Some(TypeAnnotation::Simple("Error".into())));
            }
            other => panic!("expected try/catch, got {:?}", other),
        }
    }

    #[test]
    fn test_try_catch_finally() {
        let prog = parse("try {\n    x()\n} catch (e) {\n    log(e)\n} finally {\n    cleanup()\n}").unwrap();
        match &prog.statements[0] {
            Statement::TryCatch { catches, finally, .. } => {
                assert_eq!(catches.len(), 1);
                assert!(finally.is_some());
            }
            other => panic!("expected try/catch/finally, got {:?}", other),
        }
    }

    #[test]
    fn test_try_multiple_catches() {
        let prog = parse("try {\n    x()\n} catch (e: TypeError) {\n    a()\n} catch (e: ValueError) {\n    b()\n}").unwrap();
        match &prog.statements[0] {
            Statement::TryCatch { catches, .. } => {
                assert_eq!(catches.len(), 2);
                assert_eq!(catches[0].type_annotation, Some(TypeAnnotation::Simple("TypeError".into())));
                assert_eq!(catches[1].type_annotation, Some(TypeAnnotation::Simple("ValueError".into())));
            }
            other => panic!("expected try/catch, got {:?}", other),
        }
    }

    // ── With Block ──────────────────────────────────────────────────────

    #[test]
    fn test_with_single_resource() {
        let prog = parse("with (f = open(\"file.txt\")) {\n    read(f)\n}").unwrap();
        match &prog.statements[0] {
            Statement::WithBlock { resources, body } => {
                assert_eq!(resources.len(), 1);
                assert_eq!(resources[0].0, "f");
                match &resources[0].1 {
                    Expression::FunctionCall { callee, .. } => {
                        assert_eq!(**callee, Expression::Identifier("open".into()));
                    }
                    other => panic!("expected function call, got {:?}", other),
                }
                match body.as_ref() {
                    Statement::Block { statements } => assert_eq!(statements.len(), 1),
                    other => panic!("expected block body, got {:?}", other),
                }
            }
            other => panic!("expected with block, got {:?}", other),
        }
    }

    #[test]
    fn test_with_multiple_resources() {
        let prog = parse("with (a = open(\"a\"), b = open(\"b\")) {\n    process(a, b)\n}").unwrap();
        match &prog.statements[0] {
            Statement::WithBlock { resources, .. } => {
                assert_eq!(resources.len(), 2);
                assert_eq!(resources[0].0, "a");
                assert_eq!(resources[1].0, "b");
            }
            other => panic!("expected with block, got {:?}", other),
        }
    }

    // ── Defer ───────────────────────────────────────────────────────────

    #[test]
    fn test_defer_expression() {
        let prog = parse("defer close(f)").unwrap();
        match &prog.statements[0] {
            Statement::Defer { body } => {
                match body.as_ref() {
                    Statement::ExpressionStmt(Expression::FunctionCall { callee, .. }) => {
                        assert_eq!(**callee, Expression::Identifier("close".into()));
                    }
                    other => panic!("expected expression stmt, got {:?}", other),
                }
            }
            other => panic!("expected defer, got {:?}", other),
        }
    }

    #[test]
    fn test_defer_block() {
        let prog = parse("defer {\n    cleanup()\n    close(f)\n}").unwrap();
        match &prog.statements[0] {
            Statement::Defer { body } => {
                match body.as_ref() {
                    Statement::Block { statements } => assert_eq!(statements.len(), 2),
                    other => panic!("expected block, got {:?}", other),
                }
            }
            other => panic!("expected defer, got {:?}", other),
        }
    }

    // ── Question Mark Operator ──────────────────────────────────────────

    #[test]
    fn test_question_mark_operator() {
        let prog = parse("let x = get_value()?").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value, .. } => {
                match value {
                    Expression::UnaryOp { op: UnaryOperator::Try, operand } => {
                        match operand.as_ref() {
                            Expression::FunctionCall { callee, .. } => {
                                assert_eq!(**callee, Expression::Identifier("get_value".into()));
                            }
                            other => panic!("expected function call, got {:?}", other),
                        }
                    }
                    other => panic!("expected try operator, got {:?}", other),
                }
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn test_chained_question_mark() {
        let prog = parse("a()?.b?.c()").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::FunctionCall { callee, .. }) => {
                // Should be: call(prop(try(prop(try(call(a)))), "c"))
                // Let me just verify the outermost is a function call on a property
                match callee.as_ref() {
                    Expression::PropertyAccess { property, object } => {
                        assert_eq!(property, "c");
                        match object.as_ref() {
                            Expression::UnaryOp { op: UnaryOperator::Try, operand } => {
                                match operand.as_ref() {
                                    Expression::PropertyAccess { property: p, .. } => {
                                        assert_eq!(p, "b");
                                    }
                                    other => panic!("expected property access, got {:?}", other),
                                }
                            }
                            other => panic!("expected try, got {:?}", other),
                        }
                    }
                    other => panic!("expected property access, got {:?}", other),
                }
            }
            other => panic!("expected expression stmt, got {:?}", other),
        }
    }
}
