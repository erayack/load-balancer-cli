use rand::rngs::StdRng;
use rand::SeedableRng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::algorithms::{
    pick_least_connections, pick_least_response_time, pick_round_robin, pick_weighted_round_robin,
    Algorithm,
};
use crate::error::{SimError, SimResult};
use crate::models::{Assignment, Server, ServerSummary, SimulationResult, TieBreak};

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

pub fn run_simulation(
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
