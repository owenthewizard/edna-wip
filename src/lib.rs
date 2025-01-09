#![no_std]
#![cfg_attr(feature = "forbid-unsafe", forbid(unsafe_code))]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]

#[cfg(not(feature = "forbid-unsafe"))]
use core::hint::unreachable_unchecked;

extern crate alloc;
use alloc::{borrow::Cow, string::String};

use thiserror::Error;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

pub(crate) mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

mod mapping;
pub use mapping::Mapping;

pub mod punycode;

/// The prefix used before a punycode label.
pub const PREFIX: &str = "xn--";

const PUNYCODE_PREFIX: u32 =
    ((b'-' as u32) << 24) | ((b'-' as u32) << 16) | ((b'N' as u32) << 8) | b'X' as u32;
const PUNYCODE_PREFIX_MASK: u32 = (0xFF << 24) | (0xFF << 16) | (0xDF << 8) | 0xDF;

/// `has_punycode_prefix` from idna
/// Copyright (c) 2013-2022 The rust-url developers
const fn has_punycode_prefix(slice: &[u8]) -> bool {
    if slice.len() < 4 {
        return false;
    }
    // Sadly, the optimizer doesn't figure out that more idiomatic code
    // should compile to masking on 32-bit value.
    let a = slice[0] as u32;
    let b = slice[1] as u32;
    let c = slice[2] as u32;
    let d = slice[3] as u32;
    let u = d << 24 | c << 16 | b << 8 | a;
    (u & PUNYCODE_PREFIX_MASK) == PUNYCODE_PREFIX
}

macro_rules! unwrap {
    ($opt:expr) => {{
        #[cfg(feature = "forbid-unsafe")]
        {
            $opt.unwrap()
        }
        #[cfg(not(feature = "forbid-unsafe"))]
        #[allow(unused_unsafe, reason = "may be encased in an existing unsafe block")]
        {
            // SAFETY: Caller must verify this is safe.
            unsafe { $opt.unwrap_unchecked() }
        }
    }};
}
pub(crate) use unwrap;

macro_rules! _assert {
    ($e:expr) => {{
        #[cfg(feature = "forbid-unsafe")]
        {
            assert!($e)
        }
        #[cfg(not(feature = "forbid-unsafe"))]
        #[allow(unused_unsafe, reason = "may be encased in an existing unsafe block")]
        {
            use core::hint::assert_unchecked;
            // SAFETY: Caller must verify this is safe.
            unsafe { assert_unchecked($e) }
        }
    }};
}
pub(crate) use _assert;

macro_rules! _unreachable {
    () => {{
        #[cfg(feature = "forbid-unsafe")]
        {
            unreachable!()
        }
        #[cfg(not(feature = "forbid-unsafe"))]
        #[allow(unused_unsafe, reason = "may be encased in an existing unsafe block")]
        {
            use core::hint::unreachable_unchecked;
            // SAFETY: Caller must verify this is safe.
            unsafe { unreachable_unchecked() }
        }
    }};

    ($e:expr) => {{
        #[cfg(feature = "forbid-unsafe")]
        {
            unreachable!($e)
        }
        #[cfg(not(feature = "forbid-unsafe"))]
        #[allow(unused_unsafe, reason = "may be encased in an existing unsafe block")]
        {
            use core::hint::unreachable_unchecked;
            // SAFETY: Caller must verify this is safe.
            unsafe { unreachable_unchecked() }
        }
    }};
}
pub(crate) use _unreachable;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ToAsciiError {
    #[error("invalid character")]
    InvalidCharacter,
    #[error("punycode encoding error: {0}")]
    Encode(punycode::PunyEncodeError),
}

impl From<punycode::PunyEncodeError> for ToAsciiError {
    fn from(e: punycode::PunyEncodeError) -> Self {
        Self::Encode(e)
    }
}

fn map_internal(mut new: String, old: &str) -> Result<String, ToAsciiError> {
    for c in old.chars() {
        // ASCII fast path
        if c.is_ascii() {
            new.push(c.to_ascii_lowercase());
            continue;
        };

        // SAFETY: All ASCII code points taken care of above.
        match unwrap!(Mapping::of(c)) {
            Mapping::Valid => new.push(c),
            Mapping::Ignored => continue,
            Mapping::Mapped(s) => new.push_str(s),
            Mapping::Disallowed => return Err(ToAsciiError::InvalidCharacter),
            Mapping::Deviation => {
                #[cfg(not(feature = "forbid-unsafe"))]
                // SAFETY: Deviations are filtered out in build.rs
                unsafe {
                    unreachable_unchecked()
                };
                #[cfg(feature = "forbid-unsafe")]
                unimplemented!("Transitional processing is not implemented.");
            }
        }
    }

    Ok(new)
}

