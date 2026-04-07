#!/usr/bin/env bash
# ---
# feature: Module identity
# docs: docs/spec/modules.md
# section: Module Identity
# ---
# Tests file = module identity, script vs. importable, and cycle detection.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Module Identity ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# File with top-level command runs directly
proj=$(make_project)
cat > "$proj/src/cmd.ish" << 'EOF'
println("ok")
EOF
output=$("$ISH" "$proj/src/cmd.ish" 2>&1)
assert_output "file with top-level command runs directly" "ok" "$output"
rm -rf "$proj"

# File with top-level command is not importable (E018)
proj=$(make_project)
cat > "$proj/src/cmd.ish" << 'EOF'
println("oops")
EOF
cat > "$proj/src/main.ish" << 'EOF'
use cmd
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "file with top-level command not importable" "E018" "$output"
rm -rf "$proj"

# File with only functions is importable
proj=$(make_project)
cat > "$proj/src/pure.ish" << 'EOF'
pub fn f() { return 42 }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use pure { f }
println(f())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "file with only fns importable" "42" "$output"
rm -rf "$proj"

# Cross-module cycle (E017)
proj=$(make_project)
cat > "$proj/src/a.ish" << 'EOF'
use b
pub fn fa() { return 1 }
EOF
cat > "$proj/src/b.ish" << 'EOF'
use a
pub fn fb() { return 2 }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use a
println(a.fa())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "cross-module cycle" "E017" "$output"
rm -rf "$proj"

# use path with no matching file (E016)
proj=$(make_project)
cat > "$proj/src/main.ish" << 'EOF'
use nonexistent
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "use path no ish extension" "E016" "$output"
rm -rf "$proj"

finish
