use ish_parser::parse;
use ish_ast::*;

#[test]
fn parse_simple_command() {
    let prog = parse("git status").unwrap();
    assert_eq!(prog.statements.len(), 1);
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, pipes, redirections, background } => {
            assert_eq!(command, "git");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], ShellArg::Bare(s) if s == "status"));
            assert!(pipes.is_empty());
            assert!(redirections.is_empty());
            assert!(!background);
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_command_with_flags() {
    // Flags like -la and paths like /tmp are ambiguous with operators.
    // Use force-command prefix > to ensure shell mode.
    let prog = parse("> ls -la /tmp").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, .. } => {
            assert_eq!(command, "ls");
            assert_eq!(args.len(), 2);
            assert!(matches!(&args[0], ShellArg::Bare(s) if s == "-la"));
            assert!(matches!(&args[1], ShellArg::Bare(s) if s == "/tmp"));
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_pipe() {
    let prog = parse("ls | grep foo").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, pipes, .. } => {
            assert_eq!(command, "ls");
            assert_eq!(pipes.len(), 1);
            assert_eq!(pipes[0].command, "grep");
            assert_eq!(pipes[0].args.len(), 1);
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_multi_pipe() {
    let prog = parse("cat file.txt | grep hello | wc -l").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, pipes, .. } => {
            assert_eq!(command, "cat");
            assert_eq!(args.len(), 1);
            assert_eq!(pipes.len(), 2);
            assert_eq!(pipes[0].command, "grep");
            assert_eq!(pipes[1].command, "wc");
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_redirection_stdout() {
    let prog = parse("cargo build > build.log").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, redirections, .. } => {
            assert_eq!(command, "cargo");
            assert_eq!(redirections.len(), 1);
            assert_eq!(redirections[0].kind, RedirectKind::StdoutWrite);
            assert_eq!(redirections[0].target, "build.log");
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_redirection_append() {
    let prog = parse("echo hello >> log.txt").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, redirections, .. } => {
            assert_eq!(command, "echo");
            assert_eq!(redirections.len(), 1);
            assert_eq!(redirections[0].kind, RedirectKind::StdoutAppend);
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_background() {
    let prog = parse("long_task &").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, background, .. } => {
            assert_eq!(command, "long_task");
            assert!(background);
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_glob() {
    let prog = parse("ls *.rs").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, .. } => {
            assert_eq!(command, "ls");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], ShellArg::Glob(s) if s == "*.rs"));
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_env_var_in_expression() {
    let prog = parse("let home = $HOME").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { value, .. } => {
            assert!(matches!(value, Expression::EnvVar(name) if name == "HOME"));
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_env_var_braced() {
    let prog = parse("let path = ${PATH}").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { value, .. } => {
            assert!(matches!(value, Expression::EnvVar(name) if name == "PATH"));
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_command_substitution_in_expression() {
    let prog = parse("let files = $(ls -la)").unwrap();
    match &prog.statements[0] {
        Statement::VariableDecl { value, .. } => {
            match value {
                Expression::CommandSubstitution(cmd) => {
                    match cmd.as_ref() {
                        Statement::ShellCommand { command, args, .. } => {
                            assert_eq!(command, "ls");
                            assert_eq!(args.len(), 1);
                        }
                        other => panic!("expected ShellCommand inside sub, got {:?}", other),
                    }
                }
                other => panic!("expected CommandSubstitution, got {:?}", other),
            }
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_keyword_not_command() {
    // Language keywords should parse as language, not commands
    let prog = parse("let x = 5").unwrap();
    assert!(matches!(&prog.statements[0], Statement::VariableDecl { .. }));

    let prog = parse("fn foo() {\n  return 1\n}").unwrap();
    assert!(matches!(&prog.statements[0], Statement::FunctionDecl { .. }));
}

#[test]
fn parse_force_command_prefix() {
    let prog = parse("> some_func arg1 arg2").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, .. } => {
            assert_eq!(command, "some_func");
            assert_eq!(args.len(), 2);
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}

#[test]
fn parse_env_var_in_shell() {
    let prog = parse("echo $HOME").unwrap();
    match &prog.statements[0] {
        Statement::ShellCommand { command, args, .. } => {
            assert_eq!(command, "echo");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], ShellArg::EnvVar(name) if name == "HOME"));
        }
        other => panic!("expected ShellCommand, got {:?}", other),
    }
}
