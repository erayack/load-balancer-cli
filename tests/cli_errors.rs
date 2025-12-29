use predicates::str::contains;

#[test]
fn requests_zero_fails() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "a:10",
        "--requests",
        "0",
    ]);
    cmd.assert()
        .failure()
        .stderr(contains("Error: requests must be greater than 0"));
}

#[test]
fn empty_servers_fails() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args(["--algo", "round-robin", "--servers", "", "--requests", "1"]);
    cmd.assert()
        .failure()
        .stderr(contains("Error: servers must not be empty"));
}

#[test]
fn invalid_latency_fails() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "api:ten",
        "--requests",
        "1",
    ]);
    cmd.assert()
        .failure()
        .stderr(contains("Error: invalid latency in 'api:ten'"));
}

#[test]
fn duplicate_server_names_fail() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "a:10",
        "--server",
        "a:20",
        "--requests",
        "1",
    ]);
    cmd.assert()
        .failure()
        .stderr(contains("Error: duplicate server name 'a'"));
}
