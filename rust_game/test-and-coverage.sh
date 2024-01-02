#!/bin/bash

RUST_GAME_DIR="$(readlink -f $(dirname $0))"

cd $RUST_GAME_DIR

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE="$RUST_GAME_DIR/target/coverage/cargo-test-%p-%m.profraw"
export RUST_BACKTRACE=1

cargo clean
cargo test
grcov . --binary-path ./target/debug/ -s . -t html -o target/coverage/html

