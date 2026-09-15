//! # `panels::properties::geometry::annot` — the **annotation** arm of the
//! geometry section: X, Y, W, H and, since 2026-09-07, **Angle**
//!
//! ## Why this is a separate file
//!
//! R2, and the seam was already drawn in prose before it was drawn in the file
//! system. `geometry.rs` served two subjects — a page-content object and a
//! markup annotation — behind one heading and one draft type, and its own
//! header describes them as *"two engine verbs that take different shapes"*.
//! The shared parts (the draft, the seed, the arithmetic, [`super::plan`] and
//! [`super::annot_plan`]) stay in the parent, where both arms can reach them
//! and where their tests already live. What moved is only the half that talks
//! to `EditSession`'s **annotation** verbs.
//!
//! ⇒ The split is deliberately NOT "UI here, arithmetic there". The parent
//! keeps `annot_plan`, because the whole reason that function is pure is so it
//! can be tested without a document, and moving it beside its caller would
//! have put it back in a file that needs `OpenDoc` to compile a test.
//!
//! ## What this arm does that the content arm does not
//!
//! | | content | annotation |
//! |---|---|---|
//! | move | `move_nodes` on the object's anchors | `move_annotation(id, dx, dy)` |
//! | resize | `resizing::action` factors | `resize_annotation(id, anchor, sx, sy, opts)` |
//! | **turn** | — | `rotate_annotation(id, pivot, degrees)` |
//! | lock | no such thing in a content stream | §12.5.3 bit 8, and every field is greyed |
//!
//! The **turn** row is the one this file is newest for, and the operator asked
//! for it in as many words: *"the angle should be editable from the
//! properties."* Read [`super::GeometryDraft::angle_delta`] before changing
//! anything about it — the field is absolute, the verb is a delta, and the
//! conversion has a normalisation in it that is not decoration.

use egui::Ui;
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::canvas::selection::AnnotKind;
use crate::text::panels::annotgeometry as at;
use crate::text::panels::properties as t;

use super::{
    ANGLE_REGION, ANNOT_APPLY_REGION, ANNOT_REGION, ANNOT_WIDTH_REGION, Bounds, GeometryDraft,
    Subject, annot_plan, field,
};

/// **A selected annotation's `/Rect`, normalised, in PDF user space.**
///
/// `None` when the annotation is not among the page's — reachable after an undo
/// or an external reload has removed it while the selection still names it —
/// and when it carries no `/Rect`, which `EditSession` refuses by name
/// (`EditError::AnnotationRectMissing`) rather than inventing one.
///
/// # ★★★ Read from the DOCUMENT, not from the selection's `outline`
///
/// [`AnnotSelection::outline`](crate::canvas::selection::AnnotSelection::outline)
/// is right there and is the wrong number. It is in **canvas space** — Y down
/// from the page's top-left, with `/Rotate` applied and the crop box's origin
/// subtracted — and getting back to PDF user space from it means running
/// `viewer::canvas_to_pdf_space` twice and through `f32`.
///
/// `canvas::mapping`'s header calls a second conversion *the classic silent
/// defect*, and here it would be worse than usually: the fields would show the
/// number that came back from a round trip through two transforms, the operator
/// would type `40.00`, and on a rotated page the value written into `/Rect`
/// would be neither what they typed nor what they saw. Reading the dictionary
/// is one hop and no convention.
///
/// # ★★ Normalised, because §7.9.5 does not require a `/Rect` to be
///
/// A rectangle may legitimately be written with its *upper-right* corner first,
/// and producers do it. `min`/`max` on both axes is what makes "Left" mean the
/// left edge rather than "whichever X the file happened to write first" — and
/// without it a width would come out negative, which
/// `resize_annotation` would divide by and turn into a mirror.
///
/// ★ [`crate::canvas::annotclip::rect_centre_of`] gets the same fact right by a
/// different route (it averages the pair, which needs no normalisation) and
/// says so; the two agree because both are reading §7.9.5 rather than a habit.
///
/// # Cost
///
/// One `/Annots` walk per frame, bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`. The same price the clipboard's
/// deleted `carried_options` paid for the same reason —
/// there is no public verb that models one annotation dictionary — and the same
/// order as the content arm's `doc.page_objects()`, which is also per frame.
pub(super) fn bounds_of(doc: &OpenDoc, page: usize, id: ObjId) -> Option<Bounds> {
    let graph = doc.session.graph();
    let page_ref = doc.pages.get(page)?;
    let annot = pdfcer_core::annot::page_annotations(&graph, page_ref.id)
        .into_iter()
        .find(|a| a.id == Some(id))?;
    let rect = annot.rect?;
    Some(Bounds {
        x0: rect.llx.min(rect.urx),
        y0: rect.lly.min(rect.ury),
        x1: rect.llx.max(rect.urx),
        y1: rect.lly.max(rect.ury),
    })
}

