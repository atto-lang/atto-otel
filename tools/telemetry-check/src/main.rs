use serde_json::Value;
use std::{error::Error, path::PathBuf, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn execute(root: &std::path::Path, source: &std::path::Path) -> Result<String> {
    let output = Command::new(root.join("scripts/compiler.sh"))
        .arg(source)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "guest execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    return Ok(String::from_utf8(output.stdout)?);
}

fn check_smoke(text: &str) -> Result<()> {
    let rows = text
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if rows.len() != 9 || rows.iter().any(|row| !row.is_object()) {
        return Err("expected exactly nine telemetry records".into());
    }
    let logs: Vec<_> = rows.iter().filter(|r| r["signal"] == "log").collect();
    let spans: Vec<_> = rows.iter().filter(|r| r["signal"] == "span").collect();
    let metrics: Vec<_> = rows.iter().filter(|r| r["signal"] == "metric").collect();
    if spans.len() != 2 || logs.len() != 6 || metrics.len() != 1 {
        return Err(
            "incorrect signal counts (including repeated finish, filtered and after-finish events)"
                .into(),
        );
    }
    if spans[0]["name"] != "child" || spans[0]["parent_span_id"] != spans[1]["span_id"] {
        return Err("invalid child correlation".into());
    }
    for row in &rows {
        if row["schema"] != "oreslang-otel.v1"
            || row["ores_trace_id"] != "ores-trace-test"
            || row["timestamp_unix_ms"].as_i64().unwrap_or(0) <= 1700000000000
        {
            return Err("invalid schema, trace or epoch timestamp".into());
        }
    }
    if spans
        .iter()
        .any(|r| r["duration_ns"].as_i64().unwrap_or(-1) < 0)
    {
        return Err("invalid span duration".into());
    }
    let controls: String = (0_u8..32).map(char::from).collect();
    let expected = [
        "quote\" slash\\ newline\n nul\0 tab\t emoji😀",
        controls.as_str(),
        "-9223372036854775808|9223372036854775807|0|-1|100",
        "visible warning",
        "visible all",
        "visible equal",
    ];
    for (log, body) in logs.iter().zip(expected) {
        if log["body"] != body {
            return Err(format!("incorrect log body: {}", log["body"]).into());
        }
    }
    if metrics[0]["value"].as_i64() != Some(1) {
        return Err("incorrect counter value".into());
    }
    return Ok(());
}

fn integers(root: &std::path::Path) -> Result<()> {
    let mut values = vec![
        i64::MIN,
        i64::MAX,
        0,
        -1,
        1,
        9007199254740992,
        9007199254740993,
    ];
    let mut power = 10_i64;
    for exponent in 1..=18 {
        values.extend([power - 1, power, power + 1, -power - 1, -power, -power + 1]);
        if exponent < 18 {
            power *= 10;
        }
    }
    let mut state = 0x123456789abcdef0_u64;
    for _ in 0..100 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.push(state as i64);
    }
    let mut source =
        String::from("import module otel from \"../src/otel\";\npub routine main(): void {\n");
    for value in &values {
        let literal = if *value == i64::MIN {
            "(-9223372036854775807 - 1)".to_owned()
        } else {
            value.to_string()
        };
        source.push_str(&format!("  stdio.println(otel.integer({literal}));\n"));
    }
    source.push_str("  return;\n}\n");
    std::fs::create_dir_all(root.join(".cache"))?;
    let path = root.join(".cache/integer-parity.ores");
    std::fs::write(&path, source)?;
    let result = execute(root, &path)?;
    let lines: Vec<_> = result.lines().collect();
    if lines.len() != values.len() {
        return Err("missing integer parity output".into());
    }
    for (actual, value) in lines.into_iter().zip(&values) {
        if actual != value.to_string() {
            return Err(format!("integer parity: expected {value}, received {actual}").into());
        }
    }
    println!(
        "PASS: {} exact int64 boundaries and deterministic random samples",
        values.len()
    );
    return Ok(());
}

fn run() -> Result<()> {
    if std::env::args_os().len() != 1 {
        return Err("this check accepts no arguments".into());
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    check_smoke(&execute(&root, &root.join("tests/smoke.ores"))?)?;
    println!(
        "PASS: native spans, correlation, idempotence, JSON escaping, int64 and severity gating"
    );
    integers(&root)?;
    return Ok(());
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_rejects_unrecognized_extra_telemetry_records() {
        use serde_json::json;
        let base = json!({"schema":"oreslang-otel.v1", "ores_trace_id":"ores-trace-test",
                          "timestamp_unix_ms":1700000000001_i64});
        let controls: String = (0_u8..32).map(char::from).collect();
        let bodies = [
            "quote\" slash\\ newline\n nul\0 tab\t emoji😀",
            controls.as_str(),
            "-9223372036854775808|9223372036854775807|0|-1|100",
            "visible warning",
            "visible all",
            "visible equal",
        ];
        let mut rows = Vec::new();
        for body in bodies {
            let mut row = base.clone();
            row["signal"] = json!("log");
            row["body"] = json!(body);
            rows.push(row);
        }
        for (name, id, parent) in [("child", "child-id", "root-id"), ("request", "root-id", "")] {
            let mut row = base.clone();
            row["signal"] = json!("span");
            row["name"] = json!(name);
            row["span_id"] = json!(id);
            row["parent_span_id"] = json!(parent);
            row["duration_ns"] = json!(1);
            rows.push(row);
        }
        let mut metric = base.clone();
        metric["signal"] = json!("metric");
        metric["value"] = json!(1);
        rows.push(metric);
        let encode = |rows: &[serde_json::Value]| {
            rows.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(super::check_smoke(&encode(&rows)).is_ok());
        let mut unknown = base;
        unknown["signal"] = json!("unexpected");
        rows.push(unknown);
        assert!(super::check_smoke(&encode(&rows)).is_err());
    }

    #[test]
    fn malformed_or_incomplete_records_fail_closed() {
        assert!(super::check_smoke("{broken}").is_err());
        assert!(super::check_smoke("").is_err());
        assert!(super::check_smoke("{}").is_err());
    }
}
