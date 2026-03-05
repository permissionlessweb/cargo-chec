# Benchmarks

We implement benchmarks by counting the lines and characters returned from each unwrapped subcommand, and then comparing it with the lines & characters from the wrapped subcommands. This gives us a direct comparison between what will be returned to the LLM context during agentic sessions.

Each tool compresses verbose Rust tooling output into compact JSON.

**Last updated:** 2026-03-04 23:03:51

## Summary

| Tool | Raw Output | Filtered Output | Savings |
|------|------------|-----------------|---------|
| cargo-chec (errors only) | 81205 chars, 37 lines | 4547 chars, 1 lines | **94.4%** |
| cargo-tes | 162366 chars, 73 lines | 9281 chars, 1 lines | **94.2%** |
| cargo-carpulin (llvm-cov) | 40249 chars, 2205 lines | 507 chars, 32 lines | **98.7%** |
| cargo-carpulin (tarpaulin) | 53694 chars, 430 lines | 417 chars, 27 lines | **99.2%** |

## cargo-chec

```
cargo check --message-format=json:
  Characters: 81205
  Lines:      37

cargo chec (errors only):
  Characters: 4547
  Lines:      1
  Savings:    94.4%
```

## cargo-tes

```
cargo test --message-format=json:
  Characters: 162366
  Lines:      73

cargo tes:
  Characters: 9281
  Lines:      1
  Savings:    94.2%
```

## cargo-carpulin (llvm-cov)

```
cargo llvm-cov --json:
  Characters: 40249
  Lines:      2205

cargo carpulin --tool llvm-cov:
  Characters: 507
  Lines:      32
  Savings:    98.7%
```

## cargo-carpulin (tarpaulin)

```
cargo tarpaulin --out json:
  Characters: 53694
  Lines:      430

cargo carpulin --tool tarpaulin:
  Characters: 417
  Lines:      27
  Savings:    99.2%
```

## Running Benchmarks

Run the unified benchmark to test all tools and update this file:

```bash
just benchmark
# or
./scripts/benchmark.sh
```

Detailed results are saved to `benchmark_results/benchmark_unified_TIMESTAMP.txt`.
