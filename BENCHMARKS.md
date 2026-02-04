# Benchmarks

We implement benchmarks by counting the loc and characters returned from each unwrapped subcommand, and then comapring it with the loc & characters from the wrapped subcommands. This gives us a direct comparison between what will be returned to the llm context during agentic sessions.

Each tool compresses verbose Rust tooling output into compact JSON:

| Tool | Raw Output | Filtered Output | Savings |
|------|-----------|----------------|---------|
| cargo-chec (errors only) | 80,233 chars | 5,130 chars | **93.6%** |
| cargo-chec (with warnings) | 80,233 chars | 12,397 chars | **84.5%** |
| cargo-tes | 161,502 chars | 10,515 chars | **93.4%** |
| cargo-carpulin (llvm-cov) | 8,887 chars | 565 chars | **93.6%** |
| cargo-carpulin (tarpaulin) | 42,610 chars | 479 chars | **98.8%** |

## Cargo Chec

```sh
============================================
Character Count Comparison
============================================

cargo check --message-format=json:
  Characters: 81745
  Lines: 37

cargo chec (errors only):
  Characters: 5144
  Lines: 1
  Character savings: 93.7%

cargo chec --include-warnings:
  Characters: 12411
  Lines: 1
  Character savings: 84.8%
```

## Cargo Test

```sh
============================================
Results Summary
============================================

cargo test --message-format=json -- -Z unstable-options --format=json:
  Time: 0m0.203s
  Characters: 163446
  Lines: 73

cargo tes:
  Time: 0m0.276s
  Characters: 9839
  Lines: 1
  Character savings: 93.9%

```

## Cargo Carpulin

```sh
=======================================================
llvm-cov Comparison
=======================================================

cargo llvm-cov --json:
  Characters: 8887
  Lines:      0

cargo carpulin --tool llvm-cov:
  Characters: 565
  Lines:      32
  Character savings: 93.6%

=======================================================
tarpaulin Comparison
=======================================================

cargo tarpaulin --out json:
  Characters: 42610
  Lines:      0

cargo carpulin --tool tarpaulin:
  Characters: 479
  Lines:      27
  Character savings: 98.8%
```
