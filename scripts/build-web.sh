#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Cargo.lockに固定された依存関係でWasmと対応するJSランタイムを揃える。
cargo build --locked --release --target wasm32-unknown-unknown
macroquad_manifest=$(cargo metadata --locked --filter-platform wasm32-unknown-unknown --format-version 1 | python3 -c '
import json, sys
data = json.load(sys.stdin)
print(next(p["manifest_path"] for p in data["packages"] if p["name"] == "macroquad"))
')
mkdir -p dist
cp web/index.html dist/index.html
cp "$(dirname "$macroquad_manifest")/js/mq_js_bundle.js" dist/mq_js_bundle.js
cp target/wasm32-unknown-unknown/release/ThreeBodyViewer.wasm dist/ThreeBodyViewer.wasm
echo "Web版をdist/に出力しました。python3 -m http.server 8080 --directory dist で確認できます。"
