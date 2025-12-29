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

    let expected =
        "Summary:\na: 2 requests (avg response: 10ms)\nb: 1 requests (avg response: 20ms)\n";
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args(["--config", path.to_str().unwrap(), "--summary"]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn repeatable_server_flag_parses() {
    let expected =
        "Summary:\napi: 1 requests (avg response: 10ms)\ndb: 1 requests (avg response: 20ms)\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
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
