#!/bin/bash

echo "cargo fmt --check" && cargo fmt --check &&
    echo "cargo clippy --features=while_select,mpsc_multiplexer -- --deny warnings" && cargo clippy --features=while_select,mpsc_multiplexer -- --deny warnings &&
    echo "cargo test --features=while_select,mpsc_multiplexer" && cargo test --features=while_select,mpsc_multiplexer