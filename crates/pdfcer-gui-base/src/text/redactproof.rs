//! # `text::redactproof` — what stopped the absence proof saying *verified*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/redactproof.md`.

/// **Some streams could not be decoded, so the proof could not look inside
/// them** — `AbsenceVerification::streams_undecoded`.
#[must_use]
pub fn proof_unsearched_line(streams: usize) -> String {
    format!(
        "{streams} stream(s) in the redacted file could not be decoded, so pdfcer could not search inside them for the removed text. Their stored bytes were still searched. The removal is reported above, but pdfcer cannot call it verified."
    )
}

/// **The redacted file did not read back as a PDF** —
/// `AbsenceVerification::output_unparsed`.
#[must_use]
pub fn proof_unparsed_line() -> &'static str {
    "The redacted file could not be read back as a PDF, so pdfcer searched only its raw bytes for the removed text and cannot call the removal verified."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unsearched_count_is_stated() {
        assert!(proof_unsearched_line(3).starts_with("3 stream(s)"));
    }

    #[test]
    fn neither_line_claims_verification() {
        for line in [proof_unsearched_line(1).as_str(), proof_unparsed_line()] {
            assert!(line.contains("cannot call"), "{line}");
        }
    }
}
