#!/bin/sh

set -e

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
cd "${SCRIPT_DIR}"

if [ "$BUILD_TYPE" = "" ]; then
  BUILD_TYPE="debug"
fi

./build.sh
"${SCRIPT_DIR}/../target/${BUILD_TYPE}/ves-proto-core" "${SCRIPT_DIR}/../target/wasm32-unknown-unknown/${BUILD_TYPE}/ves_proto_game.wasm"