use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("servers must not be empty")]
    EmptyServers,
    #[error("servers must not contain empty entries")]
    EmptyServerEntry,
    #[error("requests must be greater than 0")]
    RequestsZero,
    #[error("duplicate server name '{0}'")]
    DuplicateServerName(String),
    #[error("invalid server entry '{0}': expected name:latency_ms[:weight]")]
    InvalidServerEntry(String),
    #[error("invalid latency in '{0}'")]
    InvalidLatency(String),
    #[error("latency must be > 0 in '{0}'")]
    InvalidLatencyValue(String),
    #[error("invalid weight in '{0}'")]
    InvalidWeight(String),
    #[error("weight must be > 0 in '{0}'")]
    InvalidWeightValue(String),
    #[error("request rate must be > 0 (got {0})")]
    InvalidRequestRate(f64),
    #[error("request duration must be > 0 (got {0}ms)")]
    InvalidRequestDuration(u64),
    #[error("tie-break seed required when tie_break is seeded")]
    InvalidTieBreakSeed,
    #[error("{0}")]
    ConfigIo(String),
    #[error("{0}")]
    ConfigParse(String),
    #[error("unsupported config format '{0}'")]
    UnsupportedConfigFormat(String),
    #[error("{0}")]
    Cli(String),
}

pub type Result<T> = std::result::Result<T, Error>;
