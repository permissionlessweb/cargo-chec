# Cargo Chec Workspace

<center>

![](fam/fam-pic.png)

A family of cargo subcommands that wrap standard Rust tooling, filter verbose output, and return compact JSON. Built for minimizing character/token count during agentic LLM sessions.

</center>

## Tools

| Crate | Wraps | Install | Docs |
|-------|-------|---------|------|
| **[cargo-chec](wrappers/chec/)** | `cargo check` | `cargo install cargo-chec` | [README](wrappers/chec/README.md) |
| **[cargo-tes](wrappers/tes/)** | `cargo test` | `cargo install cargo-tes` | [README](wrappers/tes/README.md) |
| **[cargo-carpulin](wrappers/carpulin/)** | `cargo llvm-cov` / `cargo tarpaulin` | `cargo install cargo-carpulin` | [README](wrappers/carpulin/README.md) |

### cargo-chec

Filters `cargo check` errors and warnings into a JSON array of strings.

```bash
cargo chec
# ["Error (severity 5) from rustc in src/main.rs at line 10:5-15: cannot find value `x`"]
```

### cargo-tes

Filters `cargo test` failures into a JSON array of strings.

```bash
cargo tes
# ["Test failed: tests::my_test (exec_time: 0.001s) - assertion failed", "Suite failed: passed 5, failed 1 (exec_time: 0.003s)"]
```

### cargo-carpulin

Runs coverage tools and outputs structured JSON with per-file uncovered line ranges.

```bash
cargo carpulin --tool llvm-cov -- -p my-crate
```

```json
{
  "summary": {
    "lines": { "count": 55, "covered": 30, "percent": 54.5 },
    "functions": { "count": 10, "covered": 8, "percent": 80.0 }
  },
  "files": [
    {
      "file": "src/lib.rs",
      "coverage": { "lines": { "count": 55, "covered": 30, "percent": 54.5 } },
      "uncovered_lines": ["16-19", "28-30", "35-46", "49-54"]
    }
  ]
}
```

## Workspace Layout

```
.
├── wrappers/
│   ├── chec/          # cargo-chec   — cargo check filter
│   ├── tes/           # cargo-tes    — cargo test filter
│   └── carpulin/      # cargo-carpulin — coverage report filter
├── tools/
│   ├── coverage-test/ # Fixture crate with intentional coverage gaps
│   └── demo-outputs/  # Demo output crate for cargo-tes
├── scripts/
│   ├── benchmark.sh           # cargo check vs cargo chec
│   ├── benchmark_tes.sh       # cargo test vs cargo tes
│   └── benchmark_carpulin.sh  # raw coverage vs cargo carpulin
├── fam/               # Project images
├── Justfile           # Release task runner
└── Cargo.toml         # Workspace root
```

## Benchmarks

Learn about [benchmarking this workspace.](./BENCHMARKS.md)

Run benchmarks locally:

```bash
./scripts/benchmark.sh
./scripts/benchmark_tes.sh
./scripts/benchmark_carpulin.sh
```

## Development

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Lint
cargo clippy --workspace

# Format
cargo fmt --all
```

## Releasing

Use the provided Justfile for release tasks:

```bash
# Run all checks
just check

# Dry-run publish
just dry-run

# Full release
just release
```

Requires `just` (install via `cargo install just`).

## Contributing

Open issues/PRs on GitHub. Built for the Rust ecosystem.

---

## For AI Agents: Workspace Specifications

### Workspace Members

| Member | Type | Path |
|--------|------|------|
| `cargo-chec` | Binary (wrapper) | `wrappers/chec/src/main.rs` |
| `cargo-tes` | Binary (wrapper) | `wrappers/tes/src/main.rs` |
| `cargo-carpulin` | Binary (wrapper) | `wrappers/carpulin/src/main.rs` |
| `demo-outputs` | Library (fixture) | `tools/demo-outputs/src/lib.rs` |
| `coverage-test-crate` | Library (fixture) | `tools/coverage-test/src/lib.rs` |

### Shared Patterns

All wrapper crates follow the same structure:

- Single-file binary in `src/main.rs`
- Dependencies: `clap` 4.0 (derive) + `serde_json` 1.0
- Clap subcommand enum for `cargo <name>` invocation
- `--input FILE` or `-` for stdin to parse existing output
- Trailing `cargo_args` passed through to the underlying tool
- Piped stdout/stderr with stderr streaming thread

### Build Commands

- **Build all**: `cargo build`
- **Build one**: `cargo build -p cargo-chec`
- **Test all**: `cargo test`
- **Test one**: `cargo test -p cargo-carpulin`
- **Benchmark**: `./scripts/benchmark_carpulin.sh`
