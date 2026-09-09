//! # `app::actions::annots::inknodes` — **the three point verbs of a freehand
//! mark**, and the one body behind them
//!
//! Split out of [`super`] under **R2** on 2026-09-09, the day it was written:
//! `annots.rs` stood at 1,458 of its 1,500 lines when `pdfcer-core`
//! `Pass 278.0` (`c8a6697`) shipped `EditSession::reshape_ink`, and the
//! apply-side of consuming it is two hundred lines of function and argument.
//!
//! ## ★★ The seam is a VERB FAMILY, not a line count
//!
//! [`super::reshape`] is the one body behind the three `/Vertices` verbs
//! (`move_node`, `insert_node`, `remove_node`) and it reaches
//! `EditSession::reshape_annotation`. [`reshape_ink`] is the one body behind
//! the three `/InkList` verbs and it reaches `EditSession::reshape_ink`. They
//! are twins — same funnel, same `/M` stamp, same disclosure list — and they
//! are **two functions rather than one with a branch** for the reason the
//! engine gave when it declined to widen `VertexEdit` with an optional stroke
//! index: *"a sentence about `/Ink` appearing in code that has nothing to do
//! with it."* A `match` on the edit's family inside `reshape` would have put
//! the ink forecast's fields (`stroke`, `stroke_points_before`,
//! `appearance_was_pdfces`) into a trace line about polygons, and the polygon
//! forecast's (`measure_not_recomputed`) into one about ink.
//!
//! ⇒ So the boundary is the same one `canvas::annotnodes::Plan` draws on the
//! canvas side: `Plan::Vertex` arrives here as `AnnotAction::{MoveNode,
//! InsertNode, RemoveNode}` and goes to [`super::reshape`]; `Plan::Ink`
//! arrives as `AnnotAction::{MoveInkPoint, InsertInkPoint, RemoveInkPoint}`
//! and comes here. Neither side converts an address; the `(stroke, point)`
//! pair the canvas planned is the pair the engine is handed.
//!
//! ## ★ Why [`apply`] exists, and the one catch-all in it
//!
//! `annots.rs` stood at 1,506 lines with three destructuring arms for these
//! verbs — rustfmt lays a five-field struct pattern vertically — and R2's
//! limit is 1,500. So [`super::apply_action`] has **one** arm for the three,
//! an or-pattern binding nothing, and [`apply`] destructures them here. The
//! `_` arm that leaves is reachable only by a caller that widened the
//! or-pattern without adding a case here, which the or-pattern's own comment
//! forbids; it traces rather than panics, for the reason every other "cannot
//! happen" in an apply arm does — a silent no-op on one release is recoverable
//! and a crash mid-edit is not. `dispatch::markupnodes` accepts the identical
//! shape for the identical reason.
//!
//! ## The engine types this module consumes, by name
//!
//! `pdfcer_core::edit::InkReshape` is what `reshape_ink` returns: its
//! `forecast` (a `pdfcer_core::edit::InkForecast`, identical to what the
//! preview answered), its `appearance`, its `dropped` list and
//! `mod_date_written`. `pdfcer_core::edit::InkEditKind` rides in the forecast's
//! `edit` field and in `CommandKind::ReshapeInk`'s undo label — six variants,
//! `InkEditKind::PointMoved`, `InkEditKind::PointInserted`,
//! `InkEditKind::PointRemoved`, `InkEditKind::StrokeReplaced`,
//! `InkEditKind::StrokeMoved`, `InkEditKind::StrokeRemoved` — of which this
//! shell raises the first three. The trace line prints it through
//! `InkEditKind::as_str` rather than `{:?}`, so it reads `move-point` exactly
//! as `pdfcer ink-edit --op move-point` does. `InkEditKind::changes_stroke_count`
//! is not read: the trace carries `strokes=before->after` outright, which says
//! the same thing as a number a wrong build would get wrong.
//!
//! ## The operator's report
//!
//! > *"the draw a line that follows the pointer tool — I can't edit the nodes
//! > that make it"* (O158, 2026-09-08)
//!
//! The engine's reply, in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`
//! (`reply_2026-09-09-ink-nodes-are-editable-SHIPPED-…`), is the source for
//! every fact this module states about the verb: that it re-bakes the
//! appearance, that pdfcer bakes an `/InkList` as a polyline, that `/Rect` is
//! derived rather than preserved, and that `appearance_was_pdfces == false`
//! is *"the one disclosure you should not drop"*.

