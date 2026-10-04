//! # `app::actions::annots` — the verbs that change an annotation
//!
//! Held here rather than in [`super::apply`] so that the funnel stays inside
//! R2's 1,500-line ceiling. The seam is the one [`super::pages`] already draws
//! next door: *what class of thing does this verb act on?* — pages there,
//! annotations here, page **content** in `apply`.
//!
//! ## The routing obligation every style verb here carries
//!
//! `EditSession::set_markup_style` restyles an annotation that already exists
//! — colour, interior, width, opacity and arrowheads, keeping its object id —
//! and the Format contextual tab is the surface for it. Every one of those
//! properties becomes a verb in here.
//!
//! Each carries a routing obligation `delete` does
//! not: a **ce dimension** is a `/Line` with `/IT /LineDimension`, it passes
//! every "markup pdfcer can author" test, and restyling one through
//! `set_markup_style` regenerates it as a bare line with its label and witness
//! lines gone. `pdfcer-core` refuses it by name and points at
//! `set_dimension_style`. `canvas::selection::annot::AnnotKind` carries the
//! distinction on the selected target precisely so that routing is a `match`
//! the compiler checks — see its header.
//!
//! ## What is NOT here
//!
//! **Placing** an annotation. `Action::CommitMarkup`, `CommitTextAnnot` and
//! the measure commits stay in `apply`, because their subject is the *gesture*
//! that authored them rather than the annotation afterwards. The line is the
//! same one `pages` draws: this file is what happens to a thing that already
//! exists.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/annots.md`.

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

/// Make one markup part of the page.
mod flatten;
pub(crate) use flatten::refusal as flatten_refusal;
mod inknodes;
/// **The text-annotation restyle verb** — `set_text_annot_style` and the
/// stamp-label disclosure it owes. Its header says why this one verb is a
/// module and its neighbours are arms.
mod textannotstyle;
// pub(super) rather than private: the round-trip test that closes the
// operator's "can't enter a size in the properties box" report lives beside
// the AUTHORING verb in `actions::textannot`, and drives this one through the
// shell rather than the engine so the undo entry and the texture drop are
// exercised with it.
pub(super) use textannotstyle::set_text_annot_style;
/// **The polygon/polyline node verbs** - `move_node`, `insert_node`, `remove_node`
/// over `reshape_annotation`.
mod vertexnodes;
use vertexnodes::{insert_node, move_node, remove_node};

/// **Remove one annotation from the document.**
pub(super) fn delete(doc: &mut OpenDoc, page: usize, id: ObjId) {
    // A `/Redact` mark reaches this too (click the mark, press Delete —
    // `OPERATOR_REQUESTS.md` O161) and the engine's
    // `delete_annotation` routes it to `delete_redaction_mark` itself
    // (`AnnotationDeletionRoute::RedactionMark`), so the panel's Remove and
    // the canvas's Delete share one verb without this function naming it. The
    // test below is the guard that keeps that route reachable from here.
    super::apply::vector_edit(doc, "delete-annotation", page, 1, |session| {
        session.delete_annotation(id).map(|report| {
            crate::text::markup::deleted_collateral(
                report.popup_removed,
                report.parent_popup_cleared,
                report.replies_orphaned,
                report.group_members_promoted,
            )
            .into_iter()
            .collect()
        })
    });
    // The selection named an object that no longer exists. Cleared here rather
    // than left for the next frame to notice: an outline around a deleted
    // annotation promises that a second Delete would do something, and the
    // second Delete would refuse.
    doc.selection.clear_annot();
}

/// **Move one markup annotation by a page-space delta.**
pub(super) fn move_annot(doc: &mut OpenDoc, id: ObjId, dx: f64, dy: f64) {
    super::apply::vector_edit(doc, "move-annotation", 0, 1, |session| {
        session.move_annotation(id, dx, dy).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention `forms::import_data`
                    // records: the funnel writes its own bare-named line for
                    // the same edit and `.last()` would read that one.
                    "move-annotation-applied id={} dx={dx:.3} dy={dy:.3} keys={} popup={}",
                    id.num,
                    outcome.geometry_keys_moved.len(),
                    outcome.popup_left_behind.is_some()
                )
            });
            outcome
                .popup_left_behind
                .map(|_| vec![crate::text::markup::popup_left_behind()])
                .unwrap_or_default()
        })
    });
}

