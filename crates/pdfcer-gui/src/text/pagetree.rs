//! # `text::pagetree` — what the operator is told when a save is refused
//! because the document no longer agrees with itself
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/pagetree.md`.

/// The refusal sentence for a page-tree disagreement of the given `origin`;
/// `crate::pagetree::refusal_origin` decides which one is owed.
#[must_use]
pub fn refusal_sentence(name: &str, origin: crate::pagetree::RefusalOrigin) -> String {
    use crate::pagetree::RefusalOrigin as O;
    match origin {
        O::PreExisting {
            declared,
            reachable,
        } => save_refused_pre_existing(name, declared, reachable),
        O::Root {
            declared,
            reachable,
        } => save_refused_root(name, declared, reachable),
        O::Interior { nodes } => save_refused_interior(name, nodes),
    }
}

/// **The save was refused because the document's page count is wrong.**
///
/// `declared` is what the file says it has (what Acrobat will list),
/// `reachable` is how many pages are really there. `name` is the document's
/// file name, not its path — the operator knows which document he is looking
/// at and a full path would push the numbers off the end of the status bar.
///
/// # Why it names the document at all, then
///
/// Because this shell has document tabs, and a refusal arriving while he is
/// looking at a different tab than the one he pressed `Ctrl+S` on is a real
/// sequence. One short name is cheap insurance against a sentence that appears
/// to be about the wrong file.
#[must_use]
pub fn save_refused_root(name: &str, declared: i64, reachable: usize) -> String {
    let blanks = declared - i64::try_from(reachable).unwrap_or(i64::MAX);
    format!(
        "⊗ {name} was not saved, and your edits are still here — nothing was lost. This \
         document's page list says it has {declared} pages but only {reachable} of them are \
         really there, so saving it would give you a file that opens in Acrobat with \
         {blanks} blank {page} at the end. Undo the page removal (Ctrl+Z) and the document \
         will save normally. pdfcer will not write a file it knows is damaged.",
        page = if blanks == 1 { "page" } else { "pages" },
    )
}

/// **The save was refused because part of the page tree disagrees with itself,
/// but the document's own page count is right.**
///
/// The interior case — see the module header for why it is a separate sentence
/// and does not promise blank pages.
///
/// `nodes` is how many places disagree, and it is printed for one reason: it is
/// the difference between *"one thing went wrong"* and *"the structure is
/// broadly damaged"*, and an operator deciding whether to undo one step or to
/// go back to his last saved file wants to know which.
#[must_use]
pub fn save_refused_interior(name: &str, nodes: usize) -> String {
    format!(
        "⊗ {name} was not saved, and your edits are still here — nothing was lost. Part of \
         this document's page structure no longer agrees with itself in {nodes} \
         {place}, and other PDF readers would show the wrong pages. Undo the page \
         change (Ctrl+Z) and the document will save normally. \
         it has been reported, and pdfcer will not write a file it knows is damaged.",
        place = if nodes == 1 { "place" } else { "places" },
    )
}

/// **The save was refused, and the document was ALREADY like this when it
/// was opened.**
///
/// The third sentence, and it exists because the first two would otherwise give
/// bad advice. Both of them end *"undo the page removal (Ctrl+Z)"*, which is the
/// right remedy exactly when pdfcer caused the damage — and useless when the
/// file arrived that way. An operator who presses Ctrl+Z until the undo stack is
/// empty and still cannot save has been sent in a circle by his own tool.
///
/// # Why the save is still refused rather than merely disclosed
///
/// Because pdfcer would be putting its name on the output. An incremental save
/// keeps the base revision verbatim (§7.5.6) and appends, so a base whose page
/// count is wrong produces an output whose page count is wrong — and the file
/// that lands on his disk is one **pdfcer wrote**, whatever was wrong with its
/// input. Writing a file you know is damaged is not defensible on the grounds
/// that somebody else damaged it first.
///
/// # What it costs him, and why the sentence says so plainly
///
/// It costs him the ability to save this document at all through pdfcer, and
/// there is no remedy inside this program. That is a hard thing to be told and
/// the sentence tells him rather than hedging, because the alternative is an
/// operator pressing save repeatedly against a refusal he has been given no way
/// to understand. He is told what is wrong with the file, that pdfcer did not
/// do it, and that opening it in another tool and re-saving is what repairs it.
#[must_use]
pub fn save_refused_pre_existing(name: &str, declared: i64, reachable: usize) -> String {
    format!(
        "⊗ {name} was not saved, and your edits are still here — nothing was lost. This \
         document's page list already disagreed with itself when you opened it: it says it has \
         {declared} pages and only {reachable} are really there. pdfcer did not do this and will \
         not write a file it knows is damaged. Undo will not help — the fault is in the original \
         file. Opening it in another PDF program and saving it from there rebuilds the page list, \
         and pdfcer will accept it afterwards."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The operator's own symptom appears in the sentence.**
    #[test]
    fn the_root_sentence_names_the_symptom_he_reported() {
        let s = save_refused_root("SW41177.pdf", 36, 34);
        assert!(s.contains("2 blank pages at the end"), "{s}");
        assert!(s.contains("36"), "{s}");
        assert!(s.contains("34"), "{s}");
    }

    /// **One missing page is "1 blank page", not "1 blank pages".**
    ///
    /// Trivial, and asserted because the singular is the case the operator
    /// hits first — he deletes one page to try it.
    #[test]
    fn one_missing_page_reads_as_one_page() {
        let s = save_refused_root("a.pdf", 12, 11);
        assert!(s.contains("1 blank page at the end"), "{s}");
        assert!(!s.contains("blank pages"), "{s}");
    }

    /// **The pre-existing sentence does NOT tell him to press Ctrl+Z, and
    /// the other two do.**
    #[test]
    fn only_the_sentences_pdfcer_can_undo_offer_undo() {
        let pre = save_refused_pre_existing("a.pdf", 13, 12);
        assert!(!pre.contains("Ctrl+Z"), "{pre}");
        assert!(pre.contains("Undo will not help"), "{pre}");
        assert!(
            pre.contains("pdfcer did not do this"),
            "he is owed the origin of a fault he is being refused over: {pre}"
        );
        assert!(
            pre.contains("another PDF program"),
            "and a route out, because there is none inside pdfcer: {pre}"
        );
    }

    /// **All three sentences promise that the work survives.**
    #[test]
    fn both_sentences_say_the_work_survives() {
        for s in [
            save_refused_root("a.pdf", 3, 2),
            save_refused_interior("a.pdf", 1),
            save_refused_pre_existing("a.pdf", 3, 2),
        ] {
            assert!(s.contains("still here"), "{s}");
            assert!(s.contains("nothing was lost"), "{s}");
        }
    }

    /// **Neither sentence uses the engine's vocabulary.**
    #[test]
    fn neither_sentence_speaks_pdf() {
        for s in [
            save_refused_root("a.pdf", 3, 2),
            save_refused_interior("a.pdf", 2),
            save_refused_pre_existing("a.pdf", 3, 2),
        ] {
            for banned in ["/Count", "/Kids", "/Pages", "page tree", "node", "object"] {
                assert!(!s.contains(banned), "{banned:?} in {s:?}");
            }
        }
    }

    /// **Both sentences say what to do**, and it is the only thing that works.
    #[test]
    fn both_sentences_name_the_one_remedy() {
        for s in [
            save_refused_root("a.pdf", 3, 2),
            save_refused_interior("a.pdf", 1),
        ] {
            assert!(s.contains("Ctrl+Z"), "{s}");
        }
        // The third sentence names a DIFFERENT remedy on purpose — see
        // `only_the_sentences_pdfcer_can_undo_offer_undo`.
    }
    /// Each origin reaches its own sentence: only the pre-existing one withholds
    /// Ctrl+Z, so a swapped arm here would promise undo for a file pdfcer
    /// did not damage.
    #[test]
    fn each_refusal_origin_reaches_its_own_sentence() {
        use crate::pagetree::RefusalOrigin as O;
        let pre = refusal_sentence(
            "a.pdf",
            O::PreExisting {
                declared: 13,
                reachable: 12,
            },
        );
        assert_eq!(pre, save_refused_pre_existing("a.pdf", 13, 12));
        assert!(!pre.contains("Ctrl+Z"), "{pre}");
        let root = refusal_sentence(
            "a.pdf",
            O::Root {
                declared: 13,
                reachable: 12,
            },
        );
        assert_eq!(root, save_refused_root("a.pdf", 13, 12));
        assert!(root.contains("Ctrl+Z"), "{root}");
        assert_eq!(
            refusal_sentence("a.pdf", O::Interior { nodes: 2 }),
            save_refused_interior("a.pdf", 2)
        );
    }
}
