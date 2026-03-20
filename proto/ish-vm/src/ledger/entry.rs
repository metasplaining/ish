// ish-vm/src/ledger/entry.rs — Entries on values.

use std::collections::HashMap;

/// An entry recorded on a value in the assurance ledger.
///
/// Entries are facts about items (variables, properties, functions, types).
/// For example, a `Type` entry records a type annotation; a `Mutable` entry
/// records that a variable is mutable; an `Error` entry records that a value
/// is an error.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The entry type name (e.g., "Type", "Mutable", "Error").
    pub entry_type: String,
    /// Parameters for this entry (e.g., for a Type entry: {"type": "i32"}).
    pub params: HashMap<String, String>,
}

impl Entry {
    pub fn new(entry_type: impl Into<String>) -> Self {
        Self {
            entry_type: entry_type.into(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// A collection of entries attached to a single item.
#[derive(Clone, Debug, Default)]
pub struct EntrySet {
    pub entries: Vec<Entry>,
}

impl EntrySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Check whether this set contains an entry of the given type.
    pub fn has(&self, entry_type: &str) -> bool {
        self.entries.iter().any(|e| e.entry_type == entry_type)
    }

    /// Get the first entry of the given type, if any.
    pub fn get(&self, entry_type: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.entry_type == entry_type)
    }

    /// Remove all entries of the given type.
    pub fn remove(&mut self, entry_type: &str) {
        self.entries.retain(|e| e.entry_type != entry_type);
    }
}
