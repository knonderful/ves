#!/bin/sh

set -e
cd proto-core
cargo build --release
cd ..
cd proto-game
./build.sh
cd ..
retroarch -L proto-core/target/release/libproto_core.so proto-game/target/wasm32-unknown-unknown/release/proto_game.wasm