/// **Scale a markup annotation about an anchor.** `OPERATOR_REQUESTS.md` O51.
pub(super) fn resize(
    doc: &mut OpenDoc,
    id: ObjId,
    anchor: (f64, f64),
    (sx, sy): (f64, f64),
    uniform: bool,
    modifiers: crate::canvas::scaling::Modifiers,
) {
    // **THE OPERATOR'S SWITCHES, and the flag is never derived from the
    // geometry.**
    //
    // Deriving `scale_stroke_width` from whether the drag was proportional is
    // a **workaround for a refusal**: with a foreign appearance and a uniform
    // scale the engine refuses unless either the stroke scales or distortion
    // is allowed, and forcing the first makes the common case work with no
    // control. It also makes the operator's answer unreachable on exactly the
    // resizes where they are most likely to have one, and
    // `OPERATOR_REQUESTS.md` **O51** is a correction about precisely that
    // shape of reasoning.
    //
    // What stands in its place is the worded decline below, not a different
    // guess.
    //
    // The discriminator behind the DEFAULTS is unchanged and is the engine's,
    // promoted from this shell's own CAD argument: *is the property a length in
    // the space being transformed?* An inset is; a line weight is a drafting
    // convention. `canvas::scaling` carries the whole account.
    // A STAMP IS ARTWORK, AND SCALING ARTWORK IS THE RESIZE.
    //
    // The operator: *"there's still no way to edit the
    // size of a placed stamp … on the canvas, or by entering a different size
    // in the properties box."* The engine's `resize_annotation` re-bakes the
    // appearances it knows how to author (shapes; `/FreeText`) and, for any
    // other `/AP`, either CARRIES it through §12.5.5's placement matrix or
    // refuses: it carries exactly when the caller's options say the matrix
    // agrees with the ask — `scale_stroke_width` for a uniform scale, or
    // `allow_appearance_distortion` for anything — and refuses otherwise with
    // *"pdfcer did not draw it"*, which for a pdfcer-drawn stamp is false.
    //
    // Those two switches are about BORDERS: a rectangle whose 1 pt outline
    // should stay 1 pt when the box grows. A stamp has no such border — it is
    // a picture of text in a frame, and what the operator means by "make it
    // bigger" is precisely that the picture scales, letters and frame
    // together, as every other program scales a stamp. So for a stamp both
    // switches are set here, unconditionally: the matrix path is then the
    // correct resize, not a distortion knowingly accepted, and it reaches the
    // same `resize_annotation` from the canvas grips and from the Properties
    // width/height fields alike. (The Tool-panel switches keep their meaning
    // for every other kind.) A request for a true re-bake stands
    // (`request_resize_annotation_refuses_a_pdfcer_authored_stamp_as_foreign.md`),
    // but the carry is vector and exact, so nothing visible waits on it.
    //
    // The subtype is read through the same `page_annotations` lookup
    // `canvas::annotnodes::geometry` uses, so the two agree about which
    // annotation is meant.
    let is_stamp = doc.selection.annot().is_some_and(|a| {
        a.target.id == id
            && doc.pages.get(a.target.page).is_some_and(|page| {
                pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
                    .into_iter()
                    .any(|found| found.id == Some(id) && found.subtype.as_slice() == b"Stamp")
            })
    });
    let opts = if is_stamp {
        modifiers
            .to_options()
            .with_scale_stroke_width(true)
            .with_allow_appearance_distortion(true)
    } else {
        modifiers.to_options()
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("resize-annotation-options id={} stamp={is_stamp}", id.num)
    });
    super::apply::vector_edit(doc, "resize-annotation", 0, 1, |session| {
        session
            .resize_annotation(id, anchor, sx, sy, &opts)
            .inspect_err(|error| {
                // **The refusal is caught here and worded**, rather than
                // being left to `vector_edit`'s generic arm, which traces the
                // engine's reason and, per O116, words only *"That change was
                // refused, and the document is unchanged."* That floor ends
                // the silence; it cannot name a remedy, and naming one is the
                // whole value of catching the refusal here.
                //
                // A resize that silently does nothing is this project's
                // founding failure: the operator drags a grip, lets go, the
                // shape snaps back, and no surface anywhere says why.
                //
                // Recorded from INSIDE the closure because the condition is
                // not knowable before the call — whether an appearance is
                // pdfcer's own is a property of the file. `record_save_failure`
                // is called from the apply phase for the identical reason;
                // `record_flatten_certified` is not, because its refusal is a
                // query.
                //
                // Only this one variant. Every other `EditError` stays
                // trace-only, which is honest: wording a
                // decline is catalog work per refusal, and a `format!` of an
                // `EditError`'s `Display` would route diagnostic prose into the
                // UI — the thing `check-ui-strings`' exclusion 3 names in as
                // many words.
                if let pdfcer_core::edit::EditError::ResizeAppearanceNotRebuildable {
                    uniform: was_uniform,
                    ..
                } = error
                {
                    crate::app::status::decline::record_resize_not_rebuildable(*was_uniform);
                }
                // And its sibling: a `/Text` sticky or a `NoZoom` annotation
                // has no size to scale. Worded because the Properties panel's
                // geometry fields can raise this resize even though the
                // sticky's canvas grips are move-only. `subtype == "Text"` is
                // the engine's own test for which of `resize_annotation`'s two
                // `why` sentences it chose.
                if let pdfcer_core::edit::EditError::ResizeFixedSizeMarker { subtype, .. } = error {
                    crate::app::status::decline::record_resize_fixed_size_marker(subtype != "Text");
                }
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "resize-annotation-applied id={} sx={sx:.4} sy={sy:.4} uniform={uniform} \
                         keys={} appearance={:?} stroke={}",
                        id.num,
                        outcome.geometry_keys_scaled.len(),
                        outcome.appearance,
                        outcome.stroke_width.is_some()
                    )
                });
                let mut notes = Vec::new();
                if outcome.stroke_width.is_none() {
                    notes.push(crate::text::markup::stroke_width_unchanged());
                }
                if matches!(
                    outcome.appearance,
                    pdfcer_core::edit::ResizedAppearance::CarriedDistorted
                ) {
                    notes.push(crate::text::markup::appearance_distorted());
                }
                notes
            })
    });
}

