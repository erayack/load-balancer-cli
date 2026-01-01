use predicates::str::diff;

#[test]
fn list_algorithms_prints_supported_values() {
    let expected = concat!(
        "round-robin\n",
        "weighted-round-robin\n",
        "least-connections\n",
        "least-response-time\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.arg("list-algorithms");
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn show_config_prints_parsed_configuration() {
    let expected = concat!(
        "Algorithm: round-robin\n",
        "Requests: 3\n",
        "Tie-break: seeded(42)\n",
        "Servers:\n",
        "- api (latency: 10ms, weight: 1)\n",
        "- db (latency: 20ms, weight: 2)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "show-config",
        "--algo",
        "round-robin",
        "--servers",
        "api:10,db:20:2",
        "--requests",
        "3",
        "--seed",
        "42",
    ]);
    cmd.assert().success().stdout(diff(expected));
}
