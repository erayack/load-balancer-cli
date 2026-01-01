use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::models::{AlgoConfig, RequestProfile, ServerConfig, SimConfig, TieBreakConfig};

#[derive(Parser, Debug)]
#[command(name = "lb-sim")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[arg(long, value_enum)]
    pub algo: Option<AlgoArg>,
    #[arg(long)]
    pub servers: Option<String>,
    #[arg(long, value_name = "name:latency[:weight]")]
    pub server: Vec<String>,
    #[arg(long)]
    pub requests: Option<usize>,
    #[arg(long, help = "Send all requests at once (burst)")]
    pub burst: Option<usize>,
    #[arg(long, default_value_t = 0, help = "Burst arrival time in ms")]
    pub burst_at: u64,
    #[arg(long, help = "Use Poisson arrivals at a rate above total capacity")]
    pub overload: bool,
    #[arg(
        long,
        default_value_t = 1.1,
        help = "Overload factor applied to total weighted capacity (Poisson rate)"
    )]
    pub overload_factor: f64,
    #[arg(long, default_value_t = 1000, help = "Overload duration in ms")]
    pub overload_duration_ms: u64,
    #[arg(long)]
    pub summary: bool,
    #[arg(long, value_enum, default_value = "human")]
    pub format: FormatArg,
    #[arg(
        long,
        help = "Seed tie-breaks for least-connections/response-time; omit for stable input-order tie-breaks"
    )]
    pub seed: Option<u64>,
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the load balancer simulation
    Run(RunArgs),
    /// List available algorithms
    ListAlgorithms,
    /// Show the effective configuration
    ShowConfig(RunArgs),
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    #[arg(long, value_enum)]
    pub algo: Option<AlgoArg>,
    #[arg(long)]
    pub servers: Option<String>,
    #[arg(long, value_name = "name:latency[:weight]")]
    pub server: Vec<String>,
    #[arg(long)]
    pub requests: Option<usize>,
    #[arg(long, help = "Send all requests at once (burst)")]
    pub burst: Option<usize>,
    #[arg(long, default_value_t = 0, help = "Burst arrival time in ms")]
    pub burst_at: u64,
    #[arg(long, help = "Use Poisson arrivals at a rate above total capacity")]
    pub overload: bool,
    #[arg(
        long,
        default_value_t = 1.1,
        help = "Overload factor applied to total weighted capacity (Poisson rate)"
    )]
    pub overload_factor: f64,
    #[arg(long, default_value_t = 1000, help = "Overload duration in ms")]
    pub overload_duration_ms: u64,
    #[arg(long)]
    pub summary: bool,
    #[arg(long, value_enum, default_value = "human")]
    pub format: FormatArg,
    #[arg(
        long,
        help = "Seed tie-breaks for least-connections/response-time; omit for stable input-order tie-breaks"
    )]
    pub seed: Option<u64>,
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum AlgoArg {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LeastResponseTime,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum FormatArg {
    Human,
    Summary,
    Json,
}

impl From<AlgoArg> for AlgoConfig {
    fn from(value: AlgoArg) -> Self {
        match value {
            AlgoArg::RoundRobin => AlgoConfig::RoundRobin,
            AlgoArg::WeightedRoundRobin => AlgoConfig::WeightedRoundRobin,
            AlgoArg::LeastConnections => AlgoConfig::LeastConnections,
            AlgoArg::LeastResponseTime => AlgoConfig::LeastResponseTime,
        }
    }
}

pub fn parse_args() -> Result<CliArgs> {
    CliArgs::try_parse().map_err(|e| Error::Cli(e.to_string()))
}

pub fn parse_command() -> Result<Command> {
    let args = parse_args()?;
    match args.command {
        Some(cmd) => Ok(cmd),
        None => {
            let run_args = RunArgs {
                algo: args.algo,
                servers: args.servers,
                server: args.server,
                requests: args.requests,
                burst: args.burst,
                burst_at: args.burst_at,
                overload: args.overload,
                overload_factor: args.overload_factor,
                overload_duration_ms: args.overload_duration_ms,
                summary: args.summary,
                format: args.format,
                seed: args.seed,
                config: args.config,
            };
            Ok(Command::Run(run_args))
        }
    }
}

pub fn build_config_from_run_args(args: RunArgs) -> Result<(SimConfig, FormatArg)> {
    let format = format_arg_from_run_args(&args);
    if args.requests.is_some() && args.burst.is_some() {
        return Err(Error::Cli(
            "use either --requests or --burst, not both".to_string(),
        ));
    }
    if args.overload && (args.requests.is_some() || args.burst.is_some()) {
        return Err(Error::Cli(
            "use either --overload or --requests/--burst, not both".to_string(),
        ));
    }
    if args.overload && args.overload_factor <= 0.0 {
        return Err(Error::Cli(
            "--overload-factor must be greater than 0".to_string(),
        ));
    }
    if args.overload && args.overload_duration_ms == 0 {
        return Err(Error::Cli(
            "--overload-duration-ms must be greater than 0".to_string(),
        ));
    }
    let mut config = if let Some(path) = args.config.as_ref() {
        load_config(path)?
    } else {
        let algo = args
            .algo
            .clone()
            .ok_or_else(|| Error::Cli("missing required --algo".to_string()))?;
        let servers = parse_server_args(&args.server, args.servers.as_deref())?;
        let requests = if args.overload {
            RequestProfile::Poisson {
                rate: capacity_rps(&servers) * args.overload_factor,
                duration_ms: args.overload_duration_ms,
            }
        } else {
            match (args.requests, args.burst) {
                (Some(count), None) => RequestProfile::FixedCount(count),
                (None, Some(count)) => RequestProfile::Burst {
                    count,
                    at_ms: args.burst_at,
                },
                (None, None) => {
                    return Err(Error::Cli(
                        "missing required --requests, --burst, or --overload".to_string(),
                    ))
                }
                (Some(_), Some(_)) => {
                    return Err(Error::Cli(
                        "use either --requests or --burst, not both".to_string(),
                    ))
                }
            }
        };
        let tie_break = if args.seed.is_some() {
            TieBreakConfig::Seeded
        } else {
            TieBreakConfig::Stable
        };
        return Ok((
            create_config(servers, requests, algo, tie_break, args.seed),
            format,
        ));
    };

    if let Some(algo) = args.algo {
        config.algo = algo.into();
    }
    if let Some(requests) = args.requests {
        config.requests = RequestProfile::FixedCount(requests);
    }
    if let Some(count) = args.burst {
        config.requests = RequestProfile::Burst {
            count,
            at_ms: args.burst_at,
        };
    }
    if args.overload {
        let rate = capacity_rps(&config.servers) * args.overload_factor;
        config.requests = RequestProfile::Poisson {
            rate,
            duration_ms: args.overload_duration_ms,
        };
    }
    if !args.server.is_empty() || args.servers.is_some() {
        config.servers = parse_server_args(&args.server, args.servers.as_deref())?;
    }
    if args.seed.is_some() {
        config.seed = args.seed;
        config.tie_break = TieBreakConfig::Seeded;
    }

    Ok((config, format))
}

pub fn load_config(path: &Path) -> Result<SimConfig> {
    let contents = fs::read_to_string(path).map_err(|err| {
        Error::ConfigIo(format!(
            "failed to read config '{}': {}",
            path.display(),
            err
        ))
    })?;
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    match ext {
        "toml" => toml::from_str(&contents)
            .map_err(|err| Error::ConfigParse(format!("failed to parse TOML: {}", err))),
        "json" => serde_json::from_str(&contents)
            .map_err(|err| Error::ConfigParse(format!("failed to parse JSON: {}", err))),
        "" => Err(Error::UnsupportedConfigFormat("unknown".to_string())),
        _ => Err(Error::UnsupportedConfigFormat(ext.to_string())),
    }
}

pub fn parse_server_args(
    server_entries: &[String],
    servers_csv: Option<&str>,
) -> Result<Vec<ServerConfig>> {
    let mut entries: Vec<String> = Vec::new();

    if let Some(csv) = servers_csv {
        let trimmed = csv.trim();
        if !trimmed.is_empty() {
            for entry in trimmed.split(',') {
                let trimmed_entry = entry.trim();
                if trimmed_entry.is_empty() {
                    return Err(Error::EmptyServerEntry);
                }
                entries.push(trimmed_entry.to_string());
            }
        }
    }

    for entry in server_entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(Error::EmptyServerEntry);
        }
        entries.push(trimmed.to_string());
    }

    if entries.is_empty() {
        return Err(Error::EmptyServers);
    }

    let mut servers = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let server = parse_server_entry(&entry)?;
        if names.contains(&server.name) {
            return Err(Error::DuplicateServerName(server.name));
        }
        names.insert(server.name.clone());
        servers.push(server);
    }

    Ok(servers)
}

