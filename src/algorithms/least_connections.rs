use rand::Rng;

use crate::algorithms::{Selection, SelectionContext, SelectionStrategy};

#[derive(Default)]
pub struct LeastConnectionsStrategy {
    candidates: Vec<usize>,
}

impl SelectionStrategy for LeastConnectionsStrategy {
    fn select(&mut self, ctx: &mut SelectionContext) -> Selection {
        let mut min_count = u32::MAX;
        self.candidates.clear();
        if self.candidates.capacity() < ctx.servers.len() {
            self.candidates
                .reserve(ctx.servers.len().saturating_sub(self.candidates.len()));
        }

        for (idx, server) in ctx.servers.iter().enumerate() {
            if server.active_connections < min_count {
                min_count = server.active_connections;
                self.candidates.clear();
                self.candidates.push(idx);
            } else if server.active_connections == min_count {
                self.candidates.push(idx);
            }
        }

        let choice = if self.candidates.len() == 1 {
            self.candidates[0]
        } else {
            let pick = ctx.rng.gen_range(0..self.candidates.len());
            self.candidates[pick]
        };

        Selection {
            server_id: choice,
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
    fn least_connections_prefers_lowest_active_connections() {
        let servers = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 3,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 2,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
        ];
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut strategy = LeastConnectionsStrategy::default();
        let mut ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        assert_eq!(strategy.select(&mut ctx).server_id, 1);
    }

    #[test]
    fn least_connections_uses_seeded_tiebreak() {
        let servers = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
                next_available_ms: 0,
            },
        ];
        let candidates = [0usize, 1, 2];
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut strategy = LeastConnectionsStrategy::default();
        let mut ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        assert_eq!(strategy.select(&mut ctx).server_id, expected);
    }
}
