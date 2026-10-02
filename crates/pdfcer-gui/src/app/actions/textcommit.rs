//! # `app::actions::textcommit` — **committing an edit to text that is already
//! on the page**
//!
//! One verb: [`commit_text_edit`], the body of `Action::CommitTextEdit`.
//!
//! ## Why this is its own file
//!
//! [`super::apply`]'s match is a **router**: almost every arm in it names a
//! function somewhere else and gets out of the way — *"the arm routes; it does
//! not compute"*, as that module's own header puts it. This verb cannot be an
//! arm of that shape. It plans the edit, traces the plan's disposition, gathers
//! a page-level form report **before** the mutation because of a borrow, reads
//! the edited line's position either side of the mutation for the same reason,
//! copies one fact out of the plan so a closure can take the rest by reference,
//! classifies the engine's refusal, and appends two independent disclosure
//! sources to the report. Seven decisions, none of which is *"which verb is
//! this?"*. So it lives here, and the arm in `apply.rs` is a single call with
//! the variant's four fields.
//!
//! ## What is NOT here, and why
//!
//! `Action::CommitAddText` and `Action::CommitTextAnnot` are elsewhere. They look
//! adjacent — all three end in text on a page — but they are different
//! subjects with different failure modes: adding a run cannot destroy content
//! that was already there, and a text *annotation* never touches the content
//! stream at all. This module is specifically about **replacing** what a
//! producer authored, which is the case where an over-eager plan silently
//! loses a drawing's labels. `crate::canvas::textedit` carries that argument
//! in full; this module routes to it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/textcommit.md`.

use crate::app::state::OpenDoc;

use super::funnel::vector_edit;

/// **Replace a run of text that is already on the page.**
pub(super) fn commit_text_edit(
    doc: &mut OpenDoc,
    page: usize,
    run: usize,
    original: &str,
    replacement: &str,
    reface: Option<&pdfcer_gui_base::editmodel::reface::Reface>,
) {
    if let Some(reface) = reface
        && super::reface::try_commit(doc, page, run, original, replacement, reface)
    {
        return;
    }
    let plan = crate::canvas::textedit::plan(doc, page, run, original, replacement);
    let reason = plan.reason;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // It names the DISPOSITION and the REASON, and it has to name
        // both. A check that asserts only a relation — here, "an edit
        // happened" — is satisfied by any absurdity pointing the right
        // way. A build reverted to `EditOptions::default()` would emit
        // an identical `edit-text` line one line below this one; only
        // this line carries the value such a build gets wrong.
        format!(
            "text-edit-plan page={page} run={run} disposition={:?} reason={reason:?} \
             pinned={}",
            plan.options.disposition,
            plan.request.pinned_span.is_some()
        )
    });
    // Gathered BEFORE the edit, because it reads a `Ref` into the
    // decomposition cache and `vector_edit` wants `&mut OpenDoc`.
    // It decides whether the engine's SHARED CONTENT sentence may
    // be followed by a remedy; `report::PageLevelForms` carries the
    // whole argument, including the nested-form case where the
    // remedy would succeed and change nothing.
    let page_forms = crate::canvas::textedit::report::PageLevelForms::of(doc);
    // **O213's oracle, half one.** The left edge of the line as it stands
    // before the correction, so that a driven check can assert the thing he
    // reported rather than a proxy for it. `report::read_line` carries the
    // whole argument, including why this is not a disclosure; it costs nothing
    // unless someone is tracing, and it is taken here because the funnel below
    // takes `&mut OpenDoc`.
    let before_line = crate::canvas::textedit::report::read_line(doc, page, run);
    // Read from the document rather than from the funnel, which returns `()`
    // by design. The epoch advances only on a granted edit, so this is the
    // one discriminator available for "did anything actually commit".
    let epoch_before = doc.edit_epoch;
    vector_edit(doc, "edit-text", page, 1, |session| {
        // The line request, then the narrowed one when the line matches
        // nothing; `Plan::attempt` owns the order and why.
        let (result, tier) = plan.attempt("commit", |request| {
            session.edit_text(request, &plan.options)
        });
        result
            // **O140 — the refusal is CLASSIFIED, and the arm routes
            // rather than deciding.**
            //
            // `app::status::decline::textedit` owns *"what does the
            // text caret decline, and who says so"*; this is its
            // third decline, and its header carries the whole
            // argument — the engine's coarse `RefusalKind`, the one
            // fact the engine cannot see (whether the pinned run is
            // a single show operator), and why the tidier-looking
            // home in `canvas::textedit::report` was rejected.
            //
            // Inside the closure, through `inspect_err`, on
            // `textstyle::reflow`'s precedent: the funnel takes its
            // decline floor *before* running this and fills the slot
            // only `if slot.is_none()`, so the classified sentence
            // survives and the generic one stands aside.
            .inspect_err(|error| {
                crate::app::status::decline::record_edit_text_refusal(
                    page,
                    run,
                    plan.reached_one_operator(tier),
                    error,
                );
            })
            .map(|report| {
                // The shared-content fan-out, on the trace.
                // `canvas::textedit::trace_target` owns the whole
                // argument for why those three numbers exist and
                // what a wrong build gets wrong about them; this arm
                // routes, as every other arm here does.
                crate::canvas::textedit::report::trace_target(page, run, &report);
                let mut notes = report.disclosures.clone();
                if reason.pins_the_tail() {
                    notes.push(crate::text::textedit::pinned_tail_disclosure(reason));
                }
                // The engine's SHARED CONTENT sentence says WHAT
                // happened; it cannot say what to do, because it
                // has never heard of this shell's commands. Nothing
                // here re-words it — this is appended after it.
                notes.extend(page_forms.remedy_for(&report));
                notes
            })
    });
    // **O213's oracle, half two.** After the funnel, so `doc` is borrowable
    // again and the cache has been invalidated by the epoch bump — the second
    // reading is of the edited page, not of the one that was measured above.
    crate::canvas::textedit::report::trace_left_edge(
        page,
        run,
        doc.edit_epoch != epoch_before,
        before_line,
        crate::canvas::textedit::report::read_line(doc, page, run),
    );
}
