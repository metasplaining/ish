// Standard library functions written as ish programs.
// Each function is registered into the VM by running its AST.

use ish_ast::*;
use ish_ast::builder::ProgramBuilder;
use ish_vm::interpreter::IshVm;

/// Register all stdlib functions into the VM.
pub fn register_stdlib(vm: &mut IshVm) {
    let program = build_stdlib();
    vm.run(&program).unwrap();
}

fn build_stdlib() -> Program {
    ProgramBuilder::new()
        // abs(x) -> absolute value
        .function("abs", &["x"], |b| {
            b.if_else(
                Expression::binary(BinaryOperator::Lt, Expression::ident("x"), Expression::int(0)),
                |b| b.ret(Expression::binary(BinaryOperator::Sub, Expression::int(0), Expression::ident("x"))),
                |b| b.ret(Expression::ident("x")),
            )
        })

        // max(a, b) -> larger value
        .function("max", &["a", "b"], |b| {
            b.if_else(
                Expression::binary(BinaryOperator::Gt, Expression::ident("a"), Expression::ident("b")),
                |b| b.ret(Expression::ident("a")),
                |b| b.ret(Expression::ident("b")),
            )
        })

        // min(a, b) -> smaller value
        .function("min", &["a", "b"], |b| {
            b.if_else(
                Expression::binary(BinaryOperator::Lt, Expression::ident("a"), Expression::ident("b")),
                |b| b.ret(Expression::ident("a")),
                |b| b.ret(Expression::ident("b")),
            )
        })

        // range(n) -> list [0, 1, ..., n-1]
        .function("range", &["n"], |b| {
            b.var_decl("result", Expression::list(vec![]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::ident("n")),
                    |b| b
                        .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                            Expression::ident("result"),
                            Expression::ident("i"),
                        ]))
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::ident("result"))
        })

        // assert(cond, msg) — print error if cond is false
        .function("assert", &["cond", "msg"], |b| {
            b.if_then(
                Expression::unary(UnaryOperator::Not, Expression::ident("cond")),
                |b| b.expr_stmt(Expression::call(Expression::ident("println"), vec![
                    Expression::call(Expression::ident("str_concat"), vec![
                        Expression::string("ASSERTION FAILED: "),
                        Expression::ident("msg"),
                    ]),
                ])),
            )
            .ret(Expression::ident("cond"))
        })

        // assert_eq(a, b, msg) — check equality
        .function("assert_eq", &["a", "b", "msg"], |b| {
            b.if_then(
                Expression::binary(BinaryOperator::NotEq, Expression::ident("a"), Expression::ident("b")),
                |b| b.expr_stmt(Expression::call(Expression::ident("println"), vec![
                    Expression::call(Expression::ident("str_concat"), vec![
                        Expression::string("ASSERTION FAILED: "),
                        Expression::call(Expression::ident("str_concat"), vec![
                            Expression::ident("msg"),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::string(" (got "),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::call(Expression::ident("to_string"), vec![Expression::ident("a")]),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::string(" != "),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::call(Expression::ident("to_string"), vec![Expression::ident("b")]),
                                            Expression::string(")"),
                                        ]),
                                    ]),
                                ]),
                            ]),
                        ]),
                    ]),
                ])),
            )
            .ret(Expression::binary(BinaryOperator::Eq, Expression::ident("a"), Expression::ident("b")))
        })

        // sum(lst) -> sum of list elements
        .function("sum", &["lst"], |b| {
            b.var_decl("total", Expression::int(0))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("lst")])),
                    |b| b
                        .assign("total", Expression::binary(BinaryOperator::Add, Expression::ident("total"), Expression::call(Expression::ident("list_get"), vec![Expression::ident("lst"), Expression::ident("i")])))
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::ident("total"))
        })

        // map(lst, fn) -> new list with fn applied to each element
        .function("map", &["lst", "f"], |b| {
            b.var_decl("result", Expression::list(vec![]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("lst")])),
                    |b| b
                        .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                            Expression::ident("result"),
                            Expression::call(Expression::ident("f"), vec![
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("lst"), Expression::ident("i")]),
                            ]),
                        ]))
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::ident("result"))
        })

        // filter(lst, pred) -> new list with elements satisfying pred
        .function("filter", &["lst", "pred"], |b| {
            b.var_decl("result", Expression::list(vec![]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("lst")])),
                    |b| b
                        .var_decl("elem", Expression::call(Expression::ident("list_get"), vec![Expression::ident("lst"), Expression::ident("i")]))
                        .if_then(
                            Expression::call(Expression::ident("pred"), vec![Expression::ident("elem")]),
                            |b| b.expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                Expression::ident("result"),
                                Expression::ident("elem"),
                            ])),
                        )
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::ident("result"))
        })

        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ish_vm::value::Value;

    fn make_vm() -> IshVm {
        let mut vm = IshVm::new();
        crate::load_all(&mut vm);
        vm
    }

    #[test]
    fn test_abs() {
        let mut vm = make_vm();
        let prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(Expression::ident("abs"), vec![Expression::int(-5)])),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_max_min() {
        let mut vm = make_vm();
        let prog = Program::new(vec![
            Statement::var_decl("a", Expression::call(Expression::ident("max"), vec![Expression::int(3), Expression::int(7)])),
            Statement::var_decl("b", Expression::call(Expression::ident("min"), vec![Expression::int(3), Expression::int(7)])),
            Statement::expr_stmt(Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::ident("b"))),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Int(10)); // 7 + 3
    }

    #[test]
    fn test_range() {
        let mut vm = make_vm();
        let prog = Program::new(vec![
            Statement::var_decl("r", Expression::call(Expression::ident("range"), vec![Expression::int(5)])),
            Statement::expr_stmt(Expression::call(Expression::ident("list_length"), vec![Expression::ident("r")])),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_sum() {
        let mut vm = make_vm();
        let prog = Program::new(vec![
            Statement::var_decl("r", Expression::call(Expression::ident("range"), vec![Expression::int(5)])),
            Statement::expr_stmt(Expression::call(Expression::ident("sum"), vec![Expression::ident("r")])),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Int(10)); // 0+1+2+3+4
    }

    #[test]
    fn test_assert_eq_pass() {
        let mut vm = make_vm();
        let prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(Expression::ident("assert_eq"), vec![
                Expression::int(42),
                Expression::int(42),
                Expression::string("should be equal"),
            ])),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_map_and_filter() {
        let mut vm = make_vm();

        // double = lambda(x) => x * 2
        // result = sum(map(range(4), double))  => sum([0,2,4,6]) = 12
        let prog = Program::new(vec![
            Statement::var_decl("double", Expression::lambda(
                vec![Parameter { name: "x".into(), type_annotation: None, default_value: None }],
                Statement::Block {
                    statements: vec![
                        Statement::Return { value: Some(Expression::binary(BinaryOperator::Mul, Expression::ident("x"), Expression::int(2))) },
                    ],
                },
            )),
            Statement::var_decl("mapped", Expression::call(Expression::ident("map"), vec![
                Expression::call(Expression::ident("range"), vec![Expression::int(4)]),
                Expression::ident("double"),
            ])),
            Statement::expr_stmt(Expression::call(Expression::ident("sum"), vec![Expression::ident("mapped")])),
        ]);
        let result = vm.run(&prog).unwrap();
        assert_eq!(result, Value::Int(12));
    }
}
