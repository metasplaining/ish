#!/usr/bin/env bash
# ---
# feature: Import syntax
# docs: docs/spec/modules.md
# section: Import Syntax
# ---
# Tests for all four import forms: plain, alias, selective, selective-alias.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Import Syntax ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# Set up a project with utils module
setup_utils_project() {
    local proj
    proj=$(make_project)
    cat > "$proj/src/utils.ish" << 'EOF'
pub fn greet(name) { return "hello " + name }
pub fn farewell(name) { return "bye " + name }
EOF
    echo "$proj"
}

# Plain use — qualified access
proj=$(setup_utils_project)
cat > "$proj/src/main.ish" << 'EOF'
use utils
println(utils.greet("world"))
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "plain use" "hello world" "$output"
rm -rf "$proj"

# Alias use
proj=$(setup_utils_project)
cat > "$proj/src/main.ish" << 'EOF'
use utils as u
println(u.greet("world"))
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "alias use" "hello world" "$output"
rm -rf "$proj"

# Selective use
proj=$(setup_utils_project)
cat > "$proj/src/main.ish" << 'EOF'
use utils { greet }
println(greet("world"))
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "selective use" "hello world" "$output"
rm -rf "$proj"

# Selective use with alias
proj=$(setup_utils_project)
cat > "$proj/src/main.ish" << 'EOF'
use utils { greet as hi }
println(hi("world"))
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "selective alias use" "hello world" "$output"
rm -rf "$proj"

# Multiple selective imports
proj=$(setup_utils_project)
cat > "$proj/src/main.ish" << 'EOF'
use utils { greet, farewell }
println(greet("world"))
println(farewell("world"))
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
expected="hello world
bye world"
assert_output "multiple selective imports" "$expected" "$output"
rm -rf "$proj"

finish
