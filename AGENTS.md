# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the entry point and wires CLI parsing to the simulation run.
- `src/cli.rs` defines clap arguments and parses the `--servers` input.
- `src/models.rs` contains core data types (`Server`, `Algorithm`, results).
- `src/sim.rs` implements the load-balancer simulation logic and validates simulation inputs.

## Build, Test, and Development Commands
- `cargo build` builds the CLI binary.
- `cargo run -- --algo round-robin --servers a:10,b:20 --requests 5` runs a sample simulation.
- `cargo test` runs unit/integration tests.
- `cargo fmt` formats Rust code using rustfmt.
- `cargo clippy` runs lints; fix or justify warnings before PRs.

## Tooling
- `cargo nextest run` is the preferred test runner (config in `.config/nextest.toml`).
- `cargo xtest` is a local alias for `cargo nextest run`.

## Coding Style & Naming Conventions
- Rust 2021 edition with standard 4-space indentation.
- Types use `UpperCamelCase`; functions and variables use `snake_case`.
- Keep parsing and CLI concerns in `src/cli.rs`, and algorithm logic in `src/sim.rs`.
- Prefer small, pure functions for selection logic (e.g., `pick_*` helpers).

## Testing Guidelines
- Unit tests live in-module with `#[cfg(test)]` (e.g., selection helpers in `src/sim.rs`).
- Integration tests live under `tests/` and use `assert_cmd` + `predicates` with `--summary` for stable output.
- Use deterministic `--seed` values in CLI tests to keep tie-breaks reproducible.
- Use descriptive test names like `least_connections_prefers_lowest_pick_count`.

## Commit & Pull Request Guidelines
- No commit history is available in this repository; use concise, imperative messages
  (e.g., "Add least-response-time scoring").
- Run `cargo fmt` before opening a PR to keep formatting consistent.
- PRs should include a short description, how to run the change, and sample output
  when user-visible behavior changes.

## Configuration Notes
- `--servers` expects comma-separated `name:latency_ms` entries, e.g. `api:25,db:40`.
- `--seed` makes tie-breaks deterministic for least-connections/response-time.
- `--summary` prints only the summary in stable input order for testing.
- Duplicate server IDs are rejected by the simulator.
