use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

use crate::algorithms::{build_strategy, SelectionContext, SelectionStrategy};
use crate::error::{Error, Result};
use crate::events::{Event, Request, ScheduledEvent};
use crate::models::{RequestProfile, ServerConfig, SimConfig, TieBreakConfig};
use crate::state::{
    Assignment, EngineState, Phase1Metrics, ResponseTimePercentiles, RunMetadata, ServerState,
    ServerSummary, ServerUtilization, SimulationResult,
};

pub struct SimulationEngine {
    pub config: SimConfig,
    pub state: EngineState,
    pub strategy: Box<dyn SelectionStrategy>,
    pub rng: StdRng,
}

impl SimulationEngine {
    pub fn new(config: SimConfig, strategy: Box<dyn SelectionStrategy>) -> Self {
        let seed = match config.tie_break {
            TieBreakConfig::Seeded => config.seed.unwrap_or(0),
            TieBreakConfig::Stable => 0,
        };
        let rng = StdRng::seed_from_u64(seed);
        let state = EngineState {
            time_ms: 0,
            servers: Vec::new(),
            assignments: Vec::new(),
        };

        Self {
            config,
            state,
            strategy,
            rng,
        }
    }

    pub fn run(&mut self, store_assignments: bool) -> Result<SimulationResult> {
        validate_config(&self.config)?;
        let requests = build_requests(&self.config.requests, self.config.seed)?;

        self.state.servers = init_server_state(&self.config.servers);
        if store_assignments {
            self.state.assignments = Vec::with_capacity(requests.len());
        } else {
            self.state.assignments = Vec::new();
        }

        let mut counts = vec![0u32; self.state.servers.len()];
        let mut total_response_ms = vec![0u64; self.state.servers.len()];
        let mut total_service_ms = vec![0u64; self.state.servers.len()];
        let mut response_times = Vec::with_capacity(requests.len());
        let mut total_wait_ms = 0u64;
        let mut duration_ms = 0;
        let mut first_arrival_ms: Option<u64> = None;

        let mut events: BinaryHeap<Reverse<ScheduledEvent>> = BinaryHeap::new();
        for request in requests {
            first_arrival_ms = Some(match first_arrival_ms {
                Some(current) => current.min(request.arrival_time_ms),
                None => request.arrival_time_ms,
            });
            events.push(Reverse(ScheduledEvent::new(
                request.arrival_time_ms,
                Event::RequestArrival(request),
            )));
        }

        let mut stable_rng = StableRng;

        while let Some(Reverse(scheduled)) = events.pop() {
            self.state.time_ms = scheduled.time_ms;
            match scheduled.event {
                Event::RequestComplete { server_id, .. } => {
                    let server = &mut self.state.servers[server_id];
                    server.active_connections -= 1;
                    server.in_flight -= 1;
                }
                Event::RequestArrival(request) => {
                    let rng: &mut dyn RngCore = match self.config.tie_break {
                        TieBreakConfig::Stable => &mut stable_rng,
                        TieBreakConfig::Seeded => &mut self.rng,
                    };
                    let mut ctx = SelectionContext {
                        servers: &self.state.servers,
                        time_ms: self.state.time_ms,
                        rng,
                    };
                    let selection = self.strategy.select(&mut ctx);
                    let server_idx = selection.server_id;

                    let server = &mut self.state.servers[server_idx];
                    server.active_connections += 1;
                    server.pick_count += 1;
                    server.in_flight += 1;

                    let started_at = self.state.time_ms.max(server.next_available_ms);
                    let completed_at = started_at + server.base_latency_ms;
                    server.next_available_ms = completed_at;
                    let response_time = completed_at - request.arrival_time_ms;
                    let service_time = completed_at - started_at;
                    let wait_time = started_at.saturating_sub(request.arrival_time_ms);
                    counts[server_idx] += 1;
                    total_response_ms[server_idx] += response_time;
                    total_service_ms[server_idx] += service_time;
                    response_times.push(response_time);
                    total_wait_ms += wait_time;
                    duration_ms = duration_ms.max(completed_at);
                    events.push(Reverse(ScheduledEvent::new(
                        completed_at,
                        Event::RequestComplete {
                            server_id: server_idx,
                            request_id: request.id,
                        },
                    )));

                    if store_assignments {
                        self.state.assignments.push(Assignment {
                            request_id: request.id,
                            server_id: server_idx,
                            arrival_time_ms: request.arrival_time_ms,
                            started_at,
                            completed_at,
                            score: selection.score,
                        });
                    }
                }
            }
        }

        let totals = self
            .state
            .servers
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

        response_times.sort_unstable();
        let p95_ms = nearest_rank_percentile(&response_times, 95.0);
        let p99_ms = nearest_rank_percentile(&response_times, 99.0);
        let active_duration_ms = match self.config.requests {
            RequestProfile::Burst { at_ms, .. } if at_ms > 0 => {
                duration_ms.saturating_sub(first_arrival_ms.unwrap_or(0))
            }
            _ => duration_ms,
        };

        let per_server_utilization = self
            .state
            .servers
            .iter()
            .enumerate()
            .map(|(idx, server)| {
                let busy_time_ms = total_service_ms[idx];
                let utilization_pct = if active_duration_ms == 0 {
                    0.0
                } else {
                    (busy_time_ms as f64 / active_duration_ms as f64) * 100.0
                };
                ServerUtilization {
                    name: server.name.clone(),
                    utilization_pct: round_to(utilization_pct, 2),
                }
            })
            .collect::<Vec<_>>();
        let total_requests = counts.iter().copied().map(u64::from).sum::<u64>();
        let throughput_rps = if active_duration_ms == 0 {
            0.0
        } else {
            (total_requests as f64 / active_duration_ms as f64) * 1000.0
        };
        let avg_wait_ms = if total_requests == 0 {
            0
        } else {
            total_wait_ms / total_requests
        };
        let sum = counts.iter().copied().map(f64::from).sum::<f64>();
        let sum_sq = counts
            .iter()
            .copied()
            .map(f64::from)
            .map(|value| value * value)
            .sum::<f64>();
        let jain_fairness = if sum == 0.0 || sum_sq == 0.0 {
            0.0
        } else {
            (sum * sum) / (counts.len() as f64 * sum_sq)
        };

        Ok(SimulationResult {
            assignments: if store_assignments {
                std::mem::take(&mut self.state.assignments)
            } else {
                Vec::new()
            },
            totals,
            metadata: RunMetadata {
                algo: self.config.algo.to_string(),
                tie_break: self.config.tie_break.label_with_seed(self.config.seed),
                duration_ms: active_duration_ms,
            },
            phase1_metrics: Phase1Metrics {
                response_time: ResponseTimePercentiles { p95_ms, p99_ms },
                per_server_utilization,
                jain_fairness: round_to(jain_fairness, 4),
                throughput_rps: round_to(throughput_rps, 2),
                avg_wait_ms,
            },
        })
    }
}

