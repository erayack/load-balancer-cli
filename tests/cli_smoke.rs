use predicates::str::diff;

#[test]
fn summary_round_robin_is_stable() {
    let expected =
        "Summary:\na: 2 requests (avg response: 10ms)\nb: 1 requests (avg response: 20ms)\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "a:10",
        "--server",
        "b:20",
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
    let expected =
        "Summary:\nfast: 2 requests (avg response: 10ms)\nslow: 0 requests (avg response: 0ms)\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "least-response-time",
        "--server",
        "fast:10",
        "--server",
        "slow:30",
        "--requests",
        "2",
        "--summary",
        "--seed",
        "7",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_preserves_input_order() {
    let expected = "Summary:\nz: 1 requests (avg response: 10ms)\na: 0 requests (avg response: 0ms)\nm: 0 requests (avg response: 0ms)\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "z:10",
        "--server",
        "a:20",
        "--server",
        "m:30",
        "--requests",
        "1",
        "--summary",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_preserves_input_order_for_least_connections() {
    let expected = "Summary:\nfirst: 1 requests (avg response: 10ms)\nsecond: 1 requests (avg response: 20ms)\nthird: 2 requests (avg response: 30ms)\n";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "least-connections",
        "--server",
        "first:10",
        "--server",
        "second:20",
        "--server",
        "third:30",
        "--requests",
        "4",
        "--summary",
        "--seed",
        "11",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn full_output_least_response_time_includes_scores() {
    let expected = concat!(
        "Tie-break: seeded(7)\n",
        "Request 1 -> a (score: 10ms)\n",
        "Request 2 -> b (score: 10ms)\n",
        "Request 3 -> a (score: 20ms)\n",
        "Summary:\n",
        "a: 2 requests (avg response: 10ms)\n",
        "b: 1 requests (avg response: 10ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "least-response-time",
        "--server",
        "a:10",
        "--server",
        "b:10",
        "--requests",
        "3",
        "--seed",
        "7",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn full_output_round_robin_omits_scores() {
    let expected = concat!(
        "Tie-break: seeded(99)\n",
        "Request 1 -> a\n",
        "Request 2 -> b\n",
        "Request 3 -> a\n",
        "Summary:\n",
        "a: 2 requests (avg response: 10ms)\n",
        "b: 1 requests (avg response: 20ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("load-balancer-cli");
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "a:10",
        "--server",
        "b:20",
        "--requests",
        "3",
        "--seed",
        "99",
    ]);
    cmd.assert().success().stdout(diff(expected));
}
