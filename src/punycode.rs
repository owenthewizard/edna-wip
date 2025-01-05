use core::cmp::Ordering;

extern crate alloc;
use alloc::{string::String, vec::Vec};

use thiserror::Error;

use crate::{_assert, _unreachable, unwrap};

// You MUST adjust encode_digit() if any of the constants are modified!
const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;
const DELIMITER: char = '-';

type Utf32 = Vec<char>;

/// Encodes Unicode as Punycode.
///
/// Overflow may occur if `input` is longer than 63 characters.
/// Overflow may result in invalid output, but will never result in Undefined Behavior.
///
/// # Examples
///
/// ```
/// # use edna::punycode;
/// assert_eq!(punycode::encode("München"), "Mnchen-3ya");
/// ```
#[must_use]
#[expect(clippy::cast_possible_truncation)]
pub fn encode(input: &str) -> String {
    // UTF-32 is 4x as many bytes as UTF-8 in the worst case, but may be less
    let mut new_input = Utf32::with_capacity(input.len() * 4);
    // ASCII-Unicode split is unknown, same worst case as above
    let mut output = String::with_capacity(input.len() * 4);
    let mut non_ascii = Utf32::with_capacity(input.len() * 4);

    for c in input.chars() {
        new_input.push(c);
        if c.is_ascii() {
            output.push(c);
        } else {
            non_ascii.push(c);
        }
    }
    let input = new_input;

    non_ascii.sort_unstable();
    non_ascii.dedup();
    let mut non_ascii = non_ascii.into_iter();

    let basic_len = output.len() as u32;
    if basic_len > 0 {
        output.push(DELIMITER);
    }

    let mut cp = INITIAL_N;
    let mut delta = 0;
    let mut bias = INITIAL_BIAS;
    let mut processed = basic_len;

    while processed < input.len() as u32 {
        // SAFETY: input always contains a code point >= cp while processed < input.len()
        let min_cp = unwrap!(non_ascii.next()) as u32;
        delta += (min_cp - cp) * (processed + 1);
        cp = min_cp;
        for &c in &input {
            let c = c as u32;
            match c.cmp(&cp) {
                Ordering::Less => delta += 1,
                Ordering::Equal => {
                    let mut q = delta;
                    for k in (BASE..).step_by(BASE as usize) {
                        let t = clamped_sub(k, bias);
                        // mutants test for clamped_sub
                        // SAFETY: clamped to T_MIN ..= T_MAX
                        _assert!((T_MIN..=T_MAX).contains(&t));

                        if q < t {
                            break;
                        }

                        let value = t + ((q - t) % (BASE - t));
                        output.push(encode_digit(value));
                        q = (q - t) / (BASE - t);
                    }
                    output.push(encode_digit(q));
                    bias = adapt(delta, processed + 1, processed == basic_len);
                    delta = 0;
                    processed += 1;
                }
                Ordering::Greater => (),
            }
        }
        delta += 1;
        cp += 1;
    }

    output
}

/// Decodes Punycode to Unicode.
///
/// # Errors
///
/// - Overflow has occured.
/// - `input` is not ASCII.
/// - An invalid Punycode sequence was encountered.
///
/// # Examples
///
/// ```
/// # use edna::punycode;
/// assert_eq!(punycode::decode("Mnchen-3ya"), Ok("München".to_string()));
/// ```
#[expect(clippy::cast_possible_truncation)]
pub fn decode(input: &str) -> Result<String, PunyDecodeError> {
    let (mut output, mut encoded) = input.rsplit_once('-').map_or_else(
        || (Utf32::new(), input.chars()),
        |(l, r)| (l.chars().collect::<Utf32>(), r.chars()),
    );

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

            let t = clamped_sub(k, bias);
            // mutants test for clamped_sub
            // SAFETY: clamped to T_MIN ..= T_MAX
            _assert!((T_MIN..=T_MAX).contains(&t));

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
        // SAFETY: AFAIK this can never fail, but if that's incorrect please open a bug.
        let c = unwrap!(char::from_u32(cp));
        output.insert(i as usize, c);
        i += 1;
    }

    Ok(output.into_iter().collect::<String>())
}

