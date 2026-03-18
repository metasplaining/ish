#!/usr/bin/env bash
# ---
# feature: Data Structures
# docs: docs/spec/syntax.md
# section: Data Structures
# ---
# Tests object literals, list literals, property access, index access,
# and mutation via builtins.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Data Structures ---"

# List literal
output=$(run_ish 'println([1, 2, 3])')
assert_output "list literal" "[1, 2, 3]" "$output"

# Empty list
output=$(run_ish 'println([])')
assert_output "empty list" "[]" "$output"

# List index access
output=$(run_ish 'let l = [10, 20, 30]; println(l[0]); println(l[1]); println(l[2])')
assert_output "list index access" $'10\n20\n30' "$output"

# List push and length
output=$(run_ish 'let l = [1, 2]; list_push(l, 3); println(l); println(list_length(l))')
assert_output "list push and length" $'[1, 2, 3]\n3' "$output"

# List pop
output=$(run_ish 'let l = [1, 2, 3]; let v = list_pop(l); println(v); println(l)')
assert_output "list pop" $'3\n[1, 2]' "$output"

# List slice
output=$(run_ish 'println(list_slice([1, 2, 3, 4, 5], 1, 3))')
assert_output "list slice" "[2, 3]" "$output"

# List join
output=$(run_ish 'println(list_join(["a", "b", "c"], ", "))')
assert_output "list join" "a, b, c" "$output"

# List index assignment
output=$(run_ish 'let l = [1, 2, 3]; l[0] = 99; println(l)')
assert_output "list index assignment" "[99, 2, 3]" "$output"

# Object literal
output=$(run_ish 'let o = {name: "Alice", age: 30}; println(o.name); println(o.age)')
assert_output "object literal and access" $'Alice\n30' "$output"

# Object property assignment
output=$(run_ish 'let o = {x: 1}; o.x = 42; println(o.x)')
assert_output "object property assignment" "42" "$output"

# Object builtins
output=$(run_ish 'let o = {a: 1, b: 2}; println(obj_has(o, "a")); println(obj_has(o, "c"))')
assert_output "obj_has" $'true\nfalse' "$output"

output=$(run_ish 'let o = {a: 1, b: 2}; let keys = obj_keys(o); println(list_length(keys)); println(obj_has(o, "a")); println(obj_has(o, "b"))')
assert_output "obj_keys" $'2\ntrue\ntrue' "$output"

# obj_set and obj_get
output=$(run_ish 'let o = {}; obj_set(o, "key", "val"); println(obj_get(o, "key"))')
assert_output "obj_set and obj_get" "val" "$output"

# obj_remove
output=$(run_ish 'let o = {a: 1, b: 2}; obj_remove(o, "a"); println(obj_has(o, "a"))')
assert_output "obj_remove" "false" "$output"

finish
