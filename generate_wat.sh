#!/bin/sh

# This script generates WAT files from all WASM files that are found in the "target" directory.

if ! which wasm2wat > /dev/null
then
  >&2 echo "This script requires 'wasm2wat'. (https://github.com/WebAssembly/wabt)"
  exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
for f in $(find "${SCRIPT_DIR}"/target -name "*.wasm") ; do
  OUT="${f%%.wasm}.wat"
  # --enable-reference-types
  wasm2wat --inline-exports --inline-imports --generate-names -o "${OUT}" "${f}"
  echo "${OUT}"
done