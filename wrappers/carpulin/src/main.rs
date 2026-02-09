use clap::Parser;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// Strip absolute path prefix to make it relative to the current working directory.
/// e.g. "/Users/me/project/src/lib.rs" -> "src/lib.rs" when cwd is "/Users/me/project"
fn make_relative(path: &str) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        Path::new(path)
            .strip_prefix(&cwd)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string())
    } else {
        path.to_string()
    }
}

/// Truncate percentage to 2 decimal places using string formatting
fn round_percent(percent: f64) -> f64 {
    format!("{:.2}", percent).parse().unwrap_or(percent)
}

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cargo {
    #[command(name = "carpulin")]
    Carpulin(Args),
}

#[derive(clap::Args)]
#[command(version, about = "Structured coverage JSON from llvm-cov or tarpaulin")]
#[command(
    long_about = "Runs a coverage tool (`cargo llvm-cov` or `cargo tarpaulin`) and outputs structured \
    JSON showing per-file uncovered line ranges and coverage summaries.\n\n\
    Use --input to parse an existing coverage JSON report instead of running a tool.\n\n\
    All extra arguments are passed through to the underlying coverage tool."
)]
struct Args {
    /// Parse from file or stdin ("-") instead of running a coverage tool
    #[arg(short, long, value_name = "FILE")]
    input: Option<String>,

    /// Coverage tool to use: llvm-cov (default) or tarpaulin
    #[arg(short, long, default_value = "llvm-cov")]
    tool: String,

