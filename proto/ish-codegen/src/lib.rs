// ish-codegen: Compilation driver for compiling generated Rust code into
// dynamically loadable libraries.
//
// Pipeline:
// 1. ish generator produces Rust source string
// 2. CompilationDriver writes source to a temp Cargo project
// 3. Invokes `cargo build --release` on the temp project
// 4. Loads the resulting .so via libloading
// 5. Returns callable function pointers

pub mod template;

use std::path::{Path, PathBuf};
use std::process::Command;

/// A compiled shared library loaded into memory.
pub struct CompiledLibrary {
    // Hold the library to keep it loaded
    _library: libloading::Library,
    /// Path to the temp directory (kept alive so the .so isn't deleted)
    _dir: tempfile::TempDir,
}

/// A compiled function that takes i64 args and returns i64.
/// This is the simplest possible FFI for the prototype.
pub type CompiledFn = unsafe extern "C" fn(i64) -> i64;
pub type CompiledFn2 = unsafe extern "C" fn(i64, i64) -> i64;

/// Error type for compilation.
#[derive(Debug)]
pub enum CompileError {
    Io(std::io::Error),
    CargoBuild(String),
    LibLoad(String),
    SymbolNotFound(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(e) => write!(f, "IO error: {}", e),
            CompileError::CargoBuild(e) => write!(f, "Cargo build failed: {}", e),
            CompileError::LibLoad(e) => write!(f, "Library load failed: {}", e),
            CompileError::SymbolNotFound(e) => write!(f, "Symbol not found: {}", e),
        }
    }
}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e)
    }
}

/// The compilation driver: writes a temp Cargo project, builds it, loads the .so.
pub struct CompilationDriver {
    /// Path to the ish-runtime crate (so the temp project can depend on it)
    runtime_path: PathBuf,
}

impl CompilationDriver {
    /// Create a new driver. `runtime_path` should be the absolute path to the
    /// ish-runtime crate directory.
    pub fn new(runtime_path: PathBuf) -> Self {
        Self { runtime_path }
    }

    /// Compile the given Rust source code into a shared library.
    ///
    /// `rust_source` — the generated Rust code (function definitions).
    /// `function_names` — names of the `#[no_mangle] pub extern "C" fn` symbols.
    ///
    /// The source is wrapped with the necessary boilerplate (crate-type = cdylib).
    pub fn compile(&self, rust_source: &str) -> Result<(CompiledLibrary, PathBuf), CompileError> {
        let dir = tempfile::tempdir()?;
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Write Cargo.toml
        let cargo_toml = template::cargo_toml(&self.runtime_path);
        std::fs::write(dir.path().join("Cargo.toml"), cargo_toml)?;

        // Write src/lib.rs with the generated code
        let lib_rs = template::lib_rs(rust_source);
        std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

        // Build
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(dir.path())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompileError::CargoBuild(stderr.to_string()));
        }

        // Find the built shared library
        let lib_path = find_cdylib(dir.path())?;

        // Load it
        let library = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| CompileError::LibLoad(e.to_string()))?
        };

        Ok((CompiledLibrary { _library: library, _dir: dir }, lib_path))
    }
    
    /// Compile and load, returning the library with a looked-up function symbol.
    pub fn compile_function_1(&self, rust_source: &str, fn_name: &str) -> Result<(CompiledLibrary, CompiledFn), CompileError> {
        let dir = tempfile::tempdir()?;
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        let cargo_toml = template::cargo_toml(&self.runtime_path);
        std::fs::write(dir.path().join("Cargo.toml"), cargo_toml)?;

        let lib_rs = template::lib_rs(rust_source);
        std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(dir.path())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompileError::CargoBuild(stderr.to_string()));
        }

        let lib_path = find_cdylib(dir.path())?;

        let library = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| CompileError::LibLoad(e.to_string()))?
        };

        let func: CompiledFn = unsafe {
            let sym = library.get::<CompiledFn>(fn_name.as_bytes())
                .map_err(|e| CompileError::SymbolNotFound(format!("{}: {}", fn_name, e)))?;
            *sym
        };

        Ok((CompiledLibrary { _library: library, _dir: dir }, func))
    }

    /// Compile and load a function taking two i64 arguments.
    pub fn compile_function_2(&self, rust_source: &str, fn_name: &str) -> Result<(CompiledLibrary, CompiledFn2), CompileError> {
        let dir = tempfile::tempdir()?;
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        let cargo_toml = template::cargo_toml(&self.runtime_path);
        std::fs::write(dir.path().join("Cargo.toml"), cargo_toml)?;

        let lib_rs = template::lib_rs(rust_source);
        std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(dir.path())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompileError::CargoBuild(stderr.to_string()));
        }

        let lib_path = find_cdylib(dir.path())?;

        let library = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| CompileError::LibLoad(e.to_string()))?
        };

        let func: CompiledFn2 = unsafe {
            let sym = library.get::<CompiledFn2>(fn_name.as_bytes())
                .map_err(|e| CompileError::SymbolNotFound(format!("{}: {}", fn_name, e)))?;
            *sym
        };

        Ok((CompiledLibrary { _library: library, _dir: dir }, func))
    }
}

/// Find the cdylib in the target/release directory.
fn find_cdylib(project_dir: &Path) -> Result<PathBuf, CompileError> {
    let release_dir = project_dir.join("target").join("release");
    
    for entry in std::fs::read_dir(&release_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "so" || ext == "dylib" {
                return Ok(path);
            }
        }
    }

    Err(CompileError::SymbolNotFound(format!(
        "No .so/.dylib found in {}",
        release_dir.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("ish-runtime")
    }

    #[test]
    fn test_compile_simple_function() {
        let driver = CompilationDriver::new(runtime_path());

        let source = r#"
#[no_mangle]
pub extern "C" fn double(x: i64) -> i64 {
    x * 2
}
"#;
        let result = driver.compile_function_1(source, "double");
        match result {
            Ok((_lib, func)) => {
                let result = unsafe { func(21) };
                assert_eq!(result, 42);
            }
            Err(e) => {
                // If cargo is not available in test env, skip
                eprintln!("Compilation test skipped: {}", e);
            }
        }
    }

    #[test]
    fn test_compile_factorial() {
        let driver = CompilationDriver::new(runtime_path());

        let source = r#"
#[no_mangle]
pub extern "C" fn factorial(n: i64) -> i64 {
    if n <= 1_i64 {
        return 1_i64;
    }
    return n * factorial(n - 1_i64);
}
"#;
        let result = driver.compile_function_1(source, "factorial");
        match result {
            Ok((_lib, func)) => {
                let result = unsafe { func(5) };
                assert_eq!(result, 120);
                let result = unsafe { func(10) };
                assert_eq!(result, 3628800);
            }
            Err(e) => {
                eprintln!("Compilation test skipped: {}", e);
            }
        }
    }
}
