//! # `app::actions::redactsel` — **redact what is selected on the page**
//!
//! ## What this closes
//!
//! **Ken, 2026-08-30:** *"the redaction tool — am I able to select objects on
//! the canvas and redact them that way yet? I only tried it when it only worked
//! with the search box and it didn't work for some things. it just told me it
//! couldn't."*
//!
//! He was right on both counts, and the second half is the more interesting one.
//!
//! ## ★★★ Why the search box could not do what he wanted
//!
//! Until now this shell had exactly **two** ways to mark a redaction:
//!
//! | route | what it can reach |
//! |---|---|
//! | the search box | **text pdfcer can read as text** |
//! | mark whole page | everything, indiscriminately |
//!
//! Nothing in between. And on a CAD drawing the first is far narrower than it
//! sounds: a title-block value drawn as **vector strokes**, a scanned stamp, a
//! logo, a signature image and a run in a font whose encoding pdfcer cannot map
//! are all invisible to a text search. There is nothing to type that would find
//! them.
//!
//! ⇒ So *"it just told me it couldn't"* was the program being honest about a
//! route that genuinely could not reach the thing — not a bug, and not a thing
//! any amount of retyping would have fixed. What was missing was **a third
//! route that does not go through text at all.**
//!
//! ## What this is
//!
//! Select anything on the page — a path, an image, a text run, several at once —
//! and mark it for redaction. The geometry comes from the **selection's own
//! bounds**, which the canvas already computes to draw the outline, so what gets
//! marked is exactly the box the operator can see around what they picked.
//!
//! ★ It is `EditSession::add_redaction`, the same verb the *mark whole page*
//! control uses, with the page's crop box swapped for the selection's bounds.
//! No new engine capability was needed and none was asked for: the verb takes
//! arbitrary quads and always has.
//!
//! ## ★★ Marking is not applying, and this changes nothing about that
//!
//! A `/Redact` annotation is a **mark**: it removes no content. Applying is a
//! separate, deliberate, confirmed act (`edit.redact_apply`), and this route
//! feeds the same review list as the other two — the panel lists every mark
//! before anything is destroyed.
//!
//! That is worth stating because a *"redact this"* control on a right-click menu
//! sounds like it destroys something immediately. It does not, and the wording
//! says so.
//!
//! ## Why the bounds and not the object's exact shape
//!
//! A redaction region is quads (§12.5.6.23 `/QuadPoints`), so an L-shaped
//! polyline could in principle be covered by several. It is deliberately one
//! box per selected object, for two reasons that point the same way:
//!
//! 1. **A redaction that follows an outline tells you what was there.** The
//!    silhouette of a signature is a signature; the outline of a part number is
//!    its digit count. A bounding box discloses less, which is the entire point
//!    of the operation.
//! 2. It is what the operator can see. The selection outline *is* the box, so
//!    the mark lands exactly where the preview said it would.

use pdfcer_core::annot_author::{Quad, RedactAppearance};

use crate::app::actions::apply::vector_edit;
use crate::app::state::OpenDoc;

/// **Mark every selected object on the current page for redaction.**
///
/// One `/Redact` annotation per selected object, all in **one undo entry** —
/// `vector_edit` wraps the whole loop, so an operator who marks six things and
/// presses `Ctrl+Z` gets back the state before they started rather than five
/// marks and a headache.
///
/// # What it does when nothing is selected
///
/// Nothing, silently. The command is gated on `selection.any`, so a pointer
/// cannot reach it in that state — and a keyboard route does not exist for this
/// verb. A sentence here would be describing a state the operator cannot be in.
pub fn mark_selection(doc: &mut OpenDoc, appearance: &RedactAppearance) {
    let page_index = doc.view.page_index;
    let Some(page) = doc.pages.get(page_index).cloned() else {
        return;
    };

    // ★★ The bounds come from the SELECTION's own cached outlines — the same
    // rectangles the canvas draws — so what is marked is exactly the box the
    // operator was looking at. Deriving them again from the decomposition would
    // be a second answer to a question the canvas has already answered, and the
    // two would disagree the first time one of them was corrected.
    let quads: Vec<Quad> = doc
        .selection
        .outlines()
        .iter()
        .filter(|(entry, _)| entry.page == page_index)
        .filter_map(|(_, canvas)| {
            // Canvas space is the page's device space at scale 1.0, so the hop
            // is the page's own height and nothing else. `canvas_to_pdf_space`
            // is the one place that arithmetic lives.
            let min = crate::viewer::canvas_to_pdf_space(canvas.min, &page)?;
            let max = crate::viewer::canvas_to_pdf_space(canvas.max, &page)?;
            Some(Quad::from_rect(pdfcer_core::page_tree::Rect {
                // ★ NORMALISED, because the y flip inverts the corners: the
                // canvas rect's `min` is its TOP-left and the PDF rect's `llx`
                // / `lly` is its BOTTOM-left. A quad built from the unswapped
                // pair is inside-out, and §12.5.6.23 does not say what a
                // viewer should do with one.
                llx: f64::from(min.x.min(max.x)),
                lly: f64::from(min.y.min(max.y)),
                urx: f64::from(min.x.max(max.x)),
                ury: f64::from(min.y.max(max.y)),
            }))
        })
        .collect();

    if quads.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("redact-mark-selection-declined page={page_index} reason=no-bounds")
        });
        return;
    }

    let count = quads.len();
    // ★ `-requested`, not the bare label: `vector_edit` writes
    // `redact-mark-selection page=… n=… epoch=…` for the same edit, and
    // `Trace::last()` matches on the FIRST TOKEN. Two lines with one name means
    // a driven check asking for `quads=` gets the funnel's line and reports
    // that the verb did nothing. `tools/gates/check-trace-names.py` caught it.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("redact-mark-selection-requested page={page_index} quads={count}")
    });

    // ★★★ ONE annotation carrying every quad, not one annotation per object.
    //
    // §12.5.6.23 makes `/QuadPoints` a list precisely so one mark can cover
    // several regions, and the operator made **one** gesture — they selected a
    // group of things and asked for them to go. Six marks would mean six rows
    // in the review list and six presses to undo a decision they made once.
    //
    // ★ It also keeps the apply honest: `apply_redactions` removes what the
    // quads cover, and one annotation with six quads and six annotations with
    // one each remove exactly the same content. The difference is entirely in
    // what the operator has to manage afterwards.
    let spec = appearance.to_spec(quads);
    // ★★★ ASKED BEFORE THE EDIT, because afterwards the selection is gone.
    //
    // `vector_edit` bumps the epoch and the canvas resolves its selection
    // against the new revision, so the objects the operator picked are no
    // longer enumerable inside the closure. The question — "did any of what he
    // chose turn out to be a raster image?" — has to be answered while the
    // answer still exists. See `super::redactimg` for why it is asked at all.
    let images = crate::app::actions::redactimg::images_in_selection(doc);
    vector_edit(doc, "redact-mark-selection", page_index, count, |session| {
        session.add_redaction(page_index, &spec).map(|_| {
            let mut notes = vec![crate::text::redact::marked_selection(count)];
            // ★ Second, never first. The mark SUCCEEDED — that is the sentence
            // he is owed first, and the caveat belongs beside it rather than
            // instead of it. This module's header carries rule 4's ordering:
            // "a residual is named in the SAME sentence as the success, never
            // in a dialog that replaces it".
            if images > 0 {
                notes.push(crate::text::redact::mark_covers_image(images));
            }
            notes
        })
    });
}
