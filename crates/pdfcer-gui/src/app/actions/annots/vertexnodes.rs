//! **A markup shape's vertex verbs** — `move_node`, `insert_node` and
//! `remove_node`, one body (`reshape`) behind all three, over
//! `EditSession::reshape_annotation`.
//!
//! Split out of `annots.rs` on 2026-09-09 under R2, when the stamp-resize and
//! rotated-page work pushed that file to 1,515 lines. The seam is the one
//! `inknodes.rs` drew the same morning for `/Ink`: one file per geometry
//! family, each calling its own engine verb through the shared `vector_edit`
//! wrapper. `/Polygon`, `/PolyLine` and `/Line` are addressed by a flat
//! vertex index (`VertexEdit`); `/Ink` by `(stroke, point)` (`InkEdit`) — two
//! files because they are two index spaces, which is also the engine's
//! argument for two verbs.
//!
//! Nothing here decides whether a node may move — `canvas::annotnodes` asks
//! `reshape_annotation_preview` on every frame of the drag, so a release that
//! reaches these functions is one the engine already said yes to.

use super::*;

/// The one body behind [`move_node`], [`insert_node`] and [`remove_node`].
///
/// # ★★ What it discloses, and why both sentences are needed
///
/// | condition | sentence | why a canvas cannot say it |
/// |---|---|---|
/// | `measure_not_recomputed` | [`crate::text::markup::measure_stale`] | the number is baked into an appearance the shape still draws |
/// | `dropped` is non-empty | [`crate::text::dropped::only_the_first`]-style listing, through `markup::dropped_properties` | the re-baked appearance *looks* right; what went is what pdfcer could not reproduce |
///
/// The first is the one the engine went out of its way to give us. Acrobat
/// recomputes a `/Measure` number on a reshape and — a sourced user complaint —
/// silently clobbers a manual override doing it. pdfcer does neither, so the
/// geometry moves and the text does not, and **only a sentence can say so**.
///
/// # ★ The refusal is not caught here
///
/// `canvas::annotnodes` asks `reshape_annotation_preview` on **every frame** of
/// the drag, so a release that reaches this function is one the engine already
/// said yes to. A refusal arriving here would mean the document changed between
/// the last preview frame and the release — which cannot happen inside one
/// frame's `Vec<Action>` — and `vector_edit`'s own worded floor covers it.
fn reshape(doc: &mut OpenDoc, id: ObjId, edit: pdfcer_core::edit::VertexEdit, label: &str) {
    let modified = crate::app::clock::pdf_date_utc();
    super::super::apply::vector_edit(doc, label, 0, 1, |session| {
        session
            .reshape_annotation(id, edit, modified.as_deref())
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ `nodes=` carries BEFORE→AFTER rather than a single
                    // count, because that pair is what a wrong build gets
                    // wrong invisibly: an insert that landed on the wrong
                    // segment and a correct one both report one more node,
                    // and only the before-and-after together with the index
                    // in the shell's own line say which happened.
                    // ★★ `rect=` carries the annotation's `/Rect` BEFORE and
                    // AFTER, and it is the one number in this line a driven
                    // check can compare against pixels. A reshape rewrites
                    // three things — the geometry array, the `/Rect` and the
                    // baked `/AP` — and the engine's own note is that a shell
                    // which wrote only some of them looks correct in every
                    // renderer and is wrong in the next tool that rebuilds the
                    // appearance. The `/Rect` is the half that MOVES the
                    // painted result, so a run where the nodes changed and this
                    // pair did not is the shape of that defect, visible in one
                    // line.
                    //
                    // `rect_before` is an `Option` because a malformed
                    // annotation may carry no `/Rect` at all; the engine
                    // surfaces that rather than repairing it, and so does this.
                    let before = outcome
                        .rect_before
                        .map_or_else(|| "none".to_owned(), |r| format!("{r:?}"));
                    format!(
                        "{label}-applied id={} subtype={} edit={} nodes={}->{} \
                         rect={before}->{:?} dropped={} measure_stale={} m={}",
                        id.num,
                        outcome.subtype,
                        // ★ `as_str` and not `{:?}`: the engine spells these
                        // "move" / "insert" / "remove" itself, and a trace that
                        // read `Moved` while `pdfcer annotation-vertex` printed
                        // `move` would be two vocabularies for one fact.
                        outcome.edit.as_str(),
                        outcome.vertices_before,
                        outcome.vertices_after,
                        outcome.rect_after,
                        outcome.dropped.len(),
                        outcome.measure_not_recomputed,
                        outcome.mod_date_written
                    )
                });
                let mut notes = Vec::new();
                if outcome.measure_not_recomputed {
                    notes.push(crate::text::markup::measure_stale().to_owned());
                }
                // ★ Carried into the disclosure list rather than discarded, on
                // `SetMarkupStyle`'s stated rule: a reshape RE-BAKES the
                // appearance, and re-baking loses anything the original
                // expressed outside the model pdfcer draws — a border effect it
                // does not author, a producer's own decoration. The dictionary
                // key survives and the picture does not, so the canvas cannot
                // show what went. Same catalog as the restyle's, because it is
                // the same loss from the same bake.
                notes.extend(
                    outcome
                        .dropped
                        .iter()
                        .map(|d| crate::text::panels::properties::markup_dropped(*d).to_owned()),
                );
                notes
            })
    });
}

/// **Move one node of a markup shape.** `EditSession::move_annotation_vertex`,
/// reached through [`reshape`] so the `/M` stamp and the disclosures are one
/// rule rather than three copies of one.
pub(super) fn move_node(doc: &mut OpenDoc, id: ObjId, index: usize, dx: f64, dy: f64) {
    reshape(
        doc,
        id,
        pdfcer_core::edit::VertexEdit::Move { index, dx, dy },
        "move-annotation-vertex",
    );
}

/// **Add a node immediately after `after`**, at `at`.
/// `EditSession::insert_annotation_vertex`.
pub(super) fn insert_node(
    doc: &mut OpenDoc,
    id: ObjId,
    after: usize,
    at: pdfcer_core::vector::Point,
) {
    reshape(
        doc,
        id,
        pdfcer_core::edit::VertexEdit::Insert { after, at },
        "insert-annotation-vertex",
    );
}

/// **Take a node away.** `EditSession::remove_annotation_vertex`.
pub(super) fn remove_node(doc: &mut OpenDoc, id: ObjId, index: usize) {
    reshape(
        doc,
        id,
        pdfcer_core::edit::VertexEdit::Remove { index },
        "remove-annotation-vertex",
    );
}
