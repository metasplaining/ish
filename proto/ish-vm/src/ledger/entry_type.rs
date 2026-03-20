// ish-vm/src/ledger/entry_type.rs — Entry type definitions and registry.

use std::collections::HashMap;

/// A required property in an entry type definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredProperty {
    pub name: String,
    /// The expected type name (e.g., "String", "i32"). Simple name match for now.
    pub type_name: String,
}

/// An entry type definition.
///
/// Entry types define the kinds of entries that can be recorded in the ledger.
/// They form a hierarchy via `parent` (e.g., CodedError extends Error).
#[derive(Clone, Debug)]
pub struct EntryType {
    pub name: String,
    /// Parent entry type name (for inheritance).
    pub parent: Option<String>,
    /// Properties required for a value to qualify as this entry type.
    pub required_properties: Vec<RequiredProperty>,
}

impl EntryType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            required_properties: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_required(mut self, prop_name: impl Into<String>, type_name: impl Into<String>) -> Self {
        self.required_properties.push(RequiredProperty {
            name: prop_name.into(),
            type_name: type_name.into(),
        });
        self
    }
}

/// Registry of entry types with inheritance resolution.
#[derive(Clone, Debug, Default)]
pub struct EntryTypeRegistry {
    types: HashMap<String, EntryType>,
}

impl EntryTypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an entry type. Overwrites any existing type with the same name.
    pub fn register(&mut self, entry_type: EntryType) {
        self.types.insert(entry_type.name.clone(), entry_type);
    }

    /// Look up an entry type by name.
    pub fn get(&self, name: &str) -> Option<&EntryType> {
        self.types.get(name)
    }

    /// Resolve the full set of required properties for an entry type,
    /// including properties inherited from parent types.
    pub fn resolve_required_properties(&self, name: &str) -> Option<Vec<RequiredProperty>> {
        let entry_type = self.types.get(name)?;
        let mut props = if let Some(ref parent_name) = entry_type.parent {
            self.resolve_required_properties(parent_name)?
        } else {
            Vec::new()
        };
        // Child properties are appended (they add requirements beyond the parent).
        for prop in &entry_type.required_properties {
            if !props.iter().any(|p| p.name == prop.name) {
                props.push(prop.clone());
            }
        }
        Some(props)
    }

    /// Check whether a given entry type is a descendant of (or equal to) another.
    pub fn is_subtype(&self, child: &str, ancestor: &str) -> bool {
        if child == ancestor {
            return true;
        }
        if let Some(entry_type) = self.types.get(child) {
            if let Some(ref parent) = entry_type.parent {
                return self.is_subtype(parent, ancestor);
            }
        }
        false
    }

    /// Register built-in entry types: Error, CodedError, SystemError, Mutable, Type, Open, Closed.
    pub fn register_builtins(&mut self) {
        // Error — requires message: String
        self.register(
            EntryType::new("Error")
                .with_required("message", "String")
        );

        // CodedError extends Error — additionally requires code: String
        self.register(
            EntryType::new("CodedError")
                .with_parent("Error")
                .with_required("code", "String")
        );

        // SystemError extends CodedError
        self.register(
            EntryType::new("SystemError")
                .with_parent("CodedError")
        );

        // Domain subtypes of SystemError
        self.register(
            EntryType::new("TypeError")
                .with_parent("SystemError")
        );
        self.register(
            EntryType::new("ArgumentError")
                .with_parent("SystemError")
        );
        self.register(
            EntryType::new("FileError")
                .with_parent("SystemError")
        );
        self.register(
            EntryType::new("FileNotFoundError")
                .with_parent("FileError")
        );
        self.register(
            EntryType::new("PermissionError")
                .with_parent("FileError")
        );

        // Mutable — no required properties (marker entry)
        self.register(EntryType::new("Mutable"));

        // Type — no required properties (structural type entry)
        self.register(EntryType::new("Type"));

        // Open — marks a type as open to extra properties
        self.register(EntryType::new("Open"));

        // Closed — marks a type as closed to extra properties
        self.register(EntryType::new("Closed"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = EntryTypeRegistry::new();
        reg.register(EntryType::new("Custom"));
        assert!(reg.get("Custom").is_some());
        assert!(reg.get("Missing").is_none());
    }

    #[test]
    fn resolve_required_properties_no_parent() {
        let mut reg = EntryTypeRegistry::new();
        reg.register(EntryType::new("Marker"));
        let props = reg.resolve_required_properties("Marker").unwrap();
        assert!(props.is_empty());
    }

    #[test]
    fn resolve_required_properties_with_inheritance() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        // CodedError extends Error, so it requires both message (from Error) and code.
        let props = reg.resolve_required_properties("CodedError").unwrap();
        assert!(props.iter().any(|p| p.name == "message"));
        assert!(props.iter().any(|p| p.name == "code"));
    }

    #[test]
    fn resolve_system_error_inherits_coded_error() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        let props = reg.resolve_required_properties("SystemError").unwrap();
        // SystemError extends CodedError, so it inherits message and code.
        assert!(props.iter().any(|p| p.name == "message"));
        assert!(props.iter().any(|p| p.name == "code"));
    }

    #[test]
    fn is_subtype_positive() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        assert!(reg.is_subtype("SystemError", "CodedError"));
        assert!(reg.is_subtype("SystemError", "Error"));
        assert!(reg.is_subtype("CodedError", "Error"));
        assert!(reg.is_subtype("Error", "Error")); // reflexive
    }

    #[test]
    fn domain_subtype_hierarchy() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        // TypeError -> SystemError -> CodedError -> Error
        assert!(reg.is_subtype("TypeError", "SystemError"));
        assert!(reg.is_subtype("TypeError", "Error"));
        // ArgumentError -> SystemError -> Error
        assert!(reg.is_subtype("ArgumentError", "SystemError"));
        assert!(reg.is_subtype("ArgumentError", "Error"));
        // FileNotFoundError -> FileError -> SystemError -> Error
        assert!(reg.is_subtype("FileNotFoundError", "FileError"));
        assert!(reg.is_subtype("FileNotFoundError", "SystemError"));
        assert!(reg.is_subtype("FileNotFoundError", "Error"));
        // PermissionError -> FileError -> SystemError
        assert!(reg.is_subtype("PermissionError", "FileError"));
        assert!(reg.is_subtype("PermissionError", "SystemError"));
        // Not subtypes of each other
        assert!(!reg.is_subtype("TypeError", "ArgumentError"));
        assert!(!reg.is_subtype("FileError", "TypeError"));
    }

    #[test]
    fn is_subtype_negative() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        assert!(!reg.is_subtype("Error", "CodedError"));
        assert!(!reg.is_subtype("Mutable", "Error"));
    }

    #[test]
    fn builtins_registered() {
        let mut reg = EntryTypeRegistry::new();
        reg.register_builtins();
        for name in &[
            "Error", "CodedError", "SystemError",
            "TypeError", "ArgumentError", "FileError", "FileNotFoundError", "PermissionError",
            "Mutable", "Type", "Open", "Closed",
        ] {
            assert!(reg.get(name).is_some(), "missing builtin entry type: {}", name);
        }
    }
}
