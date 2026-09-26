//! # `text::pageclip` — the sentences the PAGE clipboard says
//!
//! Four, and three of them are things the operator **cannot see**. That is the
//! whole reason this file is as long as it is: a page paste changes what is on
//! screen dramatically and hides its two most consequential effects completely.
//!
//! ## Rule 4, and why the page clipboard is its sharpest case
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/pageclip.md`.

/// **Form fields left behind by a page copy.**
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
#[must_use]
pub const fn nothing_copied() -> &'static str {
    "No pages have been copied. Pick the sheets you want in the Pages panel and press Copy on the \
     Pages tab."
}

/// The engine declined to copy the pages, in its own words.
#[must_use]
pub fn copy_refused(engine: &str) -> String {
    format!("Those pages could not be copied. {engine}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Singular and plural are spelled out, never `field(s)`.
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

    /// Every disclosure names the REMEDY, not just the problem.
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
