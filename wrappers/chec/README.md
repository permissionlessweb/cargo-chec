# Cargo Chec

<center>

![](../../fam/ferris.png)

A cargo subcommand that wraps `cargo check`, filters Rust errors/warnings, and outputs them as a JSON array of strings. Perfect for minimizing character/token count during agentic LLM sessions.

</center>

## Quick Start

1. **Install globally**: `cargo install cargo-chec`
2. **Run in any Rust project**: `cargo chec`

Outputs a JSON array like `["Error (severity 5)...", "Related..."]`. No errors? `[]`.

## Installation

```bash
cargo install cargo-chec
```

Requires Rust and Cargo.

## Features

- **Full cargo check support**: All cargo check flags pass through (--release, --package, --all-targets, etc.)
- **Smart Filtering**: Only shows errors (severity 5) and warnings (severity 4)
- **Structured Output**: JSON array of strings for easy parsing
- **Flexible Input**: Supports files, stdin, or default cargo check
- **Fast & Lean**: Minimal dependencies (clap, serde_json)

## Usage

### Default: Run cargo check

```bash
cd your-rust-project
cargo chec
# Output: ["Error (severity 5) from rustc in src/main.rs at line 1:1-10: Message"]
```

### With cargo check flags

All cargo check flags are supported:

```bash
# Check in release mode
cargo chec --release

# Check a specific package
cargo chec -p my-package

# Check all targets
cargo chec --all-targets

# Combine flags
cargo chec --release --all-targets -p my-package
```

### Custom Input

Parse existing cargo check output instead of running cargo check:

```bash
# From file
cargo chec --input logs.json

# From stdin
cargo check --message-format=json | cargo chec --input -
```

### Output Format

JSON array of strings:

```json
[
  "Error (severity 5) from rustc in src/main.rs at line 10:5-15: cannot find value `x` in this scope",
  "Error (severity 4) from rustc in src/lib.rs at line 5:1-10: unused variable: `y` Related: In src/lib.rs at line 5:1-5: remove this line"
]
```

Empty on no issues: `[]`.

## Troubleshooting

- **Command not found?** Run `cargo install cargo-chec`.
- **No output?** Project has no errors/warnings.
- **Invalid JSON?** If using custom input, ensure valid NDJSON from cargo check.

## Benchmarks

```bash
============================================
Benchmark: cargo check vs cargo chec
============================================

cargo check --message-format=json:
  Characters: 80233
  Lines: 37

cargo chec (errors only):
  Characters: 5130
  Savings: 93.6%

cargo chec --include-warnings:
  Characters: 12397
  Savings: 84.5%
```

---

## For AI Agents

### Source Code

`src/main.rs` (single-file binary)

### Dependencies

- `clap`: CLI argument parsing with cargo subcommand support
- `serde_json`: JSON parsing and serialization

### Build Commands

- **Build**: `cargo build --release -p cargo-chec` -> `target/release/cargo-chec`
- **Lint**: `cargo clippy -p cargo-chec`
- **Format**: `cargo fmt -p cargo-chec`
- **Test**: `cargo test -p cargo-chec`
- **Publish**: `cargo publish -p cargo-chec`

### Runtime Behavior

- **Entry Point**: `main()` in `src/main.rs`
- **Input**: If no `--input`, runs `cargo check --message-format=json` with any additional args passed through
- **Filtering**: Errors (severity 5) and warnings (severity 4) only
- **Output**: JSON array of formatted error strings to stdout
