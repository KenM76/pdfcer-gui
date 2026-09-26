//! # `app::actions::xobject` — the verbs whose subject is a form XObject
//!
//! One verb today: **give this page its own private copy of a shared drawing**,
//! so that a later edit to it changes this page and no other.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/xobject.md`.

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::unshare::UnshareRefusal;

/// The verbs whose subject is a form XObject — a drawing invoked by a page,
/// possibly by many pages, possibly several times by one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XObjectAction {
    /// **Give this page its own private copy of a shared drawing.**
    ///
    /// `EditSession::unshare_form`. Raised by `crate::app::dispatch::format`'s
    /// `format.unshare_form` arm — from the Format contextual tab and from the
    /// canvas context menu — and by nothing else.
    ///
    /// # Why the operator needs this, concretely
    ///
    /// ISO 32000-1 §8.10.1 names a CAD system's standard component as the
    /// *purpose* of form XObjects, and this operator's drawing sets are exactly
    /// that: one title block, one stream object, invoked from thirty-six
    /// sheets. Since `pdfcer-core` `Pass 119.0` this shell can **edit text
    /// inside a form**, which means an operator fixing a typo on sheet 12
    /// changes all thirty-six — and pdfcer cannot prevent that structurally,
    /// because there is exactly one stream object to write.
    ///
    /// `pdfcer-core`'s decision 076 ruled that edit-in-place-and-disclose is the
    /// **default**, and `R206` requires that two defensible behaviours ship as
    /// two options. This variant is the second option. Until it existed the
    /// operator had the default and no choice at all, which is the state `R206`
    /// exists to prevent.
    ///
    /// # Both fields are load-bearing and neither is redundant
    ///
    /// `page` is **not** merely for the trace, unlike
    /// `super::annot::AnnotAction::Delete`'s. The verb's signature is
    /// `(page_index, form)` and the page is half the operand: unsharing is
    /// defined as *"re-point **this page's** references"*, and the same form on
    /// a different page is a different, equally valid call that this one must
    /// not perform.
    ///
    /// `form` is the **outermost** enclosing form's `ObjId`, resolved before the
    /// action was raised. See the module header for why that resolution is not
    /// done here and why the innermost form would be refused.
    ///
    /// # `Copy`, which its neighbours are not
    ///
    /// Both fields are `Copy` — a `usize` and an `ObjId` — so the whole enum is,
    /// and `Action` is not made heavier by carrying it. `super::annot` and
    /// `super::bookmarks` are not `Copy` because they carry `String`s and
    /// `Vec`s; nothing here needs one, and nothing here should grow one: a
    /// second copy of a name the document already holds is how a stale operand
    /// gets written back.
    Unshare {
        /// The 0-based page whose references move. Half the operand, not a
        /// trace field.
        page: usize,
        /// The **outermost** enclosing form's stream object.
        form: ObjId,
    },
}

/// Apply one form-XObject verb.
///
/// One arm today, matching its neighbours' shape: `super::bookmarks::apply` and
/// `super::attachments::apply` are both reached from `super::apply` by a single
/// line, so the family's rules live with the family rather than in the
/// interpreter's match.
pub(super) fn apply(doc: &mut OpenDoc, action: XObjectAction) {
    match action {
        XObjectAction::Unshare { page, form } => unshare(doc, page, form),
    }
}

/// **Clone a shared form XObject for one page, as one undoable command.**
fn unshare(doc: &mut OpenDoc, page: usize, form: ObjId) {
    let Some(measured) = fanout(doc, page, form) else {
        // The decline is already worded and recorded by `fanout`. Returning
        // here is the whole of "change nothing": no worker cancel, no session
        // borrow, no undo entry, no `edit_epoch` bump, and a document that is
        // exactly as clean as it was a moment ago.
        return;
    };
    super::apply::vector_edit(doc, "unshare-form", page, 1, |session| {
        session
            .unshare_form(page, form)
            .inspect_err(|error| {
                crate::app::status::decline::record_unshare(refusal_for(error));
            })
            .map(|report| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // It names BOTH object numbers and the count. The count
                    // is also on the status row, and that duplication is
                    // deliberate: a driven check must be able to assert the
                    // number without reading prose, and the two coming from one
                    // `report` value means they cannot disagree.
                    //
                    // `original` is on the line because it is the number a
                    // wrong build gets wrong in the quietest way — a verb handed
                    // the INNERMOST form instead of the outermost would refuse
                    // on a nested drawing and, on a singly-nested one, would
                    // succeed against the wrong object.
                    format!(
                        "unshare-form-applied page={page} original={} copy={} moved={}",
                        report.original.num, report.copy.num, report.references_moved
                    )
                });
                vec![crate::text::unshare::unshared(
                    report.references_moved,
                    measured,
                )]
            })
    });
}

