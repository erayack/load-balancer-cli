use clap::{Parser, ValueEnum};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::load_config;
use crate::models::{
    AlgoConfig, RequestProfile, ServerConfig, SimConfig, SimError, SimResult, TieBreakConfig,
};

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

pub fn parse_args() -> SimResult<Args> {
    Args::try_parse().map_err(|e| SimError::Cli(e.to_string()))
}

pub fn build_config(args: Args) -> SimResult<(SimConfig, bool)> {
    let mut config = if let Some(path) = args.config {
        load_config(&path)?
    } else {
        let algo = args
            .algo
            .ok_or_else(|| SimError::Cli("missing required --algo".to_string()))?;
        let requests = args
            .requests
            .ok_or_else(|| SimError::Cli("missing required --requests".to_string()))?;
        let servers = parse_server_args(&args.server, args.servers.as_deref())?;
        let tie_break = if args.seed.is_some() {
            TieBreakConfig::Seeded
        } else {
            TieBreakConfig::Stable
        };
        return Ok((
            SimConfig {
                servers,
                requests: RequestProfile::FixedCount(requests),
                algo: algo.into(),
                tie_break,
                seed: args.seed,
            },
            args.summary,
        ));
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

    Ok((config, args.summary))
}

pub fn parse_server_args(
    server_entries: &[String],
    servers_csv: Option<&str>,
) -> SimResult<Vec<ServerConfig>> {
    let mut entries: Vec<String> = Vec::new();

    if let Some(csv) = servers_csv {
        if csv.trim().is_empty() {
            return Err(SimError::EmptyServers);
        }
        for entry in csv.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                return Err(SimError::EmptyServerEntry);
            }
            entries.push(trimmed.to_string());
        }
    }

    for entry in server_entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(SimError::EmptyServerEntry);
        }
        entries.push(trimmed.to_string());
    }

    if entries.is_empty() {
        return Err(SimError::EmptyServers);
    }

    let mut servers = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let server = parse_server_entry(&entry)?;
        if names.contains(&server.name) {
            return Err(SimError::DuplicateServerName(server.name));
        }
        names.insert(server.name.clone());
        servers.push(server);
    }

    Ok(servers)
}

fn parse_server_entry(entry: &str) -> SimResult<ServerConfig> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(SimError::EmptyServerEntry);
    }

    let mut parts = trimmed.split(':');
    let name = parts.next().unwrap_or("").trim();
    let latency_str = parts.next().unwrap_or("").trim();
    let weight_str = parts.next().map(str::trim);
    if parts.next().is_some() {
        return Err(SimError::InvalidServerEntry(trimmed.to_string()));
    }
    if name.is_empty() || latency_str.is_empty() || weight_str == Some("") {
        return Err(SimError::InvalidServerEntry(trimmed.to_string()));
    }

    let latency_ms: u64 = latency_str
        .parse()
        .map_err(|_| SimError::InvalidLatency(trimmed.to_string()))?;
    if latency_ms == 0 {
        return Err(SimError::InvalidLatencyValue(trimmed.to_string()));
    }

    let weight = match weight_str {
        Some(value) => value
            .parse::<u32>()
            .map_err(|_| SimError::InvalidWeight(trimmed.to_string()))?,
        None => 1,
    };
    if weight == 0 {
        return Err(SimError::InvalidWeightValue(trimmed.to_string()));
    }

    Ok(ServerConfig {
        name: name.to_string(),
        base_latency_ms: latency_ms,
        weight,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_server_args;

    #[test]
    fn parse_servers_accepts_valid_list() {
        let servers = parse_server_args(&[], Some("api:10, db:20")).unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "api");
        assert_eq!(servers[0].base_latency_ms, 10);
        assert_eq!(servers[0].weight, 1);
        assert_eq!(servers[1].name, "db");
        assert_eq!(servers[1].base_latency_ms, 20);
        assert_eq!(servers[1].weight, 1);
    }

    #[test]
    fn parse_servers_accepts_weighted_entry() {
        let servers = parse_server_args(&["api:10:3".to_string()], None).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].weight, 3);
    }

    #[test]
    fn parse_servers_rejects_empty_input() {
        assert!(parse_server_args(&[], None).is_err());
    }

    #[test]
    fn parse_servers_rejects_invalid_format() {
        assert!(parse_server_args(&["api".to_string()], None).is_err());
        assert!(parse_server_args(&["api:10:20:30".to_string()], None).is_err());
    }

    #[test]
    fn parse_servers_rejects_invalid_latency() {
        assert!(parse_server_args(&["api:0".to_string()], None).is_err());
        assert!(parse_server_args(&["api:ten".to_string()], None).is_err());
    }

    #[test]
    fn parse_servers_rejects_invalid_weight() {
        assert!(parse_server_args(&["api:10:0".to_string()], None).is_err());
        assert!(parse_server_args(&["api:10:ten".to_string()], None).is_err());
        assert!(parse_server_args(&["api:10:".to_string()], None).is_err());
    }

    #[test]
    fn parse_servers_rejects_duplicate_names() {
        let err = parse_server_args(&[], Some("api:10, api:20")).unwrap_err();
        assert_eq!(err.to_string(), "duplicate server name 'api'");
    }

    #[test]
    fn parse_servers_rejects_trailing_commas() {
        let err = parse_server_args(&[], Some("a:10,")).unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_empty_segments() {
        let err = parse_server_args(&[], Some("a:10,,b:20")).unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_comma_only_input() {
        let err = parse_server_args(&[], Some(",")).unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_whitespace_only_input() {
        let err = parse_server_args(&[], Some(" ")).unwrap_err();
        assert_eq!(err.to_string(), "servers must not be empty");
    }
}
