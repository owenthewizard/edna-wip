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
            let mut new = String::new();
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
                let mut new = String::new();
                new.push_str(&s[..i]);
                return map_internal(new, &s[i + n..]).map(Cow::Owned);
            }

            Mapping::Mapped(r) => {
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
    #[case::ascii("www.example.com", Ok(Cow::Borrowed("www.example.com")))]
    #[case::digits("123.test", Ok(Cow::Borrowed("123.test")))]
    #[case::munchen("münchen.de", Ok(Cow::Owned("xn--mnchen-3ya.de".into())))]
    #[case::uber("www.über.com", Ok(Cow::Owned("www.xn--ber-goa.com".into())))]
    #[case::chinese_simplified("例子.测试", Ok(Cow::Owned("xn--fsqu00a.xn--0zwm56d".into())))]
    #[case::greek("παράδειγμα.δοκιμή", Ok(Cow::Owned("xn--hxajbheg2az3al.xn--jxalpdlp".into())))]
    #[case::emoji_middle("exam💩ple.com", Ok(Cow::Owned("xn--example-wg05f.com".into())))]
    #[case::emoji("🦀.☕", Ok(Cow::Owned("xn--zs9h.xn--53h".into())))]
    #[case::empty("", Ok(Cow::Borrowed("")))]
    #[case::valid("www.¡Hola!.com.mx", Ok(Cow::Owned("www.xn--hola!-xfa.com.mx".into())))]
    #[case::mapped("www.Ç.com", Ok(Cow::Owned("www.xn--7ca.com".into())))]
    #[case::invalid("\u{10FFF}", Err(ToAsciiError::InvalidCharacter))]
    #[case::ignored("www.exam\u{00ad}ple.com", Ok(Cow::Owned("www.example.com".into())))]
    #[case::deviation_eszett("faß.de", Ok(Cow::Owned("xn--fa-hia.de".into())))]
    #[case::deviation_sigma("βόλος.com", Ok(Cow::Owned("xn--nxasmm1c.com".into())))]
    #[case::deviation_zwj("ශ්‍රී.com", Ok(Cow::Owned("xn--10cl1a0b660p.com".into())))]
    #[case::deviation_zwnj("نامه‌ای.com", Ok(Cow::Owned("xn--mgba3gch31f060k.com".into())))]
    fn test_to_ascii(#[case] input: &str, #[case] expected: Result<Cow<str>, ToAsciiError>) {
        assert_eq!(to_ascii(input), expected);
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
