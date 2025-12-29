use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::models::{
    Algorithm, Assignment, Server, ServerSummary, SimError, SimResult, SimulationResult, TieBreak,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InFlight {
    completes_at: u64,
    server_id: usize,
}

impl Ord for InFlight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.completes_at
            .cmp(&other.completes_at)
            .then_with(|| self.server_id.cmp(&other.server_id))
    }
}

impl PartialOrd for InFlight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) fn run_simulation(
    mut servers: Vec<Server>,
    algo: Algorithm,
    request_count: usize,
    tie_break: TieBreak,
) -> SimResult<SimulationResult> {
    if servers.is_empty() {
        return Err(SimError::EmptyServers);
    }
    let mut id_to_index = HashMap::new();
    for (idx, server) in servers.iter().enumerate() {
        if id_to_index.insert(server.id, idx).is_some() {
            return Err(SimError::DuplicateServerId(server.id));
        }
    }
    let mut assignments = Vec::with_capacity(request_count);
    let mut rng = match &tie_break {
        TieBreak::Seeded(seed) => Some(StdRng::seed_from_u64(*seed)),
        TieBreak::Stable => None,
    };
    let mut next_idx = 0usize;
    let mut in_flight: BinaryHeap<Reverse<InFlight>> = BinaryHeap::new();

    for (current_time, request_id) in (1..=request_count).enumerate() {
        let current_time = current_time as u64;
        while let Some(Reverse(in_flight_request)) = in_flight.peek() {
            if in_flight_request.completes_at > current_time {
                break;
            }
            let completed = in_flight.pop().unwrap();
            let server_idx = completed.0.server_id;
            servers[server_idx].active_connections -= 1;
        }

        let (server_idx, score) = match algo {
            Algorithm::RoundRobin => (pick_round_robin(&mut next_idx, servers.len()), None),
            Algorithm::WeightedRoundRobin => {
                (pick_weighted_round_robin(&mut next_idx, &servers), None)
            }
            Algorithm::LeastConnections => {
                let idx = pick_least_connections(&servers, rng.as_mut());
                (idx, None)
            }
            Algorithm::LeastResponseTime => {
                let (idx, score) = pick_least_response_time(&servers, rng.as_mut());
                (idx, Some(score))
            }
        };

        servers[server_idx].active_connections += 1;
        servers[server_idx].pick_count += 1;
        let started_at = current_time;
        let completed_at = started_at + servers[server_idx].base_latency_ms;
        in_flight.push(Reverse(InFlight {
            completes_at: completed_at,
            server_id: server_idx,
        }));

        assignments.push(Assignment {
            request_id,
            server_id: servers[server_idx].id,
            server_name: servers[server_idx].name.clone(),
            score,
            started_at,
            completed_at,
        });
    }

    let mut counts = vec![0u32; servers.len()];
    let mut total_response_ms = vec![0u64; servers.len()];
    for assignment in &assignments {
        let idx = id_to_index[&assignment.server_id];
        counts[idx] += 1;
        total_response_ms[idx] += assignment.completed_at - assignment.started_at;
    }

    let totals = servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let count = counts[idx];
            let avg_response_ms = if count == 0 {
                0
            } else {
                total_response_ms[idx] / count as u64
            };
            ServerSummary {
                name: server.name.clone(),
                requests: count,
                avg_response_ms,
            }
        })
        .collect();

    Ok(SimulationResult {
        assignments,
        totals,
        tie_break,
    })
}

fn pick_round_robin(next_idx: &mut usize, len: usize) -> usize {
    let idx = *next_idx;
    *next_idx = (*next_idx + 1) % len;
    idx
}

fn pick_weighted_round_robin(next_idx: &mut usize, servers: &[Server]) -> usize {
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

fn pick_least_connections(servers: &[Server], rng: Option<&mut StdRng>) -> usize {
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
    fn least_connections_accounts_for_completed_requests() {
        let servers = vec![
            Server::test_at(0, "fast", 1, 1, 0, 0),
            Server::test_at(1, "slow", 100, 1, 0, 0),
        ];
        let result = run_simulation(servers, Algorithm::LeastConnections, 2, TieBreak::Stable)
            .expect("simulation should succeed");
        let assigned = result
            .assignments
            .iter()
            .map(|assignment| assignment.server_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(assigned, vec!["fast", "fast"]);
    }

    #[test]
    fn assignments_include_response_time_metrics() {
        let servers = vec![Server::test_at(0, "api", 5, 1, 0, 0)];
        let result = run_simulation(servers, Algorithm::RoundRobin, 2, TieBreak::Stable)
            .expect("simulation should succeed");

        let started: Vec<u64> = result
            .assignments
            .iter()
            .map(|assignment| assignment.started_at)
            .collect();
        assert_eq!(started, vec![0, 1]);

        let completed: Vec<u64> = result
            .assignments
            .iter()
            .map(|assignment| assignment.completed_at)
            .collect();
        assert_eq!(completed, vec![5, 6]);

        assert_eq!(result.totals[0].avg_response_ms, 5);
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

    #[test]
    fn summary_preserves_input_order() {
        let servers = vec![
            Server::test_at(0, "api", 10, 1, 0, 0),
            Server::test_at(1, "db", 20, 1, 0, 0),
            Server::test_at(2, "cache", 30, 1, 0, 0),
        ];
        let result = run_simulation(servers, Algorithm::RoundRobin, 2, TieBreak::Stable)
            .expect("simulation should succeed");
        let names: Vec<&str> = result
            .totals
            .iter()
            .map(|summary| summary.name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "db", "cache"]);
    }

    #[test]
    fn duplicate_server_ids_error() {
        let servers = vec![
            Server::test_at(1, "a", 10, 1, 0, 0),
            Server::test_at(1, "b", 20, 1, 0, 0),
        ];
        let result = run_simulation(servers, Algorithm::RoundRobin, 1, TieBreak::Stable);
        assert!(result.is_err());
    }

    #[test]
    fn empty_servers_error() {
        let result = run_simulation(Vec::new(), Algorithm::RoundRobin, 1, TieBreak::Stable);
        assert!(result.is_err());
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
