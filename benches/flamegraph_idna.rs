#![no_std]
#![no_main]
#![allow(clippy::missing_panics_doc)]

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
        "مثال.إختبار",
        "مثال.آزمایشی",
        "例子.测试",
        "例子.測試",
        "пример.испытание",
        "उदाहरण.परीक्षा",
        "παράδειγμα.δοκιμή",
        "실례.테스트",
        "בײַשפּיל.טעסט",
        "例え.テスト",
        "உதாரணம்.பரிட்சை ",
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
        let _ = black_box(idna::domain_to_ascii(d)).unwrap();
    }

    0
}
