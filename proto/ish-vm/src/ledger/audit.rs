// ish-vm/src/ledger/audit.rs — Stateless audit logic.

use std::collections::HashMap;
use super::standard::FeatureState;
use super::entry::EntrySet;

/// The result of auditing a statement.
#[derive(Clone, Debug, PartialEq)]
pub enum AuditResult {
    /// The statement passes audit with no issues.
    Pass,
    /// The audit auto-fixed the entries (e.g., added an inferred entry).
    AutoFix(Vec<AuditAction>),
    /// The audit detected a discrepancy.
    Discrepancy(DiscrepancyReport),
}

/// An auto-fix action produced by the audit.
#[derive(Clone, Debug, PartialEq)]
pub enum AuditAction {
    /// Add an entry to a value.
    AddEntry {
        target: String,
        entry_type: String,
        params: HashMap<String, String>,
    },
}

/// A discrepancy detected by the audit.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscrepancyReport {
    pub message: String,
    pub feature: Option<String>,
    pub trail: Vec<String>,
}

impl DiscrepancyReport {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            feature: None,
            trail: Vec::new(),
        }
    }

    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature = Some(feature.into());
        self
    }

    pub fn with_trail_entry(mut self, entry: impl Into<String>) -> Self {
        self.trail.push(entry.into());
        self
    }
}

/// Audit a statement against active feature states and current entries.
///
/// This is the core stateless audit function. Given:
/// - `active_features`: the resolved feature map from the active standard
/// - `entries`: the current entries on the item being audited
/// - `statement_kind`: what kind of statement this is (e.g., "assignment", "call", "return")
/// - `has_annotation`: whether the statement has an explicit type annotation
///
/// Returns an `AuditResult`.
pub fn audit_statement(
    active_features: &HashMap<String, FeatureState>,
    entries: &EntrySet,
    statement_kind: &str,
    has_annotation: bool,
) -> AuditResult {
    // Check type annotation requirement.
    if let Some(types_state) = active_features.get("types") {
        if types_state.annotation == super::standard::AnnotationDimension::Required
            && !has_annotation
            && matches!(statement_kind, "assignment" | "parameter" | "return")
        {
            return AuditResult::Discrepancy(
                DiscrepancyReport::new(format!(
                    "Missing type annotation on {statement_kind}"
                ))
                .with_feature("types")
                .with_trail_entry(format!(
                    "Active standard requires types(required, {:?})",
                    types_state.audit
                )),
            );
        }
    }

    // Check mutability annotation requirement.
    if let Some(immutability_state) = active_features.get("immutability") {
        if immutability_state.annotation == super::standard::AnnotationDimension::Required
            && !entries.has("Mutable")
            && statement_kind == "assignment"
        {
            // Mutable variables should have a Mutable entry. Immutable is the default,
            // so we only flag this when the standard requires explicit annotation and
            // the variable is being reassigned without a Mutable entry.
            // For now, this is a TODO — reassignment checking requires tracking
            // whether this is a first binding or a reassignment.
        }
    }

    AuditResult::Pass
}

/// Audit whether a function that performs async operations is declared `async`.
///
/// When the `async_annotation` feature is `required`, a function that contains
/// `await` or `yield` must be declared with `async fn`.
pub fn audit_async_annotation(
    active_features: &HashMap<String, FeatureState>,
    is_async: bool,
    fn_name: &str,
) -> AuditResult {
    if let Some(feature) = active_features.get("async_annotation") {
        if feature.annotation == super::standard::AnnotationDimension::Required && !is_async {
            return AuditResult::Discrepancy(
                DiscrepancyReport::new(format!(
                    "Function '{}' performs async operations but is not declared 'async fn'",
                    fn_name
                ))
                .with_feature("async_annotation")
                .with_trail_entry("Active standard requires async_annotation"),
            );
        }
    }
    AuditResult::Pass
}

/// Audit whether an async function call is properly awaited.
///
/// When the `await_required` feature is `required`, calling an async function
/// without `await` is a discrepancy.
pub fn audit_await_required(
    active_features: &HashMap<String, FeatureState>,
    call_expr: &str,
) -> AuditResult {
    if let Some(feature) = active_features.get("await_required") {
        if feature.annotation == super::standard::AnnotationDimension::Required {
            return AuditResult::Discrepancy(
                DiscrepancyReport::new(format!(
                    "Async function call '{}' is not awaited (required by active standard)",
                    call_expr
                ))
                .with_feature("await_required")
                .with_trail_entry("Active standard requires await_required"),
            );
        }
    }
    AuditResult::Pass
}