/// **Turn a markup annotation about a pivot.**
pub(super) fn rotate(doc: &mut OpenDoc, id: ObjId, pivot: (f64, f64), degrees: f64) {
    super::apply::vector_edit(doc, "rotate-annotation", 0, 1, |session| {
        session
            .rotate_annotation(id, pivot, degrees)
            .inspect_err(|error| {
                // **The refusal is caught here and worded**, rather than
                // being left to `vector_edit`'s generic arm, which since O116
                // words an un-categorised sentence naming no remedy.
                // [`resize`]'s own comment is the argument and it applies
                // unchanged: a grip that is dragged, released, and does
                // nothing with no explanation is this project's founding
                // defect.
                //
                // From INSIDE the closure, because none of these is knowable
                // before the call — whether a document's certification forbids
                // an annotation change is a census over its objects, and
                // whether the routing sent the wrong kind here is a fact about
                // the value the engine resolved.
                //
                // **Every** `EditError` is worded, unlike [`resize`], which
                // words one variant and leaves the rest to the trace. The
                // difference is that a resize refuses in several shapes with
                // distinct remedies, and this verb has essentially none the
                // operator can act on — so a catch-all that says *the page is
                // exactly as it was* is honest here where a catch-all there
                // would have been a shrug.
                crate::app::status::decline::record_rotate(refusal_for(error));
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // It carries the ANGLE, the PIVOT and both rectangles,
                    // which is what a wrong build gets wrong. A line saying only
                    // "a rotation applied" would be identical for a build that
                    // turned the other way, pivoted about a corner instead of
                    // the centre, or left the appearance `/Matrix` alone — and
                    // that last one produces a `/Rect` that grew around artwork
                    // that did not move, which looks exactly like the correct
                    // behaviour this function discloses.
                    //
                    // `-applied`, per the convention `forms::import_data`
                    // records: the funnel writes its own bare-named line for
                    // the same edit and `.last()` would read that one.
                    format!(
                        "rotate-annotation-applied id={} deg={degrees:.2} px={:.2} py={:.2} \
                         keys={} matrix={} from={:.1}x{:.1} to={:.1}x{:.1}",
                        id.num,
                        pivot.0,
                        pivot.1,
                        outcome.geometry_keys_rotated.len(),
                        u8::from(outcome.appearance_matrix_updated),
                        outcome.from.urx - outcome.from.llx,
                        outcome.from.ury - outcome.from.lly,
                        outcome.to.urx - outcome.to.llx,
                        outcome.to.ury - outcome.to.lly,
                    )
                });
                // **NO DISCLOSURE.** The outline is drawn at the mark's own
                // angle (`canvas::annotquad`, `OPERATOR_REQUESTS.md` O147), so
                // it hugs the artwork and there is no swelling box to explain.
                // `text::rotating` carries why a `rect_grew` sentence must not
                // be restored here for the O145 growth defect.
                Vec::new()
            })
    });
}

