#[cfg(test)]
mod tests {
    use ish_parser::parse;
    use ish_ast::*;

    // ── Empty program ───────────────────────────────────────────────────

    #[test]
    fn test_empty_program() {
        let prog = parse("").unwrap();
        assert_eq!(prog.statements.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let prog = parse("   \n\n  \n").unwrap();
        assert_eq!(prog.statements.len(), 0);
    }

    // ── Literals ────────────────────────────────────────────────────────

    #[test]
    fn test_integer_literal() {
        let prog = parse("42").unwrap();
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::Int(42))) => {}
            other => panic!("expected int literal, got {:?}", other),
        }
    }

    #[test]
    fn test_float_literal() {
        let prog = parse("3.14").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::Float(f))) => {
                assert!((f - 3.14).abs() < f64::EPSILON);
            }
            other => panic!("expected float literal, got {:?}", other),
        }
    }

    #[test]
    fn test_boolean_literals() {
        let prog = parse("true\nfalse").unwrap();
        assert_eq!(prog.statements.len(), 2);
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::Bool(true))) => {}
            other => panic!("expected true, got {:?}", other),
        }
        match &prog.statements[1] {
            Statement::ExpressionStmt(Expression::Literal(Literal::Bool(false))) => {}
            other => panic!("expected false, got {:?}", other),
        }
    }

    #[test]
    fn test_null_literal() {
        let prog = parse("null").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::Null)) => {}
            other => panic!("expected null, got {:?}", other),
        }
    }

    #[test]
    fn test_string_literal() {
        let prog = parse(r#""hello world""#).unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
                assert_eq!(s, "hello world");
            }
            other => panic!("expected string literal, got {:?}", other),
        }
    }

    #[test]
    fn test_string_escape_sequences() {
        let prog = parse(r#""line1\nline2\ttab""#).unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
                assert_eq!(s, "line1\nline2\ttab");
            }
            other => panic!("expected string with escapes, got {:?}", other),
        }
    }

    // ── Variables ───────────────────────────────────────────────────────

    #[test]
    fn test_let_declaration() {
        let prog = parse("let x = 5").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { name, mutable, type_annotation, value, .. } => {
                assert_eq!(name, "x");
                assert!(!mutable);
                assert!(type_annotation.is_none());
                assert_eq!(*value, Expression::Literal(Literal::Int(5)));
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn test_let_mut_declaration() {
        let prog = parse("let mut y = 10").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { name, mutable, .. } => {
                assert_eq!(name, "y");
                assert!(*mutable);
            }
            other => panic!("expected mutable var decl, got {:?}", other),
        }
    }

    #[test]
    fn test_let_with_type_annotation() {
        let prog = parse("let z: i32 = 42").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { name, type_annotation, .. } => {
                assert_eq!(name, "z");
                assert_eq!(*type_annotation, Some(TypeAnnotation::Simple("i32".into())));
            }
            other => panic!("expected typed var decl, got {:?}", other),
        }
    }

    // ── Arithmetic expressions ──────────────────────────────────────────

    #[test]
    fn test_addition() {
        let prog = parse("1 + 2").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::BinaryOp { op, left, right }) => {
                assert_eq!(*op, BinaryOperator::Add);
                assert_eq!(**left, Expression::Literal(Literal::Int(1)));
                assert_eq!(**right, Expression::Literal(Literal::Int(2)));
            }
            other => panic!("expected binary add, got {:?}", other),
        }
    }

    #[test]
    fn test_precedence_mul_over_add() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let prog = parse("1 + 2 * 3").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::BinaryOp { op, left, right }) => {
                assert_eq!(*op, BinaryOperator::Add);
                assert_eq!(**left, Expression::Literal(Literal::Int(1)));
                match right.as_ref() {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::Mul);
                    }
                    other => panic!("expected mul on right, got {:?}", other),
                }
            }
            other => panic!("expected binary add, got {:?}", other),
        }
    }

    #[test]
    fn test_comparison() {
        let prog = parse("x == 5").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::BinaryOp { op, .. }) => {
                assert_eq!(*op, BinaryOperator::Eq);
            }
            other => panic!("expected comparison, got {:?}", other),
        }
    }

    #[test]
    fn test_logical_operators() {
        let prog = parse("a and b or c").unwrap();
        // Should parse as (a and b) or c
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::BinaryOp { op, left, .. }) => {
                assert_eq!(*op, BinaryOperator::Or);
                match left.as_ref() {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::And);
                    }
                    other => panic!("expected and on left of or, got {:?}", other),
                }
            }
            other => panic!("expected or expression, got {:?}", other),
        }
    }

    #[test]
    fn test_not_operator() {
        let prog = parse("not x").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::UnaryOp { op, .. }) => {
                assert_eq!(*op, UnaryOperator::Not);
            }
            other => panic!("expected not, got {:?}", other),
        }
    }

    #[test]
    fn test_negate_operator() {
        let prog = parse("let x = -5").unwrap();
        match &prog.statements[0] {
            Statement::VariableDecl { value, .. } => {
                // -5 may parse as integer literal -5 or as negate(5)
                match value {
                    Expression::Literal(Literal::Int(-5)) => {}
                    Expression::UnaryOp { op: UnaryOperator::Negate, .. } => {}
                    other => panic!("expected negative, got {:?}", other),
                }
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn test_grouped_expression() {
        let prog = parse("(1 + 2) * 3").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::BinaryOp { op, left, .. }) => {
                assert_eq!(*op, BinaryOperator::Mul);
                match left.as_ref() {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::Add);
                    }
                    other => panic!("expected add in parens, got {:?}", other),
                }
            }
            other => panic!("expected mul, got {:?}", other),
        }
    }

    // ── Function call ───────────────────────────────────────────────────

    #[test]
    fn test_function_call() {
        let prog = parse("add(1, 2)").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::FunctionCall { callee, args }) => {
                assert_eq!(**callee, Expression::Identifier("add".into()));
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected function call, got {:?}", other),
        }
    }

    #[test]
    fn test_chained_call() {
        let prog = parse("f(1)(2)").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::FunctionCall { callee, args }) => {
                assert_eq!(args.len(), 1);
                match callee.as_ref() {
                    Expression::FunctionCall { callee: inner, args: inner_args } => {
                        assert_eq!(**inner, Expression::Identifier("f".into()));
                        assert_eq!(inner_args.len(), 1);
                    }
                    other => panic!("expected inner call, got {:?}", other),
                }
            }
            other => panic!("expected chained call, got {:?}", other),
        }
    }

    // ── Control flow ────────────────────────────────────────────────────

    #[test]
    fn test_if_statement() {
        let prog = parse("if x > 0 {\n    y\n}").unwrap();
        match &prog.statements[0] {
            Statement::If { condition, else_block, .. } => {
                match condition {
                    Expression::BinaryOp { op, .. } => assert_eq!(*op, BinaryOperator::Gt),
                    other => panic!("expected comparison, got {:?}", other),
                }
                assert!(else_block.is_none());
            }
            other => panic!("expected if, got {:?}", other),
        }
    }

    #[test]
    fn test_if_else() {
        let prog = parse("if x > 0 {\n    1\n} else {\n    2\n}").unwrap();
        match &prog.statements[0] {
            Statement::If { else_block, .. } => {
                assert!(else_block.is_some());
            }
            other => panic!("expected if/else, got {:?}", other),
        }
    }

    #[test]
    fn test_if_else_if() {
        let prog = parse("if x > 0 {\n    1\n} else if x == 0 {\n    0\n} else {\n    2\n}").unwrap();
        match &prog.statements[0] {
            Statement::If { else_block: Some(eb), .. } => {
                match eb.as_ref() {
                    Statement::If { else_block: Some(_), .. } => {}
                    other => panic!("expected else-if chain, got {:?}", other),
                }
            }
            other => panic!("expected if, got {:?}", other),
        }
    }

    #[test]
    fn test_while_loop() {
        let prog = parse("while x > 0 {\n    x\n}").unwrap();
        match &prog.statements[0] {
            Statement::While { condition, .. } => {
                match condition {
                    Expression::BinaryOp { op, .. } => assert_eq!(*op, BinaryOperator::Gt),
                    other => panic!("expected comparison, got {:?}", other),
                }
            }
            other => panic!("expected while, got {:?}", other),
        }
    }

    #[test]
    fn test_for_loop() {
        let prog = parse("for item in collection {\n    item\n}").unwrap();
        match &prog.statements[0] {
            Statement::ForEach { variable, iterable, .. } => {
                assert_eq!(variable, "item");
                assert_eq!(*iterable, Expression::Identifier("collection".into()));
            }
            other => panic!("expected for-each, got {:?}", other),
        }
    }

    // ── Functions ───────────────────────────────────────────────────────

    #[test]
    fn test_function_declaration() {
        let prog = parse("fn add(a, b) {\n    return a + b\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { name, params, return_type, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
                assert!(return_type.is_none());
            }
            other => panic!("expected fn decl, got {:?}", other),
        }
    }

    #[test]
    fn test_function_with_return_type() {
        let prog = parse("fn square(x: i32) -> i32 {\n    return x * x\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { name, params, return_type, .. } => {
                assert_eq!(name, "square");
                assert_eq!(params[0].type_annotation, Some(TypeAnnotation::Simple("i32".into())));
                assert_eq!(*return_type, Some(TypeAnnotation::Simple("i32".into())));
            }
            other => panic!("expected fn decl, got {:?}", other),
        }
    }

    #[test]
    fn test_function_with_default_param() {
        let prog = parse("fn connect(host: String, port: i32 = 8080) {\n    host\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { params, .. } => {
                assert_eq!(params.len(), 2);
                assert!(params[0].default_value.is_none());
                assert_eq!(params[1].default_value, Some(Expression::Literal(Literal::Int(8080))));
            }
            other => panic!("expected fn with defaults, got {:?}", other),
        }
    }

    #[test]
    fn test_return_statement() {
        let prog = parse("fn f() {\n    return 42\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { body, .. } => {
                match body.as_ref() {
                    Statement::Block { statements } => {
                        match &statements[0] {
                            Statement::Return { value: Some(Expression::Literal(Literal::Int(42))) } => {}
                            other => panic!("expected return 42, got {:?}", other),
                        }
                    }
                    other => panic!("expected block, got {:?}", other),
                }
            }
            other => panic!("expected fn decl, got {:?}", other),
        }
    }

    #[test]
    fn test_return_void() {
        let prog = parse("fn f() {\n    return\n}").unwrap();
        match &prog.statements[0] {
            Statement::FunctionDecl { body, .. } => {
                match body.as_ref() {
                    Statement::Block { statements } => {
                        match &statements[0] {
                            Statement::Return { value: None } => {}
                            other => panic!("expected void return, got {:?}", other),
                        }
                    }
                    other => panic!("expected block, got {:?}", other),
                }
            }
            other => panic!("expected fn decl, got {:?}", other),
        }
    }

    // ── Property and index access ───────────────────────────────────────

    #[test]
    fn test_property_access() {
        let prog = parse("obj.name").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::PropertyAccess { object, property }) => {
                assert_eq!(**object, Expression::Identifier("obj".into()));
                assert_eq!(property, "name");
            }
            other => panic!("expected property access, got {:?}", other),
        }
    }

    #[test]
    fn test_index_access() {
        let prog = parse("arr[0]").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::IndexAccess { object, index }) => {
                assert_eq!(**object, Expression::Identifier("arr".into()));
                assert_eq!(**index, Expression::Literal(Literal::Int(0)));
            }
            other => panic!("expected index access, got {:?}", other),
        }
    }

    // ── Statement termination ───────────────────────────────────────────

    #[test]
    fn test_newline_separated_statements() {
        let prog = parse("let x = 1\nlet y = 2\nlet z = 3").unwrap();
        assert_eq!(prog.statements.len(), 3);
    }

    #[test]
    fn test_semicolon_separated_statements() {
        let prog = parse("let x = 1; let y = 2; let z = 3").unwrap();
        assert_eq!(prog.statements.len(), 3);
    }

    #[test]
    fn test_mixed_separators() {
        let prog = parse("let x = 1; let y = 2\nlet z = 3").unwrap();
        assert_eq!(prog.statements.len(), 3);
    }

    // ── Comments ────────────────────────────────────────────────────────

    #[test]
    fn test_line_comment_slashes() {
        let prog = parse("// this is a comment\nlet x = 5").unwrap();
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0] {
            Statement::VariableDecl { name, .. } => assert_eq!(name, "x"),
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn test_line_comment_hash() {
        let prog = parse("# this is a comment\nlet x = 5").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_block_comment() {
        let prog = parse("/* comment */ let x = 5").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_nested_block_comment() {
        let prog = parse("/* outer /* inner */ still comment */ let x = 5").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    // ── Assignment to compound targets ──────────────────────────────────

    #[test]
    fn test_property_assignment() {
        let prog = parse("obj.x = 5").unwrap();
        match &prog.statements[0] {
            Statement::Assignment { target, .. } => {
                match target {
                    AssignTarget::Property { property, .. } => {
                        assert_eq!(property, "x");
                    }
                    other => panic!("expected property target, got {:?}", other),
                }
            }
            other => panic!("expected assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_index_assignment() {
        let prog = parse("arr[0] = 5").unwrap();
        match &prog.statements[0] {
            Statement::Assignment { target, .. } => {
                match target {
                    AssignTarget::Index { .. } => {}
                    other => panic!("expected index target, got {:?}", other),
                }
            }
            other => panic!("expected assignment, got {:?}", other),
        }
    }

    // ── Identifier vs keyword ───────────────────────────────────────────

    #[test]
    fn test_identifier_not_keyword() {
        let prog = parse("letter").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Identifier(name)) => {
                assert_eq!(name, "letter");
            }
            other => panic!("expected identifier, got {:?}", other),
        }
    }

    #[test]
    fn test_identifier_starting_with_keyword() {
        // "format" starts with "for" but should be a valid identifier
        let prog = parse("format").unwrap();
        match &prog.statements[0] {
            Statement::ExpressionStmt(Expression::Identifier(name)) => {
                assert_eq!(name, "format");
            }
            other => panic!("expected identifier 'format', got {:?}", other),
        }
    }

    // ── Multi-statement program ─────────────────────────────────────────

    #[test]
    fn test_factorial_program() {
        let src = "\
fn factorial(n) {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}";
        let prog = parse(src).unwrap();
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0] {
            Statement::FunctionDecl { name, params, .. } => {
                assert_eq!(name, "factorial");
                assert_eq!(params.len(), 1);
            }
            other => panic!("expected fn decl, got {:?}", other),
        }
    }
}
