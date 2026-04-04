#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
PUBLIC_DIR="$REPO_DIR/public"

echo "=== Building Siltty Extensions ==="
echo ""

rm -rf "$PUBLIC_DIR"
mkdir -p "$PUBLIC_DIR/extensions"

PLUGIN_COUNT=0
THEME_COUNT=0
TOTAL_COUNT=0

# ── Process each extension ───────────────────────────────────────
for ext_dir in "$REPO_DIR/extensions"/*/; do
    [ -d "$ext_dir" ] || continue

    manifest="$ext_dir/extension.toml"
    [ -f "$manifest" ] || continue

    id=$(basename "$ext_dir")
    ext_type=$(sed -n 's/^type *= *"\(.*\)"/\1/p' "$manifest" | head -1)
    version=$(sed -n '/^\[extension\]/,/^\[/{s/^version *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    [ -z "$version" ] && version="0.1.0"

    out_dir="$PUBLIC_DIR/extensions/$id"
    mkdir -p "$out_dir"

    # Copy extension.toml to output (serves as manifest for install)
    cp "$manifest" "$out_dir/extension.toml"

    # ── Plugin build ─────────────────────────────────────────────
    if [[ "$ext_type" == "plugin" || "$ext_type" == "plugin+theme" ]]; then
        echo "Building plugin: $id v$version"
        cd "$ext_dir"
        cargo build --target wasm32-wasip1 --release 2>&1 | tail -1

        wasm_file=$(find target/wasm32-wasip1/release -maxdepth 1 -name "*.wasm" -not -name "*.d" | head -1)
        if [ -z "$wasm_file" ]; then
            echo "  ERROR: no .wasm found for $id"
            cd "$REPO_DIR"
            continue
        fi

        cp "$wasm_file" "$out_dir/plugin.wasm"
        size=$(wc -c < "$out_dir/plugin.wasm" | tr -d ' ')
        echo "  → plugin.wasm ($size bytes)"

        PLUGIN_COUNT=$((PLUGIN_COUNT + 1))
        cd "$REPO_DIR"
    fi

    # ── Theme copy ───────────────────────────────────────────────
    if [[ "$ext_type" == "theme" || "$ext_type" == "plugin+theme" ]]; then
        if [ -d "$ext_dir/themes" ]; then
            theme_count=0
            for theme_file in "$ext_dir/themes"/*.toml; do
                [ -f "$theme_file" ] || continue
                cp "$theme_file" "$out_dir/"
                theme_count=$((theme_count + 1))
            done
            echo "Adding themes: $id ($theme_count theme(s))"
            THEME_COUNT=$((THEME_COUNT + theme_count))
        fi
    fi

    TOTAL_COUNT=$((TOTAL_COUNT + 1))
done

# ── Generate index.json ──────────────────────────────────────────
echo ""
echo "Generating index.json..."

INDEX="$PUBLIC_DIR/index.json"
cat > "$INDEX" << 'HEADER'
{
  "version": 3,
  "extensions": [
HEADER

FIRST=true

for ext_dir in "$PUBLIC_DIR/extensions"/*/; do
    [ -d "$ext_dir" ] || continue

    manifest="$ext_dir/extension.toml"
    [ -f "$manifest" ] || continue

    id=$(basename "$ext_dir")

    # Parse extension.toml fields
    parse_field() {
        sed -n "/^\[extension\]/,/^\[/{s/^$1 *= *\"\(.*\)\"/\1/p;}" "$manifest" | head -1
    }

    ext_name=$(parse_field "name")
    ext_type=$(parse_field "type")
    ext_ver=$(parse_field "version")
    ext_desc=$(parse_field "description")
    ext_author=$(parse_field "author")
    ext_license=$(parse_field "license")
    [ -z "$ext_name" ] && ext_name="$id"
    [ -z "$ext_type" ] && ext_type="plugin"
    [ -z "$ext_ver" ] && ext_ver="0.1.0"

    # Build files array
    files_json=""
    for f in "$ext_dir"/*; do
        [ -f "$f" ] || continue
        fname=$(basename "$f")
        [ "$fname" = "extension.toml" ] && continue
        fsize=$(wc -c < "$f" | tr -d ' ')
        fsha=$(shasum -a 256 "$f" | cut -d' ' -f1)
        [ -n "$files_json" ] && files_json="$files_json,"
        files_json="$files_json{\"path\":\"$fname\",\"size\":$fsize,\"sha256\":\"$fsha\"}"
    done

    # Build themes array for theme extensions
    themes_json=""
    if [[ "$ext_type" == "theme" || "$ext_type" == "plugin+theme" ]]; then
        # Extract [[themes]] entries — get all id fields
        theme_ids=$(grep -A1 '^\[\[themes\]\]' "$manifest" | grep '^id' | sed 's/.*"\(.*\)".*/\1/')
        for tid in $theme_ids; do
            [ -n "$themes_json" ] && themes_json="$themes_json,"
            themes_json="$themes_json\"$tid\""
        done
    fi

    # Build permissions array for plugins
    perms_json=""
    if [[ "$ext_type" == "plugin" || "$ext_type" == "plugin+theme" ]]; then
        while IFS= read -r line; do
            key=$(echo "$line" | sed 's/ *= *true//' | tr -d ' ')
            [ -n "$key" ] && { [ -n "$perms_json" ] && perms_json="$perms_json,"; perms_json="$perms_json\"$key\""; }
        done < <(sed -n '/^\[permissions\]/,/^\[/{/= *true/p;}' "$manifest")

        api_ver=$(sed -n '/^\[plugin\]/,/^\[/{s/^api_version *= *\([0-9]*\)/\1/p;}' "$manifest" | head -1)
        [ -z "$api_ver" ] && api_ver="1"
    fi

    # Write JSON entry
    [ "$FIRST" = true ] && FIRST=false || printf ',\n' >> "$INDEX"

    printf '    {\n' >> "$INDEX"
    printf '      "id": "%s",\n' "$id" >> "$INDEX"
    printf '      "type": "%s",\n' "$ext_type" >> "$INDEX"
    printf '      "name": "%s",\n' "$ext_name" >> "$INDEX"
    printf '      "version": "%s",\n' "$ext_ver" >> "$INDEX"
    printf '      "description": "%s",\n' "$ext_desc" >> "$INDEX"
    printf '      "author": "%s",\n' "$ext_author" >> "$INDEX"
    [ -n "$ext_license" ] && printf '      "license": "%s",\n' "$ext_license" >> "$INDEX"
    printf '      "files": [%s]' "$files_json" >> "$INDEX"

    if [ -n "$themes_json" ]; then
        printf ',\n      "themes": [%s]' "$themes_json" >> "$INDEX"
    fi

    if [[ "$ext_type" == "plugin" || "$ext_type" == "plugin+theme" ]]; then
        printf ',\n      "permissions": [%s]' "$perms_json" >> "$INDEX"
        printf ',\n      "api_version": %s' "$api_ver" >> "$INDEX"
    fi

    printf '\n    }' >> "$INDEX"
