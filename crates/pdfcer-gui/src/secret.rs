//! # `secret` — a string the operator typed that must never reach a log
//!
//! One type, [`Secret`], and its whole reason for existing is its [`Debug`]
//! implementation.
//!
//! ## ★★★ The hazard, stated before the type
//!
//! A document password travels from a text field, through
//! [`crate::app::actions::Action`], into the action queue, and out again in
//! [`crate::app::lifecycle`]. Every step of that is ordinary — and
//! `Action` derives `Debug`, this crate traces liberally to stderr under
//! `PDFCER_DIAG`, and **`tools/ui-verify` captures that stderr to a file it
//! keeps as evidence**.
//!
//! So a single `format!("{action:?}")` anywhere on that path writes the
//! operator's password into `target/ui-verify/*.trace.txt`, in plain text, on
//! disk, in a directory whose whole purpose is to be kept and read. It would
//! not fail anything. It would not look wrong in review — a `{:?}` on an action
//! is exactly what a diagnostic line is made of.
//!
//! ⇒ **The fix is not a rule saying "do not print it".** This project has spent
//! several corrections learning that a rule written beside the code it governs
//! is not a mechanism: the rotation-button gate had its rule in its own module
//! header sixty lines above the code that broke it. The fix is a type whose
//! `Debug` cannot print the thing.
//!
//! ## What it deliberately does NOT do
//!
//! **It does not zero its buffer on drop.** A `String`'s allocation can be
//! moved by the allocator and copied by `realloc` before any `Drop` runs, so
//! zeroing the final buffer is a gesture rather than a guarantee, and shipping
//! a gesture named `Zeroizing` would tell a reader they had a property they do
//! not. If in-memory scrubbing is ever wanted it needs a fixed buffer that is
//! never reallocated, and that is a different design with its own argument.
//!
//! What this type guarantees is exactly one thing, and it is the thing that was
//! actually going to go wrong: **the value cannot be formatted.**
//!
//! ## Why `PartialEq` is here
//!
//! [`crate::app::actions::Action`] derives it, and every variant must. The
//! comparison is the ordinary string one — deliberately **not** constant-time,
//! because nothing here compares a secret against a stored secret. The only
//! comparison that matters is `pdfcer-core`'s, inside the document's own
//! authentication.

/// A string the operator typed that must not be logged.
///
/// See the module header. The `Debug` implementation reports the **length** and
/// nothing else, which is enough for a diagnostic to say *"a password of 11
/// characters was supplied"* — the fact a reader of a trace actually needs — and
/// carries none of the value.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wrap a typed string.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The bytes, for the one caller entitled to them.
    ///
    /// ★ Named `expose` rather than `as_bytes` or `get` on purpose: a call site
    /// reading `password.expose()` says at the point of use that a boundary is
    /// being crossed, and a reviewer scanning for "where does the password
    /// actually go" has one word to grep for. There are two legitimate callers
    /// — `Document::load_with_password` and `from_bytes_with_password` — and a
    /// third would want explaining.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Whether anything was typed.
    ///
    /// ★ An **empty** password is not the same as no password at all, and the
    /// engine's own doc says so: `load_with_password(path, None)` means *"try
    /// the empty user password, then give up"*, which every conforming reader
    /// does silently before prompting. Supplying `Some(b"")` is a different
    /// request. This shell only ever reaches the prompt after the `None` attempt
    /// has already failed, so an empty box means the operator pressed Open with
    /// nothing typed — which the dialog refuses rather than sending on, because
    /// re-asking the engine the question it has already answered would look
    /// like the password was rejected.
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
    ///
    /// ★★ Not idle curiosity: `pdfcer-core` reports
    /// `DocError::PasswordRequiresNormalisation` for a `/R` 5 document when the
    /// supplied password is non-ASCII, because the spec's own step 1 applies
    /// **SASLprep** (RFC 4013) before hashing and the engine does not implement
    /// it. Its doc comment is explicit that this variant exists *"so that
    /// failure does not masquerade as `PasswordRequired`'s 'you typed it
    /// wrong', which would send the operator to re-check a password that was
    /// correct."*
    ///
    /// The dialog therefore has a different sentence for that case, and this is
    /// how it can also say the useful half — *which* characters are the problem
    /// — without the engine having to tell it.
    #[must_use]
    pub fn has_non_ascii(&self) -> bool {
        !self.0.is_ascii()
    }
}

/// ★★★ **The whole point of the type.** Reports the length and nothing else.
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

    /// ★★★ **The assertion this type exists for**, and it is written as a test
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

    /// ★★ **…and nesting it inside another `Debug` type does not defeat it**,
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
