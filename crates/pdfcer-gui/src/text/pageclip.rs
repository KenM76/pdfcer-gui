//! # `text::pageclip` — the sentences the PAGE clipboard says
//!
//! Four, and three of them are things the operator **cannot see**. That is the
//! whole reason this file is as long as it is: a page paste changes what is on
//! screen dramatically and hides its two most consequential effects completely.
//!
//! ## ★★ Rule 4, and why the page clipboard is its sharpest case
//!
//! A pasted page renders exactly as a saved-and-reopened one would. Nothing is
//! badged, tinted or outlined — the operator's standing ruling, and doubly right
//! here, because a whole sheet marked as "recently pasted" would be a permanent
//! second appearance for a document that is now simply longer.
//!
//! And the two facts that matter most are invisible by construction:
//!
//! - a **form field left behind** at the copy, because its boxes straddled a
//!   picked and an unpicked sheet;
//! - **orphaned field boxes** at the paste, which draw exactly like live fields
//!   and cannot be filled by anything.
//!
//! There is no screenshot that shows either. *Render normally; report
//! separately.* **Both.**

/// **Form fields left behind by a page copy.**
///
/// `PageClip::fields_dropped`. A field whose widgets sit on more than one sheet
/// cannot travel unless every one of those sheets was picked, so the engine
/// leaves it out and counts it.
///
/// # ★★★ Why this is said at the COPY and not at the paste
///
/// Because it is a fact about **their selection**, and it is still fixable. At
/// the copy the operator can widen the pick and copy again; by the paste the
/// clip is made and the sentence would be an autopsy.
///
/// ⇒ That is the general rule this file follows and it is worth stating once:
/// **a disclosure belongs at the moment the operator can still act on it.**
///
/// # The wording
///
/// It says *why* — "on sheets you did not pick" — because the remedy is in the
/// cause and an operator told only that something was dropped has to guess.
#[must_use]
pub fn fields_dropped(n: usize) -> String {
    if n == 1 {
        "One form field was left behind: parts of it sit on sheets you did not pick. Pick those \
         sheets too if you want it to come along."
            .to_owned()
    } else {
        format!(
            "{n} form fields were left behind: parts of them sit on sheets you did not pick. \
             Pick those sheets too if you want them to come along."
        )
    }
}

/// **Form-field boxes that arrived belonging to nothing.**
///
/// `InsertOutcome::orphaned_widgets`, and the engine flagged it by name as
/// *"the one that produces a document that looks right and is not"*.
///
/// # ★★★ The mechanism, because the sentence has to be trusted
///
/// A page's `/Annots` array reaches its form-field boxes, so they travel with
/// the page. The `/AcroForm` dictionary that **owns** them is a catalog entry
/// and does not. The boxes therefore arrive drawn, positioned and looking
/// exactly like working fields, belonging to no field at all — so nothing can
/// fill them, no form-data export sees them, and no viewer complains.
///
/// The engine measured **two** on its own smoke test, which is to say this is
/// the ordinary case for pasting a page out of a form, not an exotic one.
///
/// # Why it offers the Forms panel rather than apologising
///
/// Because the remedy exists and is one panel away: the Tab-order section lists
/// exactly these widgets and offers to register them. A sentence that named the
/// problem and not the cure would send the operator looking for a bug.
#[must_use]
pub fn orphaned_widgets(n: usize) -> String {
    let boxes = if n == 1 { "One box" } else { "Boxes" };
    format!(
        "{boxes} that look like form fields came with the pages and belong to no field, so \
         nothing can fill {}. They came from a form whose definition stayed behind. The Forms \
         panel lists {} and can adopt {}.",
        if n == 1 { "it" } else { "them" },
        if n == 1 { "it" } else { "them" },
        if n == 1 { "it" } else { "them" },
    )
}

/// **What a page copy leaves on the operating system's clipboard.**
///
/// ★ The clip **is a PDF** — the engine's own choice, because `pageops::assemble`
/// already does object copying, reference remapping and page-tree construction
/// on every split and merge, and a private page format would be a second
/// implementation of the most-exercised code in the crate.
///
/// So this sentence is not merely a marker the way the object clipboard's is:
/// what pdfcer is holding really is a document. The wording says so, because an
/// operator who learns that can save it, mail it, or open it — and would never
/// guess it from a program that only offered to paste it back.
#[must_use]
pub fn os_marker(pages: usize) -> String {
    if pages == 1 {
        "1 page copied from pdfcer. Paste it back into pdfcer, or use Paste in the Pages tab."
            .to_owned()
    } else {
        format!(
            "{pages} pages copied from pdfcer. Paste them back into pdfcer, or use Paste in the \
             Pages tab."
        )
    }
}

/// The clipboard holds no pages.
///
/// ★ It names the **command**, not the chord, because there is no chord: page
/// copy and paste are named commands on the Pages tab, and `Ctrl+C` belongs to
/// the canvas. Telling the operator to press a key that does something else is
/// worse than saying nothing. `app::dispatch::pageclip`'s header carries why.
#[must_use]
pub const fn nothing_copied() -> &'static str {
    "No pages have been copied. Pick the sheets you want in the Pages panel and press Copy on the \
     Pages tab."
}

/// The engine declined to copy the pages, in its own words.
///
/// ★ Its sentence, prefixed with what was being attempted rather than replaced.
/// The engine's refusals name the document's own state — an encryption, a
/// certification — and are written by the party that knows why; what they cannot
/// know is which gesture the operator was making when they met one.
#[must_use]
pub fn copy_refused(engine: &str) -> String {
    format!("Those pages could not be copied. {engine}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Singular and plural are spelled out, never `field(s)`.
    ///
    /// ★ A parenthesised plural is the tell of a program that could not be
    /// bothered, and every one of these sentences is read by somebody who did
    /// not expect it — which is the whole reason it exists.
    #[test]
    fn no_sentence_fakes_its_plural() {
        for s in [
            fields_dropped(1),
            fields_dropped(4),
            orphaned_widgets(1),
            orphaned_widgets(3),
            os_marker(1),
            os_marker(9),
        ] {
            assert!(!s.contains("(s)"), "parenthesised plural in: {s}");
        }
    }

    /// ★★ Every disclosure names the REMEDY, not just the problem.
    ///
    /// The two invisible facts are the ones an operator cannot investigate for
    /// themselves — a left-behind field and an orphaned box both look like
    /// nothing at all — so a sentence that stopped at the diagnosis would send
    /// them hunting for a defect.
    #[test]
    fn both_invisible_disclosures_say_what_to_do_about_it() {
        assert!(
            fields_dropped(1).contains("Pick those sheets"),
            "the remedy for a left-behind field is to widen the pick, and it is still available \
             at the moment this is said"
        );
        assert!(
            orphaned_widgets(2).contains("Forms panel"),
            "the remedy for an orphaned box is the Tab-order section, which lists exactly these \
             and offers to adopt them"
        );
    }

    /// The empty-clipboard sentence names a control, never a chord.
    #[test]
    fn the_empty_sentence_does_not_send_the_operator_to_a_key_that_does_something_else() {
        let s = nothing_copied();
        assert!(
            !s.contains("Ctrl+"),
            "★ page copy has no chord — Ctrl+C is the canvas's — so naming one would send them \
             to a key that copies a shape. Got: {s}"
        );
        assert!(
            s.contains("Pages"),
            "it must name where the control is. Got: {s}"
        );
    }
}
