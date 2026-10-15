use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use rust_raptor::*;

fn bench_raptor_berlin(c: &mut Criterion) {
    let schedule = load_schedule("data/berlin-v7.1-10pct_transitSchedule.xml");
    let network = generate_network(&schedule);

    let num_stops = network.stop_ids.len();
    let mut rng = StdRng::seed_from_u64(42);
    let queries: Vec<(usize, usize)> = (0..10_000)
        .map(|_| {
            let start = rng.random_range(0..num_stops);
            let mut dest = rng.random_range(0..num_stops);
            while dest == start { dest = rng.random_range(0..num_stops); }
            (start, dest)
        })
        .collect();

    let mut i = 0;
    c.bench_function("raptor Berlin", |b| {
        b.iter(|| {
            let (start, dest) = queries[i % queries.len()];
            i += 1;
            std::hint::black_box(
                raptor(
                    &network,
                    start,
                    dest,
                    10,
                )
            )
        })
    });
}

fn bench_raptor_dresden(c: &mut Criterion) {
    let schedule = load_schedule("data/oberlausitz-dresden-v2025.0-transitSchedule.xml");
    let network = generate_network(&schedule);

    let num_stops = network.stop_ids.len();
    let mut rng = StdRng::seed_from_u64(42);
    let queries: Vec<(usize, usize)> = (0..10_000)
        .map(|_| {
            let start = rng.random_range(0..num_stops);
            let mut dest = rng.random_range(0..num_stops);
            while dest == start { dest = rng.random_range(0..num_stops); }
            (start, dest)
        })
        .collect();

    let mut i = 0;
    c.bench_function("raptor Dresden", |b| {
        b.iter(|| {
            let (start, dest) = queries[i % queries.len()];
            i += 1;
            std::hint::black_box(
                raptor(
                    &network,
                    start,
                    dest,
                    10,
                )
            )
        })
    });
}

fn bench_raptor_simple(c: &mut Criterion) {
    let schedule = load_schedule("data/simple_transitSchedule.xml");
    let network = generate_network(&schedule);

    c.bench_function("raptor Simple", |b| {
        b.iter(|| {
            std::hint::black_box(
                raptor(
                    &network,
                    1,
                    2,
                    10,
                )
            )
        })
    });
}

criterion_group!(benches, bench_raptor_berlin, bench_raptor_dresden, bench_raptor_simple);
criterion_main!(benches);