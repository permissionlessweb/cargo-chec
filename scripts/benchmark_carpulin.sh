#!/bin/sh
# Benchmark script: cargo llvm-cov / cargo tarpaulin vs cargo carpulin
# Measures character count and line count savings when using the structured output

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date "+%Y-%m-%d_%H-%M-%S")
REPORT_FILE="$RESULTS_DIR/benchmark_carpulin_report_$TIMESTAMP.txt"
COVERAGE_PKG="coverage-test-crate"

# Colors for output (if terminal supports it)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================"
echo "Benchmark: raw coverage tools vs cargo carpulin"
echo "============================================"
echo ""

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Build cargo-carpulin first (release mode for fair comparison)
echo "Building cargo-carpulin..."
cd "$PROJECT_ROOT"
cargo build --release --quiet -p cargo-carpulin 2>/dev/null || {
    echo "${RED}Error: Failed to build cargo-carpulin${NC}"
    exit 1
}
echo "Build complete."
echo ""

# Create temp files for outputs
LLVM_RAW_OUTPUT=$(mktemp)
LLVM_CARPULIN_OUTPUT=$(mktemp)
TARP_RAW_OUTPUT=$(mktemp)
TARP_CARPULIN_OUTPUT=$(mktemp)

# Cleanup temp files on exit
cleanup() {
    rm -f "$LLVM_RAW_OUTPUT" "$LLVM_CARPULIN_OUTPUT" "$TARP_RAW_OUTPUT" "$TARP_CARPULIN_OUTPUT"
}
trap cleanup EXIT

# ============================================
# llvm-cov comparison
# ============================================

HAS_LLVM_COV=true
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "${YELLOW}Warning: cargo-llvm-cov not installed, skipping llvm-cov benchmark${NC}"
    HAS_LLVM_COV=false
fi

if [ "$HAS_LLVM_COV" = true ]; then
    echo "Running cargo llvm-cov --json -p $COVERAGE_PKG..."
    cargo llvm-cov --json -p "$COVERAGE_PKG" > "$LLVM_RAW_OUTPUT" 2>/dev/null || true
    # Strip any non-JSON prefix (llvm-cov may emit prompts)
    python3 -c "
import sys
content = open(sys.argv[1]).read()
idx = content.index('{')
open(sys.argv[1], 'w').write(content[idx:])
" "$LLVM_RAW_OUTPUT" 2>/dev/null || true
    LLVM_RAW_CHARS=$(wc -c < "$LLVM_RAW_OUTPUT" | tr -d ' ')
    LLVM_RAW_LINES=$(wc -l < "$LLVM_RAW_OUTPUT" | tr -d ' ')
    echo "  Output: $LLVM_RAW_CHARS characters, $LLVM_RAW_LINES lines"
    echo ""

    echo "Running cargo carpulin --tool llvm-cov -- -p $COVERAGE_PKG..."
    "$PROJECT_ROOT/target/release/cargo-carpulin" carpulin --tool llvm-cov -- -p "$COVERAGE_PKG" > "$LLVM_CARPULIN_OUTPUT" 2>/dev/null || true
    LLVM_CARPULIN_CHARS=$(wc -c < "$LLVM_CARPULIN_OUTPUT" | tr -d ' ')
    LLVM_CARPULIN_LINES=$(wc -l < "$LLVM_CARPULIN_OUTPUT" | tr -d ' ')
    echo "  Output: $LLVM_CARPULIN_CHARS characters, $LLVM_CARPULIN_LINES lines"
    echo ""

    # Calculate savings
    if [ "$LLVM_RAW_CHARS" -gt 0 ]; then
        LLVM_SAVINGS_CHARS=$(echo "scale=1; ($LLVM_RAW_CHARS - $LLVM_CARPULIN_CHARS) * 100 / $LLVM_RAW_CHARS" | bc)
    else
        LLVM_SAVINGS_CHARS="N/A"
    fi
fi

# ============================================
# tarpaulin comparison
# ============================================

HAS_TARPAULIN=true
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo "${YELLOW}Warning: cargo-tarpaulin not installed, skipping tarpaulin benchmark${NC}"
    HAS_TARPAULIN=false
fi