/// **Ask how widely this drawing is drawn, once, on the press.**
fn fanout(doc: &OpenDoc, page: usize, form: ObjId) -> Option<crate::text::unshare::Fanout> {
    let set = pdfcer_core::text_edit::invocation_set(&doc.session.view(), form.num);
    // Counted rather than read off `set.pages.len()`, because "other" is this
    // verb's whole subject: the page in front of the operator is not one of the
    // pages that keeps the original, and a build that forgot to subtract it
    // would over-report by exactly one on every document — the friendliest
    // possible off-by-one, since it is invisible on any file with two or more
    // sheets and wrong on every file with one.
    let other_pages = set.pages.iter().filter(|&&p| p != page).count();
    let lower_bound = set.is_lower_bound();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // Written on BOTH paths — decline and proceed — because the number
        // that decided which one ran is the evidence a driven check needs, and
        // a line emitted only on success would leave the decline provable only
        // by the absence of something.
        format!(
            "unshare-form-measured page={page} form={} places={} pages={} other={other_pages} \
             lower_bound={lower_bound}",
            form.num,
            set.count(),
            set.pages.len()
        )
    });
    if !lower_bound && set.count() > 0 && other_pages == 0 {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The decline gets a line of its OWN, and it is not redundant
            // with the measurement above. A driven check that had to prove the
            // decline from the *absence* of `unshare-form-applied` would pass
            // on every build where the row is greyed, the dispatcher has no arm
            // or the menu never opened — every possible breakage produces the
            // same absence. This line is the positive oracle: the press
            // arrived, the walk ran, and the verb chose not to act.
            format!(
                "unshare-form-declined page={page} form={} reason=not-shared places={}",
                form.num,
                set.count()
            )
        });
        crate::app::status::decline::record_unshare(UnshareRefusal::NotShared);
        return None;
    }
    Some(crate::text::unshare::Fanout {
        other_pages,
        lower_bound,
    })
}

/// Which sentence an `EditError` from `unshare_form` earns.
fn refusal_for(error: &pdfcer_core::edit::EditError) -> UnshareRefusal {
    use pdfcer_core::edit::EditError;
    match error {
        EditError::DocumentEncrypted => UnshareRefusal::Encrypted,
        EditError::CertificationForbidsChange { .. } => UnshareRefusal::Certified,
        EditError::ObjectCreationWouldExposeHiddenObjects { .. } => {
            UnshareRefusal::WouldExposeHiddenObjects
        }
        EditError::FormNestedInAnotherForm { .. } => UnshareRefusal::Nested,
        EditError::FormNotOnPage { .. } => UnshareRefusal::NotOnPage,
        EditError::ObjectNumbersExhausted => UnshareRefusal::NumbersExhausted,
        _ => UnshareRefusal::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::edit::EditError;

    /// **Every refusal the verb documents maps to its own sentence**, and
    /// none of them falls through to the catch-all.
    #[test]
    fn each_documented_refusal_earns_its_own_sentence() {
        for (error, expected) in [
            (EditError::DocumentEncrypted, UnshareRefusal::Encrypted),
            (
                EditError::CertificationForbidsChange { permission: 2 },
                UnshareRefusal::Certified,
            ),
            (
                EditError::ObjectCreationWouldExposeHiddenObjects { count: 17 },
                UnshareRefusal::WouldExposeHiddenObjects,
            ),
            (
                EditError::FormNestedInAnotherForm { form: 7 },
                UnshareRefusal::Nested,
            ),
            (
                EditError::FormNotOnPage {
                    form: 7,
                    page_index: 3,
                },
                UnshareRefusal::NotOnPage,
            ),
            (
                EditError::ObjectNumbersExhausted,
                UnshareRefusal::NumbersExhausted,
            ),
        ] {
            assert_eq!(
                refusal_for(&error),
                expected,
                "{error} fell through to the wrong sentence"
            );
        }
    }

    /// **An error the verb does not document still gets a sentence**, and it
    /// is the one that promises nothing changed.
    #[test]
    fn an_undocumented_error_falls_to_the_honest_fallback() {
        assert_eq!(
            refusal_for(&EditError::PageOutOfRange { index: 9, count: 2 }),
            UnshareRefusal::Other
        );
    }
}
