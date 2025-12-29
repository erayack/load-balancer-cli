use crate::models::TieBreakConfig;

#[derive(Clone, Debug)]
pub struct ServerState {
    pub id: usize,
    pub name: String,
    pub base_latency_ms: u64,
    pub weight: u32,
    pub active_connections: u32,
    pub pick_count: u32,
    pub in_flight: u32,
}

#[derive(Clone, Debug)]
pub struct EngineState {
    pub time_ms: u64,
    pub servers: Vec<ServerState>,
    pub assignments: Vec<Assignment>,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub id: usize,
    pub arrival_ms: u64,
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub request_id: usize,
    pub server_id: usize,
    pub started_at: u64,
    pub completed_at: u64,
    pub score: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ServerSummary {
    pub name: String,
    pub requests: u32,
    pub avg_response_ms: u64,
}

#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub assignments: Vec<Assignment>,
    pub totals: Vec<ServerSummary>,
    pub tie_break: TieBreakConfig,
    pub seed: Option<u64>,
}
