use rand::Rng;

use crate::algorithms::{Selection, SelectionContext, SelectionStrategy};

#[derive(Default)]
pub struct LeastResponseTimeStrategy;

impl SelectionStrategy for LeastResponseTimeStrategy {
    fn select(&mut self, ctx: &SelectionContext) -> Selection {
        let mut min_score = u64::MAX;
        let mut candidates = Vec::new();

        for (idx, server) in ctx.servers.iter().enumerate() {
            let score = server.base_latency_ms + (server.pick_count as u64 * 10);
            if score < min_score {
                min_score = score;
                candidates.clear();
                candidates.push(idx);
            } else if score == min_score {
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
            score: Some(min_score),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ServerState;
    use rand::SeedableRng;

    #[test]
    fn least_response_time_prefers_lowest_score() {
        let servers = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 30,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 0,
                pick_count: 2,
                in_flight: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 20,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
            },
        ];
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut strategy = LeastResponseTimeStrategy::default();
        let ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        let selection = strategy.select(&ctx);
        assert_eq!(selection.server_id, 2);
        assert_eq!(selection.score, Some(20));
    }

    #[test]
    fn least_response_time_uses_seeded_tiebreak() {
        let servers = vec![
            ServerState {
                id: 0,
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
            },
            ServerState {
                id: 1,
                name: "b".to_string(),
                base_latency_ms: 0,
                weight: 1,
                active_connections: 0,
                pick_count: 1,
                in_flight: 0,
            },
            ServerState {
                id: 2,
                name: "c".to_string(),
                base_latency_ms: 20,
                weight: 1,
                active_connections: 0,
                pick_count: 0,
                in_flight: 0,
            },
        ];
        let candidates = [0usize, 1];
        let mut rng = rand::rngs::StdRng::seed_from_u64(99);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(99);
        let mut strategy = LeastResponseTimeStrategy::default();
        let ctx = SelectionContext {
            servers: &servers,
            time_ms: 0,
            rng: &mut rng,
        };

        let selection = strategy.select(&ctx);
        assert_eq!(selection.server_id, expected);
        assert_eq!(selection.score, Some(10));
    }
}
