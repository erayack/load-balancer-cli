use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug)]
pub enum SimError {
    EmptyServers,
    RequestsZero,
    DuplicateServerName(String),
    InvalidServerEntry(String),
    InvalidLatency(String),
    InvalidLatencyValue(String),
    InvalidWeight(String),
    InvalidWeightValue(String),
    InvalidRequestRate(f64),
    InvalidRequestDuration(u64),
    InvalidTieBreakSeed,
    EmptyServerEntry,
    ConfigIo(String),
    ConfigParse(String),
    UnsupportedConfigFormat(String),
    Cli(String),
}

pub type SimResult<T> = Result<T, SimError>;

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::EmptyServers => write!(f, "servers must not be empty"),
            SimError::EmptyServerEntry => write!(f, "servers must not contain empty entries"),
            SimError::RequestsZero => write!(f, "requests must be greater than 0"),
            SimError::DuplicateServerName(name) => {
                write!(f, "duplicate server name '{}'", name)
            }
            SimError::InvalidServerEntry(entry) => write!(
                f,
                "invalid server entry '{}': expected name:latency_ms[:weight]",
                entry
            ),
            SimError::InvalidLatency(entry) => write!(f, "invalid latency in '{}'", entry),
            SimError::InvalidLatencyValue(entry) => {
                write!(f, "latency must be > 0 in '{}'", entry)
            }
            SimError::InvalidWeight(entry) => write!(f, "invalid weight in '{}'", entry),
            SimError::InvalidWeightValue(entry) => {
                write!(f, "weight must be > 0 in '{}'", entry)
            }
            SimError::InvalidRequestRate(rate) => {
                write!(f, "request rate must be > 0 (got {})", rate)
            }
            SimError::InvalidRequestDuration(duration) => {
                write!(f, "request duration must be > 0 (got {}ms)", duration)
            }
            SimError::InvalidTieBreakSeed => {
                write!(f, "tie-break seed required when tie_break is seeded")
            }
            SimError::ConfigIo(message) => write!(f, "{}", message),
            SimError::ConfigParse(message) => write!(f, "{}", message),
            SimError::UnsupportedConfigFormat(format) => {
                write!(f, "unsupported config format '{}'", format)
            }
            SimError::Cli(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimConfig {
    pub servers: Vec<ServerConfig>,
    pub requests: RequestProfile,
    pub algo: AlgoConfig,
    #[serde(default)]
    pub tie_break: TieBreakConfig,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub base_latency_ms: u64,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequestProfile {
    FixedCount(usize),
    Poisson { rate: f64, duration_ms: u64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlgoConfig {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LeastResponseTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TieBreakConfig {
    #[default]
    Stable,
    Seeded,
}

fn default_weight() -> u32 {
    1
}
