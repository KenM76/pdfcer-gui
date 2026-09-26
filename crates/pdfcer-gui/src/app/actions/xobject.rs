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
    /// ★★★ **Give this page its own private copy of a shared drawing.**
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
    /// # ★★ Both fields are load-bearing and neither is redundant
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
///
/// # ★★★ Every refusal is caught INSIDE the closure and worded
///
///
/// **This verb owes a sentence for every one of its refusals**, which is
/// unusual — `resize` words exactly one of six — and the reason is a property
/// of what a refusal *looks like* here rather than of how many there are:
///
/// | after a refusal, the operator sees | and infers |
/// |---|---|
/// | the page, unchanged | "it worked — the copy is identical, after all" |
/// | no outline moving, no colour changing | "…so I can safely type in the title block now" |
///
/// A silent decline on this command does not read as "nothing happened". It
/// reads as **success**, because success looks like nothing happening too. The
/// operator then edits a title block they still share with thirty-five other
/// sheets, believing they have privatised it. That is the most expensive
/// failure this shell can produce from one unworded branch, and it is why
/// `crate::text::unshare`'s sentences all end by restating that the sharing is
/// untouched.
///
/// ★★ Recorded from **inside** the closure rather than before the call, for
/// `record_resize_not_rebuildable`'s stated reason: whether the engine will
/// refuse is a property of the FILE — is it encrypted, is it certified, is its
/// `/Size` suppressing entries, is this form reached only from inside another —
/// and none of those is knowable from the selection the dispatcher holds. The
/// one refusal that *is* a query the shell can answer itself
/// ([`UnshareRefusal::NothingInAForm`]) is recorded in the dispatcher, which is
/// the same placement `record_inside_form` uses and for the same reason.
///
/// # The disclosure
///
/// `UnshareFormReport` names the copy, the original and how many references
/// moved. The two object ids go to the **trace** and the count goes to the
/// **status row**, which is `canvas::textedit::report`'s rule applied
/// unchanged: a number about a content stream is evidence, and evidence belongs
/// where a driven check can read it; a count of places on the sheet in front of
/// the operator is a disclosure.
///
///
/// [`fanout`] runs **first**, before `vector_edit` is called at all, and its
/// two possible answers are the two possible shapes of this whole verb:
///
/// | [`fanout`] answers | this function does |
/// |---|---|
/// | `None` — no other page draws it | **nothing**, and says so in a sentence |
/// | `Some(measurement)` | the edit, and discloses the measured number |
///
/// Before that question existed, the verb succeeded on a form invoked exactly
/// once and told the operator *"every other page still shares the original"*
/// about a document that had no other page. Neither half of that was defensible
/// — a byte-identical clone, a rewritten `/Resources`, an undo entry and a
/// dirty document, bought for nothing, and then a false statement about their
/// own file. [`UnshareRefusal::NotShared`]'s docs carry the full account.
///
/// ★★ The order matters: the walk is done **outside** `vector_edit`, not inside
/// its closure. `vector_edit` cancels the render worker and takes `&mut` on the
/// session before the closure runs, so a decline from inside it would have
/// stopped a raster mid-flight to learn that nothing was going to happen. From
/// out here a decline costs one document walk and touches nothing.
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
                    // ★ It names BOTH object numbers and the count. The count
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
///
/// Returns the measurement the disclosure is built from, or `None` when no
/// other page draws it — in which case the decline is already worded and
/// recorded and the caller must do nothing at all.
///
/// # ★★★ Why this exists: the command shipped without ever asking
///
/// "Give this page its own copy" went out on 2026-08-28 and, for one day,
/// **nothing in its chain asked whether the form was invoked more than once.**
/// `catalog/format.rs` gates the control on `selection.in_form`;
/// `conditions.rs` defines that as *"a leaf id is in the selection for this
/// page"*; `dispatch/format.rs` adds only *"the leaf resolves to a containing
/// form"*. And `EditSession::unshare_form` itself guards encryption,
/// certification, `/Size` suppression, form-not-on-page and nesting — and has
/// **no is-shared check**, by design: it is a verb, and a verb does what it is
/// told. So on an ordinary one-page CAD sheet wrapped in a single form the
/// engine allocated an object, privatised `/Resources`, committed an undo entry
/// and returned `Ok`, and the shell told the operator that every other page
/// still shared the original. There were no other pages.
///
/// ⇒ The question is the shell's to ask, and this is where it is asked.
///
/// # ★★★ THE COST, and why a whole-document walk is affordable HERE
///
/// `pdfcer_core::text_edit::invocation_set` walks **every page in the document**
/// and decodes every form it finds, recursively. Its own documentation is blunt
/// about why nothing cheaper exists: *"nothing cheaper can prove a form is not
/// also reached from a page the caller did not ask about."* A form XObject is
/// bound to no page by the standard, so the only proof of absence is a complete
/// scan. On a thirty-six-sheet drawing set that is thirty-six content streams
/// parsed and every form in them decoded — call it tens of milliseconds.
///
/// **That is fine here, and it is fine for exactly one reason: this runs once
/// per operator press.** A press is already a frame the operator expects to
/// cost something; the alternative — the edit itself — allocates an object and
/// rewrites a resource dictionary, which is not cheap either.
///
/// ## ★★★ And why the same walk must NOT go in a condition — R9
///
/// The tempting shape is to grey the control when the form is not shared. It is
/// wrong, and `crate::app::conditions`' own budget says why: conditions are
/// evaluated **on every frame**, for every command in the ribbon plan, to
/// decide what is enabled. Putting a document-wide page walk behind
/// `selection.in_form` would pay for it sixty times a second, on a document
/// nobody is editing, to learn an answer that changes only when the document
/// does. `crate::app::status::decline::Declined::FlattenCertified` already
/// records this ruling in the same words for a certification census: *"putting
/// it in the per-frame path … would pay for it sixty times a second to learn an
/// answer that never moves."*
///
/// ⇒ So the control stays live and **answers in words when it is pressed**,
/// which is R9's own remedy and this project's founding rule: a refusal is a
/// sentence, never a silence.
///
/// # ★★★ What "not shared" is measured as, and why `is_shared()` alone is not
/// # the test
///
/// `InvocationSet::is_shared()` is `sites.len() > 1` — *more than one `Do`
/// reaches this form*. That is the right predicate for the question the engine
/// asks with it (*will an in-place edit be visible somewhere the operator is
/// not looking?* — no, if every site is on the page in front of them). It is
/// **not** the right predicate for this verb, and the difference is a real
/// document rather than a hypothetical one: the engine's own corpus ships
/// `shared-form-twice.pdf`, one page invoking one form twice.
///
/// On that file `is_shared()` is `true`, and unsharing would move **both**
/// references to the copy and leave the original referenced by nothing — an
/// orphan object, a dirty document, and no sentence that is both true and worth
/// reading. So the test is the operator's question rather than the engine's:
/// **does any page other than this one draw it?**
///
/// | measurement | `is_shared()` | this function |
/// |---|---|---|
/// | drawn once, here | `false` | decline |
/// | drawn twice, both here | `true` | **decline** |
/// | drawn here and on sheet 12 | `true` | proceed, "1 other page" |
///
/// # ★★★ An incomplete walk NEVER declines
///
/// `InvocationSet::is_lower_bound()` is true when some page's scan hit the
/// depth guard or a form pdfcer could not decode. On such a document the count
/// is a floor: pages pdfcer could not read may draw this form. Declining there
/// would assert *"nothing else draws it"* from a measurement that did not
/// finish — which is the same defect this function exists to fix, committed in
/// the opposite direction, and `invocation_set`'s docs name that class in as
/// many words: *"an under-count presented as a total is the same class of
/// defect as a silent edit."*
///
/// ⇒ So a lower bound **proceeds**, and the disclosure it produces says *at
/// least* or, when it counted no other page at all, declines to name a number.
/// A `count() == 0` walk — the form was resolved from the page's object model
/// but the scan never saw it — is treated the same way and for the same reason:
/// the two disagree, and the honest response to a disagreement is not a claim.
///
/// # ★★ Read through `session.view()` ALONE, since engine v0.41.0
///
/// Until v0.41.0 this call took a **pair** — the object **graph** from
/// `session.document()` (the base revision) and the **bytes** from
/// `session.view()` (session-aware) — because that is the pairing
/// `EditSession` used at its own two internal `invocation_map` call sites.
/// `Pass 257.0` removed the split: a `DocumentView` now carries both halves,
/// every text-edit planner and helper takes `&DocumentView<'_>`, and there is
/// no `&Document → &DocumentView` coercion, so handing one of them a base
/// revision is a **compile error** rather than a latent wrong answer.
///
/// The behaviour this site wanted is unchanged and is now the only behaviour
/// available: a form whose stream *or whose resource objects* the operator has
/// already edited in this session is walked as it now stands rather than as it
/// was on disk. The half that silently read the base — the graph — was the
/// half that could disagree with the bytes, and it is gone.
fn fanout(doc: &OpenDoc, page: usize, form: ObjId) -> Option<crate::text::unshare::Fanout> {
    let set = pdfcer_core::text_edit::invocation_set(&doc.session.view(), form.num);
    // ★ Counted rather than read off `set.pages.len()`, because "other" is this
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
        // ★ Written on BOTH paths — decline and proceed — because the number
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
            // ★★★ The decline gets a line of its OWN, and it is not redundant
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
///
/// # ★★ A total match with a named catch-all, not a `_ =>` with a guess
///
/// Every variant the verb's own documentation names has an arm, in the order
/// the engine checks them, so this function and `EditSession::unshare_form`'s
/// guard ladder can be read side by side. `PageOutOfRange` and `PageTree` fall
/// to [`UnshareRefusal::Other`] deliberately rather than getting sentences of
/// their own: both mean the page vector moved under a queued command, and the
/// only honest operator-facing content for that is *nothing happened*.
///
/// ★ It is a free function rather than a `From` impl because a `From` would
/// invite this mapping to be reused for another verb's errors, and it is not
/// reusable: `FormNotOnPage` earns a sentence about re-selecting *because this
/// command's operand is derived from a selection*, which is not true of every
/// caller the engine has.
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

    /// ★★★ **Every refusal the verb documents maps to its own sentence**, and
    /// none of them falls through to the catch-all.
    ///
    /// The failure this pins is the one that is invisible in a diff: somebody
    /// adds an arm above, mistypes a variant name, and the compiler is happy
    /// because `_ =>` catches it. The operator then meets *"pdfcer could not do
    /// that"* where they should have met *"that drawing is drawn from inside
    /// another one — use Select the form first"*, which is the difference
    /// between a dead end and an instruction.
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

    /// ★ **An error the verb does not document still gets a sentence**, and it
    /// is the one that promises nothing changed.
    ///
    /// The catch-all is not a hole; it is the honest fallback. This pins that
    /// it resolves to `Other` rather than to whichever arm happens to be first,
    /// which is what a reordering accident would produce.
    #[test]
    fn an_undocumented_error_falls_to_the_honest_fallback() {
        assert_eq!(
            refusal_for(&EditError::PageOutOfRange { index: 9, count: 2 }),
            UnshareRefusal::Other
        );
    }
}
