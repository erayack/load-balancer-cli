use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

use crate::algorithms::{build_strategy, SelectionContext, SelectionStrategy};
use crate::error::{Error, Result};
use crate::events::{Event, Request, ScheduledEvent};
use crate::models::{RequestProfile, ServerConfig, SimConfig, TieBreakConfig};
use crate::state::{
    Assignment, EngineState, RunMetadata, ServerState, ServerSummary, SimulationResult,
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

    pub fn run(&mut self) -> Result<SimulationResult> {
        validate_config(&self.config)?;
        let requests = build_requests(&self.config.requests, self.config.seed)?;

        self.state.servers = init_server_state(&self.config.servers);
        self.state.assignments = Vec::with_capacity(requests.len());

        let mut events: BinaryHeap<Reverse<ScheduledEvent>> = BinaryHeap::new();
        for request in requests {
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
                    let ctx = SelectionContext {
                        servers: &self.state.servers,
                        time_ms: self.state.time_ms,
                        rng,
                    };
                    let selection = self.strategy.select(&ctx);
                    let server_idx = selection.server_id;

                    let server = &mut self.state.servers[server_idx];
                    server.active_connections += 1;
                    server.pick_count += 1;
                    server.in_flight += 1;

                    let started_at = self.state.time_ms;
                    let completed_at = started_at + server.base_latency_ms;
                    events.push(Reverse(ScheduledEvent::new(
                        completed_at,
                        Event::RequestComplete {
                            server_id: server_idx,
                            request_id: request.id,
                        },
                    )));

                    self.state.assignments.push(Assignment {
                        request_id: request.id,
                        server_id: server_idx,
                        server_name: server.name.clone(),
                        started_at,
                        completed_at,
                        score: selection.score,
                    });
                }
            }
        }

        let mut counts = vec![0u32; self.state.servers.len()];
        let mut total_response_ms = vec![0u64; self.state.servers.len()];
        for assignment in &self.state.assignments {
            let idx = assignment.server_id;
            counts[idx] += 1;
            total_response_ms[idx] += assignment.completed_at - assignment.started_at;
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

        let duration_ms = self
            .state
            .assignments
            .iter()
            .map(|assignment| assignment.completed_at)
            .max()
            .unwrap_or(0);

        Ok(SimulationResult {
            assignments: self.state.assignments.clone(),
            totals,
            metadata: RunMetadata {
                algo: self.config.algo.to_string(),
                tie_break: self.config.tie_break.label_with_seed(self.config.seed),
                duration_ms,
            },
        })
    }
}

pub fn run_simulation(config: &SimConfig) -> Result<SimulationResult> {
    let strategy = build_strategy(config.algo.clone());
    let mut engine = SimulationEngine::new(config.clone(), strategy);
    engine.run()
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