done

# Close extensions array, add backward-compat plugins array
printf '\n  ],\n  "plugins": [\n' >> "$INDEX"

FIRST=true
for ext_dir in "$PUBLIC_DIR/extensions"/*/; do
    [ -d "$ext_dir" ] || continue
    manifest="$ext_dir/extension.toml"
    [ -f "$manifest" ] || continue

    ext_type=$(sed -n '/^\[extension\]/,/^\[/{s/^type *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    [[ "$ext_type" == "plugin" || "$ext_type" == "plugin+theme" ]] || continue

    [ -f "$ext_dir/plugin.wasm" ] || continue

    id=$(basename "$ext_dir")
    pname=$(sed -n '/^\[extension\]/,/^\[/{s/^name *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    pver=$(sed -n '/^\[extension\]/,/^\[/{s/^version *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    pdesc=$(sed -n '/^\[extension\]/,/^\[/{s/^description *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    pauthor=$(sed -n '/^\[extension\]/,/^\[/{s/^author *= *"\(.*\)"/\1/p;}' "$manifest" | head -1)
    papi=$(sed -n '/^\[plugin\]/,/^\[/{s/^api_version *= *\([0-9]*\)/\1/p;}' "$manifest" | head -1)
    [ -z "$papi" ] && papi="1"

    psize=$(wc -c < "$ext_dir/plugin.wasm" | tr -d ' ')
    psha=$(shasum -a 256 "$ext_dir/plugin.wasm" | cut -d' ' -f1)

    perms=""
    while IFS= read -r line; do
        key=$(echo "$line" | sed 's/ *= *true//' | tr -d ' ')
        [ -n "$key" ] && perms="${perms}\"${key}\","
    done < <(sed -n '/^\[permissions\]/,/^\[/{/= *true/p;}' "$manifest")
    perms="${perms%,}"

    [ "$FIRST" = true ] && FIRST=false || printf ',\n' >> "$INDEX"
    printf '    {"id":"%s","name":"%s","version":"%s","description":"%s","author":"%s","download":"extensions/%s/plugin.wasm","size":%s,"sha256":"%s","permissions":[%s],"api_version":%s}' \
        "$id" "$pname" "$pver" "$pdesc" "$pauthor" "$id" "$psize" "$psha" "$perms" "$papi" >> "$INDEX"
done

printf '\n  ]\n}\n' >> "$INDEX"

# Minified version
tr -d '\n' < "$INDEX" | sed 's/  */ /g' > "$PUBLIC_DIR/index.min.json"

# Add .nojekyll for GitHub Pages
touch "$PUBLIC_DIR/.nojekyll"

echo ""
echo "=== Done ==="
echo "  Extensions: $TOTAL_COUNT ($PLUGIN_COUNT plugins, $THEME_COUNT themes)"
echo "  index.json: $(wc -c < "$INDEX" | tr -d ' ') bytes"
echo "  Deploy: public/ → gh-pages branch"
