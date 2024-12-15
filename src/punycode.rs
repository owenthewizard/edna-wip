#![cfg_attr(feature = "forbid-unsafe", forbid(unsafe_code))]

use core::cmp::Ordering;

#[cfg(not(feature = "forbid-unsafe"))]
use core::hint::unreachable_unchecked;

extern crate alloc;
use alloc::{borrow::Cow, string::String, vec::Vec};

use thiserror::Error;

const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;
const DELIMITER: char = '-';

type Utf32 = Vec<char>;

/// Encode Unicode as Punycode.
///
/// Overflow may occur if `input` is longer than 63 characters.
/// Overflow may result in invalid output, but will never result in Undefined Behavior.
///
/// # Example
/// ```
/// # use owen_idna::punycode;
/// assert_eq!(punycode::encode("München"), "Mnchen-3ya");
/// ```
pub fn encode(input: &str) -> String {
    let input = input.chars().collect::<Utf32>();
    let input_len = input.len() as u32;

    let mut output = input.iter().filter(|x| x.is_ascii()).collect::<String>();
    let basic_len = output.len() as u32;

    if basic_len > 0 {
        output.push(DELIMITER);
    }

    let mut cp = INITIAL_N;
    let mut delta = 0;
    let mut bias = INITIAL_BIAS;
    let mut processed = basic_len;

    while processed < input_len {
        let min_cp = *input.iter().filter(|&c| *c as u32 >= cp).min().unwrap() as u32;
        delta += (min_cp - cp) * (processed + 1);
        cp = min_cp;
        for c in &input {
            let c = *c as u32;
            match c.cmp(&cp) {
                Ordering::Less => delta += 1,
                Ordering::Equal => {
                    let mut q = delta;
                    let mut k = BASE;
                    loop {
                        let t = if k <= bias {
                            T_MIN
                        } else if k >= bias + T_MAX {
                            T_MAX
                        } else {
                            k - bias
                        };
                        if q < t {
                            break;
                        }
                        let value = t + ((q - t) % (BASE - t));
                        output.push(encode_digit(value));
                        q = (q - t) / (BASE - t);
                        k += BASE;
                    }
                    output.push(encode_digit(q));
                    bias = adapt(delta, processed + 1, processed == basic_len);
                    delta = 0;
                    processed += 1;
                }
                Ordering::Greater => {}
            }
        }
        delta += 1;
        cp += 1;
    }

    output
}

/// Decode Punycode to Unicode.
///
/// Overflow may occur if `input` is longer than 63 characters.
/// Overflow may result in invalid output, but will never result in Undefined Behavior.
///
/// # Example
/// ```
/// # use owen_idna::punycode;
/// assert_eq!(punycode::decode("Mnchen-3ya"), Ok(String::from("München")));
/// ```
pub fn decode(input: &str) -> Result<String, PunyDecodeError> {
    let (mut output, mut encoded) = input
        .rsplit_once('-')
        .map(|(l, r)| (l.chars().collect::<Utf32>(), r.chars()))
        .unwrap_or((Utf32::with_capacity(input.len()), input.chars()));

    let mut cp = INITIAL_N;
    let mut i: u32 = 0;
    let mut bias = INITIAL_BIAS;

    while let Some(mut byte) = encoded.next() {
        let old_i = i;
        let mut weight = 1;
        for k in (BASE..).step_by(BASE as usize) {
            let digit = decode_digit(byte).ok_or(PunyDecodeError::NonAscii)?;

            let product = digit.checked_mul(weight).ok_or(PunyDecodeError::Overflow)?;
            i = i.checked_add(product).ok_or(PunyDecodeError::Overflow)?;

            let t = if k <= bias {
                T_MIN
            } else if k >= bias + T_MAX {
                T_MAX
            } else {
                k - bias
            };

            if digit < t {
                break;
            }

            weight = weight
                .checked_mul(BASE - t)
                .ok_or(PunyDecodeError::Overflow)?;
            byte = encoded.next().ok_or(PunyDecodeError::InvalidSequence)?;
        }
        bias = adapt(i - old_i, output.len() as u32 + 1, old_i == 0);
        cp = cp
            .checked_add(i / (output.len() as u32 + 1))
            .ok_or(PunyDecodeError::Overflow)?;
        i %= output.len() as u32 + 1;
        let c = char::from_u32(cp).ok_or(PunyDecodeError::InvalidCodePoint)?;
        output.insert(i as usize, c);
        i += 1;
    }

    Ok(output.into_iter().collect::<String>())
}

