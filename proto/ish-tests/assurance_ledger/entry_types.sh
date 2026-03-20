#!/usr/bin/env bash
# ---
# feature: Assurance Ledger — Entry Types
# docs: docs/spec/assurance-ledger.md
# section: Entry Types
# ---
# Tests for entry type definition and built-in entry types.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Entry Types ---"

# Built-in entry types exist
output=$(run_ish 'println(has_entry_type("Error"))')
assert_output "built-in Error exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("CodedError"))')
assert_output "built-in CodedError exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("SystemError"))')
assert_output "built-in SystemError exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("Mutable"))')
assert_output "built-in Mutable exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("Type"))')
assert_output "built-in Type exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("Open"))')
assert_output "built-in Open exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("Closed"))')
assert_output "built-in Closed exists" "true" "$output"

# Non-existent entry type
output=$(run_ish 'println(has_entry_type("Nonexistent"))')
assert_output "nonexistent entry type returns false" "false" "$output"

# Custom entry type definition
output=$(run_ish '
entry type AuditTrail {
    severity: "high"
}
println(has_entry_type("AuditTrail"))
')
assert_output "custom entry type definition" "true" "$output"

# Multiple custom entry types
output=$(run_ish '
entry type Validated { level: "strict" }
entry type Cached { ttl: 60 }
println(has_entry_type("Validated"))
println(has_entry_type("Cached"))
')
assert_output "multiple custom entry types" $'true\ntrue' "$output"

finish
