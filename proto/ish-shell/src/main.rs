mod highlight;
mod interface_cmd;
mod repl;
mod validate;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let no_history = args.iter().any(|a| a == "--no-history");

    // Filter out flags to get positional args
    let positional: Vec<&str> = args[1..]
        .iter()
        .filter(|a| !a.starts_with("--"))
        .filter(|a| a.as_str() != "-c")
        .map(|s| s.as_str())
        .collect();

    if positional.first() == Some(&"interface") {
        if positional.get(1) == Some(&"freeze") {
            let target = positional.get(2).map(|s| s.to_string());
            let cwd = std::env::current_dir().expect("cannot determine cwd");
            interface_cmd::freeze(target, &cwd);
            return;
        }
        eprintln!("unknown interface subcommand: {:?}", positional.get(1));
        std::process::exit(1);
    }

    if let Some(idx) = args.iter().position(|a| a == "-c") {
        // Inline execution: ish -c 'code'
        let code = args.get(idx + 1).expect("missing argument to -c");
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(repl::run_inline(code));
    } else if let Some(filename) = positional.first() {
        // File execution: ish script.ish
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(repl::run_file(filename));
    } else {
        // Interactive REPL (two-thread model — manages its own runtime)
        repl::run_interactive(no_history);
    }
}
