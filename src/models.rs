use std::fmt;

#[derive(Clone, Debug)]
pub struct Server {
    pub id: usize,
    pub name: String,
    pub base_latency_ms: u64,
    pub weight: u32,
    pub active_connections: u32,
    pub pick_count: u32,
}

#[cfg(test)]
impl Server {
    pub fn test_at(
        index: usize,
        name: &str,
        latency: u64,
        weight: u32,
        active_connections: u32,
        pick_count: u32,
    ) -> Self {
        Self {
            id: index,
            name: name.to_string(),
            base_latency_ms: latency,
            weight,
            active_connections,
            pick_count,
        }
    }
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
    pub started_at: u64,
    pub completed_at: u64,
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
    pub tie_break: TieBreak,
}
