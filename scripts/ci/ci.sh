#!/usr/bin/env sh

set -e

cargo check
cargo fmt --all --check
cargo build --release
cargo test --all --release
cargo clippy --all-targets --all-features -- -D warnings