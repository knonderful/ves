#!/bin/sh

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
cd "${SCRIPT_DIR}"
cargo build --release --target wasm32-unknown-unknown
"${SCRIPT_DIR}/../../generate_wat.sh"
