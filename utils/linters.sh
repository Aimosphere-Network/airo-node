#!/bin/sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo clippy --tests -- -D warnings
