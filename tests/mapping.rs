use edna::Mapping;

/// Assert that all `char`s are mapped.
#[test]
fn main() {
    for c in '\u{0}'..'\u{10ffff}' {
        let _ = Mapping::of(c);
    }
}
