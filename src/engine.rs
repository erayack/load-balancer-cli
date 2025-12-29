use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

use crate::algorithms::{
    pick_least_connections, pick_least_response_time, pick_round_robin, pick_weighted_round_robin,
};
use crate::error::{SimError, SimResult};
use crate::models::{AlgoConfig, RequestProfile, ServerConfig, SimConfig, TieBreakConfig};
use crate::state::{Assignment, Request, ServerState, ServerSummary, SimulationResult};

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

pub fn run_simulation(config: &SimConfig) -> SimResult<SimulationResult> {
    validate_config(config)?;
    let requests = build_requests(&config.requests, config.seed)?;
    let mut servers = init_server_state(&config.servers);
    let mut assignments = Vec::with_capacity(requests.len());
    let mut rng = match config.tie_break {
        TieBreakConfig::Seeded => Some(StdRng::seed_from_u64(config.seed.unwrap())),
        TieBreakConfig::Stable => None,
    };
    let mut next_idx = 0usize;
    let mut in_flight: BinaryHeap<Reverse<InFlight>> = BinaryHeap::new();

    for request in requests {
        let current_time = request.arrival_ms;
        while let Some(Reverse(in_flight_request)) = in_flight.peek() {
            if in_flight_request.completes_at > current_time {
                break;
            }
            let completed = in_flight.pop().unwrap();
            let server_idx = completed.0.server_id;
            servers[server_idx].active_connections -= 1;
            servers[server_idx].in_flight -= 1;
        }

        let (server_idx, score) = match config.algo {
            AlgoConfig::RoundRobin => (pick_round_robin(&mut next_idx, servers.len()), None),
            AlgoConfig::WeightedRoundRobin => (
                pick_weighted_round_robin(&mut next_idx, &config.servers),
                None,
            ),
            AlgoConfig::LeastConnections => {
                let idx = pick_least_connections(&servers, rng.as_mut());
                (idx, None)
            }
            AlgoConfig::LeastResponseTime => {
                let (idx, score) =
                    pick_least_response_time(&config.servers, &servers, rng.as_mut());
                (idx, Some(score))
            }
        };

        servers[server_idx].active_connections += 1;
        servers[server_idx].pick_count += 1;
        servers[server_idx].in_flight += 1;
        let started_at = current_time;
        let completed_at = started_at + config.servers[server_idx].base_latency_ms;
        in_flight.push(Reverse(InFlight {
            completes_at: completed_at,
            server_id: server_idx,
        }));

        assignments.push(Assignment {
            request_id: request.id,
            server_id: servers[server_idx].id,
            score,
            started_at,
            completed_at,
        });
    }

    let mut counts = vec![0u32; servers.len()];
    let mut total_response_ms = vec![0u64; servers.len()];
    for assignment in &assignments {
        let idx = assignment.server_id;
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
        tie_break: config.tie_break.clone(),
        seed: config.seed,
    })
}

fn validate_config(config: &SimConfig) -> SimResult<()> {
    if config.servers.is_empty() {
        return Err(SimError::EmptyServers);
    }
    let mut names = HashSet::new();
    for server in &config.servers {
        if server.name.trim().is_empty() {
            return Err(SimError::InvalidServerEntry(server.name.clone()));
        }
        if server.base_latency_ms == 0 {
            return Err(SimError::InvalidLatencyValue(server.name.clone()));
        }
        if server.weight == 0 {
            return Err(SimError::InvalidWeightValue(server.name.clone()));
        }
        if names.contains(&server.name) {
            return Err(SimError::DuplicateServerName(server.name.clone()));
        }
        names.insert(server.name.clone());
    }

    match config.requests {
        RequestProfile::FixedCount(0) => return Err(SimError::RequestsZero),
        RequestProfile::FixedCount(_) => {}
        RequestProfile::Poisson { rate, duration_ms } => {
            if rate <= 0.0 {
                return Err(SimError::InvalidRequestRate(rate));
            }
            if duration_ms == 0 {
                return Err(SimError::InvalidRequestDuration(duration_ms));
            }
        }
    }

    if matches!(config.tie_break, TieBreakConfig::Seeded) && config.seed.is_none() {
        return Err(SimError::InvalidTieBreakSeed);
    }

    Ok(())
}

