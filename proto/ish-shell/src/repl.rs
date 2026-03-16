use ish_vm::interpreter::IshVm;
use ish_vm::value::Value;
use reedline::{
    DefaultHinter, FileBackedHistory, Reedline, Signal,
    DefaultPrompt, DefaultPromptSegment,
};
use nu_ansi_term::Style;

use crate::highlight::IshHighlighter;
use crate::validate::IshValidator;

/// Run the interactive REPL.
pub fn run_interactive(no_history: bool) {
    let mut vm = IshVm::new();
    ish_stdlib::load_all(&mut vm);

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
        ));

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
                process_input(trimmed, &mut vm);
            }
            Ok(Signal::CtrlC) => {
                // Cancel current input, continue
            }
            Ok(Signal::CtrlD) => {
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

/// Execute a source file.
pub fn run_file(filename: &str) {
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

    let mut vm = IshVm::new();
    ish_stdlib::load_all(&mut vm);

    match vm.run(&program) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}: {}", filename, e);
            std::process::exit(1);
        }
    }
}

/// Execute inline code from -c argument.
pub fn run_inline(code: &str) {
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

    let mut vm = IshVm::new();
    ish_stdlib::load_all(&mut vm);

    match vm.run(&program) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("ish: {}", e);
            std::process::exit(1);
        }
    }
}

fn process_input(input: &str, vm: &mut IshVm) {
    let program = match ish_parser::parse(input) {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors {
                eprintln!("error: {}", e);
            }
            return;
        }
    };

    if program.has_any_incomplete() {
        eprintln!("error: unexpected end of input");
        return;
    }

    match vm.run(&program) {
        Ok(Value::Null) => {}
        Ok(val) => println!("{}", val),
        Err(e) => eprintln!("error: {}", e),
    }
}

/// Get the home directory for history file placement.
fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