/// **The engine's rectangle rule as a stable single-word token**, for the trace.
const fn rect_rule_token(rule: pdfcer_core::edit::RectDerivation) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed in the UI.
    match rule {
        pdfcer_core::edit::RectDerivation::Artwork => "artwork",
        pdfcer_core::edit::RectDerivation::Geometry => "geometry",
        pdfcer_core::edit::RectDerivation::PreviousRect => "previous-rect",
        _ => "other",
    }
}

/// **Set a markup annotation's angle absolutely.**
pub(super) fn set_rotation(doc: &mut OpenDoc, id: ObjId, pivot: (f64, f64), degrees: f64) {
    super::apply::vector_edit(doc, "set-annotation-rotation", 0, 1, |session| {
        session
            .set_annotation_rotation(id, pivot, degrees)
            .inspect_err(|error| {
                crate::app::status::decline::record_rotate(refusal_for(error));
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // `asked=` and `deg=` are BOTH carried and they are
                    // different numbers: the first is the absolute angle the
                    // operator typed, the second is the delta the engine worked
                    // out to get there. A build that passed the typed value
                    // through as a delta would show them equal on the first
                    // edit of an unturned mark — which is most of them — and
                    // diverge only on the second, so a trace carrying one of
                    // the two could not see it.
                    //
                    // `rect_derived=` is `AnnotationRotate::rect_derived_from`
                    // — which of the engine's rules produced the new
                    // rectangle, and the one field an operator report would
                    // need: the artwork and geometry rules compose, the
                    // previous-rect rule cannot, and nothing else on this line
                    // distinguishes them.
                    format!(
                        "set-annotation-rotation-applied id={} asked={degrees:.2} deg={:.2} \
                         px={:.2} py={:.2} rect_derived={} to={:.1}x{:.1}",
                        id.num,
                        outcome.degrees,
                        pivot.0,
                        pivot.1,
                        rect_rule_token(outcome.rect_derived_from),
                        outcome.to.urx - outcome.to.llx,
                        outcome.to.ury - outcome.to.lly,
                    )
                });
                crate::text::rotating::rect_still_grows(outcome.rect_derived_from)
                    .into_iter()
                    .collect()
            })
    });
}

/// **Turn a ce dimension about a pivot.**
pub(super) fn rotate_dimension(
    doc: &mut OpenDoc,
    dimension: pdfcer_core::dimension::DimensionId,
    annot: ObjId,
    pivot: (f64, f64),
    degrees: f64,
) {
    super::apply::vector_edit(doc, "rotate-dimension", 0, 1, |session| {
        session
            .rotate_dimension(dimension, pivot, degrees)
            .inspect_err(|error| {
                // Same placement and the same argument as [`rotate`]'s: caught
                // inside the closure, because whether the engine refuses is not
                // knowable before the call.
                crate::app::status::decline::record_rotate(refusal_for(error));
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // `relaxed=` is the field worth tracing and the one a
                    // wrong build gets wrong: a rotation that turned the
                    // geometry and left a `Horizontal` constraint behind
                    // produces a line and a constraint that disagree, which is
                    // invisible on the canvas and shows up the next time
                    // anything regenerates from the constraint.
                    //
                    // `annot=` is carried purely so a failed run ties back to
                    // the thing the operator had selected; the verb addressed
                    // the sidecar record, not the annotation.
                    format!(
                        "rotate-dimension-applied dim={} annot={} deg={degrees:.2} \
                         px={:.2} py={:.2} relaxed={}",
                        outcome.dimension.0,
                        annot.num,
                        pivot.0,
                        pivot.1,
                        u8::from(outcome.constraint_relaxed),
                    )
                });
                if outcome.constraint_relaxed {
                    vec![crate::text::rotating::axis_lock_relaxed()]
                } else {
                    Vec::new()
                }
            })
    });
}

