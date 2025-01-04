#![no_std]
#![no_main]
#![allow(clippy::missing_panics_doc)]
#![allow(unused_imports, reason = "comments")]

use core::{hint::black_box, iter};

extern crate alloc;
use alloc::{string::String, vec::Vec};

use rand::{
    distributions::{DistString, Standard},
    prelude::SliceRandom,
    rngs::SmallRng,
    SeedableRng,
};

use edna::punycode;

const SEED: u64 = 0x5EED_5EED;

#[no_mangle]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut rng = SmallRng::seed_from_u64(SEED);

    let mut utf8 = [
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
        "🧑‍💻.com",
        "exam💩ple.com",
        "",
        "🦀.☕",
    ]
    .into_iter()
    .cycle()
    .take(65536)
    .collect::<Vec<_>>();
    utf8.shuffle(&mut rng);

    /*
    let utf8 = iter::repeat_with(|| Standard.sample_string(&mut rng, 63))
        .take(65536)
        .collect::<Vec<String>>();

    let puny = utf8
        .iter()
        .map(|x| punycode::encode(x))
        .collect::<Vec<String>>();

    for d in &utf8 {
        black_box(edna::punycode::encode(d));
    }

    for d in &puny {
        let _ = black_box(edna::punycode::decode(d)).unwrap();
    }

    #[cfg(not(feature = "forbid-unsafe"))]
    for d in &puny {
        unsafe {
            black_box(edna::punycode::decode_unchecked(d));
        }
    }
    */

    for d in &utf8 {
        let _ = black_box(edna::to_ascii(d)).unwrap();
    }

    0
}
