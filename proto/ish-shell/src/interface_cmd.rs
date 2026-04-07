use std::path::{Path, PathBuf};
use ish_ast::{Statement, TypeAnnotation, Visibility};

/// Run `ish interface freeze [target]`.
///
/// If `target` is None, walks `<project_root>/src/` and processes all .ish files.
/// If `target` is Some(module_name), resolves the name to a single .ish file.
/// For each file: parse, collect pub FunctionDecl and TypeAlias, write .ishi sibling.
pub fn freeze(target: Option<String>, project_root: &Path) {
    let src_root = project_root.join("src");
    if !src_root.exists() {
        eprintln!("error: no src/ directory found under {}", project_root.display());
        std::process::exit(1);
    }
    let files: Vec<PathBuf> = if let Some(ref mod_name) = target {
        // Resolve module_name (slash-separated) to src_root/a/b/c.ish or index.ish.
        let parts: Vec<&str> = mod_name.split('/').collect();
        let mut candidate = src_root.clone();
        for part in &parts {
            candidate.push(part);
        }
        candidate.set_extension("ish");
        if candidate.exists() {
            vec![candidate]
        } else {
            // Try index.ish
            candidate.set_extension("");
            candidate.push("index.ish");
            if candidate.exists() {
                vec![candidate]
            } else {
                eprintln!("error: module '{}' not found", mod_name);
                std::process::exit(1);
            }
        }
    } else {
        // Walk src/ recursively for all .ish files.
        collect_ish_files(&src_root)
    };

    for file in files {
        process_file(&file, &src_root);
    }
}

fn collect_ish_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_ish_files(&path));
            } else if path.extension().map_or(false, |e| e == "ish") {
                result.push(path);
            }
        }
    }
    result
}

fn process_file(file: &Path, src_root: &Path) {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading {}: {}", file.display(), e);
            return;
        }
    };
    let program = match ish_parser::parse(&content) {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors {
                eprintln!("error parsing {}: {}", file.display(), e);
            }
            return;
        }
    };

    // Collect pub FunctionDecl and TypeAlias statements.
    let pub_decls: Vec<&Statement> = program
        .statements
        .iter()
        .filter(|s| match s {
            Statement::FunctionDecl { visibility, .. } => {
                matches!(visibility, Some(Visibility::Pub))
            }
            Statement::TypeAlias { visibility, .. } => {
                matches!(visibility, Some(Visibility::Pub))
            }
            _ => false,
        })
        .collect();

    // Format as .ishi content.
    let mut ishi_content = String::new();
    for decl in &pub_decls {
        ishi_content.push_str(&format_decl(decl));
        ishi_content.push('\n');
    }

    // Write to sibling .ishi file.
    let ishi_path = file.with_extension("ishi");
    match std::fs::write(&ishi_path, &ishi_content) {
        Ok(()) => {
            let rel = file.strip_prefix(src_root).unwrap_or(file);
            println!("Wrote {}", rel.with_extension("ishi").display());
        }
        Err(e) => eprintln!("error writing {}: {}", ishi_path.display(), e),
    }
}

fn format_decl(stmt: &Statement) -> String {
    match stmt {
        Statement::FunctionDecl {
            name,
            params,
            return_type,
            type_params,
            is_async,
            ..
        } => {
            let async_prefix = if *is_async { "async " } else { "" };
            let tparams = if type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", type_params.join(", "))
            };
            let params_str = params
                .iter()
                .map(|p| {
                    if let Some(ref ann) = p.type_annotation {
                        format!("{}: {}", p.name, format_type(ann))
                    } else {
                        p.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let ret = if let Some(ref rt) = return_type {
                format!(" -> {}", format_type(rt))
            } else {
                String::new()
            };
            format!("pub {}fn {}{}({}){} {{}}", async_prefix, name, tparams, params_str, ret)
        }
        Statement::TypeAlias {
            name, definition, ..
        } => {
            format!("pub type {} = {}", name, format_type(definition))
        }
        _ => String::new(),
    }
}

fn format_type(ty: &TypeAnnotation) -> String {
    match ty {
        TypeAnnotation::Simple(s) => s.clone(),
        TypeAnnotation::List(inner) => format!("List<{}>", format_type(inner)),
        TypeAnnotation::Object(fields) => {
            let fs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_type(v)))
                .collect();
            format!("{{ {} }}", fs.join(", "))
        }
        TypeAnnotation::Function { params, ret } => {
            let ps: Vec<String> = params.iter().map(|p| format_type(p)).collect();
            format!("({}) -> {}", ps.join(", "), format_type(ret))
        }
        TypeAnnotation::Union(types) => {
            let ts: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            ts.join(" | ")
        }
        TypeAnnotation::Optional(inner) => format!("{}?", format_type(inner)),
        TypeAnnotation::Intersection(types) => {
            let ts: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            ts.join(" & ")
        }
        TypeAnnotation::Tuple(types) => {
            let ts: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            format!("({})", ts.join(", "))
        }
        TypeAnnotation::Generic { base, type_args } => {
            let args: Vec<String> = type_args.iter().map(|a| format_type(a)).collect();
            format!("{}<{}>", base, args.join(", "))
        }
    }
}
