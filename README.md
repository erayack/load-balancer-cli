# Load Balancer CLI

A Rust CLI that simulates load-balancing algorithms with event-driven simulation and pluggable strategies.

## Quick Start

```bash
cargo build
cargo run -- run --algo round-robin --servers a:10,b:20 --requests 5
cargo run -- list-algorithms
cargo run -- show-config --algo round-robin --servers a:10,b:20 --requests 5
```

## CLI Subcommands

- `run` - Run a simulation
- `list-algorithms` - List available algorithms
- `show-config` - Show effective configuration

## Run Options

| Option | Description |
|--------|-------------|
| `--algo` | Algorithm to use (required) |
| `--servers` | Comma-separated servers: `a:10,b:20` |
| `--server` | Add single server (repeatable): `name:latency[:weight]` |
| `--requests` | Number of requests (required) |
| `--format` | Output format: `human`, `summary`, `json` |
| `--seed` | Seed for deterministic tie-breaking |
| `--config` | TOML or JSON config file |

## Algorithms

- `round-robin` - Sequential cycling
- `weighted-round-robin` - Weight-proportional distribution
- `least-connections` - Fewest active connections
- `least-response-time` - Lowest latency + (pick_count × 10)

Run with: `cargo run -- run --config config.toml --requests 15`

## Development

```bash
cargo test
cargo nextest run
cargo fmt
cargo clippy
```

Or with Just: `just` to see all commands.
