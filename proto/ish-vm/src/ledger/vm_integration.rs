// ish-vm/src/ledger/vm_integration.rs — Wires the ledger engine into the VM.
//
// Provides:
// - LedgerState: standard scope stack, entry store, registries
// - Methods for pushing/popping standards, tracking entries, querying features

use std::collections::HashMap;

use super::standard::{StandardRegistry, FeatureState};
use super::entry_type::EntryTypeRegistry;
use super::entry::EntrySet;

/// The runtime ledger state maintained by the interpreter.
///
/// This struct owns the standard scope stack, the entry store (entries on named
/// items), and the built-in registries. It is stored on the `IshVm` struct,
/// not on the GC-managed environment.
#[derive(Clone, Debug)]
pub struct LedgerState {
    pub standard_registry: StandardRegistry,
    pub entry_type_registry: EntryTypeRegistry,
    /// Stack of active standard names. The last entry is the innermost scope.
    standard_stack: Vec<String>,
    /// Entries attached to named items (variables, functions, etc.).
    entries: HashMap<String, EntrySet>,
}

impl LedgerState {
    /// Create a new ledger state with built-in standards and entry types.
    pub fn new() -> Self {
        let mut standard_registry = StandardRegistry::new();
        standard_registry.register_builtins();

        let mut entry_type_registry = EntryTypeRegistry::new();
        entry_type_registry.register_builtins();

        Self {
            standard_registry,
            entry_type_registry,
            standard_stack: Vec::new(),
            entries: HashMap::new(),
        }
    }

    // ── Standard scope stack ────────────────────────────────────────────

    /// Push a standard onto the scope stack.
    pub fn push_standard(&mut self, name: String) {
        self.standard_stack.push(name);
    }

    /// Pop the most recent standard from the scope stack.
    pub fn pop_standard(&mut self) -> Option<String> {
        self.standard_stack.pop()
    }

    /// Get the name of the currently active standard (innermost scope).
    pub fn active_standard(&self) -> Option<&str> {
        self.standard_stack.last().map(|s| s.as_str())
    }

    /// Resolve the active feature states by merging the entire standard stack.
    /// Later standards override earlier ones.
    pub fn active_features(&self) -> HashMap<String, FeatureState> {
        let mut merged = HashMap::new();
        for standard_name in &self.standard_stack {
            if let Some(features) = self.standard_registry.resolve(standard_name) {
                merged.extend(features);
            }
        }
        merged
    }

    /// Query a specific feature state from the active standard.
    pub fn feature_state(&self, feature: &str) -> Option<FeatureState> {
        self.active_features().get(feature).cloned()
    }

    // ── Entry store ─────────────────────────────────────────────────────

    /// Get the entry set for a named item.
    pub fn get_entries(&self, item: &str) -> Option<&EntrySet> {
        self.entries.get(item)
    }

    /// Get or create the entry set for a named item.
    pub fn get_or_create_entries(&mut self, item: &str) -> &mut EntrySet {
        self.entries.entry(item.to_string()).or_default()
    }

    /// Add an entry to a named item.
    pub fn add_entry(&mut self, item: &str, entry: super::entry::Entry) {
        self.get_or_create_entries(item).add(entry);
    }

    /// Check whether a named item has an entry of the given type.
    pub fn has_entry(&self, item: &str, entry_type: &str) -> bool {
        self.entries
            .get(item)
            .map_or(false, |es| es.has(entry_type))
    }

    // ── Narrowing (entry snapshot save/restore/merge) ───────────────────

    /// Save the current entries as a snapshot for later restoration.
    pub fn save_entries(&self) -> HashMap<String, EntrySet> {
        self.entries.clone()
    }

    /// Restore entries from a snapshot.
    pub fn restore_entries(&mut self, snapshot: HashMap<String, EntrySet>) {
        self.entries = snapshot;
    }

    /// Set a narrowed type entry for a variable.
    /// Replaces any existing "Type" entry with one recording the narrowed type.
    pub fn narrow_type(&mut self, item: &str, type_name: &str) {
        let es = self.get_or_create_entries(item);
        es.remove("Type");
        es.add(super::entry::Entry::new("Type").with_param("type", type_name));
    }

    /// Remove null from a variable's type entry (null narrowing).
    /// Records an "ExcludeNull" marker entry.
    pub fn narrow_exclude_null(&mut self, item: &str) {
        let es = self.get_or_create_entries(item);
        if !es.has("ExcludeNull") {
            es.add(super::entry::Entry::new("ExcludeNull"));
        }
    }

