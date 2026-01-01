# lb-sim — Deterministic Load Balancing Simulator

A Rust-based **discrete-event simulation framework** for evaluating load balancing and routing policies under **queueing, bursty traffic, and overload conditions**, with a focus on **tail latency, throughput, utilization, and fairness tradeoffs**.

This project is designed to make infrastructure tradeoffs **explicit, reproducible, and measurable**, rather than implicit in production systems.

## Why

In real production systems, load balancing is not just about distributing traffic evenly.

Routing decisions directly affect:

* **Tail latency (p95 / p99)**
* **Throughput under contention**
* **Utilization of heterogeneous resources**
* **Fairness across backends**

Many common policies optimize one dimension while degrading others, especially under burst or overload scenarios.
This simulator exists to **quantify those tradeoffs deterministically**, using a minimal but expressive model.



## What This Simulates

### Core Model

* **Discrete-event simulation**

  * request arrival
  * server selection
  * service completion
* **Heterogeneous servers**

  * fixed service latency
  * optional weights
* **Deterministic execution**

  * seeded RNG for reproducibility
* **Pluggable routing policies**

### Arrival Patterns

* **Fixed-rate arrivals** (e.g. 1 req/ms)
* **Burst arrivals** (e.g. N requests at t=0)
* **Poisson overload** (arrival rate > service capacity)

### Metrics Collected

* End-to-end **p95 / p99 latency**
* **Average queue wait time**
* **Throughput** (requests / second)
* **Per-server utilization**
* **Jain’s Fairness Index**

All metrics are computed from simulation state without nondeterminism. For a full set of example runs, see `phase1_metrics_report.md`.


## Routing Policies

* **round-robin**
  Maximizes fairness; ignores latency and queue depth.

* **weighted-round-robin**
  Distributes load proportionally to configured weights.

* **least-connections**
  Routes to the backend with the fewest active requests.

* **least-response-time**
  Routes based on predicted completion time, favoring faster servers under contention.

Each policy exposes different tradeoffs between fairness, utilization, and tail latency.

## Example Results (Overload Scenario)

100 requests, heterogeneous servers (10 / 20 / 30 ms), Poisson overload factor 1.1.

| Policy              | p99 Latency | Throughput (rps) | Fairness |
| ------------------- | ----------: | ---------------: | -------: |
| Round-robin         |      848 ms |            100.1 |    0.999 |
| Weighted RR         |      226 ms |            155.2 |     0.70 |
| Least-connections   |      240 ms |            164.6 |     0.85 |
| Least-response-time |  **139 ms** |        **173.9** |     0.78 |

**Observations**

* Least-response-time reduces p99 latency by ~84% under overload
* Higher throughput comes at the cost of reduced fairness
* Policies that ignore latency amplify queueing effects under burst traffic


## Reproduce a Scenario

```bash
cargo run -- run \
  --algo least-response-time \
  --servers a:10,b:20,c:30 \
  --overload \
  --overload-factor 1.1 \
  --overload-duration-ms 1000 \
  --seed 42 \
  --format json
```

Deterministic seeds ensure runs are directly comparable.


## CLI Overview

### Subcommands

* `run` — execute a simulation
* `list-algorithms` — list available routing policies
* `show-config` — display resolved configuration

### Common Options

| Option       | Description                                      |
| ------------ | ------------------------------------------------ |
| `--algo`     | Routing policy (required)                        |
| `--servers`  | Comma-separated servers: `name:latency[:weight]` |
| `--requests` | Number of requests                               |
| `--burst`    | Burst size                                       |
| `--burst-at` | Burst start time                                 |
| `--overload` | Enable Poisson overload                          |
| `--seed`     | RNG seed for determinism                         |
| `--format`   | `human`, `summary`, or `json`                    |

## Output Formats

* **human** — readable summary
* **summary** — per-server aggregates
* **json** — machine-readable metrics for analysis or plotting

## Non-Goals

This project intentionally does **not** model:

* Network jitter or TCP behavior
* Kernel scheduling
* Adaptive autoscaling
* Real-world service dependencies

The goal is **clarity of routing behavior**, not production fidelity.

## Development

```bash
cargo test
cargo nextest run
cargo fmt
cargo clippy
```

Or run all checks with:

```bash
just
```

## License

MIT
