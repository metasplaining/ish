*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §`ish-vm` — New modules.*

---

**New module: `proto/ish-vm/src/module_loader.rs`**

This module handles all filesystem and project-structure concerns. It has no global state; callers pass in a `ProjectContext`.

```
find_project_root(start_dir: &Path) -> Option<PathBuf>
    Walk up from start_dir, return the first directory containing project.json.
    Return None if the filesystem root is reached without finding one.

derive_module_path(file_path: &Path, src_root: &Path) -> Result<Vec<String>, ModuleError>
    Strip the src_root prefix and .ish extension from file_path.
    Apply the index.ish rule: if the filename is "index", use the parent directory name instead.
    Return the path segments as a Vec<String>.

resolve_module_path(module_path: &[String], src_root: &Path) -> Result<PathBuf, ModuleError>
    Given a module path (from a use statement), find the corresponding .ish file.
    Candidates: src_root/a/b/c.ish and src_root/a/b/c/index.ish.
    If both exist: return Err(ModuleError::PathConflict { ... }).
    If neither exists: return Err(ModuleError::NotFound { ... }).
    Files without .ish extension are never considered.

check_cycle(loading_stack: &[Vec<String>], candidate: &[String]) -> bool
    Return true if candidate already appears in the loading_stack.
```

---

**New module: `proto/ish-vm/src/access_control.rs`**

```
pub struct ProjectContext {
    pub project_root: Option<PathBuf>,  // None = installation default
    pub src_root: Option<PathBuf>,      // project_root/src/
}

check_access(
    item_visibility: Visibility,
    item_project_root: Option<&Path>,
    caller_file_path: Option<&Path>,    // None = inline/REPL input
    caller_project_root: Option<&Path>,
) -> Result<(), AccessError>
    Priv:  caller must be in the same module (same file path).
    Pkg:   caller must be under item_project_root (containment check).
    Pub:   always allowed.
    Returns AccessError::Private, AccessError::PackageOnly as appropriate.

is_project_member(file_path: &Path, project_root: &Path) -> bool
    Returns true if file_path starts with project_root.
```
