#!/bin/sh

SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"

set -e
cd proto-core
cargo build --release
cd ..
cd proto-game
./build.sh
cd ..
retroarch -L "${SCRIPT_DIR}/../target/release/libproto_core.so" "${SCRIPT_DIR}/../target/wasm32-unknown-unknown/release/proto_game.wasm"

