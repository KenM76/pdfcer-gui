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

/// **Not every page's text could be read back** —
/// `AbsenceVerification::text_unextracted`.
#[must_use]
pub const fn proof_unextracted_line() -> &'static str {
    "pdfcer could not read back the text of every page of the redacted file, so its search of the page text is incomplete and it cannot call the removal verified."
}

/// **The removed text can still be read out of a page** —
/// `ResidualSite::ExtractedText`: no stream holds its bytes as one run, but
/// the page's text, decoded through its fonts, spells it.
#[must_use]
pub fn extracted_residual_line(text: &str) -> String {
    format!(
        "⚠  The removed text “{text}” can still be read out of this document's page text — it is drawn in pieces or through a font's own encoding, so no stream holds it as one run of bytes, but copying or searching the page would find it. Most likely it is another occurrence of the same words outside the area you marked; pdfcer cannot tell that apart from removed text left behind. If it is text you did not mean to remove, you can continue; if you meant to remove every occurrence, cancel and mark those too."
    )
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
        for line in [
            proof_unsearched_line(1).as_str(),
            proof_unparsed_line(),
            proof_unextracted_line(),
        ] {
            assert!(line.contains("cannot call"), "{line}");
        }
    }

    #[test]
    fn an_extracted_text_residual_never_says_it_is_not_drawn() {
        let line = crate::text::redact::raw_residual_line(
            "Confidential",
            crate::redact::ResidualSite::ExtractedText,
        );
        assert!(line.contains("page text"), "{line}");
        assert!(!line.contains("no longer appears"), "{line}");
    }
}
