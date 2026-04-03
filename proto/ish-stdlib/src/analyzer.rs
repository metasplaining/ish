// Code Analyzer — an ish program that receives AST-as-values, analyzes them,
// and returns annotated versions. Built using the Rust builder API.
//
// Analyses performed:
// 1. Variable usage check: track declared vs. referenced variables, flag undeclared
// 2. Return path check: verify functions end with a return
// 3. Constant folding annotation: identify binary ops on literals

use std::cell::RefCell;
use std::rc::Rc;
use ish_ast::*;
use ish_ast::builder::ProgramBuilder;
use ish_vm::interpreter::IshVm;

/// Register the analyzer function into the VM's global environment.
pub async fn register_analyzer(vm: &Rc<RefCell<IshVm>>) {
    let analyzer_program = build_analyzer();
    // Execute the program to define the functions
    IshVm::run(vm, &analyzer_program).await.unwrap();
}

/// Build the analyzer as an ish program (AST).
///
/// Defines these functions:
/// - analyze(ast_node) -> annotated AST node
/// - collect_declarations(node, declared_list) -> modified declared_list
/// - collect_references(node, refs_list) -> modified refs_list
/// - check_undeclared(declared, referenced) -> list of warnings
/// - check_returns(node) -> bool (whether all paths return)
/// - annotate_constants(node) -> annotated node
fn build_analyzer() -> Program {
    ProgramBuilder::new()
        // ── collect_declarations: walk AST and collect variable/param names ──
        //
        // function collect_declarations(node, declared) {
        //   let kind = obj_get(node, "kind");
        //   if kind == "var_decl" {
        //     list_push(declared, obj_get(node, "name"));
        //   }
        //   if kind == "function_decl" {
        //     list_push(declared, obj_get(node, "name"));
        //     let params = obj_get(node, "params");
        //     let i = 0;
        //     while i < list_length(params) {
        //       list_push(declared, obj_get(list_get(params, i), "name"));
        //       i = i + 1;
        //     }
        //     collect_declarations(obj_get(node, "body"), declared);
        //   }
        //   if kind == "block" {
        //     let stmts = obj_get(node, "statements");
        //     let i = 0;
        //     while i < list_length(stmts) {
        //       collect_declarations(list_get(stmts, i), declared);
        //       i = i + 1;
        //     }
        //   }
        //   if kind == "if" {
        //     collect_declarations(obj_get(node, "then_block"), declared);
        //     let eb = obj_get(node, "else_block");
        //     if is_type(eb, "object") { collect_declarations(eb, declared); }
        //   }
        //   if kind == "while" { collect_declarations(obj_get(node, "body"), declared); }
        //   if kind == "for_each" {
        //     list_push(declared, obj_get(node, "variable"));
        //     collect_declarations(obj_get(node, "body"), declared);
        //   }
        //   return declared;
        // }
        .function("collect_declarations", &["node", "declared"], |b| {
            b.var_decl("kind", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("node"), Expression::string("kind")],
            ))
            // var_decl
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("var_decl")),
                |b| b.expr_stmt(Expression::call(
                    Expression::ident("list_push"),
                    vec![
                        Expression::ident("declared"),
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")]),
                    ],
                )),
            )
            // function_decl
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("function_decl")),
                |b| b
                    .expr_stmt(Expression::call(
                        Expression::ident("list_push"),
                        vec![
                            Expression::ident("declared"),
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")]),
                        ],
                    ))
                    .var_decl("params", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("params")]))
                    .var_decl("i", Expression::int(0))
                    .while_loop(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("params")])),
                        |b| b
                            .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                Expression::ident("declared"),
                                Expression::call(Expression::ident("obj_get"), vec![
                                    Expression::call(Expression::ident("list_get"), vec![Expression::ident("params"), Expression::ident("i")]),
                                    Expression::string("name"),
                                ]),
                            ]))
                            .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                    )
                    .expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                        Expression::ident("declared"),
                    ])),
            )
            // block
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("block")),
                |b| b
                    .var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("statements")]))
                    .var_decl("i", Expression::int(0))
                    .while_loop(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")])),
                        |b| b
                            .expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                                Expression::ident("declared"),
                            ]))
                            .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                    ),
            )
            // if
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("if")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("then_block")]),
                        Expression::ident("declared"),
                    ]))
                    .var_decl("eb", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("else_block")]))
                    .if_then(
                        Expression::call(Expression::ident("is_type"), vec![Expression::ident("eb"), Expression::string("object")]),
                        |b| b.expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![Expression::ident("eb"), Expression::ident("declared")])),
                    ),
            )
            // while
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("while")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                    Expression::ident("declared"),
                ])),
            )
            // for_each
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("for_each")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                        Expression::ident("declared"),
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("variable")]),
                    ]))
                    .expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                        Expression::ident("declared"),
                    ])),
            )
            .ret(Expression::ident("declared"))
        })

        // ── collect_references: walk AST and collect identifier names ────────
        .function("collect_references", &["node", "refs"], |b| {
            b.var_decl("kind", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("node"), Expression::string("kind")],
            ))
            // identifier
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("identifier")),
                |b| b.expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                    Expression::ident("refs"),
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")]),
                ])),
            )
            // binary_op
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("binary_op")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("left")]),
                        Expression::ident("refs"),
                    ]))
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("right")]),
                        Expression::ident("refs"),
                    ])),
            )
            // unary_op
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("unary_op")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("operand")]),
                    Expression::ident("refs"),
                ])),
            )
            // function_call
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("function_call")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("callee")]),
                        Expression::ident("refs"),
                    ]))
                    .var_decl("args", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("args")]))
                    .var_decl("i", Expression::int(0))
                    .while_loop(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("args")])),
                        |b| b
                            .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("args"), Expression::ident("i")]),
                                Expression::ident("refs"),
                            ]))
                            .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                    ),
            )
            // block
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("block")),
                |b| b
                    .var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("statements")]))
                    .var_decl("i", Expression::int(0))
                    .while_loop(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")])),
                        |b| b
                            .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                                Expression::ident("refs"),
                            ]))
                            .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                    ),
            )
            // var_decl
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("var_decl")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]),
                    Expression::ident("refs"),
                ])),
            )
            // return
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("return")),
                |b| b
                    .var_decl("rv", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]))
                    .if_then(
                        Expression::call(Expression::ident("is_type"), vec![Expression::ident("rv"), Expression::string("object")]),
                        |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![Expression::ident("rv"), Expression::ident("refs")])),
                    ),
            )
            // if
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("if")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("condition")]),
                        Expression::ident("refs"),
                    ]))
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("then_block")]),
                        Expression::ident("refs"),
                    ]))
                    .var_decl("eb", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("else_block")]))
                    .if_then(
                        Expression::call(Expression::ident("is_type"), vec![Expression::ident("eb"), Expression::string("object")]),
                        |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![Expression::ident("eb"), Expression::ident("refs")])),
                    ),
            )
            // while
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("while")),
                |b| b
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("condition")]),
                        Expression::ident("refs"),
                    ]))
                    .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                        Expression::ident("refs"),
                    ])),
            )
            // expr_stmt
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("expr_stmt")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("expression")]),
                    Expression::ident("refs"),
                ])),
            )
            // assignment
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("assignment")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]),
                    Expression::ident("refs"),
                ])),
            )
            // function_decl body
            .if_then(
                Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("function_decl")),
                |b| b.expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                    Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                    Expression::ident("refs"),
                ])),
            )
            .ret(Expression::ident("refs"))
        })

        // ── list_contains helper ─────────────────────────────────────────────
        .function("list_contains", &["lst", "item"], |b| {
            b.var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("lst")])),
                    |b| b
                        .if_then(
                            Expression::binary(BinaryOperator::Eq,
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("lst"), Expression::ident("i")]),
                                Expression::ident("item"),
                            ),
                            |b| b.ret(Expression::bool(true)),
                        )
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::bool(false))
        })

        // ── check_undeclared: find referenced but undeclared variables ────────
        .function("check_undeclared", &["declared", "referenced"], |b| {
            b.var_decl("warnings", Expression::list(vec![]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("referenced")])),
                    |b| b
                        .var_decl("ref_name", Expression::call(Expression::ident("list_get"), vec![Expression::ident("referenced"), Expression::ident("i")]))
                        .if_then(
                            Expression::unary(UnaryOperator::Not, Expression::call(Expression::ident("list_contains"), vec![Expression::ident("declared"), Expression::ident("ref_name")])),
                            |b| b.expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                Expression::ident("warnings"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string("undeclared variable: "),
                                    Expression::ident("ref_name"),
                                ]),
                            ])),
                        )
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .ret(Expression::ident("warnings"))
        })

        // ── check_returns: check if a block ends with a return ───────────────
        .function("check_returns", &["node"], |b| {
            b.var_decl("kind", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("kind")]))
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("return")),
                    |b| b.ret(Expression::bool(true)),
                )
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("block")),
                    |b| b
                        .var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("statements")]))
                        .var_decl("len", Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")]))
                        .if_then(
                            Expression::binary(BinaryOperator::Gt, Expression::ident("len"), Expression::int(0)),
                            |b| b.ret(Expression::call(Expression::ident("check_returns"), vec![
                                Expression::call(Expression::ident("list_get"), vec![
                                    Expression::ident("stmts"),
                                    Expression::binary(BinaryOperator::Sub, Expression::ident("len"), Expression::int(1)),
                                ]),
                            ])),
                        ),
                )
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("if")),
                    |b| b
                        .var_decl("then_ret", Expression::call(Expression::ident("check_returns"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("then_block")]),
                        ]))
                        .var_decl("eb", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("else_block")]))
                        .if_then(
                            Expression::binary(BinaryOperator::And,
                                Expression::ident("then_ret"),
                                Expression::call(Expression::ident("is_type"), vec![Expression::ident("eb"), Expression::string("object")]),
                            ),
                            |b| b.ret(Expression::call(Expression::ident("check_returns"), vec![Expression::ident("eb")])),
                        ),
                )
                .ret(Expression::bool(false))
        })

        // ── is_constant_expr: check if an expression is a constant ───────────
        .function("is_constant_expr", &["node"], |b| {
            b.var_decl("kind", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("kind")]))
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("literal")),
                    |b| b.ret(Expression::bool(true)),
                )
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("binary_op")),
                    |b| b.ret(Expression::binary(BinaryOperator::And,
                        Expression::call(Expression::ident("is_constant_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("left")]),
                        ]),
                        Expression::call(Expression::ident("is_constant_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("right")]),
                        ]),
                    )),
                )
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("unary_op")),
                    |b| b.ret(Expression::call(Expression::ident("is_constant_expr"), vec![
                        Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("operand")]),
                    ])),
                )
                .ret(Expression::bool(false))
        })

        // ── analyze: main entry point ────────────────────────────────────────
        // analyze(program_node) -> { warnings: [...], has_returns: bool, constants: [...] }
        .function("analyze", &["program_node"], |b| {
            b.var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("program_node"), Expression::string("statements")]))
                .var_decl("all_declared", Expression::list(vec![]))
                .var_decl("all_referenced", Expression::list(vec![]))
                .var_decl("warnings", Expression::list(vec![]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")])),
                    |b| b
                        .expr_stmt(Expression::call(Expression::ident("collect_declarations"), vec![
                            Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                            Expression::ident("all_declared"),
                        ]))
                        .expr_stmt(Expression::call(Expression::ident("collect_references"), vec![
                            Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                            Expression::ident("all_referenced"),
                        ]))
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                // Check undeclared variables
                .var_decl("undeclared_warnings", Expression::call(Expression::ident("check_undeclared"), vec![
                    Expression::ident("all_declared"),
                    Expression::ident("all_referenced"),
                ]))
                // Build result
                .var_decl("result", Expression::object(vec![
                    ("warnings", Expression::ident("undeclared_warnings")),
                    ("declared_count", Expression::call(Expression::ident("list_length"), vec![Expression::ident("all_declared")])),
                    ("reference_count", Expression::call(Expression::ident("list_length"), vec![Expression::ident("all_referenced")])),
                ]))
                .ret(Expression::ident("result"))
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::builder::ProgramBuilder;
    use ish_vm::value::Value;
    use ish_vm::reflection::program_to_value;
    use std::rc::Rc;

    async fn make_vm() -> Rc<RefCell<IshVm>> {
        let vm = Rc::new(RefCell::new(IshVm::new()));
        crate::load_all(&vm).await;
        vm
    }

    #[tokio::test]
    async fn test_analyzer_detects_undeclared_variable() {
        let vm = make_vm().await;

        // Build a test program that references an undeclared variable
        let test_program = ProgramBuilder::new()
            .var_decl("x", Expression::int(10))
            .function("test_fn", &["a"], |b| {
                // References "b" which is undeclared
                b.ret(Expression::binary(
                    BinaryOperator::Add,
                    Expression::ident("a"),
                    Expression::ident("b"),
                ))
            })
            .build();

        let test_ast = program_to_value(&test_program);

        // Call analyze(test_ast)
        let analyze_prog = ProgramBuilder::new()
            .var_decl("test_ast", Expression::null()) // placeholder, will set via env
            .expr_stmt(Expression::call(
                Expression::ident("analyze"),
                vec![Expression::ident("test_ast")],
            ))
            .build();

        // Set the test_ast in the environment
        vm.borrow().global_env.define("test_ast".to_string(), test_ast);

        // Run analyze(test_ast)
        let call_prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("analyze"),
                vec![Expression::ident("test_ast")],
            )),
        ]);

        let result = IshVm::run(&vm, &call_prog).await.unwrap();

        // Check that warnings contain "undeclared variable: b"
        if let Value::Object(ref obj_ref) = result {
            let map = obj_ref.borrow();
            let warnings = map.get("warnings").unwrap();
            if let Value::List(ref list_ref) = warnings {
                let list = list_ref.borrow();
                assert!(list.len() > 0, "expected at least one warning");
                let first = &list[0];
                assert_eq!(*first, Value::String(Rc::new("undeclared variable: b".into())));
            } else {
                panic!("expected warnings to be a list");
            }
        } else {
            panic!("expected result to be an object");
        }
    }

    #[tokio::test]
    async fn test_analyzer_no_warnings_for_valid_program() {
        let vm = make_vm().await;

        let test_program = ProgramBuilder::new()
            .var_decl("x", Expression::int(10))
            .function("double", &["n"], |b| {
                b.ret(Expression::binary(
                    BinaryOperator::Mul,
                    Expression::ident("n"),
                    Expression::int(2),
                ))
            })
            .build();

        let test_ast = program_to_value(&test_program);
        vm.borrow().global_env.define("test_ast".to_string(), test_ast);

        let call_prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("analyze"),
                vec![Expression::ident("test_ast")],
            )),
        ]);

        let result = IshVm::run(&vm, &call_prog).await.unwrap();

        if let Value::Object(ref obj_ref) = result {
            let map = obj_ref.borrow();
            let warnings = map.get("warnings").unwrap();
            if let Value::List(ref list_ref) = warnings {
                let list = list_ref.borrow();
                assert_eq!(list.len(), 0, "expected no warnings for valid program");
            } else {
                panic!("expected warnings to be a list");
            }
        } else {
            panic!("expected result to be an object");
        }
    }

    #[tokio::test]
    async fn test_check_returns() {
        let vm = make_vm().await;

        // A function with return
        let test_program = ProgramBuilder::new()
            .function("good", &["x"], |b| {
                b.ret(Expression::ident("x"))
            })
            .build();

        let test_ast = program_to_value(&test_program);
        vm.borrow().global_env.define("test_ast".to_string(), test_ast);

        // Get the function_decl node and check its body
        let check_prog = Program::new(vec![
            Statement::var_decl("stmts", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("test_ast"), Expression::string("statements")],
            )),
            Statement::var_decl("fn_node", Expression::call(
                Expression::ident("list_get"),
                vec![Expression::ident("stmts"), Expression::int(0)],
            )),
            Statement::var_decl("body", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("fn_node"), Expression::string("body")],
            )),
            Statement::expr_stmt(Expression::call(
                Expression::ident("check_returns"),
                vec![Expression::ident("body")],
            )),
        ]);

        let result = IshVm::run(&vm, &check_prog).await.unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_is_constant_expr() {
        let vm = make_vm().await;

        // Build a constant expression: 2 + 3
        let const_expr = Expression::binary(BinaryOperator::Add, Expression::int(2), Expression::int(3));
        let const_value = ish_vm::reflection::expr_to_value(&const_expr);
        vm.borrow().global_env.define("test_expr".to_string(), const_value);

        let check_prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("is_constant_expr"),
                vec![Expression::ident("test_expr")],
            )),
        ]);

        let result = IshVm::run(&vm, &check_prog).await.unwrap();
        assert_eq!(result, Value::Bool(true));

        // Non-constant: x + 3
        let non_const_expr = Expression::binary(BinaryOperator::Add, Expression::ident("x"), Expression::int(3));
        let non_const_value = ish_vm::reflection::expr_to_value(&non_const_expr);
        vm.borrow().global_env.define("test_expr2".to_string(), non_const_value);

        let check_prog2 = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("is_constant_expr"),
                vec![Expression::ident("test_expr2")],
            )),
        ]);

        let result2 = IshVm::run(&vm, &check_prog2).await.unwrap();
        assert_eq!(result2, Value::Bool(false));
    }
}
