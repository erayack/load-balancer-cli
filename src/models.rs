use std::fmt;

#[derive(Clone, Debug)]
pub enum SimError {
    EmptyServers,
    RequestsZero,
    DuplicateServerId(usize),
    DuplicateServerName(String),
    InvalidServerEntry(String),
    InvalidLatency(String),
    InvalidLatencyValue(String),
    EmptyServerEntry,
    Cli(String),
}

pub type SimResult<T> = Result<T, SimError>;

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::EmptyServers => write!(f, "servers must not be empty"),
            SimError::EmptyServerEntry => write!(f, "servers must not contain empty entries"),
            SimError::RequestsZero => write!(f, "requests must be greater than 0"),
            SimError::DuplicateServerId(id) => write!(f, "duplicate server id {}", id),
            SimError::DuplicateServerName(name) => {
                write!(f, "duplicate server name '{}'", name)
            }
            SimError::InvalidServerEntry(entry) => write!(
                f,
                "invalid server entry '{}': expected name:latency_ms",
                entry
            ),
            SimError::InvalidLatency(entry) => write!(f, "invalid latency in '{}'", entry),
            SimError::InvalidLatencyValue(entry) => {
                write!(f, "latency must be > 0 in '{}'", entry)
            }
            SimError::Cli(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub id: usize,
    pub name: String,
    pub base_latency_ms: u64,
    pub active_connections: u32,
    pub pick_count: u32,
}

#[cfg(test)]
impl Server {
    pub fn test_at(
        index: usize,
        name: &str,
        latency: u64,
        active_connections: u32,
        pick_count: u32,
    ) -> Self {
        Self {
            id: index,
            name: name.to_string(),
            base_latency_ms: latency,
            active_connections,
            pick_count,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Algorithm {
    RoundRobin,
    LeastConnections,
    LeastResponseTime,
}

#[derive(Clone, Debug)]
pub enum TieBreak {
    Stable,
    Seeded(u64),
}

impl fmt::Display for TieBreak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TieBreak::Stable => write!(f, "stable"),
            TieBreak::Seeded(seed) => write!(f, "seeded({})", seed),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub request_id: usize,
    pub server_id: usize,
    pub server_name: String,
    pub score: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ServerSummary {
    pub name: String,
    pub requests: u32,
}

#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub assignments: Vec<Assignment>,
    pub totals: Vec<ServerSummary>,
    pub tie_break: TieBreak,
}