/// Which worded refusal an `EditError` from either rotation verb becomes.
fn refusal_for(error: &pdfcer_core::edit::EditError) -> crate::text::rotating::RotateRefusal {
    use crate::text::rotating::RotateRefusal;
    match error {
        // The routing backstop. Unreachable while `canvas::rotating`'s
        // `match` on `AnnotKind` holds and while `canvas::selection::annot`
        // keeps excluding `/Widget` — which is exactly why it is worded: if
        // this sentence ever appears, the routing has broken, and a broken
        // route with a sentence is a bug report rather than a dead handle.
        pdfcer_core::edit::EditError::AnnotationMoveWrongVerb { .. } => RotateRefusal::WrongVerb,
        // The one an operator meets on an ordinary file and cannot guess at:
        // a signed drawing looks exactly like an unsigned one on the canvas.
        pdfcer_core::edit::EditError::CertificationForbidsChange { .. } => RotateRefusal::Certified,
        _ => RotateRefusal::Other,
    }
}

/// `text` as a note signed by `author` (none when blank) and dated now in UTC
/// (none when the clock is before 1970). `/T` and `/M` are reachable only
/// through a note, so a mark with no words still passes an empty one.
pub(super) fn signed_note(text: &str, author: Option<&str>) -> pdfcer_core::edit::MarkupNote {
    // Builders, not a struct literal: `MarkupNote` is `#[non_exhaustive]`.
    let mut note = pdfcer_core::edit::MarkupNote::new(text);
    if let Some(author) = author.map(str::trim).filter(|a| !a.is_empty()) {
        note = note.by(author);
    }
    if let Some(stamp) = crate::app::clock::pdf_date_utc() {
        note = note.at(stamp);
    }
    note
}

/// **Write the note on an annotation that already exists** — `/Contents`, and
/// conditionally `/T` and `/M` — as one undoable command.
pub(super) fn set_note(doc: &mut OpenDoc, id: ObjId, text: &str, author: Option<&str>) {
    let note = signed_note(text, author);
    super::apply::vector_edit(doc, "set-markup-note", 0, 1, |session| {
        session.set_markup_note(id, &note).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `-applied`, per the convention `forms::import_data`
                // records: the funnel writes its own bare-named line for the
                // same edit and `.last()` would read that one instead.
                //
                // `keys` is the field worth tracing rather than the text: it is
                // the engine's own answer to "what actually moved", and the
                // whole `/T`-preservation contract above is invisible from a
                // screenshot and from the saved page alike.
                //
                // `rebaked` is the ONLY oracle for the half of this edit a
                // screenshot of the panel cannot see. The status line speaks
                // for the `false`-on-a-`/FreeText` case alone (correctly — see
                // below), so without this field a driven check could not tell
                // a re-baked text box from a sticky note, which are the two
                // outcomes that look identical from outside.
                format!(
                    // `rich_dropped` is a COUNT of the keys the engine
                    // removed, not the keys themselves: the operator-facing
                    // sentence does not name them (they mean nothing to a
                    // reviewer) and a driven check only needs to know whether
                    // the disclosure should have fired.
                    "set-markup-note-applied id={} chars={} keys={} replaced={} \
                     subtype={} rebaked={} rich_dropped={}",
                    id.num,
                    text.chars().count(),
                    change.keys_written.join("+"),
                    change.replaced.is_some(),
                    change.subtype,
                    change.appearance_rebaked,
                    change.rich_text_dropped.len()
                )
            });
            // The text box's disclosure goes FIRST, and the order is the
            // decision. `record_notes` documents the first sentence as the one
            // an operator reads if they read only one — and between "here are
            // the words you replaced" and "the page kept an appearance this
            // edit did not move", only the second is something they cannot
            // find out any other way.
            //
            // **Two gates, both the ENGINE's own answer**, and neither is
            // anything this shell inferred about the selection:
            //
            // 1. `MarkupNoteChange::subtype` — the raw `/Subtype`.
            // 2. `MarkupNoteChange::appearance_rebaked` — whether the picture
            //    moved with the words. `set_markup_note` re-bakes a
            //    `/FreeText`'s `/AP` itself, which is why this is a condition
            //    rather than a constant.
            //
            // `false` is **not a failure**: it is correct and final on a
            // sticky and on a stamp, and the sentence must not fire for them.
            // The four-row table is at `crate::text::textannot`'s edit-time
            // banner.
            //
            // …and the rich-text drop goes LAST, because it
            // is the only one that is *good news*: the operator's document was
            // inconsistent before this edit and is consistent after it. The
            // first two are things they need in order to act — what the page
            // did not do, and what words they can retype. This one is a
            // statement that a problem they never knew about has been closed,
            // and it must not displace either of them from the truncated slot.
            //
            // It is disclosed rather than swallowed because **pdfcer removed
            // a key the operator did not ask it to remove**, which is R8b rule 4
            // in its narrowest form: nothing on the page changes, nothing in
            // the panel changes, and the only other way to find out is a diff
            // of the file.
            crate::text::textannot::note_edit_disclosure(&change.subtype, change.appearance_rebaked)
                .map(str::to_owned)
                .into_iter()
                .chain(
                    change
                        .replaced
                        .as_deref()
                        .and_then(crate::text::markup::note_replaced),
                )
                .chain(crate::text::markup::rich_text_dropped(
                    &change.rich_text_dropped,
                ))
                .collect()
        })
    });
}

