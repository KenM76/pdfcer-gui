//! # `text::pagetree` — what the operator is told when a save is refused
//! because the document no longer agrees with itself
//!
//! One event, **three** sentences, and a wording rule that is stricter than
//! most of this catalog because the sentence has to do something unusual:
//! **explain a refusal whose cause is a defect in pdfcer itself, without either
//! blaming the operator or leaving him unsure whether his work still exists.**
//!
//! ## The event
//!
//! [`crate::app::save::write_copy`] built the bytes, [`crate::pagetree::audit`]
//! walked their page tree, and some node's `/Count` does not match the number
//! of pages actually beneath it. Nothing was written. See
//! [`crate::pagetree`]'s header for what that state is and why it is not
//! repaired here.
//!
//! ## ★★★ The four things the sentence has to carry, in this order
//!
//! 1. **That no file was written**, so the operator is not left looking for
//!    one. `crate::app::status::decline`'s `SaveFailed` line already says *"the
//!    copy was not written"* and this sentence is added **beside** it rather
//!    than instead of it —
//!    [`crate::app::save::redaction_refusal_note`]'s standing reason, which is
//!    that replacing the fact with an explanation leaves an operator unsure
//!    whether a file appeared.
//! 2. **That his edit is still here.** This is the part a refusal most often
//!    gets wrong. The session is untouched — `to_incremental_bytes` takes
//!    `&self` — so the document on screen after the refusal is exactly the
//!    document that was there before it, undo stack and all. An operator who
//!    believes a failed save cost him his work will do something drastic to
//!    recover it.
//! 3. **What is actually wrong, in his terms.** Not `/Count`, not `/Kids`, not
//!    "the page tree is inconsistent". The two numbers and the symptom: *this
//!    document says it has 36 pages and only 34 of them are really there;
//!    saving it would give you a file that opens in Acrobat with 2 blank pages
//!    at the end.* That is his own bug report read back to him, which is the
//!    strongest evidence a sentence can offer that it understands the problem.
//! 4. **What he can do next.** When pdfcer caused the damage, exactly one thing
//!    works and it is named: **undo the page removal.** Nothing else does — not
//!    saving somewhere else (the fault is in the document, not the disk), not
//!    saving again (it is deterministic), not closing and reopening (the file on
//!    disk is the unedited one). When pdfcer did **not** cause it, undo is a
//!    circle and the sentence says so and names a different route — see
//!    [`save_refused_pre_existing`].
//!
//! ## ★★ The wording rule: name the symptom the OTHER reader will show
//!
//! Every sentence here says what **Acrobat** will do, and that is deliberate
//! and is the only honest framing available. pdfcer's own reader walks `/Kids`
//! and sees a perfectly healthy 34-page document; if the copy described what
//! pdfcer sees, it would be describing a file that is fine. The whole defect is
//! that two readers disagree, so the operator is told about the reader whose
//! answer is wrong *and which he uses*. He named it himself in his report.
//!
//! ⇒ Corollary, binding on anyone editing these strings: **do not soften
//! "blank pages" into "may not open correctly".** He can verify "blank pages at
//! the end"; he cannot verify a hedge, and a hedge he cannot verify reads as
//! pdfcer refusing for reasons of its own.
//!
//! ## ★ Why the number of blank pages is computed rather than described
//!
//! `declared - reachable` is the count of pages Acrobat will list that are not
//! in the file, and on the delete path it is exactly the number of pages he
//! removed — which is the coincidence that let him diagnose it in one sentence
//! (*"equalling the number of pages I deleted"*). Printing the number rather
//! than saying "some" is what lets him recognise his own symptom.
//!
//! ## ★★ THREE sentences, not one, because three states are genuinely
//! different — and two of them would give bad advice in the third's place
//!
//! | | when | what only it can say |
//! |---|---|---|
//! | [`save_refused_root`] | the **root** disagrees, and the file arrived sound | the exact symptom: *n* blank pages at the end |
//! | [`save_refused_interior`] | only an interior node disagrees | that readers will show the wrong pages, without promising which |
//! | [`save_refused_pre_existing`] | the file **already** disagreed when it was opened | that pdfcer did not cause it, that **undo will not help**, and the one route that repairs it |
//!
//! ★★★ The third is the one that must not be merged away. The first two both end
//! *"undo the page removal (Ctrl+Z)"*, which is right exactly when pdfcer caused
//! the damage — and a **circle** when the file came in broken. An operator who
//! empties his undo stack against a refusal his own tool told him undo would fix
//! has lost his work as well as his time, which is strictly worse than an
//! unexplained refusal. [`tests::only_the_sentences_pdfcer_can_undo_offer_undo`]
//! is what stops the three being consolidated back into one on the grounds that
//! they say nearly the same thing. Which one applies is decided by
//! [`crate::pagetree::refusal_sentence`], from structured data — a second audit
//! of the file on disk — and never by inspecting a message.
//!
//! ★★★ **THE BLAME CLAUSE IS GONE — 2026-09-05, and its removal closed a
//! residual this header had predicted.**
//!
//! Both ordinary sentences used to end *"This is a fault in pdfcer, it has been
//! reported, and pdfcer will not write a file it knows is damaged."* Every
//! clause of that was true when written: `delete_pages` updated the immediate
//! parent's `/Count` and no ancestor, it had been reported that morning, and
//! the refusal was the only thing between the operator and a file Acrobat opens
//! with blank pages at the end.
//!
//! **`Pass 251.1` fixed it the same day.** So on this engine the sentence
//! blames pdfcer for damage pdfcer did not do — and the operator meets that
//! sentence precisely when he is least able to judge it.
//!
//! ⇒ ★★ **The remedy was to delete the attribution, not to update it.** A
//! refusal owes him three things: that nothing was lost, what is wrong in his
//! terms, and what to press. **Whose fault it is is not one of them** — it is
//! the sentence's least useful clause and its most perishable, and this header
//! already carried a paragraph predicting exactly that it would go stale.
//!
//! ★ That also closes the residual this paragraph used to name: the interior
//! sentence was *wrong* for a file that **arrived** with an interior-only
//! disagreement, because it blamed pdfcer for somebody else's writer, and
//! `refusal_sentence` could not switch to the pre-existing wording (that one
//! needs the root's two numbers, and an interior-only case has none). **With no
//! attribution in either sentence there is nothing left to be wrong about.** A
//! clause deleted is a clause that cannot drift.
//!
//! ⚠ `save_refused_pre_existing` **keeps** its attribution, and must: its whole
//! job is to say *pdfcer did not do this and undo will not help*, which is a
//! statement about the file rather than about pdfcer's record, and getting it
//! wrong costs him his undo stack as well as his time.