/// Decodes Punycode to Unicode without input validation.
///
/// # Safety
///
/// `input` must never overflow. That is to say, no code points should exceed
/// `(M - INITIAL_N) * (L + 1)`. This can never happen with labels <= 63 characters and code points
/// not not exceeding U+10FFFF. Therefore, any valid IDN label will never overflow. For more
/// information, see RFC 3492.
///
/// - `input` must be valid Punycode (implies ASCII only).
/// - `input` must not contain an invalid Punycode sequence.
///
/// Failure to uphold these invariants may result in **Undefined Behavior**.
///
/// # Examples
///
/// ```
/// # use edna::punycode;
/// # unsafe {
/// assert_eq!(punycode::decode_unchecked("Mnchen-3ya"), "München");
/// # }
/// ```
#[cfg(not(feature = "forbid-unsafe"))]
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub unsafe fn decode_unchecked(input: &str) -> String {
    let (mut output, mut encoded) = input.rsplit_once('-').map_or_else(
        || (Utf32::new(), input.chars()),
        |(l, r)| (l.chars().collect::<Utf32>(), r.chars()),
    );

    let mut cp = INITIAL_N;
    let mut i: u32 = 0;
    let mut bias = INITIAL_BIAS;

    while let Some(mut byte) = encoded.next() {
        let old_i = i;
        let mut weight = 1;

        for k in (BASE..).step_by(BASE as usize) {
            let digit = decode_digit(byte).unwrap_unchecked();

            i += digit * weight;

            let t = clamped_sub(k, bias);

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
    #[error("invalid punycode sequence")]
    InvalidSequence,
    #[error("input should be ASCII Punycode")]
    NonAscii,
    #[error("overflow")]
    Overflow,
}

#[must_use]
const fn adapt(mut delta: u32, num_points: u32, first_time: bool) -> u32 {
    // SAFETY: num_points (processed + 1) is always > 0, even on an empty string.
    _assert!(num_points > 0);

    delta /= if first_time { DAMP } else { 2 };
    delta += delta / num_points;
    let mut k = 0;
    while delta > (BASE - T_MIN) * T_MAX / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + (BASE - T_MIN + 1) * delta / (delta + SKEW)
}

#[must_use]
const fn clamped_sub(k: u32, bias: u32) -> u32 {
    if k <= bias {
        T_MIN
    } else if k >= bias + T_MAX {
        T_MAX
    } else {
        k - bias
    }
}

#[must_use]
const fn decode_digit(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some((c as u8 - b'0' + 26) as u32),
        'A'..='Z' => Some((c as u8 - b'A') as u32),
        'a'..='z' => Some((c as u8 - b'a') as u32),
        _ => None,
    }
}

#[must_use]
#[expect(clippy::cast_possible_truncation)]
const fn encode_digit(d: u32) -> char {
    // You MUST adjust this function if any of the constants are modified!

    const _: () = assert!(
        T_MIN == 1,
        "encode_digit() should be adjusted when constants are modified"
    );
    const MAX: u32 = BASE - T_MIN;

    match d {
        0..T_MAX => (d as u8 + b'a') as char,
        T_MAX..=MAX => (d as u8 - 26 + b'0') as char,
        _ => {
            // SAFETY: d is % BASE in encode()
            // Make sure to adjust this function if you change any of the constants!
            _unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::undocumented_unsafe_blocks)]
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
    // lowercase, so I'll let it be here as well.
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
    #[case::empty("", "")]
    #[case::emoji("🦀", "zs9h")]
    fn test_encode(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(encode(input), expected);
    }

    #[rstest]
    #[case::egyptian("egbpdaj6bu4bxfgehfvwxn", "ليهمابتكلموشعربي؟")]
    #[case::chinese_simplified("ihqwcrb4cv8a8dqg056pqjye", "他们为什么不说中文")]
    #[case::chinese_traditional("ihqwctvzc91f659drss3x8bo0yb", "他們爲什麽不說中文")]
    #[case::czech("Proprostnemluvesky-uyb24dma41a", "Pročprostěnemluvíčesky")]
    #[case::hebrew("4dbcagdahymbxekheh6e0a7fei0b", "למההםפשוטלאמדבריםעברית")]
    #[case::hindi(
        "i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd",
        "यहलोगहिन्दीक्योंनहींबोलसकतेहैं"
    )]
    #[case::japanese(
        "n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa",
        "なぜみんな日本語を話してくれないのか"
    )]
    #[case::korean(
        "989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c",
        "세계의모든사람들이한국어를이해한다면얼마나좋을까"
    )]
    // NOTE: RFC specifies this D should be uppercase, but both `punycode` and `idna` return
    // lowercase, so we'll let it be here as well.                       _
    #[case::russian("b1abfaaepdrnnbgefbadotcwatmq2g4l", "почемужеонинеговорятпорусски")]
    #[case::spanish(
        "PorqunopuedensimplementehablarenEspaol-fmd56a",
        "PorquénopuedensimplementehablarenEspañol"
    )]
    #[case::vietnamese(
        "TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g",
        "TạisaohọkhôngthểchỉnóitiếngViệt"
    )]
    #[case::kinpachi("3B-ww4c5e180e575a65lsy2b", "3年B組金八先生")]
    #[case::super_monkeys(
        "-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n",
        "安室奈美恵-with-SUPER-MONKEYS"
    )]
    #[case::hello_another_way(
        "Hello-Another-Way--fc4qua05auwb3674vfr0b",
        "Hello-Another-Way-それぞれの場所"
    )]
    #[case::under_one_roof("2-u9tlzr9756bt3uc0v", "ひとつ屋根の下2")]
    #[case::takeuchi("MajiKoi5-783gue6qz075azm5e", "MajiでKoiする5秒前")]
    #[case::amiyumi("de-jg4avhby1noc0d", "パフィーdeルンバ")]
    #[case::at_light_speed("d9juau41awczczp", "そのスピードで")]
    #[case::money("-> $1.00 <--", "-> $1.00 <-")]
    #[case::empty("", "")]
    #[case::emoji("zs9h", "🦀")]
    fn test_decode(#[case] input: &str, #[case] expected: &str) -> Result<(), PunyDecodeError> {
        assert_eq!(decode(input)?, expected);
        #[cfg(not(feature = "forbid-unsafe"))]
        unsafe {
            assert_eq!(decode_unchecked(input), expected);
        };

        Ok(())
    }
}