fn map_validate(s: &str) -> Result<Cow<str>, ToAsciiError> {
    // indexing doesn't seem to harm us here - maybe the bounds check gets optimized out

    // ASCII fast path
    if s.is_ascii() {
        return Ok(Cow::Owned(s.to_ascii_lowercase()));
    }

    for (i, c) in s.char_indices() {
        // ASCII fast paths
        if c.is_ascii_uppercase() {
            let mut new = String::with_capacity(s.len() * 4);
            new.push_str(&s[..i]);
            new.push(c.to_ascii_lowercase());
            return map_internal(new, &s[i + 1..]).map(Cow::Owned);
        }
        if c.is_ascii() {
            continue;
        }

        // SAFETY: All ASCII code points taken care of above.
        match unwrap!(Mapping::of(c)) {
            Mapping::Valid => continue,

            Mapping::Ignored => {
                let n = c.len_utf8();
                let mut new = String::with_capacity(s.len() * 4);
                new.push_str(&s[..i]);
                return map_internal(new, &s[i + n..]).map(Cow::Owned);
            }

            Mapping::Mapped(r) => {
                let n = c.len_utf8();
                let mut new = String::with_capacity(s.len() * 4);
                new.push_str(&s[..i]);
                new.push_str(r);
                return map_internal(new, &s[i + n..]).map(Cow::Owned);
            }

            Mapping::Disallowed => return Err(ToAsciiError::InvalidCharacter),

            Mapping::Deviation => {
                #[cfg(not(feature = "forbid-unsafe"))]
                // SAFETY: Deviations are filtered out in build.rs
                unsafe {
                    unreachable_unchecked()
                };
                #[cfg(feature = "forbid-unsafe")]
                unimplemented!("Transitional processing is not implemented.");
            }
        }
    }

    Ok(Cow::Borrowed(s))
}

pub fn validate(s: &str) -> Result<(), ToAsciiError> {
    // ASCII fast_paths
    if s.chars().any(|x| x.is_ascii_uppercase()) {
        return Err(ToAsciiError::InvalidCharacter);
    }
    if s.is_ascii() {
        return Ok(());
    }

    // All ASCII is valid except uppercase, which fails above.
    for c in s.chars().filter(|x| !x.is_ascii()) {
        if Mapping::of(c) == Some(Mapping::Valid) {
            continue;
        }
        return Err(ToAsciiError::InvalidCharacter);
    }

    Ok(())
}

pub fn to_ascii(s: &str) -> Result<String, ToAsciiError> {
    let mut s = map_validate(s)?;

    if is_nfc_quick(s.chars()) != IsNormalized::Yes {
        s = Cow::Owned(s.nfc().collect());
    }

    let mut ret = String::with_capacity(s.len() * 4);
    for (label, dot) in s
        .split_inclusive('.')
        .map(|x| x.split_once('.').map_or((x, None), |(a, b)| (a, Some(b))))
    {
        if label.is_ascii() {
            // TODO maybe check punycode
            ret.push_str(label);
        } else {
            if has_punycode_prefix(label.as_bytes()) {
                return Err(ToAsciiError::InvalidCharacter);
            }
            ret.push_str(PREFIX);
            ret.push_str(&punycode::encode(label)?);
        }

        if dot.is_some() {
            ret.push('.');
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ascii("www.example.com", Ok("www.example.com"))]
    #[case::digits("123.test", Ok("123.test"))]
    #[case::munchen("münchen.de", Ok("xn--mnchen-3ya.de"))]
    #[case::uber("www.über.com", Ok("www.xn--ber-goa.com"))]
    #[case::chinese_simplified("例子.测试", Ok("xn--fsqu00a.xn--0zwm56d"))]
    #[case::greek("παράδειγμα.δοκιμή", Ok("xn--hxajbheg2az3al.xn--jxalpdlp"))]
    #[case::emoji_middle("exam💩ple.com", Ok("xn--example-wg05f.com"))]
    #[case::emoji("🦀.☕", Ok("xn--zs9h.xn--53h"))]
    #[case::empty("", Ok(""))]
    #[case::valid("www.¡Hola!.com.mx", Ok("www.xn--hola!-xfa.com.mx"))]
    #[case::mapped("www.Ç.com", Ok("www.xn--7ca.com"))]
    #[case::invalid("\u{10FFF}", Err(ToAsciiError::InvalidCharacter))]
    #[case::ignored("www.exam\u{00ad}ple.com", Ok("www.example.com"))]
    #[case::deviation_eszett("faß.de", Ok("xn--fa-hia.de"))]
    #[case::deviation_sigma("βόλος.com", Ok("xn--nxasmm1c.com"))]
    #[case::deviation_zwj("ශ්‍රී.com", Ok("xn--10cl1a0b660p.com"))]
    #[case::deviation_zwnj("نامه‌ای.com", Ok("xn--mgba3gch31f060k.com"))]
    fn test_to_ascii(#[case] input: &str, #[case] expected: Result<&str, ToAsciiError>) {
        assert_eq!(to_ascii(input).as_deref(), expected.as_deref());
    }

    #[rstest]
    #[case::munchen("münchen.de", Ok(()))]
    #[case::uber("www.über.com", Ok(()))]
    #[case::chinese_simplified("例子.测试", Ok(()))]
    #[case::greek("παράδειγμα.δοκιμή", Ok(()))]
    #[case::emoji_middle("exam💩ple.com", Ok(()))]
    #[case::emoji("🦀.☕", Ok(()))]
    #[case::empty("", Ok(()))]
    #[case::valid("www.¡hola!.com.mx", Ok(()))]
    #[case::mapped("www.Ç.com", Err(ToAsciiError::InvalidCharacter))]
    #[case::invalid("\u{10FFF}", Err(ToAsciiError::InvalidCharacter))]
    #[case::ignored("www.exam\u{00ad}ple.com", Err(ToAsciiError::InvalidCharacter))]
    #[case::deviation("faß.de", Ok(()))]
    fn test_validate(#[case] input: &str, #[case] expected: Result<(), ToAsciiError>) {
        assert_eq!(validate(input), expected);
    }
}
