use load_balancer_cli::config;
use load_balancer_cli::engine;
use load_balancer_cli::error::SimResult;
use load_balancer_cli::output;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> SimResult<()> {
    let args = config::parse_args()?;
    let (config, summary_only) = config::build_config(args)?;
    let result = engine::run_simulation(&config)?;

    if summary_only {
        output::print_summary(&result);
        return Ok(());
    }

    output::print_full(&config, &result)?;

    Ok(())
}
