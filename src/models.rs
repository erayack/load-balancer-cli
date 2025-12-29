#[derive(Clone, Debug)]
pub struct Server {
    pub id: usize,
    pub name: String,
    pub base_latency_ms: u64,
    #[allow(dead_code)]
    pub active_connections: u32,
    pub pick_count: u32,
}

#[cfg(test)]
impl Server {
    pub fn test_at(index: usize, name: &str, latency: u64, pick_count: u32) -> Self {
        Self {
            id: index,
            name: name.to_string(),
            base_latency_ms: latency,
            active_connections: 0,
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

#[derive(Clone, Debug)]
pub struct Assignment {
    pub request_id: usize,
    pub server_id: usize,
    pub server_name: String,
    pub score: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub assignments: Vec<Assignment>,
    pub totals: Vec<(String, u32)>,
    pub tie_break: TieBreak,
}
