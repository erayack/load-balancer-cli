use crate::algorithms::{Selection, SelectionContext, SelectionStrategy};
use crate::state::ServerState;

#[derive(Default)]
pub struct WeightedRoundRobinStrategy {
    cursor: u64,
    total_weight: u64,
    prefix_sums: Vec<u64>,
    cached_len: usize,
}

impl WeightedRoundRobinStrategy {
    fn rebuild_cache(&mut self, servers: &[ServerState]) {
        self.total_weight = 0;
        self.prefix_sums.clear();
        self.prefix_sums.reserve(servers.len());

        for server in servers {
            self.total_weight += server.weight as u64;
            self.prefix_sums.push(self.total_weight);
        }

        self.cached_len = servers.len();
    }
}

impl SelectionStrategy for WeightedRoundRobinStrategy {
    fn select(&mut self, ctx: &mut SelectionContext) -> Selection {
        if self.prefix_sums.is_empty() || self.cached_len != ctx.servers.len() {
            self.rebuild_cache(ctx.servers);
        }

        let target = self.cursor % self.total_weight;
        self.cursor = (self.cursor + 1) % self.total_weight;

        let selected = self
            .prefix_sums
            .binary_search_by(|sum| {
                if *sum > target {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            })
            .unwrap_or_else(|idx| idx);

        Selection {
            server_id: selected,
            score: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ServerState;
    use rand::SeedableRng;

    #[test]
    fn weighted_round_robin_respects_weights() {
        let servers = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 2,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
        ];
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut strategy = WeightedRoundRobinStrategy::default();
        let mut ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        let picks: Vec<usize> = (0..6)
            .map(|_| strategy.select(&mut ctx).server_id)
            .collect();
        assert_eq!(picks, vec![0, 0, 1, 0, 0, 1]);
    }

    #[test]
    fn weighted_round_robin_rebuilds_cache_on_server_change() {
        let servers_v1 = vec![ServerState {
            id: 0,
            name: "a".to_string(),
            base_latency_ms: 10,
            weight: 1,
            active_connections: 0,
            pick_count: 0,
            in_flight: 0,
            next_available_ms: 0,
        }];
        let servers_v2 = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 2,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
        ];
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut strategy = WeightedRoundRobinStrategy::default();
        {
            let mut ctx_v1 = SelectionContext {
                servers: &servers_v1,
                time_ms: 0,
                rng: &mut rng,
            };

            assert_eq!(strategy.select(&mut ctx_v1).server_id, 0);
        }

        let mut ctx_v2 = SelectionContext {
            servers: &servers_v2,
            time_ms: 0,
            rng: &mut rng,
        };
        let picks: Vec<usize> = (0..2)
            .map(|_| strategy.select(&mut ctx_v2).server_id)
            .collect();
        assert_eq!(picks, vec![0, 1]);
    }
}