/// **Draw the four fields over a selected markup annotation**, returning
/// whether anything was drawn.
///
/// # ★★★ The three refusals, and why each takes the surface it takes
///
/// | condition | surface | why |
/// |---|---|---|
/// | a **ce dimension** | draws nothing, returns `false` | it is not this section's subject at all — [`super::dimension`] owns it, and both engine verbs refuse it **by name** |
/// | the annotation is **gone** or has no `/Rect` | draws nothing, returns `false` | there is no number to show; four spinners over `0.0` would be an invitation to place a mark at the sheet's corner |
/// | `/F` bit 8 — **locked** | draws the fields and Apply, **greyed**, with [`crate::text::panels::annotgeometry::locked`] on hover | R9's reserved case exactly: the capability is present and this annotation is out of bounds, so selecting a different one restores it |
///
/// ★★ **The ce-dimension guard is an [`AnnotKind`] match, never a `/Subtype`
/// string comparison**, and that is rule 15 made mechanical. A ce dimension IS
/// a `/Line` — `/IT /LineDimension` — so `subtype == "Line"` reads `true` for a
/// dimension and for a plain arrow alike, and routing a measurement into
/// `resize_annotation` would scale its rectangle and its baked appearance and
/// leave the sidecar geometry the displayed number is derived from where it
/// was. The mark would then say `1250 mm` about a line that is 900 long.
/// `canvas::selection::annot::AnnotKind`'s header states why it is an enum:
/// *a bool is a fact a caller may forget to read; a variant is one the compiler
/// makes them handle.*
///
/// ★ The engine would in fact catch it — `move_annotation` returns
/// `AnnotationMoveWrongVerb` naming `move_dimension` — so this guard is not the
/// last line of defence. It is the one that keeps the shell from **offering**
/// the affordance, which is R83: a control that can only produce a refusal is
/// not drawn.
///
/// # ★★ The foreign-appearance refusal is NOT guarded here
///
/// `resize_annotation` refuses a non-uniform scale over an `/AP` pdfcer did not
/// draw, unless `allow_appearance_distortion` is set. That condition cannot be
/// evaluated without rebuilding the appearance and comparing bytes, so there is
/// nothing honest to grey. It is surfaced **after** the press, by name, because
/// the action raised here is the same [`AnnotAction::Resize`] the eight grips
/// raise and `app::actions::annots::resize` already catches that error and
/// records `decline::record_resize_not_rebuildable`. A typed Width that the
/// engine declines therefore says exactly what a dragged one says.
/// `crate::text::panels::annotgeometry`'s header carries the whole argument.
pub(super) fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    target: &crate::canvas::selection::AnnotTarget,
    draft: &mut GeometryDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // ★ Exhaustive, so a third `AnnotKind` fails to compile here rather than
    // falling into whichever arm was written first. That is the same property
    // `annotclip::translated` buys with its exhaustive `MarkupSpec` match and
    // for the same reason: the failure of a wildcard is silent.
    match target.kind {
        AnnotKind::Markup => {}
        AnnotKind::CeDimension => return false,
    }
    let page = target.page;
    let Some(bounds) = bounds_of(doc, page, target.id) else {
        return false;
    };
    // ★★★ **The angle, read from the appearance's own `/Matrix`** — O146.
    //
    // `canvas::annotquad` is a declared workaround (its header says what is
    // filed and carries the tripwire); the engine's read model has no rotation
    // field. `degrees` is `None` for an appearance that is a shear or a mirror,
    // and `oriented` is `None` for an annotation with no appearance at all —
    // both collapse to no field being drawn, which is R9's answer for a
    // capability that is genuinely absent rather than temporarily unavailable.
    let angle = doc
        .pages
        .get(page)
        .and_then(|p| crate::canvas::annotquad::oriented_by_id(&doc.session.view(), p, target.id))
        .and_then(|q| q.degrees);
    draft.sync(
        page,
        Subject::Annot(target.id),
        doc.edit_epoch,
        bounds,
        angle,
    );

    // No `.strong()` — R84 / DEFECTS.md D11.
    //
    // ★★ The SAME heading, the SAME units note and the SAME four labels the
    // content arm draws. The units note is the load-bearing one: it says
    // *"Points, measured to the bottom-left corner. Y increases upward"*, which
    // is true of a `/Rect` in exactly the terms it is true of a path's bounding
    // box, and a second sentence phrased for annotations would be a second
    // statement of one coordinate convention. Two statements of a convention is
    // how a panel ends up measuring Y from the top in one half.
    ui.label(t::geometry_heading());
    ui.label(egui::RichText::new(t::geometry_units_note()).small().weak());

    // ★★★ **Greyed, not hidden, and the reason is on the hover** — R9. The
    // fields themselves and not only Apply, because a live spinner over a
    // locked annotation would accept a scrub and then refuse to commit it,
    // which is the "accepts a value and discards it" control this whole section
    // was once withheld to avoid.
    let locked = target.locked.then(at::locked);
    field(ui, t::geometry_x(), &mut draft.x, None, locked);
    field(ui, t::geometry_y(), &mut draft.y, None, locked);
    field(
        ui,
        t::geometry_w(),
        &mut draft.w,
        Some(ANNOT_WIDTH_REGION),
        locked,
    );
    field(ui, t::geometry_h(), &mut draft.h, None, locked);

    // ★★★ **THE ANGLE** — `OPERATOR_REQUESTS.md` O146, 2026-09-07: *"the angle
    // should be editable from the properties."*
    //
    // ★★ Drawn only when the mark HAS one. `draft.angle` is `None` for an
    // annotation with no appearance stream and for one whose `/Matrix` is a
    // shear or a mirror, and in both cases this renders **nothing** — not a
    // greyed spinner. R9's line is that greying is for *temporarily*
    // unavailable, and neither of those is temporary: no amount of operator
    // action turns a sheared stamp into one with an angle.
    //
    // ⚠ **The unit note is separate from the label and sits under the field**,
    // not in the units note at the top of the section. That note is about
    // points and about Y being measured upward; folding a second convention
    // into it would make one sentence carry two coordinate systems, which is
    // the thing its own doc comment warns against.
    if let Some(angle) = draft.angle.as_mut() {
        field(ui, t::geometry_angle(), angle, Some(ANGLE_REGION), locked);
        ui.label(egui::RichText::new(t::geometry_angle_note()).small().weak());
    }

    let changed = draft.differs_from(bounds);
    let usable = draft.is_usable();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `annot-geometry-draft`, not `geometry-draft`: two subjects writing
        // one trace name would make `TraceLog::last("geometry-draft")` return
        // whichever arm drew most recently, and a driven check reading `bw=`
        // would silently be reading the other subject's box.
        format!(
            "annot-geometry-draft id={} x={:.2} y={:.2} w={:.2} h={:.2} \
             bx={:.2} by={:.2} bw={:.2} bh={:.2} \
             angle={} seedangle={} turn={} \
             locked={} changed={changed} usable={usable}",
            target.id.num,
            draft.x,
            draft.y,
            draft.w,
            draft.h,
            bounds.x0,
            bounds.y0,
            bounds.w(),
            bounds.h(),
            // ★★ `none` rather than an omitted token, for both of these. A
            // trace that simply left the field out when there is no angle would
            // be indistinguishable from a build that forgot to emit it, and
            // this project has already been bitten by a check reading a fossil
            // for exactly that reason. Three states, three spellings.
            draft
                .angle
                .map_or_else(|| "none".to_owned(), |a| format!("{a:.2}")),
            draft
                .seed_angle
                .map_or_else(|| "none".to_owned(), |a| format!("{a:.2}")),
            draft
                .angle_delta()
                .map_or_else(|| "none".to_owned(), |a| format!("{a:.2}")),
            target.locked
        )
    });

    // ★ The three reasons, in the order that makes the most specific one win.
    // Locked first because it is a fact about the file rather than about the
    // typing — an operator whose fields are dead needs to know the file said
    // so, not that they have not typed anything yet, and "type a different
    // number" over a locked mark is advice that cannot work.
    let enabled = !target.locked && changed && usable;
    let why = if target.locked {
        at::locked()
    } else if usable {
        t::geometry_nothing_typed()
    } else {
        t::geometry_too_small()
    };
    let response = ui
        .add_enabled(enabled, egui::Button::new(t::geometry_apply()))
        .on_disabled_hover_text(why);
    crate::diag::ui_rect_visible(ANNOT_APPLY_REGION, response.rect, ui.clip_rect());

    if response.clicked() {
        let plan = annot_plan(draft, bounds);
        // ★★★ THE MOVE FIRST. `resize_annotation`'s anchor is an ABSOLUTE
        // point, so the corner the operator pinned with Left and Bottom must
        // already be where they said before it is used as the fixed point.
        // Raising the resize first would anchor on a corner the annotation is
        // about to stop having, and the mark would end up somewhere neither
        // number described — the same trap `plan` states for the content arm,
        // sharper here because a factor tolerates a stale origin and a point
        // does not.
        if let Some((dx, dy)) = plan.translate {
            // ★★ A **delta**, which is what `move_annotation(id, dx, dy)`
            // takes. The field holds an absolute Left/Bottom, so the conversion
            // is `delta`'s subtraction and it happens exactly once, in the pure
            // function, rather than in this arm where it could not be tested
            // without a document.
            actions.push(Action::Annot(AnnotAction::Move {
                id: target.id,
                dx,
                dy,
            }));
        }
        if let Some((anchor, (sx, sy))) = plan.resize {
            actions.push(Action::Annot(AnnotAction::Resize {
                id: target.id,
                anchor,
                sx,
                sy,
                // ★★ Whether the two factors are equal, computed the same way
                // `canvas::resizing` computes it from a grip drag. The engine
                // asked for this by name — it reports what the operator's hand
                // did, and a uniform scale of a foreign appearance is always
                // safe where a non-uniform one is refused.
                //
                // ★ Typing `40` into Width and leaving Height alone is a
                // NON-uniform scale even though the operator touched one field.
                // That is correct and is the case the refusal exists for.
                uniform: (sx - sy).abs() <= f64::EPSILON,
                // ★★★ **The operator's Tool-row switches, read live** —
                // `OPERATOR_REQUESTS.md` O51. `AnnotAction::Resize::modifiers`
                // documents why a *drag* must carry them rather than let the
                // apply arm read them: the gesture completed frames before the
                // queue drained. A press of Apply has no such gap — the click
                // and the read are the same frame — so this is
                // `CommitTextAnnot`'s case rather than `CommitMarkup`'s, and
                // reading them here keeps one store rather than adding a second
                // copy in the draft.
                modifiers: crate::canvas::scaling::read(ui.ctx()),
            }));
        }
        // ★★★ **THE TURN, RAISED LAST**, after the move and the resize.
        //
        // The order matters for the same reason the move-before-resize order
        // does, and more sharply. `move_annotation` and `resize_annotation`
        // both work on `/Rect`, which §12.5.2 requires **upright** — so both
        // are describing an axis-aligned box, and both would be describing a
        // *different* axis-aligned box if the rotation had already been
        // composed. Turning last means the two extent verbs act on the
        // rectangle the operator was reading the numbers off.
        //
        // ★★ **ABSOLUTE since 2026-09-07 (afternoon), and that is the whole
        // point of the field.** It raised `AnnotAction::Rotate` — a delta,
        // computed here as `typed − seed` — for a few hours, because
        // `rotate_annotation` was the only verb that existed. `Pass 155.2`
        // shipped `set_annotation_rotation` the same day, in answer to this
        // shell's request, and the argument is in the engine's doc comment
        // verbatim: *"composing a typed value as a delta requires the shell to
        // already trust its own idea of the current angle, and the first time
        // those disagree the object silently ends up somewhere else."*
        //
        // ⇒ `GeometryDraft::angle_delta` survives as the *did the operator
        // touch this field?* predicate — it still has to answer that, and its
        // normalisation into (−180, 180] is still what stops a 350-over-10 from
        // being read as a 340° turn — but the number that travels is
        // `draft.angle`, absolute.
        //
        // ★ The pivot is the `/Rect`'s CENTRE, which is the same point
        // `Grip::Rotate.pivot` answers for the rotate handle. That is not a
        // coincidence to be maintained by hand: the two routes must turn a mark
        // about the same point, or typing `45` and dragging to 45° would leave
        // it in two different places and an operator who used both would find
        // the mark walking across the page.
        if draft.angle_delta().is_some()
            && let Some(degrees) = draft.angle
        {
            actions.push(Action::Annot(AnnotAction::SetRotation {
                id: target.id,
                pivot: (bounds.x0 + bounds.w() / 2.0, bounds.y0 + bounds.h() / 2.0),
                degrees,
            }));
        }
    }

    ui.separator();
    // Published at the END, for the reason the content arm's own publication
    // gives at length: before it draws, `min_rect` is empty and `max_rect` is
    // the *available* space, which in a scroll area is the remaining viewport
    // — a region that does not contain its own Apply button is not a region.
    crate::diag::ui_rect_visible(ANNOT_REGION, ui.min_rect(), ui.clip_rect());
    true
}
