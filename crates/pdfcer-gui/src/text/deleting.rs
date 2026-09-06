//! # `text::deleting` — the sentences a Delete that removed nothing shows
//!
//! Four of them, for [`crate::canvas::deleting`], plus the rule that decides
//! which refusals get one at all.
//!
//! ## ★★ Why four and not eleven
//!
//! [`crate::canvas::deleting::Refusal`] has eleven variants and seven of them
//! describe a state the operator put themselves in and can **see** on screen:
//! nothing is selected, they are inside an image that has no parts, they are at
//! the point rung on a line of text. `canvas::moving::decline` settled the rule
//! for its own eight and it holds unchanged here — *a bar that narrates the
//! obvious stops being read*, and a surface nobody reads is worse than no
//! surface, because the next real sentence lands in a place their eye has
//! learned to skip.
//!
//! The four below are the ones an operator meets **having done nothing
//! wrong**: the shape they picked is inside a container pdfcer cannot cut into,
//! the file's own structure forbids removing this label before the next one,
//! they picked four points and pdfcer removes one at a time, or the page's
//! contents will not decompose at all. In every one of those the outline is on
//! screen, round the thing they want gone, and the key does nothing. From where
//! they sit, Delete is broken.
//!
//! ★★★ **The fourth joined them on 2026-09-05, and the reason it was silent is
//! worth keeping.** `NoObjectModel` was classified as obvious because for the
//! whole life of the deeper rungs it was only ever reachable through a frame
//! that had **forgotten to ask** for the decomposition — so it was not a state
//! the operator had put themselves in, it was a bug wearing a refusal's name,
//! and no sentence is right for a bug. `canvas::modelneed` now asks on the
//! frame the key arrives, so reaching this means the document really is
//! unreadable at that depth, and that is a limit the operator is owed in words.
//! ⇒ **A refusal classified as "obvious" while it was unreachable-except-by-bug
//! has to be re-classified when the bug is fixed**, or the fix ships a new
//! silence.
//!
//! ## The rule every sentence follows
//!
//! **Name the thing the operator can see, never the thing pdfcer models.**
//! [`crate::text::resizing`]'s header states it and this catalogue obeys it:
//! they can see a **label**, a **line**, a **corner point** and a **group**;
//! they cannot see a "show operator", a "subpath", an "anchor" or a "form
//! XObject". A refusal phrased in the file format's vocabulary reads as an
//! internal error.
//!
//! ★ And **never a bare "dimension"** — R8b Rule 15. The labels on the
//! operator's drawings are *pdf dimensions*: page content pdfcer reads and must
//! not silently alter. The word on screen is **label**, which is what he calls
//! them and what he can see.
//!
//! ## ★ Where they are shown
//!
//! The status bar's disclosure row, through
//! `crate::app::actions::disclosure::record_notes`, stamped with the epoch
//! currently on screen — so the sentence stands until the operator's next real
//! edit moves past it. Deliberately **not** drawn on the page: R8b Rule 4 is
//! about *disclosure*, and applied content must render exactly as saved content
//! will. Nothing here marks the canvas.

use crate::canvas::deleting::Refusal;

