# Load Balancer CLI

A Rust CLI that simulates load-balancing algorithms across servers with
configurable latencies and request counts.

## Quick Start

Build:

```
cargo build
```

Run a sample simulation:

```
cargo run -- --algo round-robin --servers a:10,b:20 --requests 5
cargo run -- --algo least-connections --servers a:10,b:20 --requests 5 --seed 42
cargo run -- --algo least-response-time --servers a:10,b:20 --requests 5 --seed 7
cargo run -- --algo weighted-round-robin --servers a:10:2,b:20:1 --requests 5 --su
cargo run -- --algo weighted-round-robin --servers a:10:2,b:20:1 --requests 5 --seed 11 --summary
```

## Usage

- `--servers` expects comma-separated `name:latency_ms` entries, e.g. `api:25,db:40`.
- `--seed` makes tie-breaks deterministic for least-connections/response-time; omit it to use stable input-order tie-breaks.
- `--summary` prints only the summary in stable input order for testing.
- Duplicate server IDs are rejected by the simulator.

Full output includes a single `Tie-break:` line (`stable` or `seeded(<seed>)`) before per-request assignments.

## Least-Connections Semantics

- Least-connections uses `active_connections`, not historical picks.
- Requests arrive one time unit apart; a request stays in flight for `base_latency_ms`.
- A server's active connections drop when in-flight requests complete (time-based decay).

## Project Layout

- `src/main.rs` wires CLI parsing to the simulation.
- `src/cli.rs` defines clap arguments and parses `--servers`.
- `src/models.rs` holds core types (`Server`, `Algorithm`, results).
- `src/sim.rs` implements the simulation logic.


## Development

```
cargo nextest run
cargo fmt
cargo clippy
```
