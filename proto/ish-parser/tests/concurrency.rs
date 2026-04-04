use ish_ast::*;

#[test]
fn parse_async_fn_decl() {
    let program = ish_parser::parse("async fn fetch() { return 42 }").unwrap();
    match &program.statements[0] {
        Statement::FunctionDecl { name, is_async, .. } => {
            assert_eq!(name, "fetch");
            assert!(*is_async);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_sync_fn_decl_is_not_async() {
    let program = ish_parser::parse("fn sync_fn() { return 1 }").unwrap();
    match &program.statements[0] {
        Statement::FunctionDecl { name, is_async, .. } => {
            assert_eq!(name, "sync_fn");
            assert!(!*is_async);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_pub_async_fn() {
    let program = ish_parser::parse("pub async fn handler() { return null }").unwrap();
    match &program.statements[0] {
        Statement::FunctionDecl { name, is_async, visibility, .. } => {
            assert_eq!(name, "handler");
            assert!(*is_async);
            assert!(visibility.is_some());
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_await_expression() {
    let program = ish_parser::parse("await some_call()").unwrap();
    match &program.statements[0] {
        Statement::ExpressionStmt(Expression::Await { expr }) => {
            match expr.as_ref() {
                Expression::FunctionCall { callee, args } => {
                    assert_eq!(**callee, Expression::Identifier("some_call".into()));
                    assert!(args.is_empty());
                }
                other => panic!("expected FunctionCall inside Await, got {:?}", other),
            }
        }
        other => panic!("expected ExpressionStmt(Await), got {:?}", other),
    }
}

#[test]
fn parse_spawn_expression() {
    let program = ish_parser::parse("spawn compute()").unwrap();
    match &program.statements[0] {
        Statement::ExpressionStmt(Expression::Spawn { callee, args }) => {
            assert_eq!(**callee, Expression::Identifier("compute".into()));
            assert!(args.is_empty());
        }
        other => panic!("expected ExpressionStmt(Spawn), got {:?}", other),
    }
}

#[test]
fn parse_yield_statement() {
    let program = ish_parser::parse("yield").unwrap();
    assert_eq!(program.statements[0], Statement::Yield);
}

#[test]
fn parse_for_with_yield_every() {
    let program = ish_parser::parse("for x in items yield every 100 { x }").unwrap();
    match &program.statements[0] {
        Statement::ForEach { variable, yield_every, .. } => {
            assert_eq!(variable, "x");
            assert!(yield_every.is_some());
            match yield_every.as_ref().unwrap() {
                Expression::Literal(Literal::Int(100)) => {}
                other => panic!("expected Int(100), got {:?}", other),
            }
        }
        other => panic!("expected ForEach, got {:?}", other),
    }
}

#[test]
fn parse_while_with_yield_every() {
    let program = ish_parser::parse("while true yield every 50 { yield }").unwrap();
    match &program.statements[0] {
        Statement::While { yield_every, .. } => {
            assert!(yield_every.is_some());
            match yield_every.as_ref().unwrap() {
                Expression::Literal(Literal::Int(50)) => {}
                other => panic!("expected Int(50), got {:?}", other),
            }
        }
        other => panic!("expected While, got {:?}", other),
    }
}

// ── Grammar: await accepts any expression; spawn still requires a call expression (TODO 49) ──

#[test]
fn parse_await_with_args() {
    let program = ish_parser::parse("await foo(1, 2)").unwrap();
    match &program.statements[0] {
        Statement::ExpressionStmt(Expression::Await { expr }) => {
            match expr.as_ref() {
                Expression::FunctionCall { callee, args } => {
                    assert_eq!(**callee, Expression::Identifier("foo".into()));
                    assert_eq!(args.len(), 2);
                }
                other => panic!("expected FunctionCall inside Await, got {:?}", other),
            }
        }
        other => panic!("expected Await with args, got {:?}", other),
    }
}

#[test]
fn parse_spawn_with_args() {
    let program = ish_parser::parse("spawn bar(x)").unwrap();
    match &program.statements[0] {
        Statement::ExpressionStmt(Expression::Spawn { callee, args }) => {
            assert_eq!(**callee, Expression::Identifier("bar".into()));
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected Spawn with args, got {:?}", other),
    }
}

#[test]
fn parse_await_non_call_is_valid() {
    // await now accepts any expression; await 42 is valid syntax (runtime check throws E014)
    let program = ish_parser::parse("await 42").unwrap();
    match &program.statements[0] {
        Statement::ExpressionStmt(Expression::Await { expr }) => {
            assert_eq!(**expr, Expression::Literal(Literal::Int(42)));
        }
        other => panic!("expected ExpressionStmt(Await {{ expr: Literal(42) }}), got {:?}", other),
    }
}

#[test]
fn parse_spawn_non_call_produces_incomplete() {
    let program = ish_parser::parse("spawn \"hello\"").unwrap();
    match &program.statements[0] {
        Statement::Incomplete { kind } => {
            assert_eq!(*kind, IncompleteKind::SpawnNonCall);
        }
        Statement::ExpressionStmt(Expression::Incomplete { kind }) => {
            assert_eq!(*kind, IncompleteKind::SpawnNonCall);
        }
        other => panic!("expected Incomplete(SpawnNonCall), got {:?}", other),
    }
}

#[test]
fn parse_for_without_yield_every() {
    let program = ish_parser::parse("for x in items { x }").unwrap();
    match &program.statements[0] {
        Statement::ForEach { yield_every, .. } => {
            assert!(yield_every.is_none());
        }
        other => panic!("expected ForEach, got {:?}", other),
    }
}

