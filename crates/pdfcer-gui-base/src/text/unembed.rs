//! # `text::unembed` — what the Remove-fonts window says before it takes
//! something out
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/unembed.md`.

use pdfcer_core::font_unembed::PdfaClaim;
use pdfcer_core::font_unembed::{UnembedBlocker, UnembedPlan};

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Remove embedded fonts"
}

/// The opening sentence.
#[must_use]
pub const fn intro() -> &'static str {
    "Removing an embedded font takes the outlines out of this document, so it will be drawn with \
     whatever matching font the reader's machine has. Nothing moves on the page and no text \
     changes: the letters keep their positions and their spacing, and only their shapes are the \
     viewer's rather than yours."
}

/// The document carries nothing that can be removed.
#[must_use]
pub const fn nothing_removable() -> &'static str {
    "There is no embedded font in this document that pdfcer can safely remove."
}

/// How many fonts will be removed.
#[must_use]
pub fn will_remove(count: usize) -> String {
    format!("{count} embedded font(s) will be removed:")
}

/// One font that will lose its program.
#[must_use]
pub fn remove_row(face: &str, bytes: usize, freed: bool, renamed: Option<&str>) -> String {
    let mut line = format!("{face} — {}", bytes_phrase(bytes as u64));
    if !freed {
        line.push_str(
            ", which stay in the file: another font that is not being changed uses the same \
             outlines",
        );
    }
    if let Some(new_name) = renamed {
        // The rename is the third of the four invisible consequences. A
        // §9.6.4 subset tag says *"this is part of a face"*, and once the
        // program is gone the claim is false — so the tag comes off and the
        // font's name in the file changes. Nothing on the page shows it, and a
        // tool comparing font names across two revisions will see it.
        line.push_str(&format!(", and it will be renamed to {new_name}"));
    }
    line
}

/// The heading over the fonts that will not be removed.
#[must_use]
pub fn cannot_remove(count: usize) -> String {
    format!("{count} embedded font(s) will be left alone:")
}

/// One blocked font, with the engine's reason.
#[must_use]
pub fn blocked_row(face: &str, blocker: &UnembedBlocker) -> String {
    format!("{face} — {}", blocker.reason())
}

/// The heading over names that matched nothing.
#[must_use]
pub fn unmatched(names: &[String]) -> String {
    format!(
        "These names are not fonts in this document: {}",
        names.join(", ")
    )
}

/// What the removal will and will not do to the file's size.
#[must_use]
pub fn size_note(bytes: u64) -> String {
    format!(
        "This frees {} of font data. **pdfcer's Save will not make the file smaller**, because it \
         saves by adding your changes to the end of the file and leaving the earlier version \
         intact — so the outlines are no longer used and are still there. Recovering the space \
         needs a save that rewrites the whole file, which pdfcer does not offer yet.",
        bytes_phrase(bytes)
    )
}

/// A byte count in the units an operator thinks in.
fn bytes_phrase(bytes: u64) -> String {
    let mib = bytes as f64 / (1024.0 * 1024.0);
    if mib >= 0.1 {
        format!("{mib:.1} MB")
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    }
}

/// What removal does to a PDF/A claim.
#[must_use]
pub fn pdfa_line(claim: &PdfaClaim) -> Option<String> {
    match claim {
        PdfaClaim::None => None,
        PdfaClaim::Identified { part, conformance } => {
            let named = match (part.as_deref(), conformance.as_deref()) {
                (Some(p), Some(c)) => format!("PDF/A-{p}{c}"),
                (Some(p), None) => format!("PDF/A-{p}"),
                _ => "PDF/A".to_owned(),
            };
            Some(format!(
                "This document says it is {named}, and that standard requires every font to be \
                 embedded. Removing them breaks the claim — the document will still say it is \
                 {named} and will no longer be one."
            ))
        }
        PdfaClaim::OutputIntentOnly => Some(
            "This document carries a PDF/A output intent but does not identify as PDF/A. That is \
             normal for colour management and is not a claim, so removing fonts breaks nothing \
             here."
                .to_owned(),
        ),
        _ => Some(
            "This document's metadata could not be read, so pdfcer cannot say whether it claims to \
             be PDF/A — a standard that requires every font to be embedded."
                .to_owned(),
        ),
    }
}

/// What removal does to a digital signature.
#[must_use]
pub const fn signature_line() -> &'static str {
    "This document is signed. Removing fonts changes it, so the signature will no longer cover \
     what a reader sees."
}

/// The button that performs the removal.
#[must_use]
pub const fn remove_button() -> &'static str {
    "Remove"
}

/// The button that does not.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// Why the Remove button is dead.
#[must_use]
pub const fn nothing_to_remove() -> &'static str {
    "None of these fonts can be removed. Each one below says why."
}

/// The disclosure after a removal, one sentence per fact worth stating.
#[must_use]
pub fn removed_disclosure(removed: usize, bytes: u64, renamed: bool) -> Vec<String> {
    let mut out = vec![format!(
        "Removed the embedded outlines from {removed} font(s)."
    )];
    out.push(size_note(bytes));
    if renamed {
        out.push(
            "At least one font was renamed: its name said it held part of a face, and it no \
             longer holds any of it."
                .to_owned(),
        );
    }
    out
}

/// The plan's one-line summary, for a caller with one line.
#[must_use]
pub fn plan_summary(plan: &UnembedPlan) -> String {
    format!(
        "{} font(s) can be removed, {} cannot.",
        plan.targets.len(),
        plan.blocked.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The size sentence always says pdfcer's Save will not deliver it.**
    #[test]
    fn the_size_sentence_never_promises_a_smaller_file() {
        for bytes in [1_024_u64, 500_000, 8_000_000] {
            let line = size_note(bytes);
            assert!(
                line.contains("will not make the file smaller"),
                "the trap is unstated: {line}"
            );
            assert!(
                line.contains("rewrites the whole file"),
                "the remedy is unnamed: {line}"
            );
        }
    }

    /// **A shared program is disclosed on the row.**
    ///
    /// The case an operator cannot infer: the font is unembedded, the bytes
    /// stay, and from outside that looks like the removal not working.
    #[test]
    fn a_shared_program_says_the_bytes_stay() {
        let freed = remove_row("ArialMT", 400_000, true, None);
        let shared = remove_row("ArialMT", 400_000, false, None);
        assert!(!freed.contains("stay in the file"), "{freed}");
        assert!(shared.contains("stay in the file"), "{shared}");
    }

    /// **A rename is disclosed and only when there is one.**
    #[test]
    fn a_rename_is_named() {
        let plain = remove_row("ABCDEF+ArialMT", 1000, true, None);
        let renamed = remove_row("ABCDEF+ArialMT", 1000, true, Some("ArialMT"));
        assert!(!plain.contains("renamed"), "{plain}");
        assert!(renamed.contains("renamed to ArialMT"), "{renamed}");
    }

    /// **A PDF/A claim gets the sentence that says removal BREAKS it.**
    ///
    /// The engine refuses to gate on this and says the shells must. Losing the
    /// word "breaks" would turn a gate into a note.
    #[test]
    fn a_pdfa_claim_is_told_it_will_break() {
        let line = pdfa_line(&PdfaClaim::Identified {
            part: Some("2".to_owned()),
            conformance: Some("B".to_owned()),
        })
        .expect("a claim gets a line");
        assert!(line.contains("PDF/A-2B"), "{line}");
        assert!(line.contains("breaks the claim"), "{line}");
        assert!(pdfa_line(&PdfaClaim::None).is_none());
    }

    /// **The disclosure always carries the size caveat.**
    #[test]
    fn the_disclosure_carries_the_caveat_every_time() {
        let notes = removed_disclosure(2, 900_000, false);
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert!(
            notes[1].contains("will not make the file smaller"),
            "{notes:?}"
        );
        assert_eq!(removed_disclosure(2, 900_000, true).len(), 3);
    }
}