    /// Arguments passed through to the coverage tool
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

/// Groups sorted line numbers into compact range strings.
/// e.g. `[10, 11, 12, 22, 30, 31]` → `["10-12", "22", "30-31"]`
fn group_into_ranges(lines: &[i64]) -> Vec<String> {
    if lines.is_empty() {
        return vec![];
    }
    let mut ranges = Vec::new();
    let mut start = lines[0];
    let mut end = lines[0];
    for &line in &lines[1..] {
        if line == end + 1 {
            end = line;
        } else {
            if start == end {
                ranges.push(start.to_string());
            } else {
                ranges.push(format!("{}-{}", start, end));
            }
            start = line;
            end = line;
        }
    }
    if start == end {
        ranges.push(start.to_string());
    } else {
        ranges.push(format!("{}-{}", start, end));
    }
    ranges
}

/// Parses `cargo llvm-cov --json` output into structured coverage JSON.
///
/// Segments format: `[line, col, count, hasCount, isRegionEntry, isGap]`
/// A line is uncovered if any segment on that line has count=0 and hasCount=true.
fn parse_llvm_cov(json_str: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let root: Value = serde_json::from_str(json_str)?;
    let data = root["data"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or("missing data[0]")?;

    let totals = &data["totals"];
    let summary = json!({
        "lines": {
            "count": totals["lines"]["count"],
            "covered": totals["lines"]["covered"],
            "percent": round_percent(totals["lines"]["percent"].as_f64().unwrap_or(0.0))
        },
        "functions": {
            "count": totals["functions"]["count"],
            "covered": totals["functions"]["covered"],
            "percent": round_percent(totals["functions"]["percent"].as_f64().unwrap_or(0.0))
        }
    });

    let mut files = Vec::new();
    if let Some(file_list) = data["files"].as_array() {
        for file in file_list {
            let filename = make_relative(file["filename"].as_str().unwrap_or(""));
            let segments = match file["segments"].as_array() {
                Some(s) => s,
                None => continue,
            };

            // Collect uncovered lines: segments where count==0 and hasCount==true
            let mut uncovered: BTreeSet<i64> = BTreeSet::new();
            let mut covered_set: BTreeSet<i64> = BTreeSet::new();
            for seg in segments {
                let arr = match seg.as_array() {
                    Some(a) if a.len() >= 5 => a,
                    _ => continue,
                };
                let line = arr[0].as_i64().unwrap_or(0);
                let count = arr[2].as_i64().unwrap_or(0);
                let has_count = arr[3].as_bool().unwrap_or(false);
                if !has_count {
                    continue;
                }
                if count > 0 {
                    covered_set.insert(line);
                } else {
                    uncovered.insert(line);
                }
            }
            // Remove lines that are also covered (a line can appear in multiple segments)
            for line in &covered_set {
                uncovered.remove(line);
            }

            let uncovered_vec: Vec<i64> = uncovered.into_iter().collect();
            let file_summary = &file["summary"];
            let lines_count = file_summary["lines"]["count"].as_i64().unwrap_or(0);
            let lines_covered = file_summary["lines"]["covered"].as_i64().unwrap_or(0);
            let lines_percent = round_percent(file_summary["lines"]["percent"].as_f64().unwrap_or(0.0));

            files.push(json!({
                "file": filename,
                "coverage": {
                    "lines": {
                        "count": lines_count,
                        "covered": lines_covered,
                        "percent": lines_percent
                    }
                },
                "uncovered_lines": group_into_ranges(&uncovered_vec)
            }));
        }
    }

    Ok(json!({ "summary": summary, "files": files }))
}

/// Parses `cargo tarpaulin --out json` output into structured coverage JSON.
///
/// Traces format: each trace has `line` (i64) and `stats.Line` (count).
/// A line is uncovered if `stats.Line == 0`.
fn parse_tarpaulin(json_str: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let root: Value = serde_json::from_str(json_str)?;

    let mut total_lines: i64 = 0;
    let mut total_covered: i64 = 0;

    let mut files = Vec::new();
    if let Some(file_list) = root["files"].as_array() {
        for file in file_list {
            let path_parts = file["path"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .unwrap_or_default();
            let filename = make_relative(&path_parts);

            let traces = match file["traces"].as_array() {
                Some(t) => t,
                None => continue,
            };

            if traces.is_empty() {
                continue;
            }

            let mut uncovered: Vec<i64> = Vec::new();
            let mut file_lines: i64 = 0;
            let mut file_covered: i64 = 0;

            for trace in traces {
                let line = trace["line"].as_i64().unwrap_or(0);
                let count = trace["stats"]["Line"].as_i64().unwrap_or(0);
                file_lines += 1;
                if count > 0 {
                    file_covered += 1;
                } else {
                    uncovered.push(line);
                }
            }

            total_lines += file_lines;
            total_covered += file_covered;

            uncovered.sort();
            let percent = if file_lines > 0 {
                round_percent((file_covered as f64 / file_lines as f64) * 100.0)
            } else {
                0.0
            };

            files.push(json!({
                "file": filename,
                "coverage": {
                    "lines": {
                        "count": file_lines,
                        "covered": file_covered,
                        "percent": percent
                    }
                },
                "uncovered_lines": group_into_ranges(&uncovered)
            }));
        }
    }

    let total_percent = if total_lines > 0 {
        round_percent((total_covered as f64 / total_lines as f64) * 100.0)
    } else {
        0.0
    };

    let summary = json!({
        "lines": {
            "count": total_lines,
            "covered": total_covered,
            "percent": total_percent
        }
    });

    Ok(json!({ "summary": summary, "files": files }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cargo::Carpulin(args) = Cargo::parse();

    let json_str = match &args.input {
        Some(p) if p == "-" => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            s
        }
        Some(p) => fs::read_to_string(p)?,
        None => {
            let (program, base_args): (&str, Vec<&str>) = match args.tool.as_str() {
                "tarpaulin" => (
                    "cargo",
                    vec!["tarpaulin", "--out", "json", "--output-dir", "-"],
                ),
                _ => ("cargo", vec!["llvm-cov", "--json"]),
            };
            eprintln!("⠿ Running {} {}...", program, base_args.join(" "));
            let mut cmd = Command::new(program);
            cmd.args(&base_args);
            cmd.args(&args.cargo_args);

            // For tarpaulin, --output-dir "-" doesn't work; capture stdout instead
            if args.tool == "tarpaulin" {
                // tarpaulin doesn't support stdout JSON easily,
                // so we use a temp dir approach
                let tmp = std::env::temp_dir().join("carpulin_tarpaulin");
                let _ = fs::create_dir_all(&tmp);
                let mut cmd = Command::new("cargo");
                cmd.arg("tarpaulin")
                    .arg("--out")
                    .arg("json")
                    .arg("--output-dir")
                    .arg(&tmp)
                    .args(&args.cargo_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                let mut child = cmd.spawn()?;
                let stderr = child.stderr.take().expect("capture stderr");
                let stderr_handle = thread::spawn(move || {
                    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                        let _ = writeln!(io::stderr(), "{}", line);
                    }
                });
                let _ = child.wait()?;
                let _ = stderr_handle.join();

                let report_path = tmp.join("tarpaulin-report.json");
                fs::read_to_string(&report_path)?
            } else {
                // llvm-cov outputs JSON to stdout
                let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

                let stderr = child.stderr.take().expect("capture stderr");
                let stderr_handle = thread::spawn(move || {
                    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                        let _ = writeln!(io::stderr(), "{}", line);
                    }
                });
                let mut stdout_buf = String::new();
                child
                    .stdout
                    .take()
                    .expect("capture stdout")
                    .read_to_string(&mut stdout_buf)?;
                let _ = child.wait()?;
                let _ = stderr_handle.join();
                stdout_buf
            }
        }
    };

    eprintln!("⠿ Parsing coverage data...");
    let result = match args.tool.as_str() {
        "tarpaulin" => parse_tarpaulin(&json_str)?,
        _ => parse_llvm_cov(&json_str)?,
    };

    let files_count = result["files"].as_array().map(|a| a.len()).unwrap_or(0);
    eprintln!("✓ Processed {} file(s), outputting JSON...", files_count);
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_into_ranges_empty() {
        assert_eq!(group_into_ranges(&[]), Vec::<String>::new());
    }

    #[test]
    fn test_group_into_ranges_single() {
        assert_eq!(group_into_ranges(&[5]), vec!["5"]);
    }

    #[test]
    fn test_group_into_ranges_consecutive() {
        assert_eq!(group_into_ranges(&[10, 11, 12]), vec!["10-12"]);
    }

    #[test]
    fn test_group_into_ranges_mixed() {
        assert_eq!(
            group_into_ranges(&[10, 11, 12, 22, 30, 31]),
            vec!["10-12", "22", "30-31"]
        );
    }

    #[test]
    fn test_group_into_ranges_all_individual() {
        assert_eq!(group_into_ranges(&[1, 3, 5, 7]), vec!["1", "3", "5", "7"]);
    }

    #[test]
    fn test_parse_llvm_cov_fixture() {
        let fixture =
            fs::read_to_string("../../tools/coverage-test/fixtures/llvm-cov.json").unwrap();
        let result = parse_llvm_cov(&fixture).unwrap();

        // Check summary exists
        let summary = &result["summary"];
        assert!(summary["lines"]["count"].as_i64().unwrap() > 0);
        assert!(summary["lines"]["covered"].as_i64().unwrap() > 0);
        assert!(summary["functions"]["count"].as_i64().unwrap() > 0);

        // Check files array
        let files = result["files"].as_array().unwrap();
        assert!(!files.is_empty());

        // Find our test file
        let test_file = files
            .iter()
            .find(|f| {
                f["file"]
                    .as_str()
                    .unwrap_or("")
                    .contains("coverage-test/src/lib.rs")
            })
            .expect("should find coverage-test/src/lib.rs");

        // Check it has uncovered lines
        let uncovered = test_file["uncovered_lines"].as_array().unwrap();
        assert!(
            !uncovered.is_empty(),
            "should have uncovered lines for partially tested code"
        );

        // Verify never_called lines (35-46) appear as a range
        let unc: Vec<&str> = uncovered.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            unc.iter().any(|s| s.contains("35") || s.contains("36")),
            "include : {:?}",
            unc
        );
    }

    #[test]
    fn test_parse_tarpaulin_fixture() {
        let fixture =
            fs::read_to_string("../../tools/coverage-test/fixtures/tarpaulin.json").unwrap();
        let result = parse_tarpaulin(&fixture).unwrap();

        // Check summary exists
        let summary = &result["summary"];
        assert!(summary["lines"]["count"].as_i64().unwrap() > 0);
        assert!(summary["lines"]["covered"].as_i64().unwrap() > 0);

        // Check files array
        let files = result["files"].as_array().unwrap();
        assert!(!files.is_empty());

        // Find our test file
        let test_file = files
            .iter()
            .find(|f| {
                f["file"]
                    .as_str()
                    .unwrap_or("")
                    .contains("coverage-test/src/lib.rs")
            })
            .expect("should find coverage-test/src/lib.rs");

        // Check it has uncovered lines
        let uncovered = test_file["uncovered_lines"].as_array().unwrap();
        assert!(
            !uncovered.is_empty(),
            "should have uncovered lines for partially tested code"
        );

        // Verify never_called lines appear
        let unc: Vec<&str> = uncovered.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            unc.iter().any(|s| s.contains("35") || s.contains("36")),
            "should include: {:?}",
            unc
        );
    }
}
