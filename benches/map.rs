use std::{thread::sleep, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use rand::{prelude::SliceRandom, rngs::SmallRng, SeedableRng};

use edna::*;

const SEED: u64 = 0x5EED_5EED;

fn of(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(SEED);

    let ascii = String::from("abcdefghijknmlnopqrstuvwxyZ");
    let non_ascii = String::from("abcdefghijklMnopqrstuvwxyz🦀");

    c.bench_function("map_validate(ascii)", |b| {
        b.iter_batched_ref(
            || {
                let mut x = non_ascii.clone();
                x
            },
            |i| {
                black_box(map_validate(i)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
    sleep(Duration::from_secs(5));

    c.bench_function("map_validate2(ascii)", |b| {
        b.iter_batched_ref(
            || {
                let mut x = non_ascii.clone();
                x
            },
            |i| {
                black_box(map_validate2(i)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
    sleep(Duration::from_secs(5));

    c.bench_function("validate(ascii)", |b| {
        b.iter_batched_ref(
            || {
                let mut x = non_ascii.clone();
                x
            },
            |i| {
                let _ = black_box(validate(i));
            },
            BatchSize::SmallInput,
        );
    });
    sleep(Duration::from_secs(5));

    #[cfg(not(feature = "forbid-unsafe"))]
    c.bench_function("validate2(ascii)", |b| {
        b.iter_batched_ref(
            || {
                let mut x = non_ascii.clone();
                x
            },
            |i| {
                let _ = black_box(validate2(i));
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, of);
criterion_main!(benches);
