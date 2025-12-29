use predicates::str::diff;

#[test]
fn summary_round_robin_is_stable() {
    let expected = "Summary:\na: 2 requests\nb: 1 requests\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--servers",
        "a:10,b:20",
        "--requests",
        "3",
        "--summary",
        "--seed",
        "42",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_least_response_time_is_stable() {
    let expected = "Summary:\nfast: 2 requests\nslow: 0 requests\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "least-response-time",
        "--servers",
        "fast:10,slow:30",
        "--requests",
        "2",
        "--summary",
        "--seed",
        "7",
    ]);
    cmd.assert().success().stdout(diff(expected));
}
