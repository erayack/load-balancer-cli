use serde::{Deserialize, Serialize};

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
