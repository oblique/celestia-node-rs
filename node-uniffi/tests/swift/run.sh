#!/bin/sh
set -xeuo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

cd ../../
cargo build

mkdir -p tests/swift/bindings
mkdir -p tests/swift/bindings/Headers
mkdir -p tests/swift/ios

cargo run --bin uniffi-bindgen generate --library ../target/debug/liblumina_node_uniffi.so --language swift --out-dir tests/swift/bindings
