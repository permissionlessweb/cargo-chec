use clap::Parser;
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
    thread,
};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cargo {
    #[command(name = "tes")]
    Tes(TestArgs),
}

#[derive(clap::Args)]
#[command(version, about = "Filtered cargo test failures, as JSON")]
#[command(
    long_about = "Runs `cargo test --message-format=json -- --format=json` and transforms the output into a \
    simplified JSON array of test failure strings. Useful for CI/CD pipelines, editors, and AI tools.\n\n\
    All cargo test flags are supported and passed through (e.g. --release, --package, --all-targets).\n\n\
    Use --input to parse existing cargo test output instead of running cargo test."
)]
struct TestArgs {
    /// Parse from file or stdin ("-") instead of running cargo test
    #[arg(short, long, value_name = "FILE")]
    input: Option<String>,

    /// Arguments passed through to cargo test (e.g. --release, -p foo)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,

    /// Include ignored tests in the output as warnings
    #[arg(long)]
    include_ignored: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cargo::Tes(args) = Cargo::parse();

    let (json_str, failure_status) = match &args.input {
        Some(p) if p == "-" => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            (s, None)
        }
        Some(p) => (fs::read_to_string(p)?, None),
        None => {
            // Split args: cargo flags before '--', test flags after
            let (cargo_flags, test_flags): (Vec<_>, Vec<_>) = args.cargo_args.iter()
                .partition(|arg| !arg.starts_with("--nocapture") && !arg.starts_with("--show-output"));

            let mut child = Command::new("cargo")
                .arg("test")
                .arg("--message-format=json")
                .args(cargo_flags)
                .arg("--")
                .arg("-Z")
                .arg("unstable-options")
                .arg("--format=json")
                .args(test_flags)
                .env("CARGO_TERM_COLOR", "always")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            // Take ownership of stderr for streaming in a separate thread
            let stderr = child.stderr.take().expect("capture stderr");
            let stderr_handle = thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if !line.trim().starts_with("Running ") {
                        let _ = writeln!(io::stderr(), "{}", line);
                    }
                }
            });
            let stdout: Vec<_> = BufReader::new(child.stdout.take().expect("capture stdout"))
                .lines()
                .map_while(Result::ok)
                .collect();
            let status = child.wait()?;
            let _ = stderr_handle.join();
            let json_str = stdout.join("\n");
            (json_str, Some(status))
        }
    };

    let mut results: Vec<String> = json_str.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|log| {
            // Handle compiler messages (same as check)
            if let Some(msg) = log.get("message").filter(|_| log["reason"] == "compiler-message") {
                let (severity, label) = match msg["level"].as_str()? {
                    "error" => (5, "Error"),
                    "warning" if args.include_ignored => (4, "Warning"), // Note: treating warnings as optional
                    _ => return None
                };
                let span = msg["spans"].as_array()?.first()?;
                let resource = span["file_name"].as_str()?;
                let (sl, sc, ec) = (span["line_start"].as_i64()?, span["column_start"].as_i64()?, span["column_end"].as_i64()?);
                let message: String = msg["rendered"].as_str()?.split_whitespace().collect::<Vec<_>>().join(" ");

                let related: Vec<Value> = msg["children"].as_array().unwrap_or(&vec![]).iter().filter_map(|c| {
                    let sp = c["spans"].as_array()?.first()?;
                    Some(json!({"message": c["message"].as_str()?, "resource": sp["file_name"].as_str()?,
                        "startLineNumber": sp["line_start"], "startColumn": sp["column_start"],
                        "endLineNumber": sp["line_end"], "endColumn": sp["column_end"]}))
                }).collect();

                let mut out = format!("{} (severity {}) from rustc in {} at line {}:{}-{}: {}",
                    label, severity, resource, sl, sc, ec, message);
                for r in &related {
                    out.push_str(&format!(" Related: In {} at line {}:{}-{}: {}",
                        r["resource"].as_str().unwrap_or(""), r["startLineNumber"], r["startColumn"],
                        r["endColumn"], r["message"].as_str().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ")));
                }
                Some(out)
            } else if log["type"] == "test" && log["event"] == "failed" {
                // Handle failed tests
                let name = log["name"].as_str()?;
                let exec_time = log["exec_time"].as_f64().unwrap_or(0.0);
                let stdout = log["stdout"].as_str().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
                Some(format!("Test failed: {} (exec_time: {:.3}s) - {}", name, exec_time, stdout))
            } else if log["type"] == "suite" && log["event"] == "failed" {
                // Handle failed suites
                let passed = log["passed"].as_i64()?;
                let failed = log["failed"].as_i64()?;
                let exec_time = log["exec_time"].as_f64()?;
                Some(format!("Suite failed: passed {}, failed {} (exec_time: {:.3}s)", passed, failed, exec_time))
            } else {
                None
            }
        }).collect();

    if let Some(status) = failure_status {
        if !status.success() {
            results.push(format!(
                "Cargo test failed with exit code {}",
                status.code().unwrap_or(-1)
            ));
        }
    }

    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}
