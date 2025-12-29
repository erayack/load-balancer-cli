use load_balancer_cli::config::{self, FormatArg};
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
    let args = config::parse_args()?;
    let (config, format) = config::build_config(args)?;
    let result = engine::run_simulation(&config)?;

    let formatter = formatter_for(&format);
    let output = formatter.write(&result);
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
