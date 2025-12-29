use load_balancer_cli::algorithms::Algorithm;
use load_balancer_cli::config;
use load_balancer_cli::engine;
use load_balancer_cli::error::{SimError, SimResult};
use load_balancer_cli::models::TieBreak;
use load_balancer_cli::output;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> SimResult<()> {
    let args = config::parse_args()?;
    let servers = config::parse_servers(&args.servers)?;
    if args.requests == 0 {
        return Err(SimError::RequestsZero);
    }

    let algo: Algorithm = args.algo.clone().into();
    let tie_break = match args.seed {
        Some(seed) => TieBreak::Seeded(seed),
        None => TieBreak::Stable,
    };
    let result = engine::run_simulation(servers, algo, args.requests, tie_break)?;

    if args.summary {
        output::print_summary(&result);
        return Ok(());
    }

    output::print_full(&result);

    Ok(())
}
