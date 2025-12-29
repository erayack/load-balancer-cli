use load_balancer_cli::config::{self, format_config, Command, FormatArg, RunArgs};
use load_balancer_cli::engine;
use load_balancer_cli::error::Result;
use load_balancer_cli::output::{Formatter, HumanFormatter, JsonFormatter, SummaryFormatter};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let command = config::parse_command()?;

    match command {
        Command::Run(run_args) => run_simulation(run_args),
        Command::ListAlgorithms => list_algorithms(),
        Command::ShowConfig(run_args) => show_config(run_args),
    }
}

fn run_simulation(run_args: RunArgs) -> Result<()> {
    let (config, format) = config::build_config_from_run_args(run_args)?;
    let result = engine::run_simulation(&config)?;

    let formatter = formatter_for(&format);
    let output = formatter.write(&result);
    print!("{}", output);

    Ok(())
}

fn list_algorithms() -> Result<()> {
    println!("round-robin");
    println!("weighted-round-robin");
    println!("least-connections");
    println!("least-response-time");
    Ok(())
}

fn show_config(run_args: RunArgs) -> Result<()> {
    let (config, _) = config::build_config_from_run_args(run_args)?;
    let output = format_config(&config);
    print!("{}", output);
    Ok(())
}

fn formatter_for(format: &FormatArg) -> Box<dyn Formatter> {
    match format {
        FormatArg::Human => Box::new(HumanFormatter),
        FormatArg::Summary => Box::new(SummaryFormatter),
        FormatArg::Json => Box::new(JsonFormatter),
    }
}
