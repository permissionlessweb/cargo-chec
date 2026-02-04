#!/bin/sh
set -e
echo "Publishing all wrappers to crates.io..."
echo "  - cargo-chec..."
cargo publish -p cargo-chec
echo "  - cargo-tes..."
cargo publish -p cargo-tes
echo "  - cargo-carpulin..."
cargo publish -p cargo-carpulin
echo "All packages published successfully!"