fn build_requests(profile: &RequestProfile, seed: Option<u64>) -> SimResult<Vec<Request>> {
    match profile {
        RequestProfile::FixedCount(count) => {
            if *count == 0 {
                return Err(SimError::RequestsZero);
            }
            Ok((0..*count)
                .map(|idx| Request {
                    id: idx + 1,
                    arrival_ms: idx as u64,
                })
                .collect())
        }
        RequestProfile::Poisson { rate, duration_ms } => {
            if *rate <= 0.0 {
                return Err(SimError::InvalidRequestRate(*rate));
            }
            if *duration_ms == 0 {
                return Err(SimError::InvalidRequestDuration(*duration_ms));
            }

            let mut rng = StdRng::seed_from_u64(seed.unwrap_or(0));
            let lambda_ms = rate / 1000.0;
            let mut requests = Vec::new();
            let mut time = 0.0;
            let mut id = 1usize;
            while time < *duration_ms as f64 {
                let mut u = rng.gen::<f64>();
                if u <= f64::MIN_POSITIVE {
                    u = f64::MIN_POSITIVE;
                }
                let inter_arrival = -u.ln() / lambda_ms;
                time += inter_arrival;
                if time >= *duration_ms as f64 {
                    break;
                }
                requests.push(Request {
                    id,
                    arrival_ms: time.floor() as u64,
                });
                id += 1;
            }

            if requests.is_empty() {
                return Err(SimError::RequestsZero);
            }

            Ok(requests)
        }
    }
}

fn init_server_state(servers: &[ServerConfig]) -> Vec<ServerState> {
    servers
        .iter()
        .enumerate()
        .map(|(id, server)| ServerState {
            id,
            name: server.name.clone(),
            active_connections: 0,
            pick_count: 0,
            in_flight: 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_servers(servers: Vec<ServerConfig>) -> SimConfig {
        SimConfig {
            servers,
            requests: RequestProfile::FixedCount(1),
            algo: AlgoConfig::RoundRobin,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        }
    }

    #[test]
    fn least_connections_accounts_for_completed_requests() {
        let config = SimConfig {
            servers: vec![
                ServerConfig {
                    name: "fast".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
                ServerConfig {
                    name: "slow".to_string(),
                    base_latency_ms: 100,
                    weight: 1,
                },
            ],
            requests: RequestProfile::FixedCount(2),
            algo: AlgoConfig::LeastConnections,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        };
        let result = run_simulation(&config).expect("simulation should succeed");
        let assigned = result
            .assignments
            .iter()
            .map(|assignment| assignment.server_id)
            .collect::<Vec<_>>();
        assert_eq!(assigned, vec![0, 0]);
    }

    #[test]
    fn assignments_include_response_time_metrics() {
        let config = SimConfig {
            servers: vec![ServerConfig {
                name: "api".to_string(),
                base_latency_ms: 5,
                weight: 1,
            }],
            requests: RequestProfile::FixedCount(2),
            algo: AlgoConfig::RoundRobin,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        };
        let result = run_simulation(&config).expect("simulation should succeed");

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
        let config = SimConfig {
            servers: vec![
                ServerConfig {
                    name: "api".to_string(),
                    base_latency_ms: 10,
                    weight: 1,
                },
                ServerConfig {
                    name: "db".to_string(),
                    base_latency_ms: 20,
                    weight: 1,
                },
                ServerConfig {
                    name: "cache".to_string(),
                    base_latency_ms: 30,
                    weight: 1,
                },
            ],
            requests: RequestProfile::FixedCount(2),
            algo: AlgoConfig::RoundRobin,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        };
        let result = run_simulation(&config).expect("simulation should succeed");
        let names: Vec<&str> = result
            .totals
            .iter()
            .map(|summary| summary.name.as_str())
            .collect();
        assert_eq!(names, vec!["api", "db", "cache"]);
    }

    #[test]
    fn duplicate_server_names_error() {
        let config = config_with_servers(vec![
            ServerConfig {
                name: "a".to_string(),
                base_latency_ms: 10,
                weight: 1,
            },
            ServerConfig {
                name: "a".to_string(),
                base_latency_ms: 20,
                weight: 1,
            },
        ]);
        let result = run_simulation(&config);
        assert!(result.is_err());
    }

    #[test]
    fn empty_servers_error() {
        let config = SimConfig {
            servers: Vec::new(),
            requests: RequestProfile::FixedCount(1),
            algo: AlgoConfig::RoundRobin,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        };
        let result = run_simulation(&config);
        assert!(result.is_err());
    }
}