/// **The save was refused because the document's page count is wrong.**
///
/// `declared` is what the file says it has (what Acrobat will list),
/// `reachable` is how many pages are really there. `name` is the document's
/// file name, not its path — the operator knows which document he is looking
/// at and a full path would push the numbers off the end of the status bar.
///
/// # ★ Why it names the document at all, then
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

/// ★★★ **The save was refused, and the document was ALREADY like this when it
/// was opened.**
///
/// The third sentence, and it exists because the first two would otherwise give
/// bad advice. Both of them end *"undo the page removal (Ctrl+Z)"*, which is the
/// right remedy exactly when pdfcer caused the damage — and useless when the
/// file arrived that way. An operator who presses Ctrl+Z until the undo stack is
/// empty and still cannot save has been sent in a circle by his own tool.
///
/// # ★ Why the save is still refused rather than merely disclosed
///
/// Because pdfcer would be putting its name on the output. An incremental save
/// keeps the base revision verbatim (§7.5.6) and appends, so a base whose page
/// count is wrong produces an output whose page count is wrong — and the file
/// that lands on his disk is one **pdfcer wrote**, whatever was wrong with its
/// input. Writing a file you know is damaged is not defensible on the grounds
/// that somebody else damaged it first.
///
/// # ★★ What it costs him, and why the sentence says so plainly
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
    ///
    /// He wrote *"blank pages at the end of the document equalling the number
    /// of pages I deleted"*. A sentence that described a page-tree
    /// inconsistency without saying "blank pages" would be true, accurate, and
    /// unrecognisable to the person it is for.
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

    /// ★★★ **The pre-existing sentence does NOT tell him to press Ctrl+Z, and
    /// the other two do.**
    ///
    /// The whole reason the third sentence exists. Undo is the remedy exactly
    /// when pdfcer caused the damage; on a file that arrived broken it is a
    /// circle, and an operator who empties his undo stack against a refusal he
    /// was told undo would fix has been misled by his own tool. This is the
    /// assertion that stops the three sentences being consolidated back into
    /// one on the grounds that they say nearly the same thing.
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

    /// ★★★ **All three sentences promise that the work survives.**
    ///
    /// The claim is true — `to_incremental_bytes` takes `&self` and the
    /// refusal happens before `std::fs::write` — and it is the one an operator
    /// most needs and is least likely to assume. A future edit that dropped
    /// this clause would leave a refusal that reads as data loss.
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
    ///
    /// Rule: the operator is never shown `/Count`, `/Kids`, `/Pages`, "page
    /// tree" or "node". Those are in the trace, where a reader of a machine
    /// wants them. This is the check that stops the next edit reaching for the
    /// precise word.
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
}
