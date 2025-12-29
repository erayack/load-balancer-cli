# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, and Lint Commands

```bash
cargo build              # Build the project
cargo run -- [args]      # Run with arguments, e.g. --algo round-robin --servers a:10,b:20 --requests 5
cargo nextest run        # Run all tests (preferred over cargo test)
cargo nextest run <test> # Run a specific test
cargo fmt                # Format code
cargo clippy             # Run linter
```

## Architecture

A Rust CLI that simulates load-balancing algorithms across servers with configurable latencies and weights.

### Module Structure

- **`src/main.rs`** - Entry point. Wires CLI parsing to simulation, handles output formatting.
- **`src/cli.rs`** - Defines clap arguments (`Args`, `AlgoArg`) and parses `--servers` with format `name:latency_ms[:weight]`. Rejects duplicate names.
- **`src/models.rs`** - Core types: `Server`, `Algorithm`, `TieBreak`, `Assignment`, `SimError`, result types.
- **`src/sim.rs`** - Simulation logic. Runs the load-balancing loop with time-based in-flight request tracking.

### Algorithms

1. **RoundRobin** - Cycles through servers sequentially.
2. **WeightedRoundRobin** - Distributes proportionally to weight values.
3. **LeastConnections** - Picks server with fewest `active_connections`. Uses `BinaryHeap<Reverse<InFlight>>` for time-based decay (requests complete after `base_latency_ms`).
4. **LeastResponseTime** - Picks server with lowest `base_latency_ms + (pick_count * 10)` score.

### Tie-Breaking

- **Stable** - Uses input order for ties (default, no seed).
- **Seeded** - Uses `StdRng` with provided seed for deterministic random selection.

### Least-Connections Semantics

Requests arrive one time unit apart. A request stays "in flight" for `base_latency_ms`. Active connections decay when in-flight requests complete (time-based, not count-based).
