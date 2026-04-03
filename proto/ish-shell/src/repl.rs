use std::cell::RefCell;
use std::rc::Rc;
use ish_vm::interpreter::IshVm;
use ish_vm::builtins::BuiltinConfig;
use ish_vm::value::Value;
use reedline::{
    DefaultHinter, ExternalPrinter, FileBackedHistory, Reedline, Signal,
    DefaultPrompt, DefaultPromptSegment,
};
use nu_ansi_term::Style;
use tokio::task::LocalSet;

use crate::highlight::IshHighlighter;
use crate::validate::IshValidator;

/// Message sent from the shell thread to the main thread.
enum ShellMessage {
    /// Execute a parsed program.
    Execute(ish_ast::Program),
    /// Shutdown — user pressed Ctrl-D or an error occurred.
    Shutdown,
}

/// Result sent from the main thread back to the shell thread.
enum ExecResult {
    /// Successful execution with optional display value.
    Ok(Option<String>),
    /// Runtime error message.
    Err(String),
}

/// Run the interactive REPL with two-thread architecture.
///
/// Shell thread: Reedline loop → parse input → send AST to main thread.
/// Main thread: Tokio runtime + LocalSet → receive AST → vm.run() → send result.
pub fn run_interactive(no_history: bool) {
    // Channels for communication between shell and main threads.
    let (submit_tx, submit_rx) = std::sync::mpsc::channel::<(ShellMessage, std::sync::mpsc::Sender<ExecResult>)>();

    // ExternalPrinter for output from spawned tasks / println
    let printer = ExternalPrinter::<String>::default();
    let output_sender = printer.sender();

    // Shell thread: handles Reedline UI and parsing
    let shell_handle = std::thread::spawn(move || {
        let mut editor = if no_history {
            Reedline::create()
        } else {
            let history_path = dirs_next()
                .map(|d| d.join(".ish_history"))
                .unwrap_or_else(|| std::path::PathBuf::from(".ish_history"));
            let history = FileBackedHistory::with_file(1000, history_path)
                .expect("failed to create history file");
            Reedline::create().with_history(Box::new(history))
        };

        editor = editor
            .with_validator(Box::new(IshValidator))
            .with_highlighter(Box::new(IshHighlighter))
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().italic().fg(nu_ansi_term::Color::DarkGray)),
            ))
            .with_external_printer(printer);

        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::Basic("ish".to_string()),
            DefaultPromptSegment::Empty,
        );

        loop {
            match editor.read_line(&prompt) {
                Ok(Signal::Success(input)) => {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Parse on the shell thread (R6.2, R6.9)
                    let program = match ish_parser::parse(trimmed) {
                        Ok(p) => p,
                        Err(errors) => {
                            for e in &errors {
                                eprintln!("error: {}", e);
                            }
                            continue;
                        }
                    };

                    if program.has_any_incomplete() {
                        eprintln!("error: unexpected end of input");
                        continue;
                    }

                    // Send to main thread and wait for result
                    let (result_tx, result_rx) = std::sync::mpsc::channel();
                    if submit_tx.send((ShellMessage::Execute(program), result_tx)).is_err() {
                        break;
                    }
                    match result_rx.recv() {
                        Ok(ExecResult::Ok(Some(display))) => println!("{}", display),
                        Ok(ExecResult::Ok(None)) => {}
                        Ok(ExecResult::Err(msg)) => eprintln!("error: {}", msg),
                        Err(_) => break,
                    }
                }
                Ok(Signal::CtrlC) => {
                    // Cancel current input, continue
                }
                Ok(Signal::CtrlD) => {
                    let _ = submit_tx.send((ShellMessage::Shutdown, std::sync::mpsc::channel().0));
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    let _ = submit_tx.send((ShellMessage::Shutdown, std::sync::mpsc::channel().0));
                    break;
                }
            }
        }
    });

    // Main thread: Tokio runtime with persistent LocalSet
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let local = LocalSet::new();
        local.run_until(async {
            // Create VM with ExternalPrinter output routing
            let config = BuiltinConfig {
                output_sender: Some(output_sender),
            };
            let mut vm = Rc::new(RefCell::new(IshVm::with_config(&config)));
            ish_stdlib::load_all(&vm).await;

            // Receive and execute programs from the shell thread
            loop {
                // Use tokio::task::yield_now() to allow spawned tasks to
                // make progress between submissions
                match submit_rx.try_recv() {
                    Ok((ShellMessage::Execute(program), result_tx)) => {
                        let result = IshVm::run(&vm, &program).await;
                        let exec_result = match result {
                            Ok(Value::Null) => ExecResult::Ok(None),
                            Ok(val) => ExecResult::Ok(Some(val.to_display_string())),
                            Err(e) => ExecResult::Err(e.to_string()),
                        };
                        let _ = result_tx.send(exec_result);
                    }
                    Ok((ShellMessage::Shutdown, _)) => break,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // No message yet, yield to let spawned tasks run
                        tokio::task::yield_now().await;
                        // Small sleep to avoid busy-spinning
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }).await;
    });

    shell_handle.join().expect("shell thread panicked");
}

/// Execute a source file.
pub async fn run_file(filename: &str) {
    let contents = match std::fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ish: cannot read '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    // Strip shebang line
    let source = if contents.starts_with("#!") {
        contents.splitn(2, '\n').nth(1).unwrap_or("")
    } else {
        &contents
    };

    let program = ish_parser::parse(source).unwrap_or_else(|errors| {
        for e in &errors {
            eprintln!("{}:{}", filename, e);
        }
        std::process::exit(1);
    });

    if program.has_any_incomplete() {
        eprintln!("{}: unexpected end of input", filename);
        std::process::exit(1);
    }

    // Non-interactive: single thread, no ExternalPrinter, direct stdout (R6.5)
    let vm = Rc::new(RefCell::new(IshVm::new()));

    let local = LocalSet::new();
    local.run_until(ish_stdlib::load_all(&vm)).await;
    match local.run_until(IshVm::run(&vm, &program)).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}: {}", filename, e);
            std::process::exit(1);
        }
    }
}

/// Execute inline code from -c argument.
pub async fn run_inline(code: &str) {
    let program = ish_parser::parse(code).unwrap_or_else(|errors| {
        for e in &errors {
            eprintln!("ish: {}", e);
        }
        std::process::exit(1);
    });

    if program.has_any_incomplete() {
        eprintln!("ish: unexpected end of input");
        std::process::exit(1);
    }

    // Non-interactive: single thread, no ExternalPrinter, direct stdout (R6.5)
    let vm = Rc::new(RefCell::new(IshVm::new()));

    let local = LocalSet::new();
    local.run_until(ish_stdlib::load_all(&vm)).await;
    match local.run_until(IshVm::run(&vm, &program)).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("ish: {}", e);
            std::process::exit(1);
        }
    }
}

/// Get the home directory for history file placement.
fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
