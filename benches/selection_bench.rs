use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use lb_sim::algorithms::{build_strategy, SelectionContext};
use lb_sim::models::AlgoConfig;
use lb_sim::state::ServerState;
use rand::rngs::StdRng;
use rand::SeedableRng;

const SERVERS: usize = 8;
const ITERATIONS: usize = 1_000;

fn build_servers(count: usize) -> Vec<ServerState> {
    (0..count)
        .map(|idx| ServerState {
            id: idx,
            name: format!("srv-{}", idx),
            base_latency_ms: 10 + idx as u64,
            weight: 1,
            active_connections: (idx % 3) as u32,
            pick_count: (idx % 5) as u32,
            in_flight: 0,
            next_available_ms: 0,
        })
        .collect()
}

fn bench_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("selection");
    let size_label = format!("{}x{}", ITERATIONS, SERVERS);
    let algos = [
        AlgoConfig::RoundRobin,
        AlgoConfig::WeightedRoundRobin,
        AlgoConfig::LeastConnections,
        AlgoConfig::LeastResponseTime,
    ];

    for algo in algos {
        let algo_label = algo.to_string();
        group.bench_with_input(
            BenchmarkId::new(algo_label, &size_label),
            &algo,
            |b, algo: &AlgoConfig| {
                b.iter_batched(
                    || {
                        let servers = build_servers(SERVERS);
                        let rng = StdRng::seed_from_u64(1);
                        let strategy = build_strategy(algo.clone());
                        (servers, rng, strategy)
                    },
                    |(servers, mut rng, mut strategy)| {
                        let mut ctx = SelectionContext {
                            servers: &servers,
                            time_ms: 0,
                            rng: &mut rng,
                        };
                        for _ in 0..ITERATIONS {
                            let selection = strategy.select(&mut ctx);
                            black_box(selection);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_selection);
criterion_main!(benches);
