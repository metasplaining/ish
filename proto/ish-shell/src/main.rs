// ish-shell: Command-line entry point demonstrating the full ish prototype.
//
// Runs the 6 verification demonstrations:
// 1. Interpreted function execution (factorial)
// 2. Compiled function execution (factorial via codegen)
// 3. Self-hosted analyzer (detects undeclared variable)
// 4. Self-hosted generator (produces compilable Rust)
// 5. Standard library in ish (abs, map, range)
// 6. Consistency (interpreted == compiled)

use ish_ast::*;
use ish_ast::builder::ProgramBuilder;
use ish_vm::interpreter::IshVm;
use ish_vm::value::Value;
use ish_vm::reflection::program_to_value;
use ish_codegen::CompilationDriver;
use std::path::PathBuf;

fn main() {
    println!("=== ish prototype demonstration ===\n");

    let mut passed = 0;
    let total = 6;

    // ── Demo 1: Interpreted function ────────────────────────────────────────
    print!("1. Interpreted factorial(10)... ");
    let mut vm = IshVm::new();
    ish_stdlib::load_all(&mut vm);

    let factorial_program = ProgramBuilder::new()
        .function("factorial", &["n"], |b| {
            b.if_else(
                Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
                |b| b.ret(Expression::int(1)),
                |b| b.ret(Expression::binary(
                    BinaryOperator::Mul,
                    Expression::ident("n"),
                    Expression::call(
                        Expression::ident("factorial"),
                        vec![Expression::binary(BinaryOperator::Sub, Expression::ident("n"), Expression::int(1))],
                    ),
                )),
            )
        })
        .stmt(Statement::expr_stmt(Expression::call(
            Expression::ident("factorial"),
            vec![Expression::int(10)],
        )))
        .build();

    match vm.run(&factorial_program) {
        Ok(Value::Int(3628800)) => { println!("PASS (3628800)"); passed += 1; }
        Ok(other) => println!("FAIL (got {:?})", other),
        Err(e) => println!("ERROR: {}", e),
    }

    // ── Demo 2: Compiled function ───────────────────────────────────────────
    print!("2. Compiled factorial(10)... ");
    // Get factorial function AST as a value
    let fact_ast = program_to_value(&ProgramBuilder::new()
        .function("factorial", &["n"], |b| {
            b.if_else(
                Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
                |b| b.ret(Expression::int(1)),
                |b| b.ret(Expression::binary(
                    BinaryOperator::Mul,
                    Expression::ident("n"),
                    Expression::call(
                        Expression::ident("factorial"),
                        vec![Expression::binary(BinaryOperator::Sub, Expression::ident("n"), Expression::int(1))],
                    ),
                )),
            )
        })
        .build());

    vm.global_env.define("test_ast".to_string(), fact_ast);

    // Use the ish-written generator to produce Rust code
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

    match vm.run(&gen_prog) {
        Ok(Value::String(ref rust_code)) => {
            // Add #[no_mangle] pub extern "C" to the generated function
            let code_with_ffi = rust_code.replace(
                "fn factorial(",
                "#[no_mangle]\npub extern \"C\" fn factorial(",
            );

            let runtime_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("ish-runtime");
            let driver = CompilationDriver::new(runtime_path);

            match driver.compile_function_1(&code_with_ffi, "factorial") {
                Ok((_lib, func)) => {
                    let result = unsafe { func(10) };
                    if result == 3628800 {
                        println!("PASS ({})", result);
                        passed += 1;
                    } else {
                        println!("FAIL (got {})", result);
                    }
                }
                Err(e) => println!("ERROR: {}", e),
            }
        }
        Ok(other) => println!("FAIL (generator returned {:?})", other),
        Err(e) => println!("ERROR: {}", e),
    }

    // ── Demo 3: Self-hosted analyzer ────────────────────────────────────────
    print!("3. Analyzer detects undeclared variable... ");
    let bad_program = ProgramBuilder::new()
        .stmt(Statement::expr_stmt(Expression::ident("undefined_var")))
        .build();
    let bad_ast = program_to_value(&bad_program);
    vm.global_env.define("bad_ast".to_string(), bad_ast);

    let analyze_prog = Program::new(vec![
        Statement::expr_stmt(Expression::call(
            Expression::ident("analyze"),
            vec![Expression::ident("bad_ast")],
        )),
    ]);

    match vm.run(&analyze_prog) {
        Ok(Value::Object(ref result)) => {
            let obj = result.borrow();
            if let Some(Value::List(ref warnings)) = obj.get("warnings") {
                let warns = warnings.borrow();
                if !warns.is_empty() {
                    println!("PASS ({} warning(s) found)", warns.len());
                    passed += 1;
                } else {
                    println!("FAIL (no warnings)");
                }
            } else {
                println!("FAIL (no warnings key)");
            }
        }
        Ok(other) => println!("FAIL (got {:?})", other),
        Err(e) => println!("ERROR: {}", e),
    }

    // ── Demo 4: Self-hosted generator ───────────────────────────────────────
    print!("4. Generator produces compilable Rust... ");
    let add_program = ProgramBuilder::new()
        .function("add", &["a", "b"], |b| {
            b.ret(Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::ident("b")))
        })
        .build();
    let add_ast = program_to_value(&add_program);
    vm.global_env.define("add_ast".to_string(), add_ast);

    let gen_add_prog = Program::new(vec![
        Statement::var_decl("stmts", Expression::call(
            Expression::ident("obj_get"),
            vec![Expression::ident("add_ast"), Expression::string("statements")],
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

    match vm.run(&gen_add_prog) {
        Ok(Value::String(ref code)) => {
            let code_with_ffi = code.replace("fn add(", "#[no_mangle]\npub extern \"C\" fn add(");
            let runtime_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("ish-runtime");
            let driver = CompilationDriver::new(runtime_path);
            match driver.compile_function_2(&code_with_ffi, "add") {
                Ok((_lib, func)) => {
                    let result = unsafe { func(10, 32) };
                    if result == 42 {
                        println!("PASS (add(10,32) = {})", result);
                        passed += 1;
                    } else {
                        println!("FAIL (got {})", result);
                    }
                }
                Err(e) => println!("ERROR compiling: {}", e),
            }
        }
        Ok(other) => println!("FAIL (generator returned {:?})", other),
        Err(e) => println!("ERROR: {}", e),
    }

    // ── Demo 5: Standard library in ish ─────────────────────────────────────
    print!("5. Stdlib: abs(-42) and sum(range(5))... ");
    let stdlib_prog = Program::new(vec![
        Statement::var_decl("a", Expression::call(Expression::ident("abs"), vec![Expression::int(-42)])),
        Statement::var_decl("s", Expression::call(Expression::ident("sum"), vec![
            Expression::call(Expression::ident("range"), vec![Expression::int(5)]),
        ])),
        Statement::expr_stmt(Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::ident("s"))),
    ]);

    match vm.run(&stdlib_prog) {
        Ok(Value::Int(52)) => { println!("PASS (42 + 10 = 52)"); passed += 1; }
        Ok(other) => println!("FAIL (got {:?})", other),
        Err(e) => println!("ERROR: {}", e),
    }

    // ── Demo 6: Consistency ─────────────────────────────────────────────────
    print!("6. Consistency: interpreted == compiled for factorial(5,8,12)... ");
    let test_inputs = [5i64, 8, 12];
    let mut consistent = true;

    // Get interpreted results
    let mut interpreted_results = Vec::new();
    for &n in &test_inputs {
        let prog = Program::new(vec![
            Statement::expr_stmt(Expression::call(
                Expression::ident("factorial"),
                vec![Expression::int(n)],
            )),
        ]);
        match vm.run(&prog) {
            Ok(Value::Int(v)) => interpreted_results.push(v),
            _ => { consistent = false; break; }
        }
    }

    if consistent {
        // Get compiled factorial
        let fact_ast_2 = program_to_value(&ProgramBuilder::new()
            .function("factorial", &["n"], |b| {
                b.if_else(
                    Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
                    |b| b.ret(Expression::int(1)),
                    |b| b.ret(Expression::binary(
                        BinaryOperator::Mul,
                        Expression::ident("n"),
                        Expression::call(
                            Expression::ident("factorial"),
                            vec![Expression::binary(BinaryOperator::Sub, Expression::ident("n"), Expression::int(1))],
                        ),
                    )),
                )
            })
            .build());

        vm.global_env.define("fact_ast_2".to_string(), fact_ast_2);

        let gen_prog2 = Program::new(vec![
            Statement::var_decl("stmts", Expression::call(
                Expression::ident("obj_get"),
                vec![Expression::ident("fact_ast_2"), Expression::string("statements")],
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

        match vm.run(&gen_prog2) {
            Ok(Value::String(ref code)) => {
                let code_with_ffi = code.replace(
                    "fn factorial(",
                    "#[no_mangle]\npub extern \"C\" fn factorial(",
                );
                let runtime_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join("ish-runtime");
                let driver = CompilationDriver::new(runtime_path);
                match driver.compile_function_1(&code_with_ffi, "factorial") {
                    Ok((_lib, func)) => {
                        for (i, &n) in test_inputs.iter().enumerate() {
                            let compiled = unsafe { func(n) };
                            if compiled != interpreted_results[i] {
                                println!("FAIL (factorial({}) interpreted={} compiled={})", n, interpreted_results[i], compiled);
                                consistent = false;
                                break;
                            }
                        }
                        if consistent {
                            println!("PASS");
                            passed += 1;
                        }
                    }
                    Err(e) => { println!("ERROR compiling: {}", e); consistent = false; }
                }
            }
            _ => { println!("FAIL (generator error)"); consistent = false; }
        }
    } else {
        println!("FAIL (interpreter error)");
    }

    // ── Summary ─────────────────────────────────────────────────────────────
    println!("\n=== Results: {}/{} passed ===", passed, total);
    if passed == total {
        println!("All demonstrations successful! The ish prototype is complete.");
    } else {
        println!("Some demonstrations failed.");
        std::process::exit(1);
    }
}
