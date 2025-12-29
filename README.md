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
```

## Usage

- `--servers` expects comma-separated `name:latency_ms` entries, e.g. `api:25,db:40`.
- `--seed` makes tie-breaks deterministic for least-connections/response-time; omit it to use stable input-order tie-breaks.
- `--summary` prints only the summary in stable input order for testing.

Full output includes a single `Tie-break:` line (`stable` or `seeded(<seed>)`) before per-request assignments.

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

Note: `cargo clippy` may report dead code in `src/models.rs`; we keep some types there for planned algorithms and will implement them later.
