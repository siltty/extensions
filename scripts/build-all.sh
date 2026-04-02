#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
PUBLIC_DIR="$REPO_DIR/public"

echo "=== Building Siltty Extensions ==="

mkdir -p "$PUBLIC_DIR/plugins"

PLUGINS=()
for plugin_dir in "$REPO_DIR/plugins"/*/; do
    name=$(basename "$plugin_dir")
    echo "Building: $name"

    cd "$plugin_dir"
    cargo build --target wasm32-wasip1 --release 2>&1 | tail -1

    # Find the built .wasm file
    wasm_file=$(find target/wasm32-wasip1/release -name "*.wasm" -not -name "*.d" | head -1)
    if [ -z "$wasm_file" ]; then
        echo "  ERROR: no .wasm found for $name"
        continue
    fi

    # Read version from plugin.toml
    version=$(grep 'version' plugin.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

    # Copy to public
    cp "$wasm_file" "$PUBLIC_DIR/plugins/${name}-v${version}.wasm"
    cp plugin.toml "$PUBLIC_DIR/plugins/${name}.toml"

    size=$(du -h "$PUBLIC_DIR/plugins/${name}-v${version}.wasm" | cut -f1)
    echo "  -> ${name}-v${version}.wasm ($size)"

    PLUGINS+=("$name")
    cd "$REPO_DIR"
done

# Generate index.json
echo "Generating index.json..."
python3 -c "
import json, tomllib, os

plugins = []
for name in sorted(os.listdir('$PUBLIC_DIR/plugins')):
    if not name.endswith('.toml'):
        continue
    plugin_name = name.replace('.toml', '')
    toml_path = os.path.join('$PUBLIC_DIR/plugins', name)
    with open(toml_path, 'rb') as f:
        manifest = tomllib.load(f)
    info = manifest.get('plugin', {})
    perms = manifest.get('permissions', {})
    version = info.get('version', '0.1.0')
    wasm_file = f'{plugin_name}-v{version}.wasm'
    wasm_path = os.path.join('$PUBLIC_DIR/plugins', wasm_file)
    size = os.path.getsize(wasm_path) if os.path.exists(wasm_path) else 0

    plugins.append({
        'id': plugin_name,
        'name': info.get('name', plugin_name),
        'version': version,
        'description': info.get('description', ''),
        'author': info.get('author', ''),
        'download': f'plugins/{wasm_file}',
        'size': size,
        'permissions': [k for k, v in perms.items() if v],
        'api_version': info.get('api_version', 1),
    })

index = {'version': 1, 'plugins': plugins}
with open('$PUBLIC_DIR/index.json', 'w') as f:
    json.dump(index, f, indent=2)
print(f'Generated index.json with {len(plugins)} plugins')
" 2>&1

echo ""
echo "=== Done: ${#PLUGINS[@]} plugins built ==="
echo "Output: $PUBLIC_DIR/"