pub fn run_simulation(config: &SimConfig) -> Result<SimulationResult> {
    run_simulation_with_options(config, true)
}

pub fn run_simulation_summary(config: &SimConfig) -> Result<SimulationResult> {
    run_simulation_with_options(config, false)
}

pub fn run_simulation_with_options(
    config: &SimConfig,
    store_assignments: bool,
) -> Result<SimulationResult> {
    let strategy = build_strategy(config.algo.clone());
    let mut engine = SimulationEngine::new(config.clone(), strategy);
    engine.run(store_assignments)
}

fn validate_config(config: &SimConfig) -> Result<()> {
    if config.servers.is_empty() {
        return Err(Error::EmptyServers);
    }
    let mut names = HashSet::new();
    for server in &config.servers {
        if server.name.trim().is_empty() {
            return Err(Error::InvalidServerEntry(server.name.clone()));
        }
        if server.base_latency_ms == 0 {
            return Err(Error::InvalidLatencyValue(server.name.clone()));
        }
        if server.weight == 0 {
            return Err(Error::InvalidWeightValue(server.name.clone()));
        }
        if names.contains(&server.name) {
            return Err(Error::DuplicateServerName(server.name.clone()));
        }
        names.insert(server.name.clone());
    }

    match config.requests {
        RequestProfile::FixedCount(0) => return Err(Error::RequestsZero),
        RequestProfile::FixedCount(_) => {}
        RequestProfile::Poisson { rate, duration_ms } => {
            if rate <= 0.0 {
                return Err(Error::InvalidRequestRate(rate));
            }
            if duration_ms == 0 {
                return Err(Error::InvalidRequestDuration(duration_ms));
            }
        }
        RequestProfile::Burst { count, .. } => {
            if count == 0 {
                return Err(Error::RequestsZero);
            }
        }
    }

    if matches!(config.tie_break, TieBreakConfig::Seeded) && config.seed.is_none() {
        return Err(Error::InvalidTieBreakSeed);
    }

    Ok(())
}

