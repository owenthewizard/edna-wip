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
    pub fn of(c: char) -> Self {
        use crate::{data, unwrap};
        use core::cmp::Ordering;

        // ASCII fast path
        if c.is_ascii() {
            return Mapping::Valid;
        }

        // SAFETY: All non-ASCII char ranges are contained in MAPPING.
        // Verified by `cargo test --test mapping`.
        unwrap!(data::MAPPING
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
            .and_then(|i| data::MAPPING.get(i)))
        .1
    }
}
