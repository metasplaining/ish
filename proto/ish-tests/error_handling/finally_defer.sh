#!/usr/bin/env bash
# ---
# feature: Finally and Defer
# docs: docs/spec/syntax.md
# section: Error Handling
# ---
# Tests try/finally blocks, defer statements, and with blocks
# for resource management.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Finally and Defer ---"

# Finally always runs (no throw)
output=$(run_ish 'try { println("body") } catch (e) { println("catch") } finally { println("finally") }')
assert_output "finally without throw" $'body\nfinally' "$output"

# Finally runs after catch
output=$(run_ish 'try { throw new_error("x") } catch (e) { println("caught") } finally { println("finally") }')
assert_output "finally with throw" $'caught\nfinally' "$output"

# Defer in function — runs after return
output=$(run_ish 'fn f() { defer println("deferred"); println("body"); return 0 }; f()')
assert_output "defer runs after return" $'body\ndeferred' "$output"

# Multiple defers — LIFO order
output=$(run_ish 'fn f() { defer println("first"); defer println("second"); println("body"); return 0 }; f()')
assert_output "multiple defers LIFO" $'body\nsecond\nfirst' "$output"

# With block
output=$(run_ish 'let resource = {value: 42}; with (r = resource) { println(r.value) }')
assert_output "with block" "42" "$output"

# Try-catch-finally (catch required by parser)
output=$(run_ish 'try { println("try") } catch (e) { println("catch") } finally { println("fin") }')
assert_output "try-catch-finally" $'try\nfin' "$output"

finish
