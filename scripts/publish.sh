#!/bin/sh
set -e
echo "Publishing cargo-chec to crates.io..."
cargo publish
echo "Publishing cargo-tes to crates.io..."
cargo publish -p cargo-tes