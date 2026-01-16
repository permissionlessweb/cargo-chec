use clap::Parser;
use cw_jsonfilter::{decoder::NoopDecoder, CwJsonFilter};
use serde_json::{json, Value};
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

        String::from_utf8(output.stdout)?
    };

    // Parse JSON (handle NDJSON from cargo check)
    let logs: Vec<Value> = if json_str.trim().is_empty() {
        vec![]
    } else {
        json_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    };

    // Transform cargo JSON to expected format
    let transformed_logs: Vec<Value> = logs
        .into_iter()
        .filter_map(|log| {
            if log.get("reason")?.as_str()? != "compiler-message" {
                return None;
            }
            let msg = log.get("message")?;
            let level = msg.get("level")?.as_str()?;
            let severity = match level {
                "error" => 5,
                "warning" => 4,
                _ => return None,
            };
            let message = msg.get("rendered")?.as_str()?;
            let spans = msg.get("spans")?.as_array()?;
            if spans.is_empty() {
                return None;
            }
            let span = &spans[0];
            let resource = span.get("file_name")?.as_str()?;
            let start_line = span.get("line_start")?.as_i64()?;
            let start_col = span.get("column_start")?.as_i64()?;
            let end_line = span.get("line_end")?.as_i64()?;
            let end_col = span.get("column_end")?.as_i64()?;
            let source = "rustc";
            let related = if let Some(children) = msg.get("children")?.as_array() {
                children
                    .iter()
                    .filter_map(|child| {
                        let child_msg = child.get("message")?.as_str()?;
                        let child_spans = child.get("spans")?.as_array()?;
                        if child_spans.is_empty() {
                            return None;
                        }
                        let child_span = &child_spans[0];
                        let rel_res = child_span.get("file_name")?.as_str()?;
                        let rel_start = child_span.get("line_start")?.as_i64()?;
                        let rel_start_col = child_span.get("column_start")?.as_i64()?;
                        let rel_end = child_span.get("line_end")?.as_i64()?;
                        let rel_end_col = child_span.get("column_end")?.as_i64()?;
                        Some(json!({
                            "message": child_msg,
                            "resource": rel_res,
                            "startLineNumber": rel_start,
                            "startColumn": rel_start_col,
                            "endLineNumber": rel_end,
                            "endColumn": rel_end_col
                        }))
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            Some(json!({
                "resource": resource,
                "severity": severity,
                "message": message,
                "source": source,
                "startLineNumber": start_line,
                "startColumn": start_col,
                "endLineNumber": end_line,
                "endColumn": end_col,
                "relatedInformation": related
            }))
        })
        .collect();

    // Filter for severity >= 4 (warnings/errors)
    let filter = json!({"severity": {"$gte": 4}});
    let cwjf: CwJsonFilter<NoopDecoder> = CwJsonFilter::new(None);

    let mut results = Vec::new();
    for log in transformed_logs {
        if cwjf.matches(&filter, &log).is_pass() {
            let error_str = format_error(&log);
            results.push(error_str);
        }
    }

    // Output as JSON array
    println!("{}", serde_json::to_string(&results)?);

    Ok(())
}

fn clean_message(msg: &str) -> String {
    msg.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn format_error(log: &Value) -> String {
    let resource = log["resource"].as_str().unwrap_or("unknown");
    let severity = log["severity"].as_i64().unwrap_or(0);
    let message = clean_message(log["message"].as_str().unwrap_or(""));
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
            let rel_msg = clean_message(rel["message"].as_str().unwrap_or(""));
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
