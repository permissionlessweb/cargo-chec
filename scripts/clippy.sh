#!/bin/sh
set -e
echo "Running clippy..."
cargo clippy -- -D warnings