/// **Remove an annotation's note entirely** — `/Contents`, `/T` and `/M` — as
/// one undoable command.
pub(super) fn clear_note(doc: &mut OpenDoc, id: ObjId) {
    super::apply::vector_edit(doc, "clear-markup-note", 0, 1, |session| {
        session.clear_markup_note(id).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "clear-markup-note-applied id={} keys={} had_note={} had_author={} \
                     subtype={} rebaked={} rich_dropped={}",
                    id.num,
                    change.keys_written.join("+"),
                    change.replaced.is_some(),
                    change.replaced_author.is_some(),
                    change.subtype,
                    change.appearance_rebaked,
                    change.rich_text_dropped.len()
                )
            });
            // First, for the same reason as `set_note` — and the surprise is
            // worse here. On a text box whose appearance pdfcer did not draw,
            // removing the comment takes away the only copy the operator can
            // edit and leaves the copy they cannot, still on the page, saying
            // what it always said.
            //
            // On one it DID draw, `clear_markup_note` empties the painted
            // box in the same command — so `appearance_rebaked` is `true`,
            // nothing is left unsaid, and this stays quiet.
            //
            // **And the rich-text drop applies here too**, which is easy to
            // miss because *removing* a note sounds like it could not leave a
            // stale copy behind. It could: `/RC` is a second copy of the same
            // comment, so clearing `/Contents` without it would leave the
            // pop-up — and, on a `/FreeText`, the page — still showing words
            // the operator has just deleted.
            //
            // Wired in both arms rather than in one of them. A disclosure
            // attached to one of two paths through the same engine report is
            // how a surface comes to be right on one route and wrong on the
            // other.
            crate::text::textannot::note_clear_disclosure(
                &change.subtype,
                change.appearance_rebaked,
            )
            .map(str::to_owned)
            .into_iter()
            .chain(
                change
                    .replaced
                    .as_deref()
                    .and_then(crate::text::markup::note_removed),
            )
            .chain(crate::text::markup::rich_text_dropped(
                &change.rich_text_dropped,
            ))
            .collect()
        })
    });
}

/// **Answer a comment** — `EditSession::add_reply`, §12.5.6.2 Table 170.
pub(super) fn add_reply(doc: &mut OpenDoc, parent: ObjId, text: &str, author: Option<&str>) {
    let note = signed_note(text, author);
    super::apply::vector_edit(doc, "add-reply", 0, 1, |session| {
        session.add_reply(parent, &note).map(|added| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `-applied`, per the convention `set_note` records: the
                // funnel writes its own bare-named line for the same edit and
                // `.last()` would read that one instead.
                //
                // `reply_id` and `page` are the two facts nothing on screen
                // can report. The reply is drawn inside its parent's thread,
                // at the parent's own coordinates, so a screenshot cannot tell
                // a reply that landed on page 3 from one that landed on page 1
                // — and `add_reply` chooses the page itself, by walking to the
                // parent, which is exactly the kind of engine-side decision a
                // trace exists to make visible.
                format!(
                    "add-reply-applied parent={} reply={} page={} chars={} \
                     parent_had_popup={} reply_has_popup={}",
                    added.parent_id.num,
                    added.reply_id.num,
                    added.page_index,
                    text.chars().count(),
                    added.parent_had_popup,
                    added.reply_has_popup
                )
            });
            crate::text::panels::comments::reply_posted(added.reply_has_popup)
                .map(str::to_owned)
                .into_iter()
                .collect()
        })
    });
}

