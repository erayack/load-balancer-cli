mod least_connections;
mod least_response_time;
mod round_robin;
mod weighted_round_robin;

use rand::RngCore;

use crate::models::AlgoConfig;
use crate::state::ServerState;

pub use least_connections::LeastConnectionsStrategy;
pub use least_response_time::LeastResponseTimeStrategy;
pub use round_robin::RoundRobinStrategy;
pub use weighted_round_robin::WeightedRoundRobinStrategy;

pub trait SelectionStrategy {
    fn select(&mut self, ctx: &mut SelectionContext) -> Selection;
}

pub struct SelectionContext<'a> {
    pub servers: &'a [ServerState],
    #[allow(dead_code)]
    pub time_ms: u64,
    pub rng: &'a mut dyn RngCore,
}

pub struct Selection {
    pub server_id: usize,
    pub score: Option<u64>,
}

pub fn build_strategy(algo: AlgoConfig) -> Box<dyn SelectionStrategy> {
    match algo {
        AlgoConfig::RoundRobin => Box::new(RoundRobinStrategy::default()),
        AlgoConfig::WeightedRoundRobin => Box::new(WeightedRoundRobinStrategy::default()),
        AlgoConfig::LeastConnections => Box::new(LeastConnectionsStrategy::default()),
        AlgoConfig::LeastResponseTime => Box::new(LeastResponseTimeStrategy::default()),
    }
}