/// The sentence for a refusal to delete, or `None` when the state is one the
/// operator can already see.
///
/// One function over the whole enum rather than one per variant, so a variant
/// added to [`Refusal`] is a **compile error here** instead of a Delete that
/// refuses silently — which is precisely what the three deeper rungs did for
/// the whole life of this shell until 2026-09-05. `text::resizing::refusal`
/// makes the same choice for the same reason and its header records the same
/// history.
///
/// `Option` rather than an empty string, because *"there is deliberately
/// nothing to say"* and *"the sentence is missing"* must not be the same value.
#[must_use]
pub const fn refusal(reason: Refusal) -> Option<&'static str> {
    match reason {
        // ★★★ §9.4.2, and the one refusal in this file that names a remedy —
        // which is the entire reason it is asked before the press rather than
        // left to the engine. See `canvas::deleting`'s header on R83.
        //
        // The mechanism, in the operator's terms: a label that carries no
        // position of its own starts wherever the label before it ended. Remove
        // the earlier one and the later one slides somewhere nobody put it. The
        // remedy always works and cannot fail — a label that follows a label
        // that is already gone is not a case this can produce.
        Refusal::RunWouldMoveNext(_) => Some(
            "The label after this one has no position of its own — it starts where this one \
             ends — so removing this one would move it somewhere you did not put it. Delete the \
             later label first, then this one.",
        ),
        // ★★ The operator has an outline round the thing they want gone and the
        // key does nothing. This is a real limit of the engine rather than of
        // this shell — `pdfcer-core` has one delete verb for the inside of a
        // container and it removes a whole object — and saying so is the
        // difference between a limit and a bug.
        //
        // The sentence names what DOES work, because it does: pressing Escape
        // to leave the part and deleting the whole object inside the container
        // reaches `delete_objects_in_form`, which is wired.
        Refusal::InsideForm => Some(
            "This line is inside a group that pdfcer can only remove whole. Press Escape to \
             step back out to the whole shape, then Delete.",
        ),
        // ★ Four points highlighted, one press, and pdfcer would remove one of
        // them. Refusing and saying how many is the honest answer; acting on
        // the first is the defect that let a four-anchor drag move one anchor
        // for months.
        Refusal::ManyNodes(_) => {
            Some("pdfcer removes one corner point at a time. Click a single point, then Delete.")
        }
        // ★★★ The page will not decompose, so nothing INSIDE an object can be
        // named — see this module's header for why this one is new. The
        // sentence names what the operator can still do, because they can: the
        // Object rung never needed the decomposition, so Escape and Delete
        // removes the whole shape on a page whose interior pdfcer cannot read.
        Refusal::NoObjectModel => Some(
            "pdfcer could not read the inside of this page, so it cannot remove one piece of a \
             shape here. Press Escape to step back out to the whole shape, then Delete.",
        ),
        // The seven that say nothing, listed rather than caught by a wildcard:
        // a new variant must be classified by whoever adds it, and `_ => None`
        // would classify it as "obvious" by default — which is the direction
        // that ships a silent Delete.
        Refusal::NothingSelected
        | Refusal::NoPartEntered
        | Refusal::NoNodeEntered
        | Refusal::UnaddressableObject
        | Refusal::NoPartsInObject
        | Refusal::NoNodeVerbForText => None,
    }
}

#[cfg(test)]
mod tests {
    use super::refusal;
    use crate::canvas::deleting::Refusal;

    /// ★ Every sentence this catalogue offers is finished English prose.
    ///
    /// Not a formatting nicety: `check-string-gaps.sh` exists because a lost
    /// line-continuation backslash bakes six spaces into the middle of a
    /// wrapped literal, and the result *looks deliberate in the diff and wrong
    /// in the window*. The gate greps the source; this asserts the value.
    #[test]
    fn every_sentence_is_finished_prose_with_no_baked_gap() {
        for reason in [
            Refusal::RunWouldMoveNext(3),
            Refusal::InsideForm,
            Refusal::ManyNodes(4),
            Refusal::NoObjectModel,
        ] {
            let sentence = refusal(reason).expect("this refusal is meant to speak");
            assert!(
                !sentence.contains("  "),
                "{reason:?} has a run of spaces in it: {sentence}"
            );
            assert!(
                sentence.ends_with('.'),
                "{reason:?} does not end in a full stop: {sentence}"
            );
        }
    }

    /// ★★ **No sentence may say "dimension"** — R8b Rule 15, mechanically.
    ///
    /// The project has been corrected on this once already. A *pdf dimension*
    /// is page content pdfcer reads; a *ce dimension* is what pdfcer authors;
    /// and a bare "dimension" on an operator-facing surface is ambiguous
    /// between the two at exactly the moment the operator is deciding whether
    /// their drawing is about to be altered. The word on screen is "label".
    #[test]
    fn no_sentence_writes_a_bare_dimension() {
        for reason in [
            Refusal::RunWouldMoveNext(0),
            Refusal::InsideForm,
            Refusal::ManyNodes(2),
            Refusal::NoObjectModel,
        ] {
            if let Some(sentence) = refusal(reason) {
                assert!(
                    !sentence.to_lowercase().contains("dimension"),
                    "{reason:?} writes a bare \"dimension\": {sentence}"
                );
            }
        }
    }

    /// The seven that are deliberately silent stay silent — so that a future
    /// edit which starts narrating "nothing selected" has to change a test that
    /// says why it should not.
    #[test]
    fn the_states_the_operator_can_see_say_nothing() {
        for reason in [
            Refusal::NothingSelected,
            Refusal::NoPartEntered,
            Refusal::NoNodeEntered,
            Refusal::UnaddressableObject,
            Refusal::NoPartsInObject,
            Refusal::NoNodeVerbForText,
        ] {
            assert!(
                refusal(reason).is_none(),
                "{reason:?} narrates a state the operator can already see"
            );
        }
    }
}
