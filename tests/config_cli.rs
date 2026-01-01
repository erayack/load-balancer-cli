use predicates::str::diff;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_config(contents: &str, extension: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be available")
        .as_nanos();
    path.push(format!("lb-config-{}.{}", nanos, extension));
    fs::write(&path, contents).expect("config write should succeed");
    path
}

#[test]
fn config_file_toml_summary_runs() {
    let config = r#"
algo = "round-robin"
requests = 3
tie_break = "seeded"
seed = 42
servers = [
  { name = "a", base_latency_ms = 10, weight = 1 },
  { name = "b", base_latency_ms = 20, weight = 1 }
]
"#;
    let path = write_temp_config(config, "toml");

    let expected = concat!(
        "Metadata:\n",
        "algo: round-robin\n",
        "tie_break: seeded(42)\n",
        "duration_ms: 21\n",
        "Summary:\n",
        "a: 2 requests (avg response: 14ms)\n",
        "b: 1 requests (avg response: 20ms)\n",
    );
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args(["run", "--config", path.to_str().unwrap(), "--summary"]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn repeatable_server_flag_parses() {
    let expected = concat!(
        "Metadata:\n",
        "algo: round-robin\n",
        "tie_break: stable\n",
        "duration_ms: 21\n",
        "Summary:\n",
        "api: 1 requests (avg response: 10ms)\n",
        "db: 1 requests (avg response: 20ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "run",
        "--algo",
        "round-robin",
        "--server",
        "api:10",
        "--server",
        "db:20",
        "--requests",
        "2",
        "--summary",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn empty_servers_csv_with_server_entries_succeeds() {
    let expected = concat!(
        "Metadata:\n",
        "algo: round-robin\n",
        "tie_break: stable\n",
        "duration_ms: 21\n",
        "Summary:\n",
        "web: 1 requests (avg response: 10ms)\n",
        "cache: 1 requests (avg response: 20ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "run",
        "--algo",
        "round-robin",
        "--servers",
        "",
        "--server",
        "web:10",
        "--server",
        "cache:20",
        "--requests",
        "2",
        "--summary",
    ]);
    cmd.assert().success().stdout(diff(expected));
}