fn parse_server_entry(entry: &str) -> Result<ServerConfig> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(Error::EmptyServerEntry);
    }

    let mut parts = trimmed.split(':');
    let name = parts.next().unwrap_or("").trim();
    let latency_str = parts.next().unwrap_or("").trim();
    let weight_str = parts.next().map(str::trim);
    if parts.next().is_some() {
        return Err(Error::InvalidServerEntry(trimmed.to_string()));
    }
    if name.is_empty() || latency_str.is_empty() || weight_str == Some("") {
        return Err(Error::InvalidServerEntry(trimmed.to_string()));
    }

    let latency_ms: u64 = latency_str
        .parse()
        .map_err(|_| Error::InvalidLatency(trimmed.to_string()))?;
    if latency_ms == 0 {
        return Err(Error::InvalidLatencyValue(trimmed.to_string()));
    }

    let weight = match weight_str {
        Some(value) => value
            .parse::<u32>()
            .map_err(|_| Error::InvalidWeight(trimmed.to_string()))?,
        None => 1,
    };
    if weight == 0 {
        return Err(Error::InvalidWeightValue(trimmed.to_string()));
    }

    Ok(ServerConfig {
        name: name.to_string(),
        base_latency_ms: latency_ms,
        weight,
    })
}

fn create_config(
    servers: Vec<ServerConfig>,
    requests: RequestProfile,
    algo: AlgoArg,
    tie_break: TieBreakConfig,
    seed: Option<u64>,
) -> SimConfig {
    SimConfig {
        servers,
        requests,
        algo: algo.into(),
        tie_break,
        seed,
    }
}

