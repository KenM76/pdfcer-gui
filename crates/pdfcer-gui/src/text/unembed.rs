//! # `text::unembed` — what the Remove-fonts window says before it takes
//! something out
//!
//! The copy for [`crate::dialogs::unembed`], and the destructive twin of
//! [`crate::text::embed`].
//!
//! ## ★★★ This is the window `tools.unembed_fonts` was blocked on, and the
//! blocker was TRUE
//!
//! Nine of the project's scaffolded commands turned out to be sitting behind
//! reasons that had expired. **This one was not.** Its recorded reason said:
//!
//! > `panels/fonts.rs` records that the old shell's confirmation window exists
//! > because *"three of unembedding's four consequences are invisible on the
//! > canvas (a broken PDF/A claim, an invalidated signature, a renamed font)"*.
//! > That disclosure surface is rule 4 work and is not built.
//!
//! It is built here. Every sentence below exists because of one of those four,
//! and the module is worth reading as the argument for why the window could not
//! have been skipped.
//!
//! ## ★★★ The FOURTH consequence, which nobody had written down
//!
//! **Unembedding does not make the file smaller when pdfcer saves it.**
//!
//! `UnembedPlan::bytes_reclaimable` is the number an operator is chasing, and
//! the engine is explicit about the trap: §7.5.6's update section is *appended*,
//! so the deleted objects get free cross-reference entries in a new section and
//! **their bytes remain in the prior revision, which is still in the file.** An
//! incremental save after an unembed produces a *larger* file. Only a full
//! rewrite drops the bytes.
//!
//! ★★ `crate::app::save` writes **incrementally, always**, by design and by a
//! promise in a tooltip that has been on an operator-visible surface since the
//! command was registered. So this shell cannot deliver the reclaimed bytes at
//! all today.
//!
//! ⇒ The engine's own rule is *"this number must never be reported without the
//! save mode that delivers it"*, and the honest reading of that here is not to
//! soften the number — it is to **state the number and then state that pdfcer's
//! Save will not deliver it**. Filed as an operator question in
//! `OPERATOR_REQUESTS.md`; hiding it would make the window a sales pitch.

use pdfcer_core::font_unembed::PdfaClaim;
use pdfcer_core::font_unembed::{UnembedBlocker, UnembedPlan};

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Remove embedded fonts"
}

/// The opening sentence.
///
/// ★★ It states what changes and what does not, in that order, because the
/// second half is the part an operator will not predict: **no text moves.**
/// `/Widths` is untouched and no content stream is rewritten, so every glyph
/// keeps its advance — what changes is which face draws inside those advances.
/// An operator who expected reflow and got none would think it had not worked.
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
///
/// ★★ The **shared-program** case is disclosed on the row and is the one an
/// operator cannot possibly infer: two fonts may point at the same stream, and
/// when the other one is not part of this operation the key comes out of this
/// descriptor while the **bytes stay in the file**. So the font is unembedded
/// and nothing is recovered, which looks exactly like a bug from outside.
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
        // ★ The rename is the third of the four invisible consequences. A
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
///
/// ★★★ It delegates to `UnembedBlocker::reason`, which is a deliberate
/// exception to this crate's *"every user-visible string lives in `ui_text`"*
/// rule and the reason is stated in the engine's own doc: those are *"the same
/// words the Fonts panel and `list-fonts` already show, because a font that
/// refused in the report and refuses here must refuse for the **same stated
/// reason**."*
///
/// ⇒ Two catalogs for one classifier is how the report and the refusal come to
/// disagree about the same font — and an operator reading two different reasons
/// for one font learns that neither is trustworthy. One classifier, one
/// sentence.
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

/// ★★★ What the removal will and will not do to the file's size.
///
/// **The fourth consequence, and the one that was in no register.** See the
/// module header: `bytes_reclaimable` is the number the operator wants and
/// `crate::app::save` writes incrementally, so pdfcer's own Save leaves every
/// one of those bytes in the file.
///
/// ★★ Both halves are said, in this order — the number first, because it is
/// real and is what the operation achieved, then the reason it does not reach
/// the disk. Reporting only the second would look like the feature failing;
/// reporting only the first would be the sales pitch.
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

/// ★★★ What removal does to a PDF/A claim.
///
/// **The first of the four invisible consequences.** Every part of ISO 19005
/// requires embedded fonts, so unembedding genuinely breaks a conformance claim
/// — and `pdfcer-core` deliberately does **not** refuse on it, saying in as many
/// words that it is *"a consequence the operator may knowingly accept, not a
/// structural impossibility"*, and that **the shells gate on it**.
///
/// This is that gate. It is a sentence rather than a refusal, because the
/// engine's position is that the choice is the operator's and the disclosure is
/// the shell's.
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

/// ★★★ What removal does to a digital signature.
///
/// **The second of the four invisible consequences**, and the only one that is
/// irreversible outside this session. A signature covers a byte range; an
/// incremental save appends and therefore leaves the earlier signature's range
/// intact, but the *document* it certifies no longer matches what a reader
/// renders.
///
/// ★★ Reported only when the document actually carries one, for
/// [`pdfa_line`]'s reason: a warning about signatures on every unsigned drawing
/// is noise that teaches an operator to stop reading the window.
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
///
/// ★★★ Conditional clauses, like [`crate::text::embed::embedded_disclosure`]'s,
/// and for the same reason — but the **size** clause is unconditional here and
/// that is deliberate. It is the operator's motive for the whole operation, and
/// a disclosure that omitted it whenever the number was inconvenient would be
/// omitting exactly the case they care about.
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

    /// ★★★ **The size sentence always says pdfcer's Save will not deliver it.**
    ///
    /// The one assertion in this module that guards a real trap rather than a
    /// wording preference. `bytes_reclaimable` is the number an operator opens
    /// this window for, `app::save` writes incrementally, and a sentence that
    /// reported the first without the second would promise a smaller file that
    /// pdfcer cannot produce.
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
    /// ★ The case an operator cannot infer: the font is unembedded, the bytes
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

    /// ★★ **A PDF/A claim gets the sentence that says removal BREAKS it.**
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