/// Audit whether dropping a future without awaiting is allowed.
///
/// When the `future_drop` feature is `required`, dropping a `Value::Future`
/// without awaiting it triggers a discrepancy.
pub fn audit_future_drop(
    active_features: &HashMap<String, FeatureState>,
    context: &str,
) -> AuditResult {
    if let Some(feature) = active_features.get("future_drop") {
        if feature.annotation == super::standard::AnnotationDimension::Required {
            return AuditResult::Discrepancy(
                DiscrepancyReport::new(format!(
                    "Future dropped without being awaited: {}",
                    context
                ))
                .with_feature("future_drop")
                .with_trail_entry("Active standard requires future_drop"),
            );
        }
    }
    AuditResult::Pass
}

/// Audit whether a complex function or block guarantees cooperative yielding.
///
/// When the `guaranteed_yield` feature is `required`, a function/block that is
/// both `complex` and `unyielding` is a discrepancy.
pub fn audit_guaranteed_yield(
    active_features: &HashMap<String, FeatureState>,
    entries: &EntrySet,
    item_name: &str,
) -> AuditResult {
    if let Some(feature) = active_features.get("guaranteed_yield") {
        if feature.annotation == super::standard::AnnotationDimension::Required {
            let is_complex = entries.get("Complexity")
                .and_then(|e| e.params.get("value"))
                .map_or(false, |v| v == "complex");
            let is_unyielding = entries.get("Yielding")
                .and_then(|e| e.params.get("value"))
                .map_or(false, |v| v == "unyielding");

            if is_complex && is_unyielding {
                return AuditResult::Discrepancy(
                    DiscrepancyReport::new(format!(
                        "'{}' is complex and unyielding — guaranteed yield required",
                        item_name
                    ))
                    .with_feature("guaranteed_yield")
                    .with_trail_entry("Active standard requires guaranteed_yield for complex code"),
                );
            }
        }
    }
    AuditResult::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::standard::{AnnotationDimension, AuditDimension};

    fn make_features(
        types_annotation: AnnotationDimension,
        types_audit: AuditDimension,
    ) -> HashMap<String, FeatureState> {
        let mut features = HashMap::new();
        features.insert(
            "types".to_string(),
            FeatureState { annotation: types_annotation, audit: types_audit, parameter: None },
        );
        features
    }

    #[test]
    fn pass_when_no_features() {
        let features = HashMap::new();
        let entries = EntrySet::new();
        let result = audit_statement(&features, &entries, "assignment", false);
        assert_eq!(result, AuditResult::Pass);
    }

    #[test]
    fn pass_when_types_optional() {
        let features = make_features(AnnotationDimension::Optional, AuditDimension::Runtime);
        let entries = EntrySet::new();
        let result = audit_statement(&features, &entries, "assignment", false);
        assert_eq!(result, AuditResult::Pass);
    }

    #[test]
    fn pass_when_annotation_present() {
        let features = make_features(AnnotationDimension::Required, AuditDimension::Runtime);
        let entries = EntrySet::new();
        let result = audit_statement(&features, &entries, "assignment", true);
        assert_eq!(result, AuditResult::Pass);
    }

    #[test]
    fn discrepancy_when_annotation_required_and_missing() {
        let features = make_features(AnnotationDimension::Required, AuditDimension::Runtime);
        let entries = EntrySet::new();
        let result = audit_statement(&features, &entries, "assignment", false);
        match result {
            AuditResult::Discrepancy(report) => {
                assert!(report.message.contains("Missing type annotation"));
                assert_eq!(report.feature, Some("types".to_string()));
            }
            _ => panic!("expected discrepancy, got {:?}", result),
        }
    }

    #[test]
    fn pass_for_non_annotation_statement_kinds() {
        let features = make_features(AnnotationDimension::Required, AuditDimension::Runtime);
        let entries = EntrySet::new();
        // "call" is not an annotation-required statement kind
        let result = audit_statement(&features, &entries, "call", false);
        assert_eq!(result, AuditResult::Pass);
    }
}