fn format_arg_from_run_args(args: &RunArgs) -> FormatArg {
    if args.summary {
        FormatArg::Summary
    } else {
        args.format.clone()
    }
}

pub fn format_config(config: &SimConfig) -> String {
    let algo_label = config.algo.to_string();

    let requests_label = match &config.requests {
        RequestProfile::FixedCount(n) => format!("Requests: {}", n),
        RequestProfile::Poisson { rate, duration_ms } => {
            format!(
                "Requests: poisson(rate={}, duration_ms={})",
                rate, duration_ms
            )
        }
        RequestProfile::Burst { count, at_ms } => {
            format!("Requests: burst(count={}, at_ms={})", count, at_ms)
        }
    };

    let tie_break_label = config.tie_break.label_with_seed(config.seed);

    let mut lines = vec![
        format!("Algorithm: {}", algo_label),
        requests_label,
        format!("Tie-break: {}", tie_break_label),
        "Servers:".to_string(),
    ];

    for server in &config.servers {
        lines.push(format!(
            "- {} (latency: {}ms, weight: {})",
            server.name, server.base_latency_ms, server.weight
        ));
    }

    lines.join("\n") + "\n"
}

fn capacity_rps(servers: &[ServerConfig]) -> f64 {
    servers
        .iter()
        .map(|server| (1000.0 / server.base_latency_ms as f64) * server.weight as f64)
        .sum()
}
