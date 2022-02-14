#!/bin/sh
set -e

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
BINARY_BASE_NAME="proto_game"

# Environment variables for wasm-bindgen taken from https://github.com/rustwasm/wasm-bindgen/blob/master/crates/cli-support/src/lib.rs#L95
# WASM_BINDGEN_EXTERNREF=1 WASM_INTERFACE_TYPES=1 wasm-pack build ....
#wasm-pack build --no-typescript --release
cargo build --release

WASM2WAT_OPTS="--enable-reference-types --inline-exports --inline-imports --generate-names"

ORIGINAL_BASE="${SCRIPT_DIR}/../../target/wasm32-unknown-unknown/release/${BINARY_BASE_NAME}"
ORIGINAL_WASM="${ORIGINAL_BASE}.wasm"
ORIGINAL_WAT="${ORIGINAL_BASE}.wat"
wasm2wat "${WASM2WAT_OPTS}" -o "${ORIGINAL_WAT}" "${ORIGINAL_WASM}"

#OPTIMIZED_BASE="${SCRIPT_DIR}/pkg/${BINARY_BASE_NAME}_bg"
#OPTIMIZED_WASM="${OPTIMIZED_BASE}.wasm"
#OPTIMIZED_WAT="${OPTIMIZED_BASE}.wat"
#wasm2wat ${WASM2WAT_OPTS} -o "${OPTIMIZED_WAT}" "${OPTIMIZED_WASM}"

#FINAL_BASE="${SCRIPT_DIR}/target/${BINARY_BASE_NAME}"
#FINAL_WASM="${FINAL_BASE}.wasm"
#FINAL_WAT="${FINAL_BASE}.wat"
#cargo install --path "${SCRIPT_DIR}/../../wasm-bindgen-purify"
#wasm-bindgen-purify --original "${ORIGINAL_WASM}" --optimized "${OPTIMIZED_WASM}" --out "${FINAL_WASM}"
#wasm2wat ${WASM2WAT_OPTS} -o "${FINAL_WAT}" "${FINAL_WASM}"

#echo "Build successful. The WASM binary is located at ${FINAL_WASM}."
