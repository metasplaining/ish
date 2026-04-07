use std::path::{Path, PathBuf};
use ish_ast::Visibility;

/// Project context discovered at interpreter startup.
pub struct ProjectContext {
    /// The project root directory (contains `project.json`). `None` if using the installation default.
    pub project_root: Option<PathBuf>,
    /// The source root (`project_root/src/`). `None` if no project root.
    pub src_root: Option<PathBuf>,
}

/// Errors returned by access control checks.
#[derive(Debug, Clone)]
pub enum AccessError {
    Private { symbol: String },
    PackageOnly { symbol: String },
}

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessError::Private { symbol } => {
                write!(f, "Cannot access private symbol '{}'", symbol)
            }
            AccessError::PackageOnly { symbol } => {
                write!(f, "Cannot access package-private symbol '{}' from outside the project", symbol)
            }
        }
    }
}

/// Check whether a caller has access to a symbol with the given visibility.
///
/// - `Priv`: caller must be in the same file (same module). `None` caller always fails.
/// - `Pkg`: caller must be under the item's project root (containment check). `None` caller always fails.
/// - `Pub`: always allowed.
pub fn check_access(
    item_visibility: &Visibility,
    item_file_path: Option<&Path>,
    item_project_root: Option<&Path>,
    caller_file_path: Option<&Path>,
    _caller_project_root: Option<&Path>,
) -> Result<(), AccessError> {
    match item_visibility {
        Visibility::Pub => Ok(()),
        Visibility::Priv => {
            match (caller_file_path, item_file_path) {
                (Some(caller), Some(item)) if caller == item => Ok(()),
                _ => Err(AccessError::Private {
                    symbol: String::new(),
                }),
            }
        }
        Visibility::Pkg => {
            match (caller_file_path, item_project_root) {
                (Some(caller), Some(project_root)) => {
                    if is_project_member(caller, project_root) {
                        Ok(())
                    } else {
                        Err(AccessError::PackageOnly {
                            symbol: String::new(),
                        })
                    }
                }
                _ => Err(AccessError::PackageOnly {
                    symbol: String::new(),
                }),
            }
        }
    }
}

/// Returns `true` if `file_path` is physically located under `project_root`.
pub fn is_project_member(file_path: &Path, project_root: &Path) -> bool {
    file_path.starts_with(project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn priv_same_file() {
        let result = check_access(
            &Visibility::Priv,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn priv_same_project() {
        let result = check_access(
            &Visibility::Priv,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/b.ish")),
            Some(Path::new("/proj")),
        );
        assert!(matches!(result, Err(AccessError::Private { .. })));
    }

    #[test]
    fn priv_external() {
        let result = check_access(
            &Visibility::Priv,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/other/src/b.ish")),
            Some(Path::new("/other")),
        );
        assert!(matches!(result, Err(AccessError::Private { .. })));
    }

    #[test]
    fn pkg_same_file() {
        let result = check_access(
            &Visibility::Pkg,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pkg_same_project() {
        let result = check_access(
            &Visibility::Pkg,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/b.ish")),
            Some(Path::new("/proj")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pkg_external() {
        let result = check_access(
            &Visibility::Pkg,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/other/src/b.ish")),
            Some(Path::new("/other")),
        );
        assert!(matches!(result, Err(AccessError::PackageOnly { .. })));
    }

    #[test]
    fn pub_same_file() {
        let result = check_access(
            &Visibility::Pub,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pub_same_project() {
        let result = check_access(
            &Visibility::Pub,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/proj/src/b.ish")),
            Some(Path::new("/proj")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pub_external() {
        let result = check_access(
            &Visibility::Pub,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            Some(Path::new("/other/src/b.ish")),
            Some(Path::new("/other")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn pkg_inline_caller() {
        let result = check_access(
            &Visibility::Pkg,
            Some(Path::new("/proj/src/a.ish")),
            Some(Path::new("/proj")),
            None,
            None,
        );
        assert!(matches!(result, Err(AccessError::PackageOnly { .. })));
    }
}
