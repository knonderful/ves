#!/bin/sh

set -e

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
cd "${SCRIPT_DIR}"

if [ "$BUILD_TYPE" = "release" ]; then
  CARGO_ARGS="--release"
else
  CARGO_ARGS=""
fi

cd common
# shellcheck disable=SC2086
cargo build ${CARGO_ARGS}
cd ../logger
# shellcheck disable=SC2086
cargo build ${CARGO_ARGS}
cd ../core
# shellcheck disable=SC2086
cargo build ${CARGO_ARGS}
cd ../game
# shellcheck disable=SC2086
cargo build ${CARGO_ARGS}
"${SCRIPT_DIR}/../generate_wat.sh"
