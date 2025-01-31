#!/bin/sh
set -xeuo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

cd ../../
cargo build

rm -rf ../target/kotlin-uniffi-bindings
mkdir -p ../target/kotlin-uniffi-bindings

cargo run --bin uniffi-bindgen generate --library ../target/debug/liblumina_node_uniffi.so --language kotlin --out-dir ../target/kotlin-uniffi-bindings