fn build_requests(profile: &RequestProfile, seed: Option<u64>) -> Result<Vec<Request>> {
    match profile {
        RequestProfile::FixedCount(count) => {
            if *count == 0 {
                return Err(Error::RequestsZero);
            }
            Ok((0..*count)
                .map(|idx| Request {
                    id: idx + 1,
                    arrival_time_ms: idx as u64,
                })
                .collect())
        }
        RequestProfile::Poisson { rate, duration_ms } => {
            if *rate <= 0.0 {
                return Err(Error::InvalidRequestRate(*rate));
            }
            if *duration_ms == 0 {
                return Err(Error::InvalidRequestDuration(*duration_ms));
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
                    arrival_time_ms: time.floor() as u64,
                });
                id += 1;
            }

            if requests.is_empty() {
                return Err(Error::RequestsZero);
            }

            Ok(requests)
        }
        RequestProfile::Burst { count, at_ms } => {
            if *count == 0 {
                return Err(Error::RequestsZero);
            }
            Ok((0..*count)
                .map(|idx| Request {
                    id: idx + 1,
                    arrival_time_ms: *at_ms,
                })
                .collect())
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
            base_latency_ms: server.base_latency_ms,
            weight: server.weight,
            active_connections: 0,
            pick_count: 0,
            in_flight: 0,
            next_available_ms: 0,
        })
        .collect()
}

struct StableRng;

impl RngCore for StableRng {
    fn next_u32(&mut self) -> u32 {
        0
    }

    fn next_u64(&mut self) -> u64 {
        0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> std::result::Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

fn nearest_rank_percentile(sorted: &[u64], percentile: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((percentile / 100.0) * sorted.len() as f64).ceil() as usize;
    let idx = rank.saturating_sub(1).min(sorted.len() - 1);
    Some(sorted[idx])
}

fn round_to(value: f64, decimals: u32) -> f64 {
    if decimals == 0 {
        return value.round();
    }
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AlgoConfig;

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
    fn seeded_tiebreak_is_deterministic_in_engine() {
        let config = SimConfig {
            servers: vec![
                ServerConfig {
                    name: "a".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
                ServerConfig {
                    name: "b".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
                ServerConfig {
                    name: "c".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
            ],
            requests: RequestProfile::FixedCount(3),
            algo: AlgoConfig::LeastConnections,
            tie_break: TieBreakConfig::Seeded,
            seed: Some(42),
        };
        let result_a = run_simulation(&config).expect("simulation should succeed");
        let result_b = run_simulation(&config).expect("simulation should succeed");

        let actual = result_a
            .assignments
            .iter()
            .map(|assignment| assignment.server_id)
            .collect::<Vec<_>>();
        let expected = result_b
            .assignments
            .iter()
            .map(|assignment| assignment.server_id)
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
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
        assert_eq!(started, vec![0, 5]);

        let completed: Vec<u64> = result
            .assignments
            .iter()
            .map(|assignment| assignment.completed_at)
            .collect();
        assert_eq!(completed, vec![5, 10]);

        let arrivals: Vec<u64> = result
            .assignments
            .iter()
            .map(|assignment| assignment.arrival_time_ms)
            .collect();
        assert_eq!(arrivals, vec![0, 1]);

        assert_eq!(result.totals[0].avg_response_ms, 7);
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

    #[test]
    fn phase1_metrics_are_deterministic() {
        let config = SimConfig {
            servers: vec![
                ServerConfig {
                    name: "a".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
                ServerConfig {
                    name: "b".to_string(),
                    base_latency_ms: 1,
                    weight: 1,
                },
            ],
            requests: RequestProfile::FixedCount(2),
            algo: AlgoConfig::RoundRobin,
            tie_break: TieBreakConfig::Stable,
            seed: None,
        };
        let result = run_simulation(&config).expect("simulation should succeed");

        assert_eq!(result.phase1_metrics.response_time.p95_ms, Some(1));
        assert_eq!(result.phase1_metrics.response_time.p99_ms, Some(1));
        assert_eq!(
            result
                .phase1_metrics
                .per_server_utilization
                .iter()
                .map(|entry| entry.utilization_pct)
                .collect::<Vec<_>>(),
            vec![50.0, 50.0]
        );
        assert_eq!(result.phase1_metrics.jain_fairness, 1.0);
        assert_eq!(result.phase1_metrics.throughput_rps, 1000.0);
        assert_eq!(result.phase1_metrics.avg_wait_ms, 0);
    }
}