/// **Record a comment's pop-up state IN THE FILE** —
/// `EditSession::set_annotation_open`.
pub(super) fn set_open(doc: &mut OpenDoc, id: ObjId, open: bool) {
    super::apply::vector_edit(doc, "set-annotation-open", 0, 1, |session| {
        session.set_annotation_open(id, open).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `was` is the field worth tracing beside the request: it is
                // the engine's answer to *"did the file already say this?"*,
                // and `None` distinguishes an absent key from an explicit
                // `false`. Nothing on screen can show that difference, and it
                // is the difference between honouring another producer's
                // intent and defaulting over it.
                format!(
                    "set-annotation-open-applied id={} subtype={} open={open} was={:?} \
                     annot_written={} popup_written={}",
                    id.num,
                    change.subtype,
                    change.was,
                    change.annotation_written,
                    change.popup_written
                )
            });
            crate::text::annotpopup::open_state_written(
                change.annotation_written,
                change.popup_written,
            )
            .map(str::to_owned)
            .into_iter()
            .collect()
        })
    });
}

// ===========================================================================
// The router
// ===========================================================================

/// **Route one annotation verb to its body.**
pub(super) fn apply_action(
    doc: &mut OpenDoc,
    action: crate::app::actions::annot::AnnotAction,
    author_name: &str,
) {
    use crate::app::actions::annot::AnnotAction as A;
    match action {
        // The move takes no page for the reason the variant states:
        // `move_annotation` finds the annotation by id, and the disclosure it
        // owes is about a pop-up rather than a sheet.
        A::Move { id, dx, dy } => move_annot(doc, id, dx, dy),
        // The one arm here whose body is in another family's module, and it
        // is deliberate: `app::actions::reorder` owns the `/Annots`
        // permutation, because the tab-order panel needs it too. A second
        // implementation beside `reorder_annotations` — same engine verb, same
        // disclosures, different words — is precisely the drift that module's
        // own header is about.
        A::Arrange { page, id, to } => crate::app::actions::reorder::arrange(doc, page, id, to),
        A::Resize {
            id,
            anchor,
            sx,
            sy,
            uniform,
            modifiers,
        } => resize(doc, id, anchor, (sx, sy), uniform, modifiers),
        // Two rotation arms, not one with a kind flag: the engine refuses a
        // ce dimension from the annotation verb by name. See
        // [`rotate_dimension`].
        A::Rotate { id, pivot, degrees } => rotate(doc, id, pivot, degrees),
        A::SetRotation { id, pivot, degrees } => set_rotation(doc, id, pivot, degrees),
        A::RotateDimension {
            dimension,
            annot,
            pivot,
            degrees,
        } => rotate_dimension(doc, dimension, annot, pivot, degrees),
        A::Delete { page, id } => delete(doc, page, id),
        A::Flatten { page, id } => flatten::flatten(doc, page, id),
        A::FlattenPage { page } => flatten::flatten_page(doc, page),
        A::SetNote {
            id,
            text,
            keep_author,
        } => {
            let author = if keep_author {
                None
            } else {
                Some(author_name).filter(|a| !a.is_empty())
            };
            set_note(doc, id, &text, author);
        }
        A::ClearNote { id } => clear_note(doc, id),
        // A reply, and note what this arm does NOT do: it asks no
        // `keep_author` question. The author preference is filtered by the
        // same rule two lines up and handed straight in, because a reply is a
        // new annotation with no prior `/T` to preserve. Folding it into the
        // `SetNote` arm above would put a reviewer's answer one boolean away
        // from overwriting the comment it answers — see `AnnotAction::Reply`.
        A::Reply { parent, text } => {
            add_reply(
                doc,
                parent,
                &text,
                Some(author_name).filter(|a| !a.is_empty()),
            );
        }
        // The **document's** `/Open`, which is a different subject from
        // whether a bubble is showing on screen — `canvas::notepopup::open`
        // owns that and raises nothing. Reached only from an explicit control;
        // the variant's docs carry the undo argument.
        A::SetOpen { id, open } => set_open(doc, id, open),
        // The node verbs, and the operator's *"I also can't edit or delete
        // nodes of a markup shape once it is drawn."*
        //
        // One arm per variant rather than one carrying a `VertexEdit`: the
        // shell's action bus does not carry the engine's enum, so a new
        // `VertexEdit` variant arrives as a compile error in [`reshape`]
        // rather than as a silent `..` here.
        A::MoveNode { id, index, dx, dy } => move_node(doc, id, index, dx, dy),
        A::InsertNode { id, after, at } => insert_node(doc, id, after, at),
        A::RemoveNode { id, index } => remove_node(doc, id, index),
        // The ink point verbs — O158 — carry a `(stroke, point)` address and
        // reach a different planner; [`inknodes::apply`] destructures them.
        // One arm here so this file stays under R2.
        A::MoveInkPoint { .. } | A::InsertInkPoint { .. } | A::RemoveInkPoint { .. } => {
            inknodes::apply(doc, action);
        }
        // **The only report a refused node edit produces.** The gesture
        // preflights through `reshape_annotation_preview`, so no verb is
        // reached, no funnel is entered and no `EditRefused` is recorded — if
        // this arm is removed the operator drags a corner of a triangle out of
        // the shape, releases, and the triangle is still a triangle with
        // nothing anywhere saying why. That silence is the report this whole
        // feature answers.
        //
        // Recorded here rather than at the gesture because the decline store
        // is `pub(super)` inside `crate::app` and the canvas is outside that
        // boundary — the same crossing `DimensionAction::DeclineVertexEdit`
        // makes for the ce-dimension twin.
        A::DeclineNodeEdit { why } => {
            crate::app::status::decline::record_markup_node_refused(why);
        }
        // The SECOND style verb, and the one arm here that answers a
        // different engine function from its neighbours. `set_markup_style`
        // reads through `spec_from_dict`, which has no `/Text` arm;
        // `set_text_annot_style` reads through `text_spec_from_dict`. Routing
        // between them is a `match` in the panel that raises this, not a
        // subtype string compared here.
        A::SetTextAnnotStyle { id, style } => set_text_annot_style(doc, id, &style),
    }
}

