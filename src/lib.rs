#![no_std]
#![cfg_attr(feature = "forbid-unsafe", forbid(unsafe_code))]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]

#[cfg(not(feature = "forbid-unsafe"))]
use core::hint::unreachable_unchecked;

extern crate alloc;
use alloc::{borrow::Cow, string::String};

use thiserror::Error;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

#[cfg(feature = "rpmalloc")]
#[global_allocator]
static ALLOC: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

pub(crate) mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

mod mapping;
pub use mapping::Mapping;

pub mod punycode;

/// The prefix used before a punycode label.
pub const PREFIX: &str = "xn--";

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

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ToAsciiError {
    #[error("invalid character")]
    InvalidCharacter,
}

fn map_internal(mut new: String, old: &str) -> Result<String, ToAsciiError> {
    for c in old.chars() {
        // ASCII fast paths
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            new.push(c);
            continue;
        };
        if c.is_ascii_uppercase() {
            // SAFETY: `s` already contains `c` or else we couldn't get here.
            new.push(c.to_ascii_lowercase());
            continue;
        }

        match Mapping::of(c) {
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

    for c in s.chars() {
        // ASCII fast paths
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            continue;
        };
        if c.is_ascii_uppercase() {
            // SAFETY: `s` already contains `c` or else we couldn't get here.
            let i = unwrap!(s.find(c));
            let n = c.len_utf8();
            let mut new = String::new();
            new.push_str(&s[..i]);
            new.push(c.to_ascii_lowercase());
            return map_internal(new, &s[i + n..]).map(Cow::Owned);
        }

        match Mapping::of(c) {
            Mapping::Valid => continue,

            Mapping::Ignored => {
                // SAFETY: `s` already contains `c` or else we couldn't get here.
                let i = unwrap!(s.find(c));
                let n = c.len_utf8();
                let mut new = String::new();
                new.push_str(&s[..i]);
                return map_internal(new, &s[i + n..]).map(Cow::Owned);
            }

            Mapping::Mapped(r) => {
                // SAFETY: `s` already contains `c` or else we couldn't get here.
                let i = unwrap!(s.find(c));
                let n = c.len_utf8();
                let mut new = String::new();
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
    for c in s.chars() {
        // All ASCII code points are Valid.
        if c.is_ascii() {
            continue;
        }

        match Mapping::of(c) {
            Mapping::Valid => continue,
            _ => return Err(ToAsciiError::InvalidCharacter),
        }
    }

    Ok(())
}

pub fn to_ascii(s: &str) -> Result<Cow<str>, ToAsciiError> {
    let mut s = map_validate(s)?;

    if is_nfc_quick(s.chars()) != IsNormalized::Yes {
        s = Cow::Owned(s.nfc().collect());
    }

    if s.is_ascii() {
        return Ok(s);
    }

    let mut ret = String::new();
    for (label, dot) in s
        .split_inclusive('.')
        .map(|x| x.split_once('.').map_or((x, None), |(a, b)| (a, Some(b))))
    {
        if label.is_ascii() {
            ret.push_str(label);
        } else {
            ret.push_str(PREFIX);
            ret.push_str(&punycode::encode(label));
        }

        if dot.is_some() {
            ret.push('.');
        }
    }

    Ok(Cow::Owned(ret))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::munchen("münchen.de", Ok("xn--mnchen-3ya.de"))]
    #[case::uber("www.über.com", Ok("www.xn--ber-goa.com"))]
    #[case::chinese_simplified("例子.测试", Ok("xn--fsqu00a.xn--0zwm56d"))]
    #[case::greek("παράδειγμα.δοκιμή", Ok("xn--hxajbheg2az3al.xn--jxalpdlp"))]
    #[case::zwj("🧑‍💻.com", Ok("xn--1ugx175ptgd.com"))]
    #[case::emoji_middle("exam💩ple.com", Ok("xn--example-wg05f.com"))]
    #[case::empty("", Ok(""))]
    #[case::emoji("🦀.☕", Ok("xn--zs9h.xn--53h"))]
    #[case::invalid("\u{10FFF}", Err(ToAsciiError::InvalidCharacter))]
    fn test_to_ascii(#[case] input: &str, #[case] expected: Result<&str, ToAsciiError>) {
        assert_eq!(to_ascii(input).as_deref(), expected.as_deref());
    }
}
