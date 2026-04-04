#!/usr/bin/env bash
# Integration tests for non-interactive async file execution.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / non_interactive"

# --- -c mode with async code ---
output=$(run_ish 'async fn compute() { return 2 + 3 }; println(await compute())')
assert_output "-c mode await" "5" "$output"

# --- File execution with async code via temp file ---
TMPFILE=$(mktemp /tmp/ish_test_XXXXXX.ish)
cat > "$TMPFILE" <<'EOF'
async fn factorial(n: int) -> int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

let r1 = await factorial(5)
let r2 = await factorial(6)
println(r1)
println(r2)
EOF
output=$("$ISH" "$TMPFILE" 2>&1)
assert_output "file execution with await" "120
720" "$output"
rm -f "$TMPFILE"

# --- Multiple sequential awaits ---
output=$(run_ish 'async fn id(x: int) { return x }; let results = {a: 0, b: 0, c: 0}; results.a = await id(1); results.b = await id(2); results.c = await id(3); println(results.a + results.b + results.c)')
assert_output "three sequential awaits" "6" "$output"

finish