#[cfg(test)]
mod tests {
    //! `OPERATOR_REQUESTS.md` O161: a redaction mark selected on the canvas
    //! and deleted leaves through `delete_redaction_mark`, the same verb the
    //! Redact panel's Remove uses.

    use super::*;
    use crate::app::state::open_local_fixture;
    use pdfcer_core::annot::page_annotations;
    use pdfcer_core::annot_author::{Quad, RedactSpec};
    use pdfcer_core::page_tree::Rect;
    use pdfcer_core::vartext::Quadding;

    fn marks_on_page_0(doc: &OpenDoc) -> Vec<ObjId> {
        page_annotations(&doc.session.graph(), doc.pages[0].id)
            .into_iter()
            .filter(|a| a.subtype.as_slice() == b"Redact")
            .filter_map(|a| a.id)
            .collect()
    }

    /// The general `delete_annotation` refuses a `/Redact` by name, so
    /// routing this through it would leave the mark on the page with no
    /// sentence.
    #[test]
    fn deleting_a_selected_redaction_mark_unmarks_it() {
        let mut doc = open_local_fixture("four-pages.pdf");
        let spec = RedactSpec {
            quads: vec![Quad::from_rect(Rect {
                llx: 100.0,
                lly: 100.0,
                urx: 200.0,
                ury: 120.0,
            })],
            fill: None,
            overlay_text: None,
            quadding: Quadding::Left,
        };
        super::super::apply::vector_edit(&mut doc, "mark", 0, 1, |session| {
            session.add_redaction(0, &spec).map(|_| Vec::new())
        });
        let marks = marks_on_page_0(&doc);
        assert_eq!(marks.len(), 1);
        // And the mark is something the canvas will SELECT — the whole route
        // is click-the-mark, press Delete, and the first half is the one a
        // verb-level test cannot see.
        {
            let view = doc.session.view();
            let candidates = crate::canvas::selection::annot::selectable_on(
                &view,
                &doc.pages[0],
                0,
                &std::collections::BTreeSet::new(),
                &std::collections::BTreeMap::new(),
            );
            assert!(
                candidates
                    .iter()
                    .any(|c| c.target.id == marks[0] && c.target.subtype == "Redact"),
                "a redaction mark must be a selection candidate on the canvas: {candidates:?}"
            );
        }
        delete(&mut doc, 0, marks[0]);
        assert!(
            marks_on_page_0(&doc).is_empty(),
            "Delete on a selected mark must take the mark off"
        );
        assert!(doc.session.can_undo(), "and it is one undo step");
    }
}
