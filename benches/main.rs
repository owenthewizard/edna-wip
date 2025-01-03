use std::{thread::sleep, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::{prelude::SliceRandom, rngs::SmallRng, SeedableRng};

#[cfg(feature = "benchmark-idna")]
use idna::uts46::{AsciiDenyList, DnsLength, Hyphens, Uts46};

const LIBRARIES: [&str; 2] = ["edna", "idna"];

const DATA: &[&str] = &[
    "ﻞﻴﻬﻣﺎﺒﺘﻜﻠﻣﻮﺸﻋﺮﺒﻳ؟",
    "他们为什么不说中文",
    "他們爲什麽不說中文",
    "Pročprostěnemluvíčesky",
    "למההםפשוטלאמדבריםעברית",
    "यहलोगहिन्दीक्योंनहींबोलसकतेहैं",
    "なぜみんな日本語を話してくれないのか",
    "세계의모든사람들이한국어를이해한다면얼마나좋을까",
    "почемужеонинеговорятпорусски",
    "PorquénopuedensimplementehablarenEspañol",
    "TạisaohọkhôngthểchỉnóitiếngViệt",
    "3年B組金八先生",
    "安室奈美恵-with-SUPER-MONKEYS",
    "Hello-Another-Way-それぞれの場所",
    "ひとつ屋根の下2",
    "MajiでKoiする5秒前",
    "パフィーdeルンバ",
    "そのスピードで",
    "-> $1.00 <-",
    "münchen.de",
    "例子.测试",
    "παράδειγμα.δοκιμή",
    "www.über.com",
    //    "🧑‍💻.com",
    "exam💩ple.com",
    "",
    "🦀.☕",
];

const SEED: u64 = 0x5EED_5EED;

#[cfg(feature = "benchmark-idna")]
const UTS46: Uts46 = Uts46::new();

fn to_ascii(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut group = c.benchmark_group("to_ascii");

    for lib in LIBRARIES {
        match lib {
            #[cfg(feature = "benchmark-idna")]
            "idna" => {
                group.bench_function(lib, |b| {
                    b.iter_batched_ref(
                        || {
                            let mut x = DATA.to_vec();
                            x.shuffle(&mut rng);
                            x.into_iter()
                        },
                        |i| {
                            for x in i {
                                let _ = UTS46
                                    .to_ascii(
                                        black_box(x).as_bytes(),
                                        AsciiDenyList::EMPTY,
                                        Hyphens::Allow,
                                        DnsLength::Ignore,
                                    )
                                    // as_ptr to force returning borrowed data with invalid lifetime
                                    .unwrap()
                                    .as_ptr();
                            }
                        },
                        BatchSize::SmallInput,
                    );
                });
                sleep(Duration::from_secs(5));
            }

            "edna" => {
                group.bench_function(lib, |b| {
                    b.iter_batched_ref(
                        || {
                            let mut x = DATA.to_vec();
                            x.shuffle(&mut rng);
                            x.into_iter()
                        },
                        |i| {
                            for x in i {
                                edna::to_ascii(black_box(x)).unwrap();
                            }
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

criterion_group!(benches, to_ascii);
criterion_main!(benches);
