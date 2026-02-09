#!/bin/sh

# Run all tests
test:
    ./scripts/test.sh

# Check code formatting
fmt-check:
    ./scripts/fmt_check.sh

# Run clippy linter
lint:
    ./scripts/clippy.sh

# Run all checks (test, format, lint)
check: test fmt-check lint

# Build and install all workspace binaries to ~/.cargo/bin
install:
    @echo "Building workspace in release mode..."
    cargo build --release --workspace
    @echo ""
    @echo "Installing binaries to ~/.cargo/bin..."
    @mkdir -p ~/.cargo/bin
    @cp -v target/release/cargo-chec ~/.cargo/bin/
    @cp -v target/release/cargo-tes ~/.cargo/bin/
    @cp -v target/release/cargo-carpulin ~/.cargo/bin/
    @echo ""
    @echo "✓ Installation complete!"
    @echo ""
    @echo "You can now use:"
    @echo "  cargo chec"
    @echo "  cargo tes"
    @echo "  cargo carpulin"

# Build all binaries in release mode
build:
    cargo build --release --workspace

# Run unified benchmark
benchmark:
    ./scripts/benchmark.sh

# Dry-run publish to crates.io
dry-run:
    ./scripts/publish_dry_run.sh

# Publish to crates.io
publish:
    ./scripts/publish.sh

# Full release workflow
release: check benchmark dry-run publish