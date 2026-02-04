#!/bin/sh
# Benchmark script: cargo check (JSON) vs cargo chec
# Measures character count savings when using the filtered output

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$PROJECT_ROOT/tools/broken-tests"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date "+%Y-%m-%d_%H-%M-%S")
REPORT_FILE="$RESULTS_DIR/benchmark_report_$TIMESTAMP.txt"

# Colors for output (if terminal supports it)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================"
echo "Benchmark: cargo check vs cargo chec"
echo "============================================"
echo ""

# Check if test directory exists
if [ ! -d "$TEST_DIR" ]; then
    echo "${RED}Error: Test directory not found at $TEST_DIR${NC}"
    exit 1
fi

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Build cargo-chec first (release mode for fair comparison)
echo "Building cargo-chec..."
cd "$PROJECT_ROOT"
cargo build --release --quiet 2>/dev/null || {
    echo "${RED}Error: Failed to build cargo-chec${NC}"
    exit 1
}
echo "Build complete."
echo ""

# Navigate to test directory
cd "$TEST_DIR"

# Create temp files for outputs
CHECK_OUTPUT=$(mktemp)
CHEC_OUTPUT=$(mktemp)
CHEC_WARNINGS_OUTPUT=$(mktemp)

# Cleanup temp files on exit
cleanup() {
    rm -f "$CHECK_OUTPUT" "$CHEC_OUTPUT" "$CHEC_WARNINGS_OUTPUT"
}
trap cleanup EXIT

echo "Running cargo check --message-format=json..."
# Run cargo check with JSON output (capture stdout, ignore stderr and exit code)
cargo check --message-format=json 2>/dev/null > "$CHECK_OUTPUT" || true
CHECK_CHARS=$(wc -c < "$CHECK_OUTPUT" | tr -d ' ')
CHECK_LINES=$(wc -l < "$CHECK_OUTPUT" | tr -d ' ')
echo "  Output: $CHECK_CHARS characters, $CHECK_LINES lines"
echo ""

echo "Running cargo chec (errors only)..."
# Run cargo chec (capture stdout, ignore stderr)
"$PROJECT_ROOT/target/release/cargo-chec" chec 2>/dev/null > "$CHEC_OUTPUT" || true
CHEC_CHARS=$(wc -c < "$CHEC_OUTPUT" | tr -d ' ')
CHEC_LINES=$(wc -l < "$CHEC_OUTPUT" | tr -d ' ')
echo "  Output: $CHEC_CHARS characters, $CHEC_LINES lines"
echo ""

echo "Running cargo chec --include-warnings..."
# Run cargo chec with warnings
"$PROJECT_ROOT/target/release/cargo-chec" chec --include-warnings 2>/dev/null > "$CHEC_WARNINGS_OUTPUT" || true
CHEC_WARNINGS_CHARS=$(wc -c < "$CHEC_WARNINGS_OUTPUT" | tr -d ' ')
CHEC_WARNINGS_LINES=$(wc -l < "$CHEC_WARNINGS_OUTPUT" | tr -d ' ')
echo "  Output: $CHEC_WARNINGS_CHARS characters, $CHEC_WARNINGS_LINES lines"
echo ""

# Calculate savings percentages
if [ "$CHECK_CHARS" -gt 0 ]; then
    SAVINGS_ERRORS=$(echo "scale=1; ($CHECK_CHARS - $CHEC_CHARS) * 100 / $CHECK_CHARS" | bc)
    SAVINGS_WARNINGS=$(echo "scale=1; ($CHECK_CHARS - $CHEC_WARNINGS_CHARS) * 100 / $CHECK_CHARS" | bc)
else
    SAVINGS_ERRORS="N/A"
    SAVINGS_WARNINGS="N/A"
fi

# Display results
echo "============================================"
echo "Results Summary"
echo "============================================"
echo ""
echo "cargo check --message-format=json:"
echo "  Characters: $CHECK_CHARS"
echo "  Lines: $CHECK_LINES"
echo ""
echo "cargo chec (errors only):"
echo "  Characters: $CHEC_CHARS"
echo "  Savings: ${SAVINGS_ERRORS}%"
echo ""
echo "cargo chec --include-warnings:"
echo "  Characters: $CHEC_WARNINGS_CHARS"
echo "  Savings: ${SAVINGS_WARNINGS}%"
echo ""

# Generate detailed report
cat > "$REPORT_FILE" << EOF
Benchmark Report - cargo chec vs cargo check
============================================

Date: $(date "+%Y-%m-%d %H:%M:%S")
Test Codebase: $TEST_DIR

============================================
Character Count Comparison
============================================

cargo check --message-format=json:
  Characters: $CHECK_CHARS
  Lines: $CHECK_LINES

cargo chec (errors only):
  Characters: $CHEC_CHARS
  Lines: $CHEC_LINES
  Character savings: ${SAVINGS_ERRORS}%

cargo chec --include-warnings:
  Characters: $CHEC_WARNINGS_CHARS
  Lines: $CHEC_WARNINGS_LINES
  Character savings: ${SAVINGS_WARNINGS}%

============================================
Sample Outputs
============================================

--- cargo check output (first 1000 chars) ---
$(head -c 1000 "$CHECK_OUTPUT")

--- cargo chec output (errors only) ---
$(cat "$CHEC_OUTPUT")

--- cargo chec output (with warnings) ---
$(cat "$CHEC_WARNINGS_OUTPUT")

============================================
Analysis
============================================

The cargo chec tool reduces the character count by transforming verbose
JSON diagnostic messages into a simplified, single-line format suitable
for programmatic parsing by editors, CI/CD pipelines, and AI tools.

Key benefits:
- Reduced token usage for LLM-based tools
- Easier parsing with consistent format
- Focused output (errors only by default, warnings optional)
EOF

echo "Report saved to: $REPORT_FILE"
echo ""
echo "Done!"