use pdfcer_core::object::ObjId;

use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;

/// **Route one of the three ink point actions to its body.**
///
/// Called from [`super::apply_action`]'s single or-pattern arm over
/// `MoveInkPoint | InsertInkPoint | RemoveInkPoint`, and from nowhere else. The
/// module header carries why the routing is split across two files and what
/// the `_` arm means.
pub(super) fn apply(doc: &mut OpenDoc, action: AnnotAction) {
    match action {
        AnnotAction::MoveInkPoint {
            id,
            stroke,
            point,
            dx,
            dy,
        } => move_ink_point(doc, id, stroke, point, dx, dy),
        AnnotAction::InsertInkPoint {
            id,
            stroke,
            after,
            at,
        } => insert_ink_point(doc, id, stroke, after, at),
        AnnotAction::RemoveInkPoint { id, stroke, point } => {
            remove_ink_point(doc, id, stroke, point);
        }
        other => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // Reachable only if `apply_action`'s or-pattern is widened without
            // a case being added above. Traced, not panicked: see the header.
            format!("ink-node-misrouted action={other:?}")
        }),
    }
}

/// **Apply one point edit to a freehand mark**, as one undoable command —
/// `EditSession::reshape_ink` (`pdfcer-core` `Pass 278.0`, `c8a6697`; the
/// operator's O158, *"the draw a line that follows the pointer tool — I can't
/// edit the nodes that make it"*).
///
/// The one body behind [`move_ink_point`], [`insert_ink_point`] and
/// [`remove_ink_point`], and [`super::reshape`]'s twin for the second verb
/// family: same funnel, same `/M` stamp from `app::clock::pdf_date_utc` (the
/// engine's wrappers pass `modified: None` and *"leave `/M` exactly as it was
/// and say so"*), same disclosure list.
///
/// # ★★ What it discloses, and the one sentence that is new
///
/// | condition | sentence | why a canvas cannot say it |
/// |---|---|---|
/// | `forecast.appearance_was_pdfces == false` | [`crate::text::markup::ink_redrawn_straight`] | the stroke was another producer's artwork; re-baking replaced a smoothed curve with pdfcer's straight segments — the geometry moved as asked and the *look* changed more than the drag explains |
/// | `dropped` is non-empty | `markup_dropped`, as for a polygon | the re-baked appearance *looks* right; what went is what pdfcer could not reproduce |
///
/// The first is the disclosure the engine's reply said not to drop, and said
/// why it is a *sentence*: the geometry has moved, so carrying the old
/// appearance is impossible — *"it would paint the stroke where it no longer
/// is"*. It goes on the status line, off-canvas, exactly as `measure_stale`
/// does for a polygon; nothing on the canvas is tinted or badged (R8b rule 4).
/// No `measure_stale` here: an `/Ink` carries no `/Measure`, and the ink
/// forecast has no such field to read.
///
/// # ★ Why `reshape_ink` and not the three wrappers
///
/// [`super::reshape`]'s own argument, unchanged: the wrappers pass
/// `modified: None` and can never stamp `/M`; this shell knows the time and
/// the engine reads no clock, on purpose. `EditSession::move_ink_point`,
/// `insert_ink_point` and `remove_ink_point` are therefore named here and
/// called nowhere.
///
/// # ★ The refusal is not caught here
///
/// `canvas::annotnodes` asks `reshape_ink_preview` on **every frame** of the
/// drag — it shares `ink_plan` with this verb — so a release that reaches this
/// function is one the engine already said yes to. `vector_edit`'s own worded
/// floor covers the impossible case.
///
/// # The trace line
///
/// `ink-reshape-applied id=… edit=… stroke=… points=before→after
/// stroke_points=before→after strokes=before→after rect=before→after
/// was_pdfces=… ap=… dropped=… m=…`, and the first token is deliberately
/// **not** one of the three funnel labels (`move-ink-point`,
/// `insert-ink-point`, `remove-ink-point`) — `tools/gates/check-trace-names.py`
/// carries the three days that rule cost. `stroke=` and the per-stroke counts
/// are what a wrong build gets wrong invisibly: an insert that landed in the
/// neighbouring stroke and a correct one both report one more point in total,
/// and only the stroke index with its before/after says which. `rect=` is the
/// one number a driven check can compare against pixels — an ink `/Rect` is
/// **derived** by the engine from the new geometry, so a run where the points
/// changed and this pair did not is the shape of a half-written edit.
fn reshape_ink(doc: &mut OpenDoc, id: ObjId, edit: &pdfcer_core::edit::InkEdit, label: &str) {
    let modified = crate::app::clock::pdf_date_utc();
    crate::app::actions::apply::vector_edit(doc, label, 0, 1, |session| {
        session
            .reshape_ink(id, edit, modified.as_deref())
            .map(|outcome| {
                let f = outcome.forecast;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    let before = f
                        .rect_before
                        .map_or_else(|| "none".to_owned(), |r| format!("{r:?}"));
                    format!(
                        "ink-reshape-applied id={} edit={} stroke={} points={}->{} \
                         stroke_points={}->{} strokes={}->{} rect={before}->{:?} \
                         was_pdfces={} ap={:?} dropped={} m={}",
                        f.annot_id.num,
                        // ★ `as_str`, the engine's own stable token — `move-point`,
                        // `insert-point`, `remove-point` — so this line and
                        // `pdfcer ink-edit --op …` spell one fact one way.
                        f.edit.as_str(),
                        f.stroke,
                        f.points_before,
                        f.points_after,
                        f.stroke_points_before,
                        f.stroke_points_after,
                        f.strokes_before,
                        f.strokes_after,
                        f.rect_after,
                        u8::from(f.appearance_was_pdfces),
                        outcome.appearance,
                        outcome.dropped.len(),
                        outcome.mod_date_written
                    )
                });
                let mut notes = Vec::new();
                // ★★ THE disclosure the engine said not to drop. `false` means
                // the appearance on disk was another producer's and has just
                // been replaced by pdfcer's polyline rendering. Said here, on
                // the release, rather than drawn anywhere on the canvas.
                if !f.appearance_was_pdfces {
                    notes.push(crate::text::markup::ink_redrawn_straight().to_owned());
                }
                // Same catalog as a polygon's reshape, because it is the same
                // loss from the same bake — see [`super::reshape`].
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

/// **Move one point of one stroke of a freehand mark.**
/// `EditSession::move_ink_point`, reached through [`reshape_ink`] so the `/M`
/// stamp and the disclosures are one rule rather than three copies of one.
pub(super) fn move_ink_point(
    doc: &mut OpenDoc,
    id: ObjId,
    stroke: usize,
    point: usize,
    dx: f64,
    dy: f64,
) {
    reshape_ink(
        doc,
        id,
        &pdfcer_core::edit::InkEdit::MovePoint {
            stroke,
            point,
            dx,
            dy,
        },
        "move-ink-point",
    );
}

/// **Add a point to one stroke immediately after `after`**, at `at`.
/// `EditSession::insert_ink_point`. After a stroke's last point this extends
/// the stroke — the engine's rule, and the canvas offers no other reading.
pub(super) fn insert_ink_point(
    doc: &mut OpenDoc,
    id: ObjId,
    stroke: usize,
    after: usize,
    at: pdfcer_core::vector::Point,
) {
    reshape_ink(
        doc,
        id,
        &pdfcer_core::edit::InkEdit::InsertPoint { stroke, after, at },
        "insert-ink-point",
    );
}

/// **Take one point out of one stroke.** `EditSession::remove_ink_point`;
/// the engine keeps two per stroke.
pub(super) fn remove_ink_point(doc: &mut OpenDoc, id: ObjId, stroke: usize, point: usize) {
    reshape_ink(
        doc,
        id,
        &pdfcer_core::edit::InkEdit::RemovePoint { stroke, point },
        "remove-ink-point",
    );
}
