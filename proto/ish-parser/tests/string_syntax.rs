/// String syntax tests — single-quoted literals, double-quoted interpolation,
/// triple-quoted strings, char literals, extended delimiters, env vars in strings.
use ish_parser::parse;
use ish_ast::*;

// ── Single-quoted (literal) strings ─────────────────────────────────────

#[test]
fn single_quoted_simple() {
    let prog = parse("'hello world'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "hello world");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn single_quoted_escaped_quote() {
    let prog = parse(r"'it\'s fine'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "it's fine");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn single_quoted_escaped_backslash() {
    let prog = parse(r"'path\\to'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, r"path\to");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn single_quoted_no_special_escapes() {
    // \n is NOT an escape in single-quoted strings
    let prog = parse(r"'hello\nworld'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, r"hello\nworld");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

// ── Double-quoted (interpolating) strings ───────────────────────────────

#[test]
fn double_quoted_no_interpolation() {
    let prog = parse("\"hello world\"").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "hello world");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn double_quoted_escape_sequences() {
    let prog = parse(r#""line1\nline2\ttab""#).unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "line1\nline2\ttab");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn double_quoted_with_interpolation() {
    let prog = parse("\"hello {name}\"").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
            assert_eq!(parts.len(), 2);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "hello "));
            assert!(matches!(&parts[1], StringPart::Expr(Expression::Identifier(n)) if n == "name"));
        }
        other => panic!("expected string interpolation, got {:?}", other),
    }
}

#[test]
fn double_quoted_env_var_interpolation() {
    let prog = parse("\"home: $HOME\"").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
            assert_eq!(parts.len(), 2);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "home: "));
            assert!(matches!(&parts[1], StringPart::Expr(Expression::EnvVar(n)) if n == "HOME"));
        }
        other => panic!("expected string interpolation, got {:?}", other),
    }
}

#[test]
fn double_quoted_escaped_brace() {
    let prog = parse(r#""\{not interpolated\}""#).unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "{not interpolated}");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn double_quoted_escaped_dollar() {
    let prog = parse(r#""\$not_a_var""#).unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "$not_a_var");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn double_quoted_expr_interpolation() {
    let prog = parse("\"result: {1 + 2}\"").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::StringInterpolation(parts)) => {
            assert_eq!(parts.len(), 2);
            assert!(matches!(&parts[1], StringPart::Expr(Expression::BinaryOp { op: BinaryOperator::Add, .. })));
        }
        other => panic!("expected string interpolation, got {:?}", other),
    }
}

// ── Char literals ───────────────────────────────────────────────────────

#[test]
fn char_literal_simple() {
    let prog = parse("c'A'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::Char(c))) => {
            assert_eq!(*c, 'A');
        }
        other => panic!("expected char literal, got {:?}", other),
    }
}

#[test]
fn char_literal_escape_newline() {
    let prog = parse(r"c'\n'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::Char(c))) => {
            assert_eq!(*c, '\n');
        }
        other => panic!("expected char literal, got {:?}", other),
    }
}

#[test]
fn char_literal_escape_null() {
    let prog = parse(r"c'\0'").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::Char(c))) => {
            assert_eq!(*c, '\0');
        }
        other => panic!("expected char literal, got {:?}", other),
    }
}

// ── Extended delimiter strings ──────────────────────────────────────────

#[test]
fn extended_double_string() {
    let prog = parse(r#"~"hello "world""~"#).unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, r#"hello "world""#);
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

#[test]
fn extended_single_string() {
    let prog = parse("~'hello world'~").unwrap();
    match &prog.statements[0] {
        Statement::ExpressionStmt(Expression::Literal(Literal::String(s))) => {
            assert_eq!(s, "hello world");
        }
        other => panic!("expected string literal, got {:?}", other),
    }
}

// ── Let with single-quoted string ───────────────────────────────────────

#[test]
fn let_with_single_quoted_string() {
    let prog = parse("let x = 'hello'").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { name, value: Expression::Literal(Literal::String(s)), .. } => {
            assert_eq!(name, "x");
            assert_eq!(s, "hello");
        }
        other => panic!("expected var decl with string, got {:?}", other),
    }
}

// ── Display round-trip ──────────────────────────────────────────────────

#[test]
fn display_single_quoted_string() {
    let prog = ish_ast::builder::ProgramBuilder::new()
        .expr_stmt(Expression::string("hello"))
        .build();
    let display = format!("{}", prog);
    assert!(display.contains("'hello'"), "display was: {}", display);
}

#[test]
fn display_char_literal() {
    let prog = ish_ast::builder::ProgramBuilder::new()
        .expr_stmt(Expression::char_lit('A'))
        .build();
    let display = format!("{}", prog);
    assert!(display.contains("c'A'"), "display was: {}", display);
}

#[test]
fn display_interpolation_no_f_prefix() {
    let prog = ish_ast::builder::ProgramBuilder::new()
        .expr_stmt(Expression::StringInterpolation(vec![
            StringPart::Text("hello ".to_string()),
            StringPart::Expr(Expression::ident("name")),
        ]))
        .build();
    let display = format!("{}", prog);
    assert!(!display.contains("f\""), "display should not contain f prefix: {}", display);
    assert!(display.contains("\"hello {name}\""), "display was: {}", display);
}

#[test]
fn display_env_var_in_interpolation() {
    let prog = ish_ast::builder::ProgramBuilder::new()
        .expr_stmt(Expression::StringInterpolation(vec![
            StringPart::Text("home: ".to_string()),
            StringPart::Expr(Expression::EnvVar("HOME".to_string())),
        ]))
        .build();
    let display = format!("{}", prog);
    assert!(display.contains("\"home: $HOME\""), "display was: {}", display);
}
