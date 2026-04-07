use std::path::{Path, PathBuf};
use ish_runtime::error::{ErrorCode, RuntimeError};

/// Walk up from `start_dir`, returning the first directory containing `project.json`.
/// Returns `None` if the filesystem root is reached without finding one.
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        if current.join("project.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Derive a module path from a file path relative to the source root.
///
/// Strips the `src_root` prefix and `.ish` extension. Applies the `index.ish` rule:
/// if the filename (without extension) is `"index"`, the parent directory name is used instead.
pub fn derive_module_path(file_path: &Path, src_root: &Path) -> Result<Vec<String>, RuntimeError> {
    let relative = file_path.strip_prefix(src_root).map_err(|_| {
        RuntimeError::system_error(
            format!("File {:?} is not under source root {:?}", file_path, src_root),
            ErrorCode::ModuleNotFound,
        )
    })?;

    let stem = relative.file_stem().ok_or_else(|| {
        RuntimeError::system_error(
            format!("File {:?} has no filename", file_path),
            ErrorCode::ModuleNotFound,
        )
    })?;

    let ext = relative.extension();
    if ext != Some(std::ffi::OsStr::new("ish")) {
        return Err(RuntimeError::system_error(
            format!("File {:?} does not have .ish extension", file_path),
            ErrorCode::ModuleNotFound,
        ));
    }

    let parent = relative.parent();

    let mut segments: Vec<String> = if let Some(p) = parent {
        p.iter().map(|s| s.to_string_lossy().to_string()).collect()
    } else {
        Vec::new()
    };

    let stem_str = stem.to_string_lossy().to_string();
    if stem_str == "index" {
        // index.ish rule: use the parent directory name, don't add "index" as a segment.
        // If segments is empty (index.ish at src root), that's the root module.
        // The segments already represent the module path from the parent dirs.
    } else {
        segments.push(stem_str);
    }

    Ok(segments)
}

/// Resolve a module path (from a `use` statement) to a `.ish` file under `src_root`.
///
/// Checks two candidates:
///   - `src_root/a/b/c.ish`
///   - `src_root/a/b/c/index.ish`
///
/// Returns E019 (`ModulePathConflict`) if both exist, E016 (`ModuleNotFound`) if neither exists.
pub fn resolve_module_path(module_path: &[String], src_root: &Path) -> Result<PathBuf, RuntimeError> {
    let mut file_candidate = src_root.to_path_buf();
    for segment in module_path {
        file_candidate.push(segment);
    }

    let mut index_candidate = file_candidate.clone();
    file_candidate.set_extension("ish");
    index_candidate.push("index.ish");

    let file_exists = file_candidate.exists();
    let index_exists = index_candidate.exists();

    match (file_exists, index_exists) {
        (true, true) => Err(RuntimeError::system_error(
            format!(
                "Module path conflict: both {:?} and {:?} exist for module path '{}'",
                file_candidate,
                index_candidate,
                module_path.join("/")
            ),
            ErrorCode::ModulePathConflict,
        )),
        (true, false) => Ok(file_candidate),
        (false, true) => Ok(index_candidate),
        (false, false) => Err(RuntimeError::system_error(
            format!(
                "Module not found: no file at {:?} or {:?} for module path '{}'",
                file_candidate,
                index_candidate,
                module_path.join("/")
            ),
            ErrorCode::ModuleNotFound,
        )),
    }
}

/// Check whether `candidate` already appears in the loading stack (cycle detection).
/// Returns `true` if a cycle is detected.
pub fn check_cycle(loading_stack: &[Vec<String>], candidate: &[String]) -> bool {
    loading_stack.iter().any(|entry| entry.as_slice() == candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_project_root_at_current_dir() {
        let dir = std::env::temp_dir().join("ish_test_fpr_current");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("project.json"), "{}").unwrap();
        let result = find_project_root(&dir);
        assert_eq!(result, Some(dir.clone()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_project_root_via_walk() {
        let parent = std::env::temp_dir().join("ish_test_fpr_walk");
        let child = parent.join("child");
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.join("project.json"), "{}").unwrap();
        let result = find_project_root(&child);
        assert_eq!(result, Some(parent.clone()));
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn find_project_root_not_found() {
        let dir = std::env::temp_dir().join("ish_test_fpr_nf").join("deep").join("nested");
        let _ = fs::remove_dir_all(std::env::temp_dir().join("ish_test_fpr_nf"));
        fs::create_dir_all(&dir).unwrap();
        // No project.json anywhere in this chain (temp_dir itself might have one)
        let result = find_project_root(&dir);
        // We check that it either returns None or returns something above our test dir.
        // Since /tmp won't have project.json, result should be None.
        assert!(result.is_none() || !result.as_ref().unwrap().starts_with(&dir));
        let _ = fs::remove_dir_all(std::env::temp_dir().join("ish_test_fpr_nf"));
    }

    #[test]
    fn derive_module_path_standard() {
        let src_root = PathBuf::from("src");
        let file_path = PathBuf::from("src/net/http.ish");
        let result = derive_module_path(&file_path, &src_root).unwrap();
        assert_eq!(result, vec!["net", "http"]);
    }

    #[test]
    fn derive_module_path_index() {
        let src_root = PathBuf::from("src");
        let file_path = PathBuf::from("src/net/index.ish");
        let result = derive_module_path(&file_path, &src_root).unwrap();
        assert_eq!(result, vec!["net"]);
    }

    #[test]
    fn resolve_module_path_found_file() {
        let dir = std::env::temp_dir().join("ish_test_rmp_file");
        let _ = fs::remove_dir_all(&dir);
        let src = dir.join("src");
        fs::create_dir_all(src.join("net")).unwrap();
        fs::write(src.join("net/http.ish"), "").unwrap();
        let result = resolve_module_path(&["net".into(), "http".into()], &src).unwrap();
        assert_eq!(result, src.join("net/http.ish"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_module_path_found_index() {
        let dir = std::env::temp_dir().join("ish_test_rmp_index");
        let _ = fs::remove_dir_all(&dir);
        let src = dir.join("src");
        fs::create_dir_all(src.join("net")).unwrap();
        fs::write(src.join("net/index.ish"), "").unwrap();
        let result = resolve_module_path(&["net".into()], &src).unwrap();
        assert_eq!(result, src.join("net/index.ish"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_module_path_not_found() {
        let dir = std::env::temp_dir().join("ish_test_rmp_nf");
        let _ = fs::remove_dir_all(&dir);
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        let result = resolve_module_path(&["nonexistent".into()], &src);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("E016") || format!("{:?}", err).contains("ModuleNotFound"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_module_path_conflict() {
        let dir = std::env::temp_dir().join("ish_test_rmp_conflict");
        let _ = fs::remove_dir_all(&dir);
        let src = dir.join("src");
        fs::create_dir_all(src.join("foo")).unwrap();
        fs::write(src.join("foo.ish"), "").unwrap();
        fs::write(src.join("foo/index.ish"), "").unwrap();
        let result = resolve_module_path(&["foo".into()], &src);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("E019") || format!("{:?}", err).contains("ModulePathConflict"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_cycle_detected() {
        let stack = vec![vec!["net".to_string(), "http".to_string()]];
        assert!(check_cycle(&stack, &["net".to_string(), "http".to_string()]));
    }

    #[test]
    fn check_cycle_not_detected() {
        let stack = vec![vec!["net".to_string()]];
        assert!(!check_cycle(&stack, &["net".to_string(), "http".to_string()]));
    }
}
