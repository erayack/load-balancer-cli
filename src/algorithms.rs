use rand::rngs::StdRng;
use rand::Rng;

use crate::models::Server;

#[derive(Clone, Debug)]
pub enum Algorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LeastResponseTime,
}

pub(crate) fn pick_round_robin(next_idx: &mut usize, len: usize) -> usize {
    let idx = *next_idx;
    *next_idx = (*next_idx + 1) % len;
    idx
}

pub(crate) fn pick_weighted_round_robin(next_idx: &mut usize, servers: &[Server]) -> usize {
    let total_weight: u64 = servers.iter().map(|server| server.weight as u64).sum();
    let target = (*next_idx as u64) % total_weight;
    *next_idx = (*next_idx + 1) % (total_weight as usize);

    let mut cursor = 0u64;
    for (idx, server) in servers.iter().enumerate() {
        cursor += server.weight as u64;
        if target < cursor {
            return idx;
        }
    }

    0
}

pub(crate) fn pick_least_connections(servers: &[Server], rng: Option<&mut StdRng>) -> usize {
    let mut min_count = u32::MAX;
    let mut candidates = Vec::new();

    for (idx, server) in servers.iter().enumerate() {
        if server.active_connections < min_count {
            min_count = server.active_connections;
            candidates.clear();
            candidates.push(idx);
        } else if server.active_connections == min_count {
            candidates.push(idx);
        }
    }

    pick_index(&candidates, rng)
}

pub(crate) fn pick_least_response_time(
    servers: &[Server],
    rng: Option<&mut StdRng>,
) -> (usize, u64) {
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
    use rand::SeedableRng;

    #[test]
    fn round_robin_cycles_indices() {
        let mut next_idx = 0usize;
        assert_eq!(pick_round_robin(&mut next_idx, 3), 0);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 1);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 2);
        assert_eq!(pick_round_robin(&mut next_idx, 3), 0);
    }

    #[test]
    fn least_connections_prefers_lowest_active_connections() {
        let servers = vec![
            Server::test_at(0, "a", 10, 1, 3, 0),
            Server::test_at(1, "b", 10, 1, 1, 0),
            Server::test_at(2, "c", 10, 1, 2, 0),
        ];
        let idx = pick_least_connections(&servers, None);
        assert_eq!(idx, 1);
    }

    #[test]
    fn least_connections_tiebreaks_stably_without_seed() {
        let servers = vec![
            Server::test_at(0, "a", 10, 1, 1, 0),
            Server::test_at(1, "b", 10, 1, 2, 0),
            Server::test_at(2, "c", 10, 1, 1, 0),
        ];
        let idx = pick_least_connections(&servers, None);
        assert_eq!(idx, 0);
    }

    #[test]
    fn least_response_time_prefers_lowest_score() {
        let servers = vec![
            Server::test_at(0, "a", 30, 1, 0, 0),
            Server::test_at(1, "b", 10, 1, 0, 2),
            Server::test_at(2, "c", 20, 1, 0, 0),
        ];
        let (idx, score) = pick_least_response_time(&servers, None);
        assert_eq!(idx, 2);
        assert_eq!(score, 20);
    }

    #[test]
    fn least_connections_uses_seeded_tiebreak() {
        let servers = vec![
            Server::test_at(0, "a", 10, 1, 1, 0),
            Server::test_at(1, "b", 10, 1, 1, 0),
            Server::test_at(2, "c", 10, 1, 1, 0),
        ];
        let candidates = [0usize, 1, 2];
        let mut rng = StdRng::seed_from_u64(42);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };
        let mut rng = StdRng::seed_from_u64(42);
        let actual = pick_least_connections(&servers, Some(&mut rng));
        assert_eq!(actual, expected);
    }

    #[test]
    fn least_response_time_uses_seeded_tiebreak() {
        let servers = vec![
            Server::test_at(0, "a", 10, 1, 0, 0),
            Server::test_at(1, "b", 0, 1, 0, 1),
            Server::test_at(2, "c", 20, 1, 0, 0),
        ];
        let candidates = [0usize, 1];
        let mut rng = StdRng::seed_from_u64(99);
        let expected = {
            let choice = rng.gen_range(0..candidates.len());
            candidates[choice]
        };
        let mut rng = StdRng::seed_from_u64(99);
        let (actual, score) = pick_least_response_time(&servers, Some(&mut rng));
        assert_eq!(actual, expected);
        assert_eq!(score, 10);
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
        let actual = super::pick_index(&candidates, Some(&mut rng));
        assert_eq!(actual, expected);
    }

    #[test]
    fn weighted_round_robin_respects_weights() {
        let servers = vec![
            Server::test_at(0, "a", 10, 2, 0, 0),
            Server::test_at(1, "b", 10, 1, 0, 0),
        ];
        let mut cursor = 0usize;
        let picks: Vec<usize> = (0..6)
            .map(|_| pick_weighted_round_robin(&mut cursor, &servers))
            .collect();
        assert_eq!(picks, vec![0, 0, 1, 0, 0, 1]);
    }
}
