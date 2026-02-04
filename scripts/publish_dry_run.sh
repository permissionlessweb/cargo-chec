#!/bin/sh
set -e
echo "Running publish dry-run for all wrappers..."
echo "  - cargo-chec..."
cargo publish -p cargo-chec --dry-run
echo "  - cargo-tes..."
cargo publish -p cargo-tes --dry-run
echo "  - cargo-carpulin..."
cargo publish -p cargo-carpulin --dry-run
echo "All dry-runs completed successfully!"