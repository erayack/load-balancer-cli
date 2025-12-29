mod cli;
mod config;
mod models;
mod sim;
mod state;

use crate::models::{SimError, SimResult, TieBreakConfig};
use crate::state::SimulationResult;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> SimResult<()> {
    let args = cli::parse_args()?;
    let (config, summary_only) = cli::build_config(args)?;
    let result = sim::run_simulation(&config)?;

    if summary_only {
        print_summary(&result);
        return Ok(());
    }

    print_tie_break(&result);

    for assignment in &result.assignments {
        let server_name = config
            .servers
            .get(assignment.server_id)
            .map(|server| server.name.as_str())
            .ok_or(SimError::InvalidServerEntry(format!(
                "missing server for id {}",
                assignment.server_id
            )))?;
        if let Some(score) = assignment.score {
            println!(
                "Request {} -> {} (score: {}ms)",
                assignment.request_id, server_name, score
            );
        } else {
            println!("Request {} -> {}", assignment.request_id, server_name);
        }
    }

    print_summary(&result);

    Ok(())
}

fn print_tie_break(result: &SimulationResult) {
    match result.tie_break {
        TieBreakConfig::Stable => println!("Tie-break: stable"),
        TieBreakConfig::Seeded => {
            let seed = result.seed.unwrap_or_default();
            println!("Tie-break: seeded({})", seed);
        }
    }
}

fn print_summary(result: &SimulationResult) {
    println!("Summary:");
    for summary in &result.totals {
        println!(
            "{}: {} requests (avg response: {}ms)",
            summary.name, summary.requests, summary.avg_response_ms
        );
    }
}
