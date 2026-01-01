use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use lb_sim::events::{Event, Request, ScheduledEvent};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const EVENT_COUNTS: &[usize] = &[128, 1_024, 8_192, 65_536];

fn build_events(count: usize) -> Vec<ScheduledEvent> {
    (0..count)
        .map(|idx| {
            let time_ms = idx as u64;
            if idx % 2 == 0 {
                ScheduledEvent::new(
                    time_ms,
                    Event::RequestArrival(Request {
                        id: idx,
                        arrival_time_ms: time_ms,
                    }),
                )
            } else {
                ScheduledEvent::new(
                    time_ms,
                    Event::RequestComplete {
                        server_id: idx % 8,
                        request_id: idx,
                    },
                )
            }
        })
        .collect()
}

fn bench_engine_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_queue");

    for &count in EVENT_COUNTS {
        group.bench_with_input(BenchmarkId::new("push_pop", count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let events = build_events(count);
                    let heap = BinaryHeap::with_capacity(events.len());
                    (heap, events)
                },
                |(mut heap, events)| {
                    for event in events {
                        heap.push(Reverse(event));
                    }
                    while let Some(event) = heap.pop() {
                        black_box(event);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_engine_queue);
criterion_main!(benches);
