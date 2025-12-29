use clap::{Parser, ValueEnum};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::models::{AlgoConfig, RequestProfile, ServerConfig, SimConfig, TieBreakConfig};

#[derive(Parser, Debug)]
#[command(name = "load-balancer-cli")]
pub struct Args {
    #[arg(long, value_enum)]
    pub algo: Option<AlgoArg>,
    #[arg(long)]
    pub servers: Option<String>,
    #[arg(long, value_name = "name:latency[:weight]")]
    pub server: Vec<String>,
    #[arg(long)]
    pub requests: Option<usize>,
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

pub fn parse_args() -> Result<Args> {
    Args::try_parse().map_err(|e| Error::Cli(e.to_string()))
}

pub fn build_config(args: Args) -> Result<(SimConfig, FormatArg)> {
    let format = format_arg(&args);
    let mut config = if let Some(path) = args.config.as_ref() {
        load_config(path)?
    } else {
        let algo = args
            .algo
            .clone()
            .ok_or_else(|| Error::Cli("missing required --algo".to_string()))?;
        let requests = args
            .requests
            .ok_or_else(|| Error::Cli("missing required --requests".to_string()))?;
        let servers = parse_server_args(&args.server, args.servers.as_deref())?;
        let tie_break = if args.seed.is_some() {
            TieBreakConfig::Seeded
        } else {
            TieBreakConfig::Stable
        };
        return Ok((create_config(servers, requests, algo, tie_break, args.seed), format));
    };

    if let Some(algo) = args.algo {
        config.algo = algo.into();
    }
    if let Some(requests) = args.requests {
        config.requests = RequestProfile::FixedCount(requests);
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
        if csv.trim().is_empty() {
            return Err(Error::EmptyServers);
        }
        for entry in csv.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                return Err(Error::EmptyServerEntry);
            }
            entries.push(trimmed.to_string());
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
    requests: usize,
    algo: AlgoArg,
    tie_break: TieBreakConfig,
    seed: Option<u64>,
) -> SimConfig {
    SimConfig {
        servers,
        requests: RequestProfile::FixedCount(requests),
        algo: algo.into(),
        tie_break,
        seed,
    }
}

fn format_arg(args: &Args) -> FormatArg {
    if args.summary {
        FormatArg::Summary
    } else {
        args.format.clone()
    }
}
