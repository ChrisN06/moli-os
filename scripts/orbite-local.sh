#!/bin/sh
# Launch a loopback-only demo. No household driver or production configuration is loaded.
set -eu
cd "$(dirname "$0")/.."
unset MOLI_MASTER_KEY MOLI_MASTER_KEY_FILE MOLI_UI_PIN_FILE MOLI_CONFIG
export MOLI_UI_PIN=123456
npm run build --prefix web
cargo build --offline -p moli-os
exec target/debug/moli-os --config demo/orbite/moli.toml serve
