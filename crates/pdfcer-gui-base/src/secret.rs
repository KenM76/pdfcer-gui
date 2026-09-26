//! # `secret` — a string the operator typed that must never reach a log
//!
//! One type, [`Secret`], and its whole reason for existing is its [`Debug`]
//! implementation.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/secret.md`.

/// A string the operator typed that must not be logged.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wrap a typed string.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The bytes, for the one caller entitled to them.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// **The value, as a string slice.**
    #[must_use]
    pub fn expose_str(&self) -> &str {
        &self.0
    }

    /// Whether anything was typed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many characters were typed. For a diagnostic, never for a decision.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.chars().count()
    }

    /// Whether the value contains anything outside ASCII.
    #[must_use]
    pub fn has_non_ascii(&self) -> bool {
        !self.0.is_ascii()
    }
}

/// **The whole point of the type.** Reports the length and nothing else.
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ui-text-exempt: a Debug rendering, never displayed in the UI.
        write!(f, "Secret(<redacted, {} chars>)", self.len())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "a test that cannot unwrap has failed")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;

    /// **The assertion this type exists for**, and it is written as a test
    /// rather than as a comment because a comment cannot fail.
    ///
    /// Formatting a `Secret` must not produce the value, at any width, through
    /// any of the three formatting paths a diagnostic might take.
    #[test]
    fn formatting_a_secret_never_yields_the_secret() {
        let s = Secret::new("hunter2".to_owned());
        for rendered in [format!("{s:?}"), format!("{s:#?}"), format!("{:>40?}", s)] {
            assert!(
                !rendered.contains("hunter2"),
                "a Secret rendered as `{rendered}`, which carries the value — every \
                 `{{:?}}` on an Action containing one writes it to the trace file that \
                 `tools/ui-verify` keeps as evidence"
            );
        }
    }

    /// **…and nesting it inside another `Debug` type does not defeat it**,
    /// which is the shape it will actually be formatted in: a `Secret` reaches a
    /// trace as a field of an `Action`, never on its own.
    #[test]
    fn a_secret_inside_a_derived_debug_is_still_redacted() {
        #[derive(Debug)]
        #[allow(dead_code, reason = "the fields exist to be formatted")] // ui-text-exempt: clippy lint justification, never displayed
        struct Carrier {
            path: &'static str,
            password: Secret,
        }
        let c = Carrier {
            path: "drawing.pdf",
            password: Secret::new("correct horse".to_owned()),
        };
        let rendered = format!("{c:?}");
        assert!(!rendered.contains("correct horse"), "{rendered}");
        assert!(rendered.contains("redacted"), "{rendered}");
        assert!(
            rendered.contains("drawing.pdf"),
            "the rest of the value must still be readable, or the redaction has \
             cost the diagnostic its usefulness: {rendered}"
        );
    }

    /// The length is reported, because *"a password of N characters was
    /// supplied"* is the fact a trace reader needs and it carries no value.
    #[test]
    fn the_length_is_reported_and_counts_characters_not_bytes() {
        assert_eq!(Secret::new("abc".to_owned()).len(), 3);
        // Four characters, more than four bytes. A byte count would be a
        // (small) leak of the value's shape and would also be wrong.
        assert_eq!(Secret::new("héllo".to_owned()).len(), 5);
    }

    /// The non-ASCII probe, which is what separates "you typed it wrong" from
    /// "pdfcer cannot normalise this password" — see [`Secret::has_non_ascii`].
    #[test]
    fn non_ascii_is_detected() {
        assert!(!Secret::new("plain".to_owned()).has_non_ascii());
        assert!(Secret::new("pläin".to_owned()).has_non_ascii());
    }

    /// An empty password is distinguishable, because it is a different request
    /// from *no* password — see [`Secret::is_empty`].
    #[test]
    fn empty_is_not_the_same_as_absent() {
        assert!(Secret::new(String::new()).is_empty());
        assert!(!Secret::new(" ".to_owned()).is_empty());
    }
}
