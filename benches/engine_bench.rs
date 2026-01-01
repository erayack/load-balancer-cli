use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use lb_sim::engine::run_simulation;
use lb_sim::models::{AlgoConfig, RequestProfile, ServerConfig, SimConfig, TieBreakConfig};

const REQUESTS: usize = 1_000;
const SERVERS: usize = 8;

fn build_servers(count: usize) -> Vec<ServerConfig> {
    (0..count)
        .map(|idx| ServerConfig {
            name: format!("srv-{}", idx),
            base_latency_ms: 10 + idx as u64,
            weight: 1,
        })
        .collect()
}

fn build_config(algo: AlgoConfig) -> SimConfig {
    SimConfig {
        servers: build_servers(SERVERS),
        requests: RequestProfile::FixedCount(REQUESTS),
        algo,
        tie_break: TieBreakConfig::Stable,
        seed: None,
    }
}

fn bench_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine");
    let size_label = format!("{}x{}", REQUESTS, SERVERS);
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
                    || build_config(algo.clone()),
                    |config| {
                        let result = run_simulation(&config).expect("simulation should succeed");
                        black_box(result);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);