if [ "$HAS_TARPAULIN" = true ]; then
    TARP_TMPDIR=$(mktemp -d)

    echo "Running cargo tarpaulin -p $COVERAGE_PKG --out json..."
    cargo tarpaulin -p "$COVERAGE_PKG" --out json --output-dir "$TARP_TMPDIR" > /dev/null 2>&1 || true
    if [ -f "$TARP_TMPDIR/tarpaulin-report.json" ]; then
        cp "$TARP_TMPDIR/tarpaulin-report.json" "$TARP_RAW_OUTPUT"
    fi
    rm -rf "$TARP_TMPDIR"
    TARP_RAW_CHARS=$(wc -c < "$TARP_RAW_OUTPUT" | tr -d ' ')
    TARP_RAW_LINES=$(wc -l < "$TARP_RAW_OUTPUT" | tr -d ' ')
    echo "  Output: $TARP_RAW_CHARS characters, $TARP_RAW_LINES lines"
    echo ""

    echo "Running cargo carpulin --tool tarpaulin -- -p $COVERAGE_PKG..."
    "$PROJECT_ROOT/target/release/cargo-carpulin" carpulin --tool tarpaulin -- -p "$COVERAGE_PKG" > "$TARP_CARPULIN_OUTPUT" 2>/dev/null || true
    TARP_CARPULIN_CHARS=$(wc -c < "$TARP_CARPULIN_OUTPUT" | tr -d ' ')
    TARP_CARPULIN_LINES=$(wc -l < "$TARP_CARPULIN_OUTPUT" | tr -d ' ')
    echo "  Output: $TARP_CARPULIN_CHARS characters, $TARP_CARPULIN_LINES lines"
    echo ""

    # Calculate savings
    if [ "$TARP_RAW_CHARS" -gt 0 ]; then
        TARP_SAVINGS_CHARS=$(echo "scale=1; ($TARP_RAW_CHARS - $TARP_CARPULIN_CHARS) * 100 / $TARP_RAW_CHARS" | bc)
    else
        TARP_SAVINGS_CHARS="N/A"
    fi
fi

# ============================================
# Display results
# ============================================

echo "============================================"
echo "Results Summary"
echo "============================================"
echo ""

if [ "$HAS_LLVM_COV" = true ]; then
    echo "--- llvm-cov ---"
    echo "cargo llvm-cov --json:"
    echo "  Characters: $LLVM_RAW_CHARS"
    echo "  Lines:      $LLVM_RAW_LINES"
    echo ""
    echo "cargo carpulin --tool llvm-cov:"
    echo "  Characters: $LLVM_CARPULIN_CHARS"
    echo "  Lines:      $LLVM_CARPULIN_LINES"
    echo "  Character savings: ${LLVM_SAVINGS_CHARS}%"
    echo ""
fi

if [ "$HAS_TARPAULIN" = true ]; then
    echo "--- tarpaulin ---"
    echo "cargo tarpaulin --out json:"
    echo "  Characters: $TARP_RAW_CHARS"
    echo "  Lines:      $TARP_RAW_LINES"
    echo ""
    echo "cargo carpulin --tool tarpaulin:"
    echo "  Characters: $TARP_CARPULIN_CHARS"
    echo "  Lines:      $TARP_CARPULIN_LINES"
    echo "  Character savings: ${TARP_SAVINGS_CHARS}%"
    echo ""
fi

# ============================================
# Generate detailed report
# ============================================

cat > "$REPORT_FILE" << REPORT_EOF
Benchmark Report - cargo carpulin vs raw coverage tools
=======================================================

Date: $(date "+%Y-%m-%d %H:%M:%S")
Coverage Target: $COVERAGE_PKG

=======================================================
REPORT_EOF

if [ "$HAS_LLVM_COV" = true ]; then
    cat >> "$REPORT_FILE" << LLVM_EOF
llvm-cov Comparison
=======================================================

cargo llvm-cov --json:
  Characters: $LLVM_RAW_CHARS
  Lines:      $LLVM_RAW_LINES

cargo carpulin --tool llvm-cov:
  Characters: $LLVM_CARPULIN_CHARS
  Lines:      $LLVM_CARPULIN_LINES
  Character savings: ${LLVM_SAVINGS_CHARS}%

--- cargo llvm-cov output (first 500 chars) ---
$(head -c 500 "$LLVM_RAW_OUTPUT")

--- cargo carpulin (llvm-cov) output ---
$(cat "$LLVM_CARPULIN_OUTPUT")

=======================================================
LLVM_EOF
fi

if [ "$HAS_TARPAULIN" = true ]; then
    cat >> "$REPORT_FILE" << TARP_EOF
tarpaulin Comparison
=======================================================

cargo tarpaulin --out json:
  Characters: $TARP_RAW_CHARS
  Lines:      $TARP_RAW_LINES

cargo carpulin --tool tarpaulin:
  Characters: $TARP_CARPULIN_CHARS
  Lines:      $TARP_CARPULIN_LINES
  Character savings: ${TARP_SAVINGS_CHARS}%

--- cargo tarpaulin output (first 500 chars) ---
$(head -c 500 "$TARP_RAW_OUTPUT")

--- cargo carpulin (tarpaulin) output ---
$(cat "$TARP_CARPULIN_OUTPUT")

=======================================================
TARP_EOF
fi

cat >> "$REPORT_FILE" << ANALYSIS_EOF
Analysis
=======================================================

The cargo carpulin tool transforms verbose coverage JSON into a compact
structured format with per-file uncovered line ranges and summaries.
This reduces output size for easier consumption by editors, CI/CD
pipelines, and AI tools.

Key benefits:
- Uncovered lines grouped into compact ranges (e.g. "35-46" vs 12 segments)
- Uniform output format regardless of underlying tool (llvm-cov or tarpaulin)
- Per-file and overall coverage summaries in a single JSON object
- Reduced token usage for LLM-based tools
ANALYSIS_EOF

echo "Report saved to: $REPORT_FILE"
echo ""
echo "Done!"
