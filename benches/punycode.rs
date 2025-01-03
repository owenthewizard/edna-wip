#![allow(clippy::missing_panics_doc)]
#![allow(unused_imports)]

use std::thread::sleep;
use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use rand::{
    distributions::{DistString, Standard},
    rngs::SmallRng,
    SeedableRng,
};

const SEED: u64 = 0x5EED_5EED;
const BYTES: [usize; 3] = [16, 32, 63];
const LIBRARIES: [&str; 3] = ["edna", "idna", "punycode"];

fn encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    let mut _rng = SmallRng::seed_from_u64(SEED);

    for lib in LIBRARIES {
        for bytes in BYTES {
            group.throughput(Throughput::Bytes(u64::try_from(bytes).unwrap()));

            match lib {
                #[cfg(all(feature = "benchmark-encode", feature = "benchmark-idna"))]
                "idna" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || Standard.sample_string(&mut _rng, bytes),
                            |i| black_box(idna::punycode::encode_str(i)).unwrap(),
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                #[cfg(feature = "benchmark-encode")]
                "edna" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || Standard.sample_string(&mut _rng, bytes),
                            |i| {
                                black_box(edna::punycode::encode(i));
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                #[cfg(all(feature = "benchmark-encode", feature = "benchmark-punycode"))]
                "punycode" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || Standard.sample_string(&mut _rng, bytes),
                            |i| {
                                black_box(punycode::encode(i)).unwrap();
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                // skip disabled benchmark
                #[allow(unreachable_patterns)]
                _ => (),
            }
        }
    }
}

fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    let mut _rng = SmallRng::seed_from_u64(SEED);

    for lib in LIBRARIES {
        for bytes in BYTES {
            group.throughput(Throughput::Bytes(u64::try_from(bytes).unwrap()));

            match lib {
                #[cfg(all(feature = "benchmark-decode", feature = "benchmark-idna"))]
                "idna" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || edna::punycode::encode(&Standard.sample_string(&mut _rng, bytes)),
                            |i| {
                                black_box(idna::punycode::decode_to_string(i)).unwrap();
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                #[cfg(feature = "benchmark-decode")]
                "edna" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || edna::punycode::encode(&Standard.sample_string(&mut _rng, bytes)),
                            |i| {
                                black_box(edna::punycode::decode(i)).unwrap();
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));

                    #[cfg(not(feature = "forbid-unsafe"))]
                    group.bench_function(BenchmarkId::new("edna_unchecked", bytes), |b| {
                        b.iter_batched_ref(
                            || edna::punycode::encode(&Standard.sample_string(&mut _rng, bytes)),
                            |i| {
                                unsafe { black_box(edna::punycode::decode_unchecked(i)) };
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                #[cfg(all(feature = "benchmark-decode", feature = "benchmark-punycode"))]
                "punycode" => {
                    group.bench_function(BenchmarkId::new(lib, bytes), |b| {
                        b.iter_batched_ref(
                            || edna::punycode::encode(&Standard.sample_string(&mut _rng, bytes)),
                            |i| {
                                black_box(punycode::decode(i)).unwrap();
                            },
                            BatchSize::SmallInput,
                        );
                    });
                    sleep(Duration::from_secs(5));
                }

                // skip disabled benchmark
                #[allow(unreachable_patterns)]
                _ => (),
            }
        }
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
