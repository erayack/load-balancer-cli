# Phase 1 Metrics Report

Deterministic runs with seed 42 and 100 requests. Response time percentiles use end-to-end latency (arrival -> completion).

## Fixed arrivals (1 request/ms)

### round-robin

Command:

```bash
cargo run -- run --algo round-robin --servers a:10,b:20,c:30 --requests 100 --seed 42 --format json
```

Metrics:

- duration_ms: 992
- p95_ms: 759
- p99_ms: 867
- avg_wait_ms: 271
- jain_fairness: 0.9998
- throughput_rps: 100.81
- per_server_utilization (%):
  - a: 34.27
  - b: 66.53
  - c: 99.8

### weighted-round-robin

Command:

```bash
cargo run -- run --algo weighted-round-robin --servers a:10:5,b:20:2,c:30:1 --requests 100 --seed 42 --format json
```

Metrics:

- duration_ms: 640
- p95_ms: 499
- p99_ms: 532
- avg_wait_ms: 229
- jain_fairness: 0.6921
- throughput_rps: 156.25
- per_server_utilization (%):
  - a: 100.0
  - b: 75.0
  - c: 56.25

### least-connections

Command:

```bash
cargo run -- run --algo least-connections --servers a:10,b:20,c:30 --requests 100 --seed 42 --format json
```

Metrics:

- duration_ms: 931
- p95_ms: 697
- p99_ms: 807
- avg_wait_ms: 256
- jain_fairness: 0.9938
- throughput_rps: 107.41
- per_server_utilization (%):
  - a: 39.74
  - b: 68.74
  - c: 99.89

### least-response-time

Command:

```bash
cargo run -- run --algo least-response-time --servers a:10,b:20,c:30 --requests 100 --seed 42 --format json
```

Metrics:

- duration_ms: 550
- p95_ms: 428
- p99_ms: 446
- avg_wait_ms: 216
- jain_fairness: 0.8174
- throughput_rps: 181.82
- per_server_utilization (%):
  - a: 100.0
  - b: 98.18
  - c: 98.18


## Burst arrivals (100 at t=0)

### round-robin

Command:

```bash
cargo run -- run --algo round-robin --servers a:10,b:20,c:30 --burst 100 --burst-at 0 --seed 42 --format json
```

Metrics:

- duration_ms: 990
- p95_ms: 840
- p99_ms: 960
- avg_wait_ms: 320
- jain_fairness: 0.9998
- throughput_rps: 101.01
- per_server_utilization (%):
  - a: 34.34
  - b: 66.67
  - c: 100.0

### weighted-round-robin

Command:

```bash
cargo run -- run --algo weighted-round-robin --servers a:10:5,b:20:2,c:30:1 --burst 100 --burst-at 0 --seed 42 --format json
```

Metrics:

- duration_ms: 640
- p95_ms: 590
- p99_ms: 630
- avg_wait_ms: 276
- jain_fairness: 0.6921
- throughput_rps: 156.25
- per_server_utilization (%):
  - a: 100.0
  - b: 75.0
  - c: 56.25

### least-connections

Command:

```bash
cargo run -- run --algo least-connections --servers a:10,b:20,c:30 --burst 100 --burst-at 0 --seed 42 --format json
```

Metrics:

- duration_ms: 1020
- p95_ms: 870
- p99_ms: 990
- avg_wait_ms: 326
- jain_fairness: 0.9998
- throughput_rps: 98.04
- per_server_utilization (%):
  - a: 32.35
  - b: 64.71
  - c: 100.0

### least-response-time

Command:

```bash
cargo run -- run --algo least-response-time --servers a:10,b:20,c:30 --burst 100 --burst-at 0 --seed 42 --format json
```

Metrics:

- duration_ms: 550
- p95_ms: 520
- p99_ms: 540
- avg_wait_ms: 264
- jain_fairness: 0.8174
- throughput_rps: 181.82
- per_server_utilization (%):
  - a: 100.0
  - b: 98.18
  - c: 98.18


## Overload Poisson arrivals (factor 1.1)

### round-robin

Command:

```bash
cargo run -- run --algo round-robin --servers a:10,b:20,c:30 --overload --overload-factor 1.1 --overload-duration-ms 1000 --seed 42 --format json
```

Metrics:

- duration_ms: 1868
- p95_ms: 776
- p99_ms: 848
- avg_wait_ms: 192
- jain_fairness: 0.9999
- throughput_rps: 100.11
- per_server_utilization (%):
  - a: 33.73
  - b: 66.38
  - c: 99.57

### weighted-round-robin

Command:

```bash
cargo run -- run --algo weighted-round-robin --servers a:10:5,b:20:2,c:30:1 --overload --overload-factor 1.1 --overload-duration-ms 1000 --seed 42 --format json
```

Metrics:

- duration_ms: 4320
- p95_ms: 3062
- p99_ms: 3277
- avg_wait_ms: 1427
- jain_fairness: 0.7097
- throughput_rps: 159.72
- per_server_utilization (%):
  - a: 100.0
  - b: 79.63
  - c: 59.72

### least-connections

Command:

```bash
cargo run -- run --algo least-connections --servers a:10,b:20,c:30 --overload --overload-factor 1.1 --overload-duration-ms 1000 --seed 42 --format json
```

Metrics:

- duration_ms: 1136
- p95_ms: 202
- p99_ms: 240
- avg_wait_ms: 59
- jain_fairness: 0.8503
- throughput_rps: 164.61
- per_server_utilization (%):
  - a: 86.27
  - b: 93.31
  - c: 95.07

### least-response-time

Command:

```bash
cargo run -- run --algo least-response-time --servers a:10,b:20,c:30 --overload --overload-factor 1.1 --overload-duration-ms 1000 --seed 42 --format json
```

Metrics:

- duration_ms: 1075
- p95_ms: 133
- p99_ms: 139
- avg_wait_ms: 61
- jain_fairness: 0.785
- throughput_rps: 173.95
- per_server_utilization (%):
  - a: 99.53
  - b: 93.02
  - c: 83.72

