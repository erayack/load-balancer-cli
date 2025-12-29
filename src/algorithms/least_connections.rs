use rand::Rng;

use crate::algorithms::{Selection, SelectionContext, SelectionStrategy};

#[derive(Default)]
pub struct LeastConnectionsStrategy;

impl SelectionStrategy for LeastConnectionsStrategy {
    fn select(&mut self, ctx: &SelectionContext) -> Selection {
        let mut min_count = u32::MAX;
        let mut candidates = Vec::new();

        for (idx, server) in ctx.servers.iter().enumerate() {
            if server.active_connections < min_count {
                min_count = server.active_connections;
                candidates.clear();
                candidates.push(idx);
            } else if server.active_connections == min_count {
                candidates.push(idx);
            }
        }

        let choice = if candidates.len() == 1 {
            candidates[0]
        } else {
            #[allow(invalid_reference_casting)]
            let rng =
                unsafe { &mut *(ctx.rng as *const dyn rand::RngCore as *mut dyn rand::RngCore) };
            let pick = rng.gen_range(0..candidates.len());
            candidates[pick]
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
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 2,
                pick_count: 0,
                in_flight: 0,
            },
        ];
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut strategy = LeastConnectionsStrategy;
        let ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        assert_eq!(strategy.select(&ctx).server_id, 1);
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
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 1,
                pick_count: 0,
                in_flight: 0,
            },
        ];
        let candidates = [0usize, 1, 2];
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut strategy = LeastConnectionsStrategy;
        let ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        assert_eq!(strategy.select(&ctx).server_id, expected);
    }
}
