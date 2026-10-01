#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo check --locked -p maqai-ui --target wasm32-unknown-unknown
cargo clippy --locked -p maqai-ui --target wasm32-unknown-unknown -- -D warnings
trunk build
cargo clippy --locked -p maqai --all-targets -- -D warnings
cargo test --locked -p maqai --lib
