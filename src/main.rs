use clap::Parser;
use serde_json::{json, Value};
use std::{fs,io::{self, Read}};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cargo {
    #[command(name = "chec")]
    Chec(Args),
}

#[derive(clap::Args)]
#[command(version,about = "Filtered cargo check errors/warnings, as JSON")]
#[command(
    long_about = "Runs `cargo check --message-format=json` and transforms the output into a \
    simplified JSON array of error strings. Useful for CI/CD pipelines, editors, and AI tools.\n\n\
    All cargo check flags are supported and passed through (e.g. --release, --package, --all-targets).\n\n\
    Use --input to parse existing cargo check output instead of running cargo check."
)]
struct Args {
    /// Parse from file or stdin ("-") instead of running cargo check
    #[arg(short, long, value_name = "FILE")]
    input: Option<String>,

    /// Arguments passed through to cargo check (e.g. --release, -p foo)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,

    /// Include warnings in the output
    #[arg(long)]
    include_warnings: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cargo::Chec(args) = Cargo::parse();

    let (json_str, failure_opt) = match &args.input {
        Some(p) if p == "-" => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            (s, None)
        }
        Some(p) => (fs::read_to_string(p)?, None),
        None => {
            let output = std::process::Command::new("cargo")
                .arg("check")
                .arg("--message-format=json")
                .args(&args.cargo_args)
                .output()?;
            let status = output.status;
            let stderr = output.stderr;
            (String::from_utf8(output.stdout)?, Some((status, stderr)))
        }
    };

    let mut results: Vec<String> = json_str.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|log| {
            let msg = log.get("message").filter(|_| log["reason"] == "compiler-message")?;
            let severity = match msg["level"].as_str()? { 
                "error" => 5, 
                "warning" if args.include_warnings => 4, 
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

            let mut out = format!("Error (severity {}) from rustc in {} at line {}:{}-{}: {}",
                severity, resource, sl, sc, ec, message);
            for r in &related {
                out.push_str(&format!(" Related: In {} at line {}:{}-{}: {}",
                    r["resource"].as_str().unwrap_or(""), r["startLineNumber"], r["startColumn"],
                    r["endColumn"], r["message"].as_str().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ")));
            }
            Some(out)
        }).collect();

    if let Some((status, stderr)) = failure_opt {
        if !status.success() {
            results.push(format!(
                "Cargo check failed with exit code {}: {}",
                status.code().unwrap_or(-1),
                String::from_utf8_lossy(&stderr)
            ));
        }
    }

    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}
