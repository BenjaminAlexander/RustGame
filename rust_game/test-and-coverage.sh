#!/bin/bash

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE="$PWD/target/coverage/cargo-test-%p-%m.profraw"

cargo clean
cargo test
grcov . --binary-path ./target/debug/ -s . -t html -o target/coverage/html
