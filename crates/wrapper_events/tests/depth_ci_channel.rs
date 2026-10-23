//! Synthetic V1 source-contract proof; never a claim about a production CLI release.
use wrapper_events::ValidatedChannelString;

#[test]
fn accepts_complete_ascii_grammar() {
    for value in ["a", "Z", "0", "aZ09._/-", "9foo/bar-baz_qux.ext"] {
        assert!(ValidatedChannelString::new(value).is_some(), "{value}");
    }
}
#[test]
fn accepts_exact_maximum() {
    assert_eq!(ValidatedChannelString::MAX_BYTES, 64);
    let value = "a".repeat(64);
    assert_eq!(ValidatedChannelString::new(&value).unwrap().as_str(), value);
}
#[test]
fn rejects_empty() {
    assert!(ValidatedChannelString::new("").is_none());
}
#[test]
fn rejects_over_limit() {
    for size in [65, 128, 1024] {
        assert!(ValidatedChannelString::new(&"a".repeat(size)).is_none());
    }
}
#[test]
fn rejects_non_ascii() {
    for value in ["é", "aé", "a\u{0080}", "a🦀"] {
        assert!(ValidatedChannelString::new(value).is_none(), "{value}");
    }
}
#[test]
fn rejects_invalid_prefix() {
    for value in [".", "_", "/", "-", ".a", "_a", "/a", "-a", " a"] {
        assert!(ValidatedChannelString::new(value).is_none(), "{value}");
    }
}
#[test]
fn rejects_invalid_characters() {
    for byte in 0..=127u8 {
        let ch = char::from(byte);
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '/' | '-')) {
            let value = format!("a{ch}z");
            assert!(ValidatedChannelString::new(&value).is_none(), "byte={byte}");
        }
    }
}
#[test]
fn preserves_case_and_path_separators() {
    for value in ["A/b.C_d-0", "a//B", "Z.../___"] {
        assert_eq!(ValidatedChannelString::new(value).unwrap().as_str(), value);
    }
}