/// Decode Punycode to Unicode without input validation.
///
/// `input` must never overflow, that is to say, no code points should exceed `(M - INITIAL_N) * (L
/// + 1)`. For more information, see RFC 3492.
///
/// Failure to uphold these invariants may result in **Undefined Behavior**.
///
/// # Example
/// ```
/// # use owen_idna::punycode;
/// # unsafe {
/// assert_eq!(punycode::decode_unchecked("Mnchen-3ya"), "München");
/// # }
/// ```
#[cfg(not(feature = "forbid-unsafe"))]
pub unsafe fn decode_unchecked(input: &str) -> String {
    let (mut output, mut encoded) = input
        .rsplit_once('-')
        .map(|(l, r)| (l.chars().collect::<Utf32>(), r.chars()))
        .unwrap_or((Utf32::with_capacity(input.len()), input.chars()));

    let mut cp = INITIAL_N;
    let mut i: u32 = 0;
    let mut bias = INITIAL_BIAS;

    while let Some(mut byte) = encoded.next() {
        let old_i = i;
        let mut weight = 1;
        for k in (BASE..).step_by(BASE as usize) {
            let digit = decode_digit(byte).unwrap_unchecked();

            i += digit * weight;

            let t = if k <= bias {
                T_MIN
            } else if k >= bias + T_MAX {
                T_MAX
            } else {
                k - bias
            };

            if digit < t {
                break;
            }

            weight *= BASE - t;
            byte = encoded.next().unwrap_unchecked();
        }
        bias = adapt(i - old_i, output.len() as u32 + 1, old_i == 0);
        cp += i / (output.len() as u32 + 1);
        i %= output.len() as u32 + 1;
        let c = char::from_u32_unchecked(cp);
        output.insert(i as usize, c);
        i += 1;
    }

    output.into_iter().collect::<String>()
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum PunyDecodeError {
    #[error("input should be ASCII Punycode")]
    NonAscii,
    #[error("invalid punycode sequence")]
    InvalidSequence,
    #[error("overflow")]
    Overflow,
    #[error("invalid code point")]
    InvalidCodePoint,
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
const fn encode_digit(d: u32) -> char {
    debug_assert!(d < 36);
    //(d + 22 + (if d < 26 { 75 } else { 0 })) as u8 as char

    match d {
        0..=25 => (d as u8 + b'a') as char,
        26..=35 => (d as u8 - 26 + b'0') as char,
        _ => {
            #[cfg(feature = "forbid-unsafe")]
            unreachable!();
            #[cfg(not(feature = "forbid-unsafe"))]
            unsafe {
                unreachable_unchecked()
            };
        }
    }
}

#[inline]
const fn decode_digit(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some((c as u8 - b'0' + 26) as u32),
        'A'..='Z' => Some((c as u8 - b'A') as u32),
        'a'..='z' => Some((c as u8 - b'a') as u32),
        _ => None,
    }
}

#[inline]
const fn adapt(mut delta: u32, num_points: u32, first_time: bool) -> u32 {
    delta /= if first_time { DAMP } else { 2 };
    delta += delta / num_points;
    let mut k = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + (((BASE - T_MIN + 1) * delta) / (delta + SKEW))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::egyptian("ليهمابتكلموشعربي؟", "egbpdaj6bu4bxfgehfvwxn")]
    #[case::chinese_simplified("他们为什么不说中文", "ihqwcrb4cv8a8dqg056pqjye")]
    #[case::chinese_traditional("他們爲什麽不說中文", "ihqwctvzc91f659drss3x8bo0yb")]
    #[case::czech("Pročprostěnemluvíčesky", "Proprostnemluvesky-uyb24dma41a")]
    #[case::hebrew("למההםפשוטלאמדבריםעברית", "4dbcagdahymbxekheh6e0a7fei0b")]
    #[case::hindi(
        "यहलोगहिन्दीक्योंनहींबोलसकतेहैं",
        "i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd"
    )]
    #[case::japanese(
        "なぜみんな日本語を話してくれないのか",
        "n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa"
    )]
    #[case::korean(
        "세계의모든사람들이한국어를이해한다면얼마나좋을까",
        "989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c"
    )]
    // NOTE: RFC specifies this D should be uppercase, but both `punycode` and `idna` return
    // lowercase, so I'll let it be here as well.                       _
    #[case::russian("почемужеонинеговорятпорусски", "b1abfaaepdrnnbgefbadotcwatmq2g4l")]
    #[case::spanish(
        "PorquénopuedensimplementehablarenEspañol",
        "PorqunopuedensimplementehablarenEspaol-fmd56a"
    )]
    #[case::vietnamese(
        "TạisaohọkhôngthểchỉnóitiếngViệt",
        "TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g"
    )]
    #[case::kinpachi("3年B組金八先生", "3B-ww4c5e180e575a65lsy2b")]
    #[case::super_monkeys(
        "安室奈美恵-with-SUPER-MONKEYS",
        "-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n"
    )]
    #[case::hello_another_way(
        "Hello-Another-Way-それぞれの場所",
        "Hello-Another-Way--fc4qua05auwb3674vfr0b"
    )]
    #[case::under_one_roof("ひとつ屋根の下2", "2-u9tlzr9756bt3uc0v")]
    #[case::takeuchi("MajiでKoiする5秒前", "MajiKoi5-783gue6qz075azm5e")]
    #[case::amiyumi("パフィーdeルンバ", "de-jg4avhby1noc0d")]
    #[case::at_light_speed("そのスピードで", "d9juau41awczczp")]
    #[case::money("-> $1.00 <-", "-> $1.00 <--")]
    fn test_encode(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(encode(input), expected);
    }

    #[rstest]
    //#[case::egyptian("egbpdaj6bu4bxfgehfvwxn", "ليهمابتكلموشعربي؟")]
    #[case::chinese_simplified("ihqwcrb4cv8a8dqg056pqjye", "他们为什么不说中文")]
    fn test_decode(#[case] input: &str, #[case] expected: &str) -> Result<(), PunyDecodeError> {
        assert_eq!(decode(input)?, expected);

        Ok(())
    }
}
