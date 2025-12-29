mod cli;
mod models;
mod sim;

use crate::models::{Algorithm, SimulationResult, TieBreak};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = cli::parse_args()?;
    let servers = cli::parse_servers(&args.servers)?;
    if args.requests == 0 {
        return Err("requests must be greater than 0".to_string());
    }

    let algo: Algorithm = args.algo.clone().into();
    let tie_break = match args.seed {
        Some(seed) => TieBreak::Seeded(seed),
        None => TieBreak::Stable,
    };
    let result = sim::run_simulation(servers, algo, args.requests, tie_break)?;

    if args.summary {
        print_summary(&result);
        return Ok(());
    }

    match &result.tie_break {
        TieBreak::Seeded(seed) => println!("Tie-break: seeded({})", seed),
        TieBreak::Stable => println!("Tie-break: stable"),
    }

    for assignment in &result.assignments {
        if let Some(score) = assignment.score {
            println!(
                "Request {} -> {} (score: {}ms)",
                assignment.request_id, assignment.server_name, score
            );
        } else {
            println!(
                "Request {} -> {}",
                assignment.request_id, assignment.server_name
            );
        }
    }

    print_summary(&result);

    Ok(())
}

fn print_summary(result: &SimulationResult) {
    println!("Summary:");
    for summary in &result.totals {
        println!("{}: {} requests", summary.name, summary.requests);
    }
}
