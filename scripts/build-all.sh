#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
PUBLIC_DIR="$REPO_DIR/public"

echo "=== Building Siltty Extensions ==="

mkdir -p "$PUBLIC_DIR/plugins"

PLUGIN_COUNT=0

for plugin_dir in "$REPO_DIR/plugins"/*/; do
    [ -d "$plugin_dir" ] || continue
    name=$(basename "$plugin_dir")
    echo "Building: $name"

    cd "$plugin_dir"
    cargo build --target wasm32-wasip1 --release

    wasm_file=$(find target/wasm32-wasip1/release -maxdepth 1 -name "*.wasm" -not -name "*.d" | head -1)
    if [ -z "$wasm_file" ]; then
        echo "  ERROR: no .wasm found for $name"
        cd "$REPO_DIR"
        continue
    fi

    version=$(grep '^version' plugin.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    cp "$wasm_file" "$PUBLIC_DIR/plugins/${name}-v${version}.wasm"
    cp plugin.toml "$PUBLIC_DIR/plugins/${name}.toml"

    size=$(wc -c < "$PUBLIC_DIR/plugins/${name}-v${version}.wasm")
    echo "  → ${name}-v${version}.wasm (${size} bytes)"

    PLUGIN_COUNT=$((PLUGIN_COUNT + 1))
    cd "$REPO_DIR"
done

# Generate index.json — pure bash, no Python
echo "Generating index.json..."
INDEX="$PUBLIC_DIR/index.json"
printf '{\n  "version": 1,\n  "plugins": [\n' > "$INDEX"

FIRST=true
for toml_file in "$PUBLIC_DIR/plugins"/*.toml; do
    [ -f "$toml_file" ] || continue
    id=$(basename "$toml_file" .toml)
    pname=$(sed -n '/^\[plugin\]/,/^\[/{s/^name *= *"\(.*\)"/\1/p;}' "$toml_file" | head -1)
    pver=$(sed -n '/^\[plugin\]/,/^\[/{s/^version *= *"\(.*\)"/\1/p;}' "$toml_file" | head -1)
    pdesc=$(sed -n '/^\[plugin\]/,/^\[/{s/^description *= *"\(.*\)"/\1/p;}' "$toml_file" | head -1)
    pauthor=$(sed -n '/^\[plugin\]/,/^\[/{s/^author *= *"\(.*\)"/\1/p;}' "$toml_file" | head -1)
    papi=$(sed -n '/^\[plugin\]/,/^\[/{s/^api_version *= *\([0-9]*\)/\1/p;}' "$toml_file" | head -1)
    [ -z "$papi" ] && papi="1"
    [ -z "$pname" ] && pname="$id"
    [ -z "$pver" ] && pver="0.1.0"

    wasm="$PUBLIC_DIR/plugins/${id}-v${pver}.wasm"
    psize=0
    [ -f "$wasm" ] && psize=$(wc -c < "$wasm" | tr -d ' ')

    perms=""
    while IFS= read -r line; do
        key=$(echo "$line" | sed 's/ *= *true//' | tr -d ' ')
        [ -n "$key" ] && perms="${perms}\"${key}\","
    done < <(sed -n '/^\[permissions\]/,/^\[/{/= *true/p;}' "$toml_file")
    perms="${perms%,}"

    [ "$FIRST" = true ] && FIRST=false || printf ',\n' >> "$INDEX"
    printf '    {"id":"%s","name":"%s","version":"%s","description":"%s","author":"%s","download":"plugins/%s-v%s.wasm","size":%s,"permissions":[%s],"api_version":%s}' \
        "$id" "$pname" "$pver" "$pdesc" "$pauthor" "$id" "$pver" "$psize" "$perms" "$papi" >> "$INDEX"
done

printf '\n  ]\n}\n' >> "$INDEX"

echo ""
echo "=== Done: ${PLUGIN_COUNT} plugins built ==="
cat "$INDEX"
