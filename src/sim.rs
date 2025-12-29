use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::models::{Algorithm, Assignment, Server, SimulationResult};

pub fn run_simulation(
    mut servers: Vec<Server>,
    algo: Algorithm,
    request_count: usize,
    seed: Option<u64>,
) -> SimulationResult {
    let mut assignments = Vec::with_capacity(request_count);
    let mut rng = seed.map(StdRng::seed_from_u64);
    let mut next_idx = 0usize;

    for request_id in 1..=request_count {
        let (server_idx, score) = match algo {
            Algorithm::RoundRobin => (pick_round_robin(&mut next_idx, servers.len()), None),
            Algorithm::LeastConnections => {
                let idx = pick_least_connections(&servers, rng.as_mut());
                (idx, None)
            }
            Algorithm::LeastResponseTime => {
                let (idx, score) = pick_least_response_time(&servers, rng.as_mut());
                (idx, Some(score))
            }
        };

        // Cumulative pick tracking for LC/LRT; active_connections is reserved for future duration modeling.
        servers[server_idx].pick_count += 1;

        assignments.push(Assignment {
            request_id,
            server_id: servers[server_idx].id,
            server_name: servers[server_idx].name.clone(),
            score,
        });
    }

    let mut counts = vec![0u32; servers.len()];
    for assignment in &assignments {
        counts[assignment.server_id] += 1;
    }

    let totals = servers
        .iter()
        .zip(counts.into_iter())
        .map(|(server, count)| (server.name.clone(), count))
        .collect();

    SimulationResult {
        assignments,
        totals,
    }
}

fn pick_round_robin(next_idx: &mut usize, len: usize) -> usize {
    let idx = *next_idx;
    *next_idx = (*next_idx + 1) % len;
    idx
}

fn pick_least_connections(servers: &[Server], rng: Option<&mut StdRng>) -> usize {
    let mut min_count = u32::MAX;
    let mut candidates = Vec::new();

    for (idx, server) in servers.iter().enumerate() {
        if server.pick_count < min_count {
            min_count = server.pick_count;
            candidates.clear();
            candidates.push(idx);
        } else if server.pick_count == min_count {
            candidates.push(idx);
        }
    }

    pick_index(&candidates, rng)
}

fn pick_least_response_time(servers: &[Server], rng: Option<&mut StdRng>) -> (usize, u64) {
    let mut min_score = u64::MAX;
    let mut candidates = Vec::new();

    for (idx, server) in servers.iter().enumerate() {
        let score = server.base_latency_ms + (server.pick_count as u64 * 10);
        if score < min_score {
            min_score = score;
            candidates.clear();
            candidates.push(idx);
        } else if score == min_score {
            candidates.push(idx);
        }
    }

    let idx = pick_index(&candidates, rng);
    (idx, min_score)
}

fn pick_index(candidates: &[usize], rng: Option<&mut StdRng>) -> usize {
    if candidates.is_empty() {
        return 0;
    }

    match rng {
        Some(rng) => {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        }
        None => candidates[0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_cycles_indices() {
        let mut next_idx = 0usize;
        assert_eq!(pick_round_robin(&mut next_idx, 3), 0);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 1);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 2);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 0);
    }

    #[test]
    fn least_connections_prefers_lowest_pick_count() {
        let servers = vec![
            Server::test_at(0, "a", 10, 3),
            Server::test_at(1, "b", 10, 1),
            Server::test_at(2, "c", 10, 2),
        ];
        let idx = pick_least_connections(&servers, None);
        assert_eq!(idx, 1);
    }

    #[test]
    fn least_connections_tiebreaks_stably_without_seed() {
        let servers = vec![
            Server::test_at(0, "a", 10, 1),
            Server::test_at(1, "b", 10, 2),
            Server::test_at(2, "c", 10, 1),
        ];
        let idx = pick_least_connections(&servers, None);
        assert_eq!(idx, 0);
    }

    #[test]
    fn least_response_time_prefers_lowest_score() {
        let servers = vec![
            Server::test_at(0, "a", 30, 0),
            Server::test_at(1, "b", 10, 2),
            Server::test_at(2, "c", 20, 0),
        ];
        let (idx, score) = pick_least_response_time(&servers, None);
        assert_eq!(idx, 2);
        assert_eq!(score, 20);
    }

    #[test]
    fn pick_index_uses_seeded_rng() {
        let candidates = vec![10usize, 20, 30];
        let mut rng = StdRng::seed_from_u64(7);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };
        let mut rng = StdRng::seed_from_u64(7);
        let actual = pick_index(&candidates, Some(&mut rng));
        assert_eq!(actual, expected);
    }
}
