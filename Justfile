#!/bin/sh

test:
    ./scripts/test.sh

fmt-check:
    ./scripts/fmt_check.sh

lint:
    ./scripts/clippy.sh

check: test fmt-check lint

benchmark: 
    ./scripts/benchmark_carpulin.sh
    ./scripts/benchmark_tes.sh
    ./scripts/benchmark_chec.sh

dry-run:
    ./scripts/publish_dry_run.sh

publish:
    ./scripts/publish.sh

release: check benchmark dry-run publish