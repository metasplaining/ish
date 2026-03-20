// ish-vm/src/ledger/standard.rs — Standards and feature states.

use std::collections::HashMap;

/// Annotation dimension: controls whether annotations are required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationDimension {
    /// The annotation is not required; if present, it is checked.
    Optional,
    /// The annotation is required; absence produces a discrepancy.
    Required,
}

/// Audit dimension: controls when checking occurs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditDimension {
    /// Checked during runtime audit (execution time).
    Runtime,
    /// Checked during build audit (declaration time / compile time).
    Build,
}

/// A feature state within a standard, combining annotation and audit dimensions
/// plus an optional feature-specific parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureState {
    pub annotation: AnnotationDimension,
    pub audit: AuditDimension,
    /// Feature-specific parameter (e.g., "wrapping" for overflow, "deny" for
    /// implicit_conversions). `None` for features that use only the two
    /// standard dimensions.
    pub parameter: Option<String>,
}

impl FeatureState {
    pub fn new(annotation: AnnotationDimension, audit: AuditDimension) -> Self {
        Self { annotation, audit, parameter: None }
    }

    pub fn with_parameter(mut self, param: impl Into<String>) -> Self {
        self.parameter = Some(param.into());
        self
    }
}

/// A named standard that sets feature states within a scope.
#[derive(Clone, Debug)]
pub struct Standard {
    pub name: String,
    /// Name of the parent standard (for inheritance via `extends`).
    pub parent: Option<String>,
    /// Feature states explicitly set by this standard (not including inherited).
    pub features: HashMap<String, FeatureState>,
}

impl Standard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            features: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_feature(mut self, name: impl Into<String>, state: FeatureState) -> Self {
        self.features.insert(name.into(), state);
        self
    }
}

/// Registry of named standards with inheritance resolution.
#[derive(Clone, Debug, Default)]
pub struct StandardRegistry {
    standards: HashMap<String, Standard>,
}

impl StandardRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a standard. Overwrites any existing standard with the same name.
    pub fn register(&mut self, standard: Standard) {
        self.standards.insert(standard.name.clone(), standard);
    }

    /// Look up a standard by name (direct definition, no inheritance resolution).
    pub fn get(&self, name: &str) -> Option<&Standard> {
        self.standards.get(name)
    }

    /// Resolve the full feature map for a standard, including inherited features.
    /// Returns `None` if the standard (or any ancestor) is not registered.
    pub fn resolve(&self, name: &str) -> Option<HashMap<String, FeatureState>> {
        let standard = self.standards.get(name)?;
        let mut resolved = if let Some(ref parent_name) = standard.parent {
            self.resolve(parent_name)?
        } else {
            HashMap::new()
        };
        // Child features override parent features.
        for (feat, state) in &standard.features {
            resolved.insert(feat.clone(), state.clone());
        }
        Some(resolved)
    }

    /// Register the three built-in standards: streamlined, cautious, rigorous.
    pub fn register_builtins(&mut self) {
        // streamlined: no features required (empty).
        self.register(Standard::new("streamlined"));

        // cautious: types, null_safety, immutability required at runtime.
        let required_runtime = FeatureState::new(
            AnnotationDimension::Required,
            AuditDimension::Runtime,
        );
        self.register(
            Standard::new("cautious")
                .with_feature("types", required_runtime.clone())
                .with_feature("null_safety", required_runtime.clone())
                .with_feature("immutability", required_runtime),
        );

        // rigorous: extends cautious, all features required at build time.
        let required_build = FeatureState::new(
            AnnotationDimension::Required,
            AuditDimension::Build,
        );
        self.register(
            Standard::new("rigorous")
                .with_parent("cautious")
                .with_feature("types", required_build.clone())
                .with_feature("null_safety", required_build.clone())
                .with_feature("immutability", required_build.clone())
                .with_feature("overflow", required_build.clone().with_parameter("panicking"))
                .with_feature("numeric_precision", required_build.clone())
                .with_feature("implicit_conversions", required_build.clone().with_parameter("deny"))
                .with_feature("undeclared_errors", FeatureState::new(
                    AnnotationDimension::Required,
                    AuditDimension::Build,
                ).with_parameter("none"))
                .with_feature("exhaustiveness", required_build.clone())
                .with_feature("unused_variables", required_build.clone())
                .with_feature("unreachable_statements", required_build.clone())
                .with_feature("memory_model", required_build.clone().with_parameter("auto"))
                .with_feature("polymorphism_strategy", required_build.clone().with_parameter("auto"))
                .with_feature("open_closed_objects", required_build.clone())
                .with_feature("visibility", required_build.clone())
                .with_feature("sync_async", required_build.clone())
                .with_feature("blocking", required_build.clone().with_parameter("deny"))
                .with_feature("pure_functions", required_build),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = StandardRegistry::new();
        reg.register(Standard::new("test_std").with_feature(
            "types",
            FeatureState::new(AnnotationDimension::Required, AuditDimension::Runtime),
        ));
        assert!(reg.get("test_std").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn resolve_no_inheritance() {
        let mut reg = StandardRegistry::new();
        reg.register(Standard::new("standalone").with_feature(
            "types",
            FeatureState::new(AnnotationDimension::Required, AuditDimension::Runtime),
        ));
        let features = reg.resolve("standalone").unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(features["types"].annotation, AnnotationDimension::Required);
        assert_eq!(features["types"].audit, AuditDimension::Runtime);
    }

    #[test]
    fn resolve_with_inheritance() {
        let mut reg = StandardRegistry::new();
        reg.register(Standard::new("parent").with_feature(
            "types",
            FeatureState::new(AnnotationDimension::Optional, AuditDimension::Runtime),
        ).with_feature(
            "null_safety",
            FeatureState::new(AnnotationDimension::Optional, AuditDimension::Runtime),
        ));
        reg.register(Standard::new("child").with_parent("parent").with_feature(
            "types",
            FeatureState::new(AnnotationDimension::Required, AuditDimension::Build),
        ));
        let features = reg.resolve("child").unwrap();
        assert_eq!(features.len(), 2);
        // Child overrides parent for types.
        assert_eq!(features["types"].annotation, AnnotationDimension::Required);
        assert_eq!(features["types"].audit, AuditDimension::Build);
        // null_safety inherited from parent.
        assert_eq!(features["null_safety"].annotation, AnnotationDimension::Optional);
    }

    #[test]
    fn builtin_streamlined_is_empty() {
        let mut reg = StandardRegistry::new();
        reg.register_builtins();
        let features = reg.resolve("streamlined").unwrap();
        assert!(features.is_empty());
    }

    #[test]
    fn builtin_cautious_has_three_features() {
        let mut reg = StandardRegistry::new();
        reg.register_builtins();
        let features = reg.resolve("cautious").unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.contains_key("types"));
        assert!(features.contains_key("null_safety"));
        assert!(features.contains_key("immutability"));
        for state in features.values() {
            assert_eq!(state.annotation, AnnotationDimension::Required);
            assert_eq!(state.audit, AuditDimension::Runtime);
        }
    }

    #[test]
    fn builtin_rigorous_extends_cautious() {
        let mut reg = StandardRegistry::new();
        reg.register_builtins();
        let features = reg.resolve("rigorous").unwrap();
        // rigorous inherits cautious's 3 features and adds many more.
        assert!(features.len() > 3);
        // All features should be build-time.
        for state in features.values() {
            assert_eq!(state.audit, AuditDimension::Build);
        }
    }
}
