use clap::{Parser, ValueEnum};
use std::collections::HashSet;

use crate::models::{Algorithm, Server, SimError, SimResult};

#[derive(Parser, Debug)]
#[command(name = "load-balancer-cli")]
pub struct Args {
    #[arg(long, value_enum)]
    pub algo: AlgoArg,
    #[arg(long)]
    pub servers: String,
    #[arg(long)]
    pub requests: usize,
    #[arg(long)]
    pub summary: bool,
    #[arg(
        long,
        help = "Seed tie-breaks for least-connections/response-time; omit for stable input-order tie-breaks"
    )]
    pub seed: Option<u64>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum AlgoArg {
    RoundRobin,
    LeastConnections,
    LeastResponseTime,
}

impl From<AlgoArg> for Algorithm {
    fn from(value: AlgoArg) -> Self {
        match value {
            AlgoArg::RoundRobin => Algorithm::RoundRobin,
            AlgoArg::LeastConnections => Algorithm::LeastConnections,
            AlgoArg::LeastResponseTime => Algorithm::LeastResponseTime,
        }
    }
}

pub fn parse_args() -> SimResult<Args> {
    Args::try_parse().map_err(|e| SimError::Cli(e.to_string()))
}

pub fn parse_servers(input: &str) -> SimResult<Vec<Server>> {
    let mut servers = Vec::new();
    let mut names = HashSet::new();

    if input.trim().is_empty() {
        return Err(SimError::EmptyServers);
    }

    for (id, entry) in input.split(',').enumerate() {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(SimError::EmptyServerEntry);
        }

        let mut parts = trimmed.split(':');
        let name = parts.next().unwrap_or("").trim();
        let latency_str = parts.next().unwrap_or("").trim();
        if parts.next().is_some() {
            return Err(SimError::InvalidServerEntry(trimmed.to_string()));
        }
        if name.is_empty() || latency_str.is_empty() {
            return Err(SimError::InvalidServerEntry(trimmed.to_string()));
        }

        if names.contains(name) {
            return Err(SimError::DuplicateServerName(name.to_string()));
        }
        names.insert(name.to_string());

        let latency_ms: u64 = latency_str
            .parse()
            .map_err(|_| SimError::InvalidLatency(trimmed.to_string()))?;
        if latency_ms == 0 {
            return Err(SimError::InvalidLatencyValue(trimmed.to_string()));
        }

        servers.push(Server {
            id,
            name: name.to_string(),
            base_latency_ms: latency_ms,
            active_connections: 0,
            pick_count: 0,
        });
    }

    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::parse_servers;

    #[test]
    fn parse_servers_accepts_valid_list() {
        let servers = parse_servers("api:10, db:20").unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id, 0);
        assert_eq!(servers[0].name, "api");
        assert_eq!(servers[0].base_latency_ms, 10);
        assert_eq!(servers[0].active_connections, 0);
        assert_eq!(servers[0].pick_count, 0);
        assert_eq!(servers[1].id, 1);
        assert_eq!(servers[1].name, "db");
        assert_eq!(servers[1].base_latency_ms, 20);
    }

    #[test]
    fn parse_servers_rejects_empty_input() {
        assert!(parse_servers("").is_err());
    }

    #[test]
    fn parse_servers_rejects_invalid_format() {
        assert!(parse_servers("api").is_err());
        assert!(parse_servers("api:10:20").is_err());
    }

    #[test]
    fn parse_servers_rejects_invalid_latency() {
        assert!(parse_servers("api:0").is_err());
        assert!(parse_servers("api:ten").is_err());
    }

    #[test]
    fn parse_servers_rejects_duplicate_names() {
        let err = parse_servers("api:10, api:20").unwrap_err();
        assert_eq!(err.to_string(), "duplicate server name 'api'");
    }

    #[test]
    fn parse_servers_rejects_trailing_commas() {
        let err = parse_servers("a:10,").unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_empty_segments() {
        let err = parse_servers("a:10,,b:20").unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_comma_only_input() {
        let err = parse_servers(",").unwrap_err();
        assert_eq!(err.to_string(), "servers must not contain empty entries");
    }

    #[test]
    fn parse_servers_rejects_whitespace_only_input() {
        let err = parse_servers(" ").unwrap_err();
        assert_eq!(err.to_string(), "servers must not be empty");
    }
}
