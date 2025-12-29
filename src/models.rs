use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for AlgoConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            AlgoConfig::RoundRobin => "round-robin",
            AlgoConfig::WeightedRoundRobin => "weighted-round-robin",
            AlgoConfig::LeastConnections => "least-connections",
            AlgoConfig::LeastResponseTime => "least-response-time",
        };
        write!(f, "{}", label)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TieBreakConfig {
    #[default]
    Stable,
    Seeded,
}

impl TieBreakConfig {
    pub fn label_with_seed(&self, seed: Option<u64>) -> String {
        match self {
            TieBreakConfig::Stable => "stable".to_string(),
            TieBreakConfig::Seeded => match seed {
                Some(value) => format!("seeded({})", value),
                None => "seeded".to_string(),
            },
        }
    }
}

fn default_weight() -> u32 {
    1
}
