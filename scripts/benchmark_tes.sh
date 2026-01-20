#!/bin/sh
# Benchmark script: cargo test (JSON) vs cargo tes
# Measures execution time and output size comparison

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$PROJECT_ROOT/test"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date "+%Y-%m-%d_%H-%M-%S")
REPORT_FILE="$RESULTS_DIR/benchmark_tes_report_$TIMESTAMP.txt"

# Colors for output (if terminal supports it)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================"
echo "Benchmark: cargo test (JSON) vs cargo tes"
echo "============================================"
echo ""

# Check if test directory exists
if [ ! -d "$TEST_DIR" ]; then
    echo "${RED}Error: Test directory not found at $TEST_DIR${NC}"
    exit 1
fi

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Build cargo-tes first (release mode for fair comparison)
echo "Building cargo-tes..."
cd "$PROJECT_ROOT"
cargo build --release --quiet -p cargo-tes 2>/dev/null || {
    echo "${RED}Error: Failed to build cargo-tes${NC}"
    exit 1
}
echo "Build complete."
echo ""

# Navigate to test directory
cd "$TEST_DIR"

# Create temp files for outputs
TEST_OUTPUT=$(mktemp)
TES_OUTPUT=$(mktemp)

# Cleanup temp files on exit
cleanup() {
    rm -f "$TEST_OUTPUT" "$TES_OUTPUT"
}
trap cleanup EXIT

echo "Running cargo test --message-format=json -- -Z unstable-options --format=json..."
# Run cargo test with JSON output (capture stdout, ignore stderr and exit code)
{ time cargo test --message-format=json -- -Z unstable-options --format=json > "$TEST_OUTPUT" 2>/dev/null; } 2> /tmp/time_test || true
TEST_TIME=$(grep real /tmp/time_test | awk '{print $2}')
TEST_CHARS=$(wc -c < "$TEST_OUTPUT" | tr -d ' ')
TEST_LINES=$(wc -l < "$TEST_OUTPUT" | tr -d ' ')
echo "  Time: $TEST_TIME"
echo "  Output: $TEST_CHARS characters, $TEST_LINES lines"
echo ""

echo "Running cargo tes..."
# Run cargo tes (capture stdout, ignore stderr)
{ time "$PROJECT_ROOT/target/release/cargo-tes" tes > "$TES_OUTPUT" 2>/dev/null; } 2> /tmp/time_tes || true
TES_TIME=$(grep real /tmp/time_tes | awk '{print $2}')
TES_CHARS=$(wc -c < "$TES_OUTPUT" | tr -d ' ')
TES_LINES=$(wc -l < "$TES_OUTPUT" | tr -d ' ')
echo "  Time: $TES_TIME"
echo "  Output: $TES_CHARS characters, $TES_LINES lines"
echo ""

# Calculate savings percentages
if [ "$TEST_CHARS" -gt 0 ]; then
    SAVINGS_CHARS=$(echo "scale=1; ($TEST_CHARS - $TES_CHARS) * 100 / $TEST_CHARS" | bc)
else
    SAVINGS_CHARS="N/A"
fi

# Display results
echo "============================================"
echo "Results Summary"
echo "============================================"
echo ""
echo "cargo test --message-format=json -- -Z unstable-options --format=json:"
echo "  Time: $TEST_TIME"
echo "  Characters: $TEST_CHARS"
echo "  Lines: $TEST_LINES"
echo ""
echo "cargo tes:"
echo "  Time: $TES_TIME"
echo "  Characters: $TES_CHARS"
echo "  Lines: $TES_LINES"
echo "  Character savings: ${SAVINGS_CHARS}%"
echo ""

# Generate detailed report
cat > "$REPORT_FILE" << EOF
Benchmark Report - cargo tes vs cargo test
===========================================

Date: $(date "+%Y-%m-%d %H:%M:%S")
Test Codebase: $TEST_DIR
Test Suite: 14 tests (5 passing, 7 failing, 2 ignored)

===========================================
Performance Comparison
===========================================

cargo test --message-format=json -- -Z unstable-options --format=json:
  Execution time: $TEST_TIME
  Characters: $TEST_CHARS
  Lines: $TEST_LINES

cargo tes:
  Execution time: $TES_TIME
  Characters: $TES_CHARS
  Lines: $TES_LINES
  Character savings: ${SAVINGS_CHARS}%

===========================================
Sample Outputs
===========================================

--- cargo test output (first 1000 chars) ---
$(head -c 1000 "$TEST_OUTPUT")

--- cargo tes output ---
$(cat "$TES_OUTPUT")

===========================================
Analysis
===========================================

The cargo tes tool filters raw cargo test JSON output to produce a clean
JSON array of test failure messages, minimizing whitespace and focusing
only on errors (failed tests and suites). This reduces output size for
easier parsing by editors, CI/CD pipelines, and AI tools.

Key benefits:
- Focused error-only output (ignores passing/ignored tests)
- Space-minimized messages for reduced character count
- Consistent JSON array format for programmatic consumption
- Faster processing for large test suites with many failures
EOF

echo "Report saved to: $REPORT_FILE"
echo ""
echo "Done!"