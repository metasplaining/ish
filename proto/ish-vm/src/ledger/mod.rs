// ish-vm/src/ledger/mod.rs — Assurance ledger engine.
//
// The ledger engine manages standards, entry types, and audit logic.
// It provides the behavioral core; the VM module wires it into the interpreter.

pub mod entry_type;
pub mod standard;
pub mod audit;
pub mod entry;
pub mod vm_integration;
pub mod type_compat;
pub mod narrowing;

pub use entry_type::{EntryType, EntryTypeRegistry};
pub use standard::{Standard, StandardRegistry, FeatureState, AnnotationDimension, AuditDimension};
pub use audit::{audit_statement, AuditResult, AuditAction, DiscrepancyReport};
pub use entry::Entry;
pub use vm_integration::LedgerState;
pub use type_compat::{is_compatible, types_compatible};
