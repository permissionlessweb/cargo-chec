# Justfile for cargo-chec release tasks

# Run tests
test:
    ./scripts/test.sh

# Check formatting
fmt-check:
    ./scripts/fmt_check.sh

# Run linting
lint:
    ./scripts/clippy.sh

# Run all pre-publish checks
check: test fmt-check lint

# Dry-run publish
dry-run:
    ./scripts/publish_dry_run.sh

# Publish to crates.io
publish:
    ./scripts/publish.sh

# Full release: check, dry-run, then publish
release: check dry-run publish