mod cli;
mod models;
mod sim;

use crate::models::{Algorithm, SimulationResult};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = cli::parse_args()?;
    let servers = cli::parse_servers(&args.servers)?;
    if servers.is_empty() {
        return Err("servers must not be empty".to_string());
    }
    if args.requests == 0 {
        return Err("requests must be greater than 0".to_string());
    }

    let algo: Algorithm = args.algo.clone().into();
    let result = sim::run_simulation(servers, algo, args.requests, args.seed);

    if args.summary {
        print_summary(&result);
        return Ok(());
    }

    if args.seed.is_some() {
        println!("Tie-break: seeded");
    } else {
        println!("Tie-break: stable index tie-break");
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
