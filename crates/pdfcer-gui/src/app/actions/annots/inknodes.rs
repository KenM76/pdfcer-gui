//! # `app::actions::annots::inknodes` — **the three point verbs of a freehand
//! mark**, and the one body behind them
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/annots/inknodes.md`.

use pdfcer_core::object::ObjId;

use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;

/// **Route one of the three ink point actions to its body.**
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
/// `EditSession::reshape_ink`, for the operator's O158, *"the draw a line that
/// follows the pointer tool — I can't edit the nodes that make it"*.
///
/// The one body behind [`move_ink_point`], [`insert_ink_point`] and
/// [`remove_ink_point`], and [`super::reshape`]'s twin for the second verb
/// family: same funnel, same `/M` stamp from `app::clock::pdf_date_utc` (the
/// engine's wrappers pass `modified: None` and *"leave `/M` exactly as it was
/// and say so"*), same disclosure list.
///
/// # What it discloses, and the one sentence that is new
///
/// | condition | sentence | why a canvas cannot say it |
/// |---|---|---|
/// | `forecast.appearance_was_pdfces == false` | [`crate::text::markup::ink_redrawn_straight`] | the stroke was another producer's artwork; re-baking replaced a smoothed curve with pdfcer's straight segments — the geometry moved as asked and the *look* changed more than the drag explains | (old-name-exempt: the engine's own field name)
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
/// # Why `reshape_ink` and not the three wrappers
///
/// [`super::reshape`]'s own argument, unchanged: the wrappers pass
/// `modified: None` and can never stamp `/M`; this shell knows the time and
/// the engine reads no clock, on purpose. `EditSession::move_ink_point`,
/// `insert_ink_point` and `remove_ink_point` are therefore named here and
/// called nowhere.
///
/// # The refusal is not caught here
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
/// was_pdfces=… ap=… dropped=… m=…`, and the first token is deliberately (old-name-exempt: the engine's own field name)
/// **not** one of the three funnel labels (`move-ink-point`,
/// `insert-ink-point`, `remove-ink-point`): a module line sharing its first
/// token with the funnel's makes `Trace::last(name)` return the funnel's,
/// which carries none of the keys this line exists to publish.
/// `tools/gates/check-trace-names.py` is what holds that.
/// `stroke=` and the per-stroke counts
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
                         was_pdfces={} ap={:?} dropped={} m={}", // old-name-exempt: the engine's own field name
                        f.annot_id.num,
                        // `as_str`, the engine's own stable token — `move-point`,
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
                        u8::from(f.appearance_was_pdfces), // old-name-exempt: the engine's own field name
                        outcome.appearance,
                        outcome.dropped.len(),
                        outcome.mod_date_written
                    )
                });
                let mut notes = Vec::new();
                // THE disclosure the engine said not to drop. `false` means
                // the appearance on disk was another producer's and has just
                // been replaced by pdfcer's polyline rendering. Said here, on
                // the release, rather than drawn anywhere on the canvas.
                // old-name-exempt: the engine's own field name, `appearance_was_pdfces`
                let foreign_stroke = !f.appearance_was_pdfces; // old-name-exempt: engine field
                if foreign_stroke {
                    // old-name-exempt: the engine's own field name
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
