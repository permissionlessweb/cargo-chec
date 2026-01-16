use clap::Parser;
use cw_jsonfilter::{CwJsonFilter, decoder::NoopDecoder};
use serde_json::{Value, json};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "cargo-chec")]
#[command(about = "Cargo subcommand to check and filter Rust errors")]
struct Args {
    /// Input file path (optional; use - for stdin, or omit to run cargo check)
    #[arg(short, long)]
    input: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read input
    let json_str = if let Some(path) = &args.input {
        if path == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else {
            fs::read_to_string(path)?
        }
    } else {
        // Run cargo check
        let output = std::process::Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .output()?;
        if !output.status.success() {
            eprintln!("Cargo check failed");
        }
        String::from_utf8(output.stdout)?
    };

    // Parse JSON (handle NDJSON from cargo check)
    let logs: Vec<Value> = if json_str.trim().is_empty() {
        vec![]
    } else {
        json_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).map_err(|_| "Invalid JSON input."))
            .collect::<Result<Vec<_>, _>>()?
    };

    // Filter for severity >= 4 (warnings/errors)
    let filter = json!({"severity": {"$gte": 4}});
    let cwjf: CwJsonFilter<NoopDecoder> = CwJsonFilter::new(None);

    let mut results = Vec::new();
    for log in logs {
        if cwjf.matches(&filter, &log).is_pass() {
            let error_str = format_error(&log);
            results.push(error_str);
        }
    }

    // Output as JSON array
    println!("{}", serde_json::to_string(&results)?);

    Ok(())
}

fn format_error(log: &Value) -> String {
    let resource = log["resource"].as_str().unwrap_or("unknown");
    let severity = log["severity"].as_i64().unwrap_or(0);
    let message = log["message"].as_str().unwrap_or("");
    let source = log["source"].as_str().unwrap_or("");
    let start_line = log["startLineNumber"].as_i64().unwrap_or(0);
    let start_col = log["startColumn"].as_i64().unwrap_or(0);
    let _end_line = log["endLineNumber"].as_i64().unwrap_or(0);
    let end_col = log["endColumn"].as_i64().unwrap_or(0);

    let mut output = format!(
        "Error (severity {}) from {} in {} at line {}:{}-{}: {}",
        severity, source, resource, start_line, start_col, end_col, message
    );

    // Related information
    if let Some(related) = log["relatedInformation"].as_array() {
        for rel in related {
            let rel_msg = rel["message"].as_str().unwrap_or("");
            let rel_res = rel["resource"].as_str().unwrap_or("");
            let rel_start = rel["startLineNumber"].as_i64().unwrap_or(0);
            let rel_start_col = rel["startColumn"].as_i64().unwrap_or(0);
            let _rel_end = rel["endLineNumber"].as_i64().unwrap_or(0);
            let rel_end_col = rel["endColumn"].as_i64().unwrap_or(0);
            output.push_str(&format!(
                " Related: In {} at line {}:{}-{}: {}",
                rel_res, rel_start, rel_start_col, rel_end_col, rel_msg
            ));
        }
    }

    output
}
