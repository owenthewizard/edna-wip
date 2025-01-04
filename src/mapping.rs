#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(
    dead_code,
    reason = "Deviation is only required for transitional processing, but may be useful in the future."
)]
pub enum Mapping<'a> {
    Valid,
    Ignored,
    Mapped(&'a str),
    Disallowed,
    Deviation,
}

#[cfg(ugly_hack)]
impl Mapping<'_> {
    #[must_use]
    pub fn of(c: char) -> Option<Self> {
        use crate::data;
        use core::cmp::Ordering;

        data::MAPPING
            .binary_search_by(|(range, _)| {
                if range.contains(&c) {
                    Ordering::Equal
                } else if c < *range.start() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .ok()
            .and_then(|i| data::MAPPING.get(i))
            .map(|x| x.1)
    }
}

#[cfg(all(ugly_hack, test))]
mod test {
    use super::*;

    /// Asserts that all ASCII `char`s are excluded from the map.
    #[test]
    fn of_ascii() {
        for c in '\u{0}'..='\u{7f}' {
            assert!(Mapping::of(c).is_none());
        }
    }

    /// Asserts that all non-ASCII `char`s are mapped.
    #[test]
    fn of_unicode() {
        for c in '\u{80}'..'\u{10ffff}' {
            let _ = Mapping::of(c).unwrap();
        }
    }
}
