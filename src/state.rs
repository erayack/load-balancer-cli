use serde::Serialize;

#[derive(Clone, Debug)]
pub struct ServerState {
    pub id: usize,
    pub name: String,
    pub base_latency_ms: u64,
    pub weight: u32,
    pub active_connections: u32,
    pub pick_count: u32,
    pub in_flight: u32,
    pub next_available_ms: u64,
}

#[derive(Clone, Debug)]
pub struct EngineState {
    pub time_ms: u64,
    pub servers: Vec<ServerState>,
    pub assignments: Vec<Assignment>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Assignment {
    pub request_id: usize,
    pub server_id: usize,
    pub arrival_time_ms: u64,
    pub started_at: u64,
    pub completed_at: u64,
    pub score: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServerSummary {
    pub name: String,
    pub requests: u32,
    pub avg_response_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResponseTimePercentiles {
    pub p95_ms: Option<u64>,
    pub p99_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServerUtilization {
    pub name: String,
    pub utilization_pct: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Phase1Metrics {
    pub response_time: ResponseTimePercentiles,
    pub per_server_utilization: Vec<ServerUtilization>,
    pub jain_fairness: f64,
    pub throughput_rps: f64,
    pub avg_wait_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunMetadata {
    pub algo: String,
    pub tie_break: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationResult {
    pub assignments: Vec<Assignment>,
    pub totals: Vec<ServerSummary>,
    pub metadata: RunMetadata,
    pub phase1_metrics: Phase1Metrics,
}
