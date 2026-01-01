use predicates::str::diff;

#[test]
fn summary_round_robin_is_stable() {
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
    cmd.args([
        "--algo",
        "round-robin",
        "--server",
        "a:10",
        "--server",
        "b:20",
        "--requests",
        "3",
        "--format",
        "summary",
        "--seed",
        "42",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_least_response_time_is_stable() {
    let expected = concat!(
        "Metadata:\n",
        "algo: least-response-time\n",
        "tie_break: seeded(7)\n",
        "duration_ms: 20\n",
        "Summary:\n",
        "fast: 2 requests (avg response: 14ms)\n",
        "slow: 0 requests (avg response: 0ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "--algo",
        "least-response-time",
        "--server",
        "fast:10",
        "--server",
        "slow:30",
        "--requests",
        "2",
        "--format",
        "summary",
        "--seed",
        "7",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_preserves_input_order() {
    let expected = concat!(
        "Metadata:\n",
        "algo: round-robin\n",
        "tie_break: stable\n",
        "duration_ms: 10\n",
        "Summary:\n",
        "z: 1 requests (avg response: 10ms)\n",
        "a: 0 requests (avg response: 0ms)\n",
        "m: 0 requests (avg response: 0ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
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
        "--format",
        "summary",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn summary_preserves_input_order_for_least_connections() {
    let expected = concat!(
        "Metadata:\n",
        "algo: least-connections\n",
        "tie_break: seeded(11)\n",
        "duration_ms: 42\n",
        "Summary:\n",
        "first: 1 requests (avg response: 10ms)\n",
        "second: 2 requests (avg response: 29ms)\n",
        "third: 1 requests (avg response: 30ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
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
        "--format",
        "summary",
        "--seed",
        "11",
    ]);
    cmd.assert().success().stdout(diff(expected));
}

#[test]
fn full_output_least_response_time_includes_scores() {
    let expected = concat!(
        "Metadata:\n",
        "algo: least-response-time\n",
        "tie_break: seeded(7)\n",
        "duration_ms: 20\n",
        "Assignments:\n",
        "Request 1 -> a (score: 10ms)\n",
        "Request 2 -> b (score: 11ms)\n",
        "Request 3 -> a (score: 20ms)\n",
        "Summary:\n",
        "a: 2 requests (avg response: 14ms)\n",
        "b: 1 requests (avg response: 10ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "--format",
        "human",
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
        "Metadata:\n",
        "algo: round-robin\n",
        "tie_break: seeded(99)\n",
        "duration_ms: 21\n",
        "Assignments:\n",
        "Request 1 -> a\n",
        "Request 2 -> b\n",
        "Request 3 -> a\n",
        "Summary:\n",
        "a: 2 requests (avg response: 14ms)\n",
        "b: 1 requests (avg response: 20ms)\n",
    );

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lb-sim");
    cmd.args([
        "--format",
        "human",
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
