#!/usr/bin/env bash
# ---
# feature: Module mapping
# docs: docs/spec/modules.md
# section: Module Mapping
# ---
# Tests for file-to-module path mapping, index.ish rule, and conflict detection.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Module Mapping ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# use net/http resolves to src/net/http.ish
proj=$(make_project)
mkdir -p "$proj/src/net"
cat > "$proj/src/net/http.ish" << 'EOF'
pub fn get() { return "ok" }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use net/http { get }
println(get())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "use net/http resolves to file" "ok" "$output"
rm -rf "$proj"

# use net resolves to src/net/index.ish
proj=$(make_project)
mkdir -p "$proj/src/net"
cat > "$proj/src/net/index.ish" << 'EOF'
pub fn connect() { return "connected" }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use net { connect }
println(connect())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "use net resolves to index" "connected" "$output"
rm -rf "$proj"

# Path conflict error (E019): both foo.ish and foo/index.ish exist
proj=$(make_project)
mkdir -p "$proj/src/foo"
echo 'pub fn x() { return 1 }' > "$proj/src/foo.ish"
echo 'pub fn y() { return 2 }' > "$proj/src/foo/index.ish"
cat > "$proj/src/main.ish" << 'EOF'
use foo
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "path conflict error" "E019" "$output"
rm -rf "$proj"

# Not found error (E016)
proj=$(make_project)
cat > "$proj/src/main.ish" << 'EOF'
use nonexistent
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "not found error" "E016" "$output"
rm -rf "$proj"

finish