    /// Merge two entry snapshots: for each item, entries from either branch are kept.
    /// This implements the "union of entries from both branches" merge strategy.
    pub fn merge_entries(
        &mut self,
        then_entries: HashMap<String, EntrySet>,
        else_entries: HashMap<String, EntrySet>,
    ) {
        let mut merged = HashMap::new();
        // Collect all item names from both branches.
        let all_keys: std::collections::HashSet<&String> =
            then_entries.keys().chain(else_entries.keys()).collect();
        for key in all_keys {
            let then_set = then_entries.get(key.as_str());
            let else_set = else_entries.get(key.as_str());
            match (then_set, else_set) {
                (Some(ts), Some(es)) => {
                    // Both branches have entries — merge (union).
                    let mut combined = ts.clone();
                    for entry in &es.entries {
                        if !combined.has(&entry.entry_type) {
                            combined.add(entry.clone());
                        }
                    }
                    merged.insert(key.clone(), combined);
                }
                (Some(ts), None) => {
                    merged.insert(key.clone(), ts.clone());
                }
                (None, Some(es)) => {
                    merged.insert(key.clone(), es.clone());
                }
                (None, None) => unreachable!(),
            }
        }
        self.entries = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::entry::Entry;
    use crate::ledger::standard::AnnotationDimension;

    #[test]
    fn new_has_builtin_standards() {
        let state = LedgerState::new();
        assert!(state.standard_registry.get("streamlined").is_some());
        assert!(state.standard_registry.get("cautious").is_some());
        assert!(state.standard_registry.get("rigorous").is_some());
    }

    #[test]
    fn new_has_builtin_entry_types() {
        let state = LedgerState::new();
        for name in &["Error", "Mutable", "Type", "Open", "Closed"] {
            assert!(state.entry_type_registry.get(name).is_some(), "missing: {}", name);
        }
        // Structural error types are NOT entry types
        for name in &["CodedError", "SystemError"] {
            assert!(state.entry_type_registry.get(name).is_none(), "should not be registered: {}", name);
        }
    }

    #[test]
    fn standard_scope_stack() {
        let mut state = LedgerState::new();
        assert!(state.active_standard().is_none());

        state.push_standard("cautious".to_string());
        assert_eq!(state.active_standard(), Some("cautious"));

        state.push_standard("rigorous".to_string());
        assert_eq!(state.active_standard(), Some("rigorous"));

        state.pop_standard();
        assert_eq!(state.active_standard(), Some("cautious"));

        state.pop_standard();
        assert!(state.active_standard().is_none());
    }

    #[test]
    fn active_features_merge() {
        let mut state = LedgerState::new();
        state.push_standard("cautious".to_string());
        let features = state.active_features();
        assert_eq!(features.len(), 3);
        assert!(features.contains_key("types"));

        // Push rigorous on top — its features override cautious.
        state.push_standard("rigorous".to_string());
        let features = state.active_features();
        assert!(features.len() > 3);
        // types should now be build-time.
        assert_eq!(
            features["types"].audit,
            crate::ledger::standard::AuditDimension::Build
        );
    }

    #[test]
    fn entry_store() {
        let mut state = LedgerState::new();
        assert!(!state.has_entry("x", "Mutable"));

        state.add_entry("x", Entry::new("Mutable"));
        assert!(state.has_entry("x", "Mutable"));
        assert!(!state.has_entry("x", "Type"));
    }

    #[test]
    fn feature_state_query() {
        let mut state = LedgerState::new();
        assert!(state.feature_state("types").is_none());

        state.push_standard("cautious".to_string());
        let fs = state.feature_state("types").unwrap();
        assert_eq!(fs.annotation, AnnotationDimension::Required);
    }

    #[test]
    fn save_and_restore_entries() {
        let mut state = LedgerState::new();
        state.add_entry("x", Entry::new("Type").with_param("type", "i32"));
        let snapshot = state.save_entries();

        // Modify entries.
        state.narrow_type("x", "String");
        assert!(state.has_entry("x", "Type"));

        // Restore snapshot — x should have original Type entry.
        state.restore_entries(snapshot);
        let entries = state.get_entries("x").unwrap();
        let type_entry = entries.get("Type").unwrap();
        assert_eq!(type_entry.params.get("type").map(|s| s.as_str()), Some("i32"));
    }

    #[test]
    fn narrow_type_replaces_entry() {
        let mut state = LedgerState::new();
        state.add_entry("x", Entry::new("Type").with_param("type", "i32 | null"));
        state.narrow_type("x", "i32");
        let entries = state.get_entries("x").unwrap();
        let type_entry = entries.get("Type").unwrap();
        assert_eq!(type_entry.params.get("type").map(|s| s.as_str()), Some("i32"));
    }

    #[test]
    fn narrow_exclude_null_adds_marker() {
        let mut state = LedgerState::new();
        state.add_entry("x", Entry::new("Type").with_param("type", "i32 | null"));
        assert!(!state.has_entry("x", "ExcludeNull"));
        state.narrow_exclude_null("x");
        assert!(state.has_entry("x", "ExcludeNull"));
    }

    #[test]
    fn merge_entries_union() {
        let mut state = LedgerState::new();
        state.add_entry("x", Entry::new("Type").with_param("type", "i32"));

        // Simulate then-branch: also has Mutable.
        let mut then_state = LedgerState::new();
        then_state.add_entry("x", Entry::new("Type").with_param("type", "i32"));
        then_state.add_entry("x", Entry::new("Mutable"));
        let then_entries = then_state.save_entries();

        // Simulate else-branch: same Type, no Mutable, but has "ExcludeNull".
        let mut else_state = LedgerState::new();
        else_state.add_entry("x", Entry::new("Type").with_param("type", "i32"));
        else_state.add_entry("x", Entry::new("ExcludeNull"));
        let else_entries = else_state.save_entries();

        // Merge: should have union of entries from both branches.
        state.merge_entries(then_entries, else_entries);
        assert!(state.has_entry("x", "Type"));
        assert!(state.has_entry("x", "Mutable"));
        assert!(state.has_entry("x", "ExcludeNull"));
    }
}
