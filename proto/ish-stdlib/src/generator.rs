// Rust Generator — an ish program that receives AST-as-values and produces
// Rust source code strings. Built using the Rust builder API.
//
// Generates valid Rust for functions containing:
// - Integer/float/bool/string literals
// - Variable declarations (let)
// - Arithmetic and comparison operators
// - If/else
// - While loops
// - Function calls
// - Return

use ish_ast::*;
use ish_ast::builder::ProgramBuilder;
use ish_vm::interpreter::IshVm;

/// Register the generator functions into the VM's global environment.
pub async fn register_generator(vm: &mut IshVm) {
    let generator_program = build_generator();
    vm.run(&generator_program).await.unwrap();
}

/// Build the Rust generator as an ish program (AST).
///
/// Defines these functions:
/// - generate_rust(ast_node) -> string of Rust code
/// - generate_expr(node) -> string
/// - generate_stmt(node, indent_str) -> string
/// - generate_block(node, indent_str) -> string
fn build_generator() -> Program {
    ProgramBuilder::new()
        // ── rust_op: map ish op names to Rust operator symbols ──────────────
        .function("rust_op", &["op"], |b| {
            b.if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("add")), |b| b.ret(Expression::string("+")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("sub")), |b| b.ret(Expression::string("-")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("mul")), |b| b.ret(Expression::string("*")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("div")), |b| b.ret(Expression::string("/")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("mod")), |b| b.ret(Expression::string("%")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("eq")), |b| b.ret(Expression::string("==")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("neq")), |b| b.ret(Expression::string("!=")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("lt")), |b| b.ret(Expression::string("<")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("gt")), |b| b.ret(Expression::string(">")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("lteq")), |b| b.ret(Expression::string("<=")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("gteq")), |b| b.ret(Expression::string(">=")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("and")), |b| b.ret(Expression::string("&&")))
                .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("or")), |b| b.ret(Expression::string("||")))
                .ret(Expression::ident("op"))
        })

        // ── generate_expr: generate Rust expression string ──────────────────
        .function("generate_expr", &["node"], |b| {
            b.var_decl("kind", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("kind")]))
                // literal
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("literal")),
                    |b| b
                        .var_decl("lt", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("literal_type")]))
                        .var_decl("val", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]))
                        .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("lt"), Expression::string("int")),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::call(Expression::ident("to_string"), vec![Expression::ident("val")]),
                                Expression::string("_i64"),
                            ])),
                        )
                        .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("lt"), Expression::string("float")),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::call(Expression::ident("to_string"), vec![Expression::ident("val")]),
                                Expression::string("_f64"),
                            ])),
                        )
                        .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("lt"), Expression::string("bool")),
                            |b| b.ret(Expression::call(Expression::ident("to_string"), vec![Expression::ident("val")])),
                        )
                        .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("lt"), Expression::string("string")),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::string("\""),
                                Expression::call(Expression::ident("str_concat"), vec![Expression::ident("val"), Expression::string("\"")]),
                            ])),
                        )
                        .if_then(Expression::binary(BinaryOperator::Eq, Expression::ident("lt"), Expression::string("char")),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::string("'"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::call(Expression::ident("to_string"), vec![Expression::ident("val")]),
                                    Expression::string("'"),
                                ]),
                            ])),
                        )
                        .ret(Expression::string("()")), // null
                )
                // identifier
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("identifier")),
                    |b| b.ret(Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")])),
                )
                // binary_op
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("binary_op")),
                    |b| b
                        .var_decl("left_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("left")]),
                        ]))
                        .var_decl("right_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("right")]),
                        ]))
                        .var_decl("op_str", Expression::call(Expression::ident("rust_op"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("op")]),
                        ]))
                        // Build: (left op right)
                        .ret(Expression::call(Expression::ident("str_concat"), vec![
                            Expression::string("("),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::ident("left_str"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string(" "),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::ident("op_str"),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::string(" "),
                                            Expression::call(Expression::ident("str_concat"), vec![
                                                Expression::ident("right_str"),
                                                Expression::string(")"),
                                            ]),
                                        ]),
                                    ]),
                                ]),
                            ]),
                        ])),
                )
                // unary_op
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("unary_op")),
                    |b| b
                        .var_decl("operand_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("operand")]),
                        ]))
                        .var_decl("op", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("op")]))
                        .if_then(
                            Expression::binary(BinaryOperator::Eq, Expression::ident("op"), Expression::string("negate")),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![Expression::string("-"), Expression::ident("operand_str")])),
                        )
                        .ret(Expression::call(Expression::ident("str_concat"), vec![Expression::string("!"), Expression::ident("operand_str")])),
                )
                // function_call
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("function_call")),
                    |b| b
                        .var_decl("callee_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("callee")]),
                        ]))
                        .var_decl("args", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("args")]))
                        .var_decl("arg_strs", Expression::list(vec![]))
                        .var_decl("i", Expression::int(0))
                        .while_loop(
                            Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("args")])),
                            |b| b
                                .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                    Expression::ident("arg_strs"),
                                    Expression::call(Expression::ident("generate_expr"), vec![
                                        Expression::call(Expression::ident("list_get"), vec![Expression::ident("args"), Expression::ident("i")]),
                                    ]),
                                ]))
                                .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                        )
                        .ret(Expression::call(Expression::ident("str_concat"), vec![
                            Expression::ident("callee_str"),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::string("("),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::call(Expression::ident("list_join"), vec![Expression::ident("arg_strs"), Expression::string(", ")]),
                                    Expression::string(")"),
                                ]),
                            ]),
                        ])),
                )
                .ret(Expression::string("/* unknown expr */"))
        })

        // ── generate_stmt: generate Rust statement string ───────────────────
        .function("generate_stmt", &["node", "indent"], |b| {
            b.var_decl("kind", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("kind")]))
                // var_decl
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("var_decl")),
                    |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                        Expression::ident("indent"),
                        Expression::call(Expression::ident("str_concat"), vec![
                            Expression::string("let mut "),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")]),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string(" = "),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::call(Expression::ident("generate_expr"), vec![
                                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]),
                                        ]),
                                        Expression::string(";\n"),
                                    ]),
                                ]),
                            ]),
                        ]),
                    ])),
                )
                // assignment
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("assignment")),
                    |b| b
                        .var_decl("target", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("target")]))
                        .var_decl("target_name", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("target"), Expression::string("name")]))
                        .ret(Expression::call(Expression::ident("str_concat"), vec![
                            Expression::ident("indent"),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::ident("target_name"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string(" = "),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::call(Expression::ident("generate_expr"), vec![
                                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]),
                                        ]),
                                        Expression::string(";\n"),
                                    ]),
                                ]),
                            ]),
                        ])),
                )
                // return
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("return")),
                    |b| b
                        .var_decl("rv", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("value")]))
                        .if_else(
                            Expression::call(Expression::ident("is_type"), vec![Expression::ident("rv"), Expression::string("null")]),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![Expression::ident("indent"), Expression::string("return;\n")])),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::ident("indent"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string("return "),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::call(Expression::ident("generate_expr"), vec![Expression::ident("rv")]),
                                        Expression::string(";\n"),
                                    ]),
                                ]),
                            ])),
                        ),
                )
                // expr_stmt
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("expr_stmt")),
                    |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                        Expression::ident("indent"),
                        Expression::call(Expression::ident("str_concat"), vec![
                            Expression::call(Expression::ident("generate_expr"), vec![
                                Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("expression")]),
                            ]),
                            Expression::string(";\n"),
                        ]),
                    ])),
                )
                // if
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("if")),
                    |b| b
                        .var_decl("cond_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("condition")]),
                        ]))
                        .var_decl("then_str", Expression::call(Expression::ident("generate_block"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("then_block")]),
                            Expression::ident("indent"),
                        ]))
                        .var_decl("eb", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("else_block")]))
                        .if_else(
                            Expression::call(Expression::ident("is_type"), vec![Expression::ident("eb"), Expression::string("object")]),
                            |b| b
                                .var_decl("else_str", Expression::call(Expression::ident("generate_block"), vec![Expression::ident("eb"), Expression::ident("indent")]))
                                .ret(Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::ident("indent"),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::string("if "),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::ident("cond_str"),
                                            Expression::call(Expression::ident("str_concat"), vec![
                                                Expression::string(" "),
                                                Expression::call(Expression::ident("str_concat"), vec![
                                                    Expression::ident("then_str"),
                                                    Expression::call(Expression::ident("str_concat"), vec![
                                                        Expression::string(" else "),
                                                        Expression::call(Expression::ident("str_concat"), vec![
                                                            Expression::ident("else_str"),
                                                            Expression::string("\n"),
                                                        ]),
                                                    ]),
                                                ]),
                                            ]),
                                        ]),
                                    ]),
                                ])),
                            |b| b.ret(Expression::call(Expression::ident("str_concat"), vec![
                                Expression::ident("indent"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string("if "),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::ident("cond_str"),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::string(" "),
                                            Expression::call(Expression::ident("str_concat"), vec![
                                                Expression::ident("then_str"),
                                                Expression::string("\n"),
                                            ]),
                                        ]),
                                    ]),
                                ]),
                            ])),
                        ),
                )
                // while
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("while")),
                    |b| b
                        .var_decl("cond_str", Expression::call(Expression::ident("generate_expr"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("condition")]),
                        ]))
                        .var_decl("body_str", Expression::call(Expression::ident("generate_block"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                            Expression::ident("indent"),
                        ]))
                        .ret(Expression::call(Expression::ident("str_concat"), vec![
                            Expression::ident("indent"),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::string("while "),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::ident("cond_str"),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::string(" "),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::ident("body_str"),
                                            Expression::string("\n"),
                                        ]),
                                    ]),
                                ]),
                            ]),
                        ])),
                )
                // block (as a standalone statement)
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("block")),
                    |b| b.ret(Expression::call(Expression::ident("generate_block"), vec![Expression::ident("node"), Expression::ident("indent")])),
                )
                .ret(Expression::call(Expression::ident("str_concat"), vec![Expression::ident("indent"), Expression::string("/* unknown stmt */\n")]))
        })

        // ── generate_block: generate a { ... } block ────────────────────────
        .function("generate_block", &["node", "indent"], |b| {
            b.var_decl("result", Expression::call(Expression::ident("str_concat"), vec![Expression::ident("indent"), Expression::string("{\n")]))
                .var_decl("inner_indent", Expression::call(Expression::ident("str_concat"), vec![Expression::ident("indent"), Expression::string("    ")]))
                .var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("statements")]))
                .var_decl("i", Expression::int(0))
                .while_loop(
                    Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")])),
                    |b| b
                        .assign("result", Expression::call(Expression::ident("str_concat"), vec![
                            Expression::ident("result"),
                            Expression::call(Expression::ident("generate_stmt"), vec![
                                Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                                Expression::ident("inner_indent"),
                            ]),
                        ]))
                        .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                )
                .assign("result", Expression::call(Expression::ident("str_concat"), vec![
                    Expression::ident("result"),
                    Expression::call(Expression::ident("str_concat"), vec![Expression::ident("indent"), Expression::string("}")]),
                ]))
                .ret(Expression::ident("result"))
        })

        // ── generate_rust: main entry point for function declarations ───────
        // generate_rust(function_decl_node) -> Rust source string
        .function("generate_rust", &["node"], |b| {
            b.var_decl("kind", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("kind")]))
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("function_decl")),
                    |b| b
                        .var_decl("name", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("name")]))
                        .var_decl("params", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("params")]))
                        .var_decl("param_strs", Expression::list(vec![]))
                        .var_decl("i", Expression::int(0))
                        .while_loop(
                            Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("params")])),
                            |b| b
                                .var_decl("pname", Expression::call(Expression::ident("obj_get"), vec![
                                    Expression::call(Expression::ident("list_get"), vec![Expression::ident("params"), Expression::ident("i")]),
                                    Expression::string("name"),
                                ]))
                                .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                    Expression::ident("param_strs"),
                                    Expression::call(Expression::ident("str_concat"), vec![Expression::ident("pname"), Expression::string(": i64")]),
                                ]))
                                .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                        )
                        .var_decl("params_str", Expression::call(Expression::ident("list_join"), vec![Expression::ident("param_strs"), Expression::string(", ")]))
                        .var_decl("body_str", Expression::call(Expression::ident("generate_block"), vec![
                            Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("body")]),
                            Expression::string(""),
                        ]))
                        // fn name(params) -> i64 { body }
                        .ret(Expression::call(Expression::ident("str_concat"), vec![
                            Expression::string("fn "),
                            Expression::call(Expression::ident("str_concat"), vec![
                                Expression::ident("name"),
                                Expression::call(Expression::ident("str_concat"), vec![
                                    Expression::string("("),
                                    Expression::call(Expression::ident("str_concat"), vec![
                                        Expression::ident("params_str"),
                                        Expression::call(Expression::ident("str_concat"), vec![
                                            Expression::string(") -> i64 "),
                                            Expression::ident("body_str"),
                                        ]),
                                    ]),
                                ]),
                            ]),
                        ])),
                )
                // program
                .if_then(
                    Expression::binary(BinaryOperator::Eq, Expression::ident("kind"), Expression::string("program")),
                    |b| b
                        .var_decl("stmts", Expression::call(Expression::ident("obj_get"), vec![Expression::ident("node"), Expression::string("statements")]))
                        .var_decl("parts", Expression::list(vec![]))
                        .var_decl("i", Expression::int(0))
                        .while_loop(
                            Expression::binary(BinaryOperator::Lt, Expression::ident("i"), Expression::call(Expression::ident("list_length"), vec![Expression::ident("stmts")])),
                            |b| b
                                .expr_stmt(Expression::call(Expression::ident("list_push"), vec![
                                    Expression::ident("parts"),
                                    Expression::call(Expression::ident("generate_rust"), vec![
                                        Expression::call(Expression::ident("list_get"), vec![Expression::ident("stmts"), Expression::ident("i")]),
                                    ]),
                                ]))
                                .assign("i", Expression::binary(BinaryOperator::Add, Expression::ident("i"), Expression::int(1))),
                        )
                        .ret(Expression::call(Expression::ident("list_join"), vec![Expression::ident("parts"), Expression::string("\n\n")])),
                )
                .ret(Expression::string("/* unsupported node */"))
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::builder::ProgramBuilder;
    use ish_vm::value::Value;
    use ish_vm::reflection::program_to_value;

    async fn make_vm() -> IshVm {
        let mut vm = IshVm::new();
        crate::load_all(&mut vm).await;
        vm
    }

    #[tokio::test]
    async fn test_generate_simple_add_function() {
        let mut vm = make_vm().await;

        // Build: fn add(a, b) { return a + b; }
        let test_program = ProgramBuilder::new()
            .function("add", &["a", "b"], |b| {
                b.ret(Expression::binary(
                    BinaryOperator::Add,
                    Expression::ident("a"),
                    Expression::ident("b"),
                ))
            })
            .build();

        let test_ast = program_to_value(&test_program);

        // Get the function_decl node
        vm.global_env.define("test_ast".to_string(), test_ast);

        let gen_prog = Program::new(vec![
            Statement::var_decl("stmts", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("test_ast"), Expression::string("statements")],
            )),
            Statement::var_decl("fn_node", Expression::call(
                Expression::ident("list_get"),
                vec![Expression::ident("stmts"), Expression::int(0)],
            )),
            Statement::expr_stmt(Expression::call(
                Expression::ident("generate_rust"),
                vec![Expression::ident("fn_node")],
            )),
        ]);

        let result = vm.run(&gen_prog).await.unwrap();
        if let Value::String(s) = &result {
            let code = s.as_ref();
            assert!(code.contains("fn add(a: i64, b: i64) -> i64"), "got: {}", code);
            assert!(code.contains("return (a + b)"), "got: {}", code);
        } else {
            panic!("expected string, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_generate_factorial() {
        let mut vm = make_vm().await;

        let test_program = ProgramBuilder::new()
            .function("factorial", &["n"], |b| {
                b.if_else(
                    Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
                    |b| b.ret(Expression::int(1)),
                    |b| b.ret(Expression::binary(
                        BinaryOperator::Mul,
                        Expression::ident("n"),
                        Expression::call(
                            Expression::ident("factorial"),
                            vec![Expression::binary(
                                BinaryOperator::Sub,
                                Expression::ident("n"),
                                Expression::int(1),
                            )],
                        ),
                    )),
                )
            })
            .build();

        let test_ast = program_to_value(&test_program);
        vm.global_env.define("test_ast".to_string(), test_ast);

        let gen_prog = Program::new(vec![
            Statement::var_decl("stmts", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("test_ast"), Expression::string("statements")],
            )),
            Statement::var_decl("fn_node", Expression::call(
                Expression::ident("list_get"),
                vec![Expression::ident("stmts"), Expression::int(0)],
            )),
            Statement::expr_stmt(Expression::call(
                Expression::ident("generate_rust"),
                vec![Expression::ident("fn_node")],
            )),
        ]);

        let result = vm.run(&gen_prog).await.unwrap();
        if let Value::String(s) = &result {
            let code = s.as_ref();
            assert!(code.contains("fn factorial(n: i64) -> i64"), "got: {}", code);
            assert!(code.contains("if"), "got: {}", code);
            assert!(code.contains("return"), "got: {}", code);
            assert!(code.contains("factorial("), "got: {}", code);
        } else {
            panic!("expected string, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_generate_program_with_multiple_functions() {
        let mut vm = make_vm().await;

        let test_program = ProgramBuilder::new()
            .function("add", &["a", "b"], |b| {
                b.ret(Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::ident("b")))
            })
            .function("main_fn", &[], |b| {
                b.var_decl("result", Expression::call(Expression::ident("add"), vec![Expression::int(10), Expression::int(32)]))
                    .ret(Expression::ident("result"))
            })
            .build();

        let test_ast = program_to_value(&test_program);
        vm.global_env.define("test_ast".to_string(), test_ast);

        let gen_prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("generate_rust"),
                vec![Expression::ident("test_ast")],
            )),
        ]);

        let result = vm.run(&gen_prog).await.unwrap();
        if let Value::String(s) = &result {
            let code = s.as_ref();
            assert!(code.contains("fn add("), "got: {}", code);
            assert!(code.contains("fn main_fn("), "got: {}", code);
        } else {
            panic!("expected string, got {:?}", result);
        }
    }
}
