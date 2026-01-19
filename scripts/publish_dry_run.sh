#!/bin/sh
set -e
echo "Running publish dry-run for cargo-chec..."
cargo publish --dry-run
echo "Running publish dry-run for cargo-tes..."
cargo publish -p cargo-tes --dry-run