//! # `app::actions::annots` — the verbs that change an annotation
//!
//! Split out of [`super::apply`] under **R2** on 2026-08-18, when annotation
//! selection landed and took that file past 1,500 lines. The seam is the one
//! [`super::pages`] already draws next door: *what class of thing does this
//! verb act on?* — pages there, annotations here, page **content** in `apply`.
//!
//! ## Why it is worth its own file today rather than when it is bigger
//!
//! Because it is about to get bigger, and for a reason that is already
//! scheduled. `EditSession::set_markup_style` shipped on 2026-08-18 —
//! colour, interior, width, opacity and arrowheads on an existing annotation,
//! keeping its object id — and the Format contextual tab is the surface for
//! it. Every one of those becomes a verb in here.
//!
//! ★ And each of them will carry the same routing obligation `delete` does
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

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

mod inknodes;
/// **The polygon/polyline node verbs** - `move_node`, `insert_node`, `remove_node`
/// over `reshape_annotation`. Split out 2026-09-09 under R2.
mod vertexnodes;
use vertexnodes::{insert_node, move_node, remove_node};

/// **Remove one annotation from the document.**
///
/// Reached from `format.delete` and from the canvas's Delete key, both only
/// while an annotation is selected.
///
/// # Why it goes through `vector_edit` like everything else
///
/// So the undo entry, the epoch bump, the cache invalidation and the
/// disclosure happen the one way they happen for every other document change.
/// The closure returns the disclosure list, which is where the **collateral**
/// goes: the operator named one annotation and the engine may legitimately
/// have removed or altered more — a `/Popup` companion (§12.5.6.14 is a
/// `shall`), replies orphaned, group members promoted.
///
/// # `page` is for the message, not for the verb
///
/// `delete_annotation` finds the annotation by id wherever it lives, and it
/// has to: a reply may sit on a different page from the comment it replies to,
/// so a page-scoped delete would miss it.
///
/// # ★ This is not redaction
///
/// It removes an entry from `/Annots`. It does not touch page content, and an
/// incremental save leaves the previous revision in the file.
/// `docs/core-api/03-capabilities.md` §3.4 states that rule, and
/// [`crate::text::markup::deleted_collateral`] observes it in the wording it
/// chooses — never "removed".
pub(super) fn delete(doc: &mut OpenDoc, page: usize, id: ObjId) {
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
///
/// Reached from `canvas::annotdrag` on the release of a drag, and from nothing
/// else.
///
/// # ★★★ The disclosure is about the half the canvas cannot show
///
/// A move writes `/Rect` *and* the absolute-coordinate geometry keys, and the
/// canvas renders from the appearance stream, so the operator sees the same
/// picture whether one half was written or both. There is therefore nothing to
/// disclose about the move having worked -- they can see that.
///
/// What they cannot see is the **pop-up left behind**. §12.5.6.14 makes a
/// pop-up a separate annotation with its own placement and leaves whether it
/// follows to the reader; `pdfcer-core` reports the object number and says the
/// decision is the shell's. This shell does not draw pop-ups at all, so one
/// stranded across the sheet is invisible here and visible in Acrobat.
///
/// ⇒ ★★ **That is Rule 4's surviving half exactly**: an inference or a
/// consequence the operator cannot see still owes an off-canvas report. Render
/// normally; report separately. Both.
///
/// # ★ What is deliberately NOT disclosed
///
/// **`geometry_keys_moved` being empty**, which the engine warns about by name:
/// a Text note, a Stamp or a Link has no geometry key because its `/Rect` *is*
/// its geometry, so empty is a correct answer and reporting it would manufacture
/// an anomaly out of the commonest case.
///
/// **`rect_differences_untouched`**, for a different reason: `/RD` holds inset
/// distances rather than coordinates, translating them would deform the
/// annotation, and not translating them is therefore not a limitation to
/// confess but the only correct behaviour. A sentence about it would teach an
/// operator to worry about something that is right.
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
///
/// ★★★ The disclosure is the operator's own ruling, carried through. He asked
/// for Inkscape's toggles — *"default should be what it said, but there should
/// be an option that they do scale with resize"* — and the sentence that
/// belongs beside a default is the one that says the default fired.
///
/// ★★ **`stroke_width: None` is the case that owes a sentence**, which is the
/// engine's own instruction: *"an operator who scaled a square 3× and expected
/// a heavier border needs telling it stayed."* That is Rule 4's surviving half
/// — a line weight left alone is invisible on the canvas, because the shape
/// grew around it and nothing says the border did not.
///
/// ★ **`CarriedDistorted` is the other one**, and it is not a defect: neither
/// PDF nor SVG has a per-axis stroke width, so a non-uniform scale of an
/// appearance pdfcer did not author produces an anisotropic border by
/// arithmetic. The engine refuses that case unless it is allowed; where it
/// proceeds, the operator is told.
pub(super) fn resize(
    doc: &mut OpenDoc,
    id: ObjId,
    anchor: (f64, f64),
    (sx, sy): (f64, f64),
    uniform: bool,
    modifiers: crate::canvas::scaling::Modifiers,
) {
    // ★★★ **THE OPERATOR'S SWITCHES, and they replaced a derivation.**
    //
    // Until 2026-08-28 this read `with_scale_stroke_width(uniform)` — the flag
    // taken from whether the drag was proportional rather than from anything
    // anybody asked for. That was a **workaround for a refusal**: with a
    // foreign appearance and a uniform scale the engine refuses unless either
    // the stroke scales or distortion is allowed, and forcing the first made
    // the common case work when no control existed.
    //
    // ⇒ It also made the operator's answer unreachable, on exactly the resizes
    // where they were most likely to have one. `OPERATOR_REQUESTS.md` **O51**
    // is a correction about precisely this shape of reasoning, so deriving the
    // flag from geometry after building the control would be making the same
    // mistake twice in one file.
    //
    // ★ What replaced the workaround is the worded decline below, not a
    // different guess.
    //
    // The discriminator behind the DEFAULTS is unchanged and is the engine's,
    // promoted from this shell's own CAD argument: *is the property a length in
    // the space being transformed?* An inset is; a line weight is a drafting
    // convention. `canvas::scaling` carries the whole account.
    // ★★★ A STAMP IS ARTWORK, AND SCALING ARTWORK IS THE RESIZE — 2026-09-09.
    //
    // The operator, twice in one morning: *"there's still no way to edit the
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
                // ★★★ **The refusal is caught here and worded**, rather than
                // being left to `vector_edit`'s generic arm, which traces the
                // engine's reason and — since O116, 2026-09-04 — words only
                // *"That change was refused, and the document is unchanged."*
                // That floor ends the silence; it cannot name a remedy, and
                // naming one is the whole value of catching the refusal here.
                //
                // A resize that silently did nothing is this project's founding
                // failure: the operator drags a grip, lets go, the shape snaps
                // back, and no surface anywhere says why. It is the same shape
                // as the annotation drag that was consumed and discarded, and
                // the same shape as the markup move that had no branch.
                //
                // ★★ Recorded from INSIDE the closure because the condition is
                // not knowable before the call — whether an appearance is
                // pdfcer's own is a property of the file. `record_save_failure`
                // is called from the apply phase for the identical reason;
                // `record_flatten_certified` is not, because its refusal is a
                // query.
                //
                // ★ Only this one variant. Every other `EditError` keeps
                // today's trace-only behaviour, which is honest: wording a
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
                // ★ And its sibling since `Pass 277.0` (2026-09-09): a `/Text`
                // sticky or a `NoZoom` annotation has no size to scale. Worded
                // because the Properties panel's geometry fields can raise this
                // resize even though the sticky's canvas grips are move-only.
                // `subtype == "Text"` is the engine's own test for which of its
                // two `why` sentences it chose (`edit.rs`, `resize_annotation`).
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

/// **Turn a markup annotation about a pivot.** `Pass 155.0`.
///
/// Reached from `canvas::rotating` on the release of a rotate-handle drag, and
/// from nothing else.
///
/// # ★★★ There is no options type, and its absence is the feature
///
/// [`resize`] one screen up takes `crate::canvas::scaling::Modifiers` — the
/// operator's Tool-row switches — because a resize has a genuine question to
/// ask: *does a line weight scale with the shape?* A rotation has no such
/// question, because **a rotation is an isometry**. Every length is preserved,
/// including the drawn stroke width, so there is nothing for a switch to
/// decide and no switch is offered.
///
/// `pdfcer-core` drew the consequence for this shell's grip UI in one line, and
/// it is the line that shaped this whole gesture: *"if your grip UI offers
/// rotate and resize together, **rotate needs no confirmation step and no
/// distortion warning.** Resize does."*
///
/// # ★★ And unlike [`resize`], a FOREIGN appearance turns correctly
///
/// `resize_annotation` has to refuse artwork pdfcer did not draw — §12.5.5's
/// placement matrix scales it *after* stroking, and no scalar `/BS /W`
/// describes an anisotropic stroke. That refusal is why [`resize`] carries a
/// worded decline for `ResizeAppearanceNotRebuildable` at all.
///
/// **Rotation has no equivalent**, and the reason is in the standard rather
/// than in an implementation choice: step (a) transforms the appearance `BBox`
/// through its **own** `/Matrix`, so pdfcer composes the rotation into the
/// matrix a producer already wrote. Nothing is redrawn and nobody's artwork is
/// replaced — it works on a stamp Acrobat made.
///
/// # ★★★ The disclosure is about the box, not about the mark
///
/// **`/Rect` grows.** §12.5.2 requires it upright, and the upright box bounding
/// a rotated rectangle is larger at any angle that is not a quarter turn. That
/// is correct behaviour and it is *invisible as such*: this shell draws its
/// selection outline **from `/Rect`**, so an operator turning a stamp 30°
/// watches a dashed box swell around artwork that did not change size.
///
/// ⇒ **That was Rule 4's surviving half, and it stopped applying on
/// 2026-09-07.** `canvas::annotquad` draws the outline at the mark's own angle
/// (`OPERATOR_REQUESTS.md` O147), so the box hugs the artwork and no longer
/// swells — the consequence had a subject and the subject is gone. The
/// disclosure `text::rotating::rect_grew` used to return here is deleted, and
/// the reason it must not be restored for the O145 growth defect is recorded at
/// its old site.
///
/// ⚠ **`/Rect` still grows.** That is correct and normative and is now simply
/// invisible, which is the right place for a fact an operator cannot act on.
///
/// # ★ What is deliberately NOT disclosed
///
/// **`rect_differences_untouched`** (`/RD`), for the reason [`move_annot`]
/// already gives about the same key: at an angle that is not a quarter turn
/// **no** axis-aligned inset expresses the rotated result, so pdfcer does not
/// invent one and leaving it alone is the only correct behaviour. A sentence
/// about it would teach an operator to worry about something that is right.
///
/// **`appearance_matrix_updated`** — that is *how* a rotation is expressed, not
/// a consequence of it. It goes in the trace, where implementation facts
/// belong, and it is the field a wrong build would get wrong.
pub(super) fn rotate(doc: &mut OpenDoc, id: ObjId, pivot: (f64, f64), degrees: f64) {
    super::apply::vector_edit(doc, "rotate-annotation", 0, 1, |session| {
        session
            .rotate_annotation(id, pivot, degrees)
            .inspect_err(|error| {
                // ★★★ **The refusal is caught here and worded**, rather than
                // being left to `vector_edit`'s generic arm, which since O116
                // words an un-categorised sentence naming no remedy.
                // [`resize`]'s own comment is the
                // argument and it applies unchanged: a grip that is dragged,
                // released, and does nothing with no explanation is this
                // project's founding defect.
                //
                // ★★ From INSIDE the closure, because none of these is knowable
                // before the call — whether a document's certification forbids
                // an annotation change is a census over its objects, and
                // whether the routing sent the wrong kind here is a fact about
                // the value the engine resolved.
                //
                // ★ **Every** `EditError` is worded, unlike [`resize`], which
                // words one variant and leaves the rest to the trace. The
                // difference is that a resize genuinely has six refusal shapes
                // with six remedies, and this verb has essentially none the
                // operator can act on — so a catch-all that says *the page is
                // exactly as it was* is honest here where a catch-all there
                // would have been a shrug.
                crate::app::status::decline::record_rotate(refusal_for(error));
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★★ It carries the ANGLE, the PIVOT and both rectangles,
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
                // ★★★ **NO DISCLOSURE, since 2026-09-07.** `rect_grew` used to
                // return a sentence here explaining that the dashed box had
                // swelled while the artwork had not. The outline is now drawn
                // at the mark's own angle (`canvas::annotquad`,
                // `OPERATOR_REQUESTS.md` O147), so it hugs the artwork and
                // there is no swelling box to explain. The full account,
                // including why it must not be restored for the O145 growth
                // defect, is at the deleted function's site in
                // `text::rotating`.
                Vec::new()
            })
    });
}

/// **The engine's rectangle rule as a stable single-word token**, for the trace.
///
/// # ★★ Why not `{:?}`
///
/// Because a `Debug` rendering is a formatting detail of somebody else's enum
/// and a driven check that greps for `rect_derived=Artwork` would go quiet the
/// day `RectDerivation` gains a field, gets renamed, or has its derive removed
/// — quietly, and in the direction that reads as *the feature stopped
/// happening*. This project has already shipped one check that reported the
/// opposite of the truth while quoting the truth in its own message, from a
/// `{:?}` on a tuple.
///
/// ★ The **match is exhaustive with no wildcard**, so a fourth rule is a
/// compile error here rather than an unnamed token in a log. `RectDerivation`
/// is `#[non_exhaustive]`, which is why the last arm exists at all and why it
/// says `other` — an honest *"this build does not know that one"* rather than a
/// guess.
///
/// The tokens match what the engine's own CLI prints (`rect_derived=`), so a
/// trace here and a `pdfcer` command line can be compared without a lookup
/// table.
const fn rect_rule_token(rule: pdfcer_core::edit::RectDerivation) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed in the UI.
    match rule {
        pdfcer_core::edit::RectDerivation::Artwork => "artwork",
        pdfcer_core::edit::RectDerivation::Geometry => "geometry",
        pdfcer_core::edit::RectDerivation::PreviousRect => "previous-rect",
        _ => "other",
    }
}

/// **Set a markup annotation's angle absolutely.** `Pass 155.2`.
///
/// Reached from the Properties panel's typed Angle field, and from nothing
/// else — the rotate grip is a drag and goes to [`rotate`], which is a delta.
///
/// # ★★★ Why this is a second function rather than an argument on [`rotate`]
///
/// Because they take different things and refuse for different reasons. A
/// delta needs no starting angle and therefore cannot fail to read one; an
/// absolute set **must** read the current angle and refuses by name when it
/// cannot (`AnnotationRotationUnreadable`). Folding them would mean one
/// function whose failure modes depend on a boolean, and a caller reading the
/// refusal would have to know which mode it was in to know what the sentence
/// meant.
///
/// # ★★ The disclosure is about the RECTANGLE RULE, not about the turn
///
/// `Pass 155.1` made `/Rect` a function of the artwork rather than of the
/// previous rectangle, so a rotation composes — *N* turns totalling θ now draw
/// the same size as one turn of θ. **With one exception the engine named and
/// this shell must honour:** an annotation with **neither an appearance stream
/// nor rotatable geometry** — a `/Square` or `/Circle` with no `/AP` — has
/// nowhere an orientation could be recorded, because its artwork *is* its
/// rectangle and §12.5.2 requires that upright. `RectDerivation::PreviousRect`
/// is that case, it **still grows on every turn**, and the engine's reply is
/// explicit that ignoring it *"re-introduces the operator's bug one level up,
/// on exactly the annotations that cannot be fixed."*
///
/// ⇒ [`crate::text::rotating::rect_still_grows`] is the sentence, and it fires
/// **only** on that rule. It is a far better disclosure than the one it
/// replaced: the old `rect_grew` fired on every non-quarter turn of everything,
/// which trained the operator to ignore it.
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
                    // ★★ `asked=` and `deg=` are BOTH carried and they are
                    // different numbers: the first is the absolute angle the
                    // operator typed, the second is the delta the engine worked
                    // out to get there. A build that passed the typed value
                    // through as a delta would show them equal on the first
                    // edit of an unturned mark — which is most of them — and
                    // diverge only on the second, so a trace carrying one of
                    // the two could not see it.
                    //
                    // ★ `rect_derived=` is `AnnotationRotate::rect_derived_from`
                    // — which of the engine's three rules produced the new
                    // rectangle. It is the field the whole of `Pass 155.1`
                    // turns on and the one an operator report would need: two
                    // of the three compose and one cannot, and nothing else on
                    // this line distinguishes them.
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

/// **Turn a ce dimension about a pivot.** `Pass 159.0`.
///
/// Reached from `canvas::rotating` on the release of a rotate-handle drag over
/// a selected ce dimension, and from nothing else.
///
/// # ★★★ Why this is a second function rather than a branch in [`rotate`]
///
/// Because `rotate_annotation` **refuses a ce dimension by name** and points
/// here, with its reason attached: *"a ce dimension's orientation is part of
/// its measurement, so turning it must re-measure rather than spin a
/// rectangle."*
///
/// A ce dimension is a `/Line` with `/IT /LineDimension` and a record in the
/// document's `/PieceInfo` sidecar. It passes every *"is this markup pdfcer can
/// author?"* test. Turning it as an annotation would rotate the `/Rect` and the
/// baked `/AP` and leave the **sidecar geometry** — the thing the displayed
/// number is derived from — exactly where it was, so the dimension would draw
/// at one angle and measure along another.
///
/// ⇒ This is the same routing obligation this module's header records for
/// `set_markup_style`, and `canvas::selection::annot::AnnotKind` carries the
/// distinction on the selected target precisely so the fork is a `match` the
/// compiler checks.
///
/// # ★★★ The measured value CANNOT change, and nothing says otherwise
///
/// A rotation preserves every distance, so the number is identical either side
/// of it **by construction** rather than because pdfcer holds it. The engine
/// therefore returns no before/after pair — deliberately, and it says why:
/// reporting *"5.000 m → 5.000 m"* would invite a reader to look for a change
/// that cannot exist.
///
/// ⇒ So there is no disclosure here saying the measurement is unchanged, and
/// there must not be. A live readout that does not move during the drag is
/// **correct, not a stale binding**.
///
/// # ★★★ The one disclosure, commissioned by the engine by name
///
/// A `Linear` dimension may be constrained to `Horizontal` or `Vertical`. Turn
/// it 30° and that constraint can no longer describe what is drawn. Three
/// options existed and two are wrong — refusing makes rotation impossible for
/// most of a CAD drawing; keeping the constraint leaves the line and its own
/// stated constraint disagreeing, invisibly, until something regenerates from
/// it. The engine relaxes to `Aligned` and reports `constraint_relaxed`, with
/// this instruction attached:
///
/// > **Say so**: an operator whose dimension silently stopped being axis-locked
/// > will find out later and blame something else.
///
/// [`crate::text::rotating::axis_lock_relaxed`] is that sentence. It fires only
/// when the flag is set — a rotation by a whole number of turns leaves the
/// constraint alone, because nothing moved.
///
/// # ★ Scaling a dimension is not here, and will not be
///
/// Not unbuilt — **declined**, by the engine and by the operator, on the ground
/// that it has no honest reading: either the displayed value stays fixed while
/// the geometry grows, so the dimension lies about the drawing, or both change,
/// so nothing was measured. The operation actually wanted is `set_group_scale`
/// — points per unit — which already ships on the Measure surface. That is why
/// `pressing::grabbable` hands a selected dimension `GripSet::rotate_only()`
/// rather than the full nine.
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
                    // ★ `relaxed=` is the field worth tracing and the one a
                    // wrong build gets wrong: a rotation that turned the
                    // geometry and left a `Horizontal` constraint behind
                    // produces a line and a constraint that disagree, which is
                    // invisible on the canvas and shows up the next time
                    // anything regenerates from the constraint.
                    //
                    // ★ `annot=` is carried purely so a failed run ties back to
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
///
/// # ★★★ One function over the two verbs, and it is the compiler's job to keep
/// it complete
///
/// Both `rotate_annotation` and `rotate_dimension` refuse from the same short
/// list, so a second copy of this mapping would be a second place for a variant
/// to be forgotten — and a forgotten variant here is a grip that is dragged,
/// released, and does nothing with no explanation.
///
/// # ★★ Why the fallback is a sentence rather than the error's `Display`
///
/// `tools/gates/check-ui-strings.sh`' exclusion 3 names the failure in as many
/// words: a `format!` of an `EditError` routes **diagnostic prose into the UI**.
/// [`crate::text::rotating::RotateRefusal::Other`] is a hand-written sentence
/// that says the one thing an operator needs about an unrecognised refusal —
/// *the page is exactly as it was* — and names no cause it does not know.
fn refusal_for(error: &pdfcer_core::edit::EditError) -> crate::text::rotating::RotateRefusal {
    use crate::text::rotating::RotateRefusal;
    match error {
        // ★ The routing backstop. Unreachable while `canvas::rotating`'s
        // `match` on `AnnotKind` holds and while `canvas::selection::annot`
        // keeps excluding `/Widget` — which is exactly why it is worded: if
        // this sentence ever appears, the routing has broken, and a broken
        // route with a sentence is a bug report rather than a dead handle.
        pdfcer_core::edit::EditError::AnnotationMoveWrongVerb { .. } => RotateRefusal::WrongVerb,
        // ★ The one an operator meets on an ordinary file and cannot guess at:
        // a signed drawing looks exactly like an unsigned one on the canvas.
        pdfcer_core::edit::EditError::CertificationForbidsChange { .. } => RotateRefusal::Certified,
        _ => RotateRefusal::Other,
    }
}

/// **Write the note on an annotation that already exists** — `/Contents`, and
/// conditionally `/T` and `/M` — as one undoable command.
///
/// Reached from the Comments panel's editor and from nothing else.
///
/// # ★★★ The three keys are not written as a group, and that is the contract
///
/// `pdfcer-core` leaves an **omitted** key untouched rather than clearing it,
/// and its reply to this shell called getting that wrong *"the easiest way to
/// get this wrong"*:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// So `author` is `None` on two quite different occasions and both must send
/// nothing: the annotation already has a byline that is not ours to move, or
/// the operator has left their name blank in Settings ▸ Comments, which is a
/// supported choice and means *comment anonymously*. `crate::app::actions::apply`
/// resolves which; this function only has to not invent one.
///
/// # ★★ `/M` is always written, and it is a modification date
///
/// §12.5.6.4 Table 170 defines `/M` as the date the annotation was **modified**,
/// and this call modifies it — so leaving it alone would leave a comment whose
/// date describes an earlier version of its own text. `crate::app::clock` is
/// the only place this shell reads a wall clock and its header carries the
/// whole argument for UTC; `None` there means the system clock is before 1970,
/// and omitting `/M` beats writing a comment dated 1969.
///
/// # ★ TWO disclosures, and they are about opposite things
///
/// **The words that are gone.** A note that replaced another one usually
/// leaves no trace on the canvas: the shape is unchanged, and a sticky's words
/// live in a pop-up window this shell does not draw.
/// `MarkupNoteChange::replaced` carries the previous text — the text, not a
/// count — precisely so the operator can be offered it back, which is what
/// `crate::text::markup::note_replaced` does.
///
/// ★ *"usually"* is doing work there, and it is the one subtype that breaks
/// the family: a `/FreeText`'s `/Contents` **is** its painted words, so on one
/// the replaced text was on the canvas, and as of `pdfcer-core` `95a936e` the
/// engine re-bakes the appearance in this same command so the page follows the
/// edit.
///
/// **The half that did not move.** Which is the second disclosure, and it fires
/// on exactly one shape of outcome — a `/FreeText` whose appearance pdfcer did
/// not author, which is preserved rather than replaced. See the call site below
/// and `crate::text::textannot`'s edit-time banner, which carries the four-row
/// table and the record of the disclosure that was deleted when the engine
/// closed its cause.
pub(super) fn set_note(doc: &mut OpenDoc, id: ObjId, text: &str, author: Option<&str>) {
    // Builders, not a struct literal: `MarkupNote` is `#[non_exhaustive]`,
    // which is what keeps a future field a non-breaking addition for us.
    let mut note = pdfcer_core::edit::MarkupNote::new(text);
    if let Some(author) = author.map(str::trim).filter(|a| !a.is_empty()) {
        note = note.by(author);
    }
    if let Some(stamp) = crate::app::clock::pdf_date_utc() {
        note = note.at(stamp);
    }
    super::apply::vector_edit(doc, "set-markup-note", 0, 1, |session| {
        session.set_markup_note(id, &note).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ `-applied`, per the convention `forms::import_data`
                // records: the funnel writes its own bare-named line for the
                // same edit and `.last()` would read that one instead.
                //
                // `keys` is the field worth tracing rather than the text: it is
                // the engine's own answer to "what actually moved", and the
                // whole `/T`-preservation contract above is invisible from a
                // screenshot and from the saved page alike.
                //
                // ★ `rebaked` is the ONLY oracle for the half of this edit a
                // screenshot of the panel cannot see. The status line speaks
                // for the `false`-on-a-`/FreeText` case alone (correctly — see
                // below), so without this field a driven check could not tell
                // a re-baked text box from a sticky note, which are the two
                // outcomes that look identical from outside.
                format!(
                    // ★ `rich_dropped` is a COUNT of the keys the engine
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
            // ★★★ The text box's disclosure goes FIRST, and the order is the
            // decision. `record_notes` documents the first sentence as the one
            // an operator reads if they read only one — and between "here are
            // the words you replaced" and "the page kept an appearance this
            // edit did not move", only the second is something they cannot
            // find out any other way.
            //
            // ★★ **Two gates, both the ENGINE's own answer**, and neither is
            // anything this shell inferred about the selection:
            //
            // 1. `MarkupNoteChange::subtype` — the raw `/Subtype`.
            // 2. `MarkupNoteChange::appearance_rebaked` — whether the picture
            //    moved with the words. `set_markup_note` re-bakes a
            //    `/FreeText`'s `/AP` itself as of `pdfcer-core` `95a936e`,
            //    which is why this is now a condition rather than a constant.
            //
            // `false` is **not a failure**: it is correct and final on a
            // sticky and on a stamp, and the sentence must not fire for them.
            // The four-row table, the deleted before-the-write hint and the
            // record of the morning this shell spent disclosing a defect the
            // engine closed by the afternoon are all at
            // `crate::text::textannot`'s edit-time banner.
            //
            // ★★★ …and the rich-text drop goes LAST of the three, because it
            // is the only one that is *good news*: the operator's document was
            // inconsistent before this edit and is consistent after it. The
            // first two are things they need in order to act — what the page
            // did not do, and what words they can retype. This one is a
            // statement that a problem they never knew about has been closed,
            // and it must not displace either of them from the truncated slot.
            //
            // ⚠ It is disclosed rather than swallowed because **pdfcer removed
            // a key the operator did not ask it to remove**, which is rule 4
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
///
/// Reached from the Comments panel's *Remove note* control and from nothing
/// else.
///
/// # ★★ It is not a delete, and the disclosure says so because nothing else can
///
/// The markup stays on the page with its geometry untouched. A shape with a
/// note and the same shape without one are **the same picture**, so an operator
/// who pressed the wrong button has no way to see either what they did or what
/// it cost them. `crate::text::markup::note_removed` states both — the words
/// that went, and the fact that the shape did not.
///
/// # ★ Why a separate verb from writing an empty note
///
/// `pdfcer-core`'s reason, adopted rather than re-derived: *"an empty comment is
/// a comment, and a reviewer deleting their remark is not the same as leaving a
/// blank one."* An empty `/Contents` beside a `/T` and an `/M` says somebody
/// wrote nothing; no `/Contents` at all says nobody wrote anything.
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
            // ★ On one it DID draw, `clear_markup_note` empties the painted
            // box in the same command — so `appearance_rebaked` is `true`,
            // nothing is left unsaid, and this stays quiet.
            //
            // ★★ **And the rich-text drop applies here too**, which is easy to
            // miss because *removing* a note sounds like it could not leave a
            // stale copy behind. It could: `/RC` is a second copy of the same
            // comment, so clearing `/Contents` without it would leave the
            // pop-up — and, on a `/FreeText`, the page — still showing words
            // the operator has just deleted.
            //
            // ⇒ Wired in both arms rather than only in the one the defect was
            // reported against. A disclosure attached to one of two paths
            // through the same engine report is how a surface comes to be
            // right on Tuesday and wrong on Wednesday.
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

/// ★★★ **Answer a comment** — `EditSession::add_reply`, `pdfcer-core`
/// `Pass 253.0`, §12.5.6.2 Table 170.
///
/// Reached from the Comments panel's editor when its draft is aimed at a reply,
/// and from nothing else. This shell filed *"we can read a comment thread and
/// cannot add to it"* on 2026-09-05 and carried it as an open gap until the
/// verb landed; this is the write half of a surface that had been listening the
/// whole time.
///
/// # ★★★ `/M` is OURS, and a reply without one is a note from nowhen
///
/// `pdfcer-core` reads no clock, by policy — determinism, and rule 4's refusal
/// to let a library invent a claim about when something happened
/// (`crate::app::clock`'s header carries the whole argument). So the stamp is
/// supplied here, from the crate's **one** wall-clock reader, exactly as
/// [`set_note`] supplies it. A reply that reached the file with no `/M` would
/// render in every reviewer UI in the class as a comment with a blank date
/// beside a parent that has one — which reads as a corrupted thread rather than
/// as a missing key.
///
/// ★ [`crate::app::clock::pdf_date_utc`] returns `None` before the Unix epoch,
/// and in that case no `/M` is sent rather than a plausible one being invented.
/// Absent is honest; wrong is not.
///
/// # ★★ The author is the operator's, always, with no `keep_author` question
///
/// `author` here is the same `Prefs::author_name` [`apply_action`] hands
/// [`set_note`], filtered by the same rule at the same seam — one source, so
/// the two surfaces cannot come to sign comments differently. What is *absent*
/// is `SetNote`'s `keep_author`, and the asymmetry is load-bearing: that verb
/// may be correcting somebody else's comment and must be able to leave their
/// `/T` alone, whereas a reply is a **new annotation this operator is
/// authoring** and has no prior byline to preserve. See
/// `AnnotAction::Reply`'s own docs.
///
/// A blank preference means *comment anonymously*, which is a supported choice
/// and not a missing value — so no `/T` is written, and no name is invented.
///
/// # ★★★ The disclosure: the reply's OWN `/Popup`, which nothing here draws
///
/// `add_reply` authors a `/Popup` companion for the reply (§12.5.6.14), and
/// `ReplyAdded::reply_has_popup` reports it because this shell asked to be
/// told:
///
/// > *"a reply that quietly acquired a second window at a second location is
/// > something we would rather be told about than discover on a screenshot."*
///
/// That question has a shell-side answer and it is deliberate:
/// `canvas::notepopup::model::notes_on` **excludes replies** from the notes it
/// draws windows for, so pdfcer shows a reply inside its parent's thread and
/// never as a second bubble on top of the comment it answers. See that
/// function's exclusion table for why the alternative is worse than untidy —
/// the reply is placed at the parent's own `/Rect`, so a shell that drew it as
/// an independent note would make the parent unclickable the moment anybody
/// answered it.
///
/// ⇒ The window still **exists in the file**, and another reader will draw it.
/// That is a fact about the document that no surface in this program can show,
/// which is precisely the test for what belongs in a disclosure —
/// [`crate::text::panels::comments::reply_posted`].
pub(super) fn add_reply(doc: &mut OpenDoc, parent: ObjId, text: &str, author: Option<&str>) {
    // Builders, not a struct literal: `MarkupNote` is `#[non_exhaustive]`, and
    // the same shape `set_note` uses two screens up.
    let mut note = pdfcer_core::edit::MarkupNote::new(text);
    if let Some(author) = author.map(str::trim).filter(|a| !a.is_empty()) {
        note = note.by(author);
    }
    if let Some(stamp) = crate::app::clock::pdf_date_utc() {
        note = note.at(stamp);
    }
    super::apply::vector_edit(doc, "add-reply", 0, 1, |session| {
        session.add_reply(parent, &note).map(|added| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ `-applied`, per the convention `set_note` records: the
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

/// ★★★ **Record a comment's pop-up state IN THE FILE** —
/// `EditSession::set_annotation_open`, `pdfcer-core` `Pass 253.3`.
///
/// Reached from the canvas pop-up's *Open by default* control and from nothing
/// else. **Not** from opening or closing a bubble on screen: that stays with
/// `crate::canvas::notepopup::open`, which owns per-document interface state
/// and raises no action at all. `AnnotAction::SetOpen`'s docs carry the whole
/// argument for the split, and the short form is that a reviewer reading six
/// comments must not end the session with six undo entries and a dirty file.
///
/// # ★★ Both objects, and why one call rather than two
///
/// Table 170 gives geometric markup **no `/Open` of its own**, so a `/Square`'s
/// window state lives only on its `/Popup`; a `/Text` has one on itself and
/// `pdfcer-core`'s own author writes both. The engine therefore treats the
/// state as a property of the **pair** and writes whichever halves exist, in
/// one command and one undo entry. A shell that issued two calls would put a
/// `Ctrl+Z` between them and could leave the two disagreeing on exactly the
/// subtype the operator uses most.
///
/// # ★ The no-op case is disclosed rather than hidden
///
/// When the annotation takes no `/Open` of its own **and** has no `/Popup`,
/// `set_annotation_open` succeeds, writes nothing and pushes **no undo entry**
/// — reported as a no-op rather than refused, so that a caller acting over a
/// mixed selection need not filter by subtype. The affordance is gated on
/// [`crate::canvas::notepopup::model::can_record_open_state`] under R83 so this should
/// not be reachable from the control; it is disclosed anyway, because *"the
/// button did nothing and said nothing"* is the one outcome an operator cannot
/// tell from a bug.
pub(super) fn set_open(doc: &mut OpenDoc, id: ObjId, open: bool) {
    super::apply::vector_edit(doc, "set-annotation-open", 0, 1, |session| {
        session.set_annotation_open(id, open).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ `was` is the field worth tracing beside the request: it is
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
// The NODES of a markup shape — `Pass 255.0`, the operator's report of
// 2026-09-05
// ===========================================================================
//
// > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//
// Three engine wrappers over one planner. They share everything except which
// `VertexEdit` they build, which is why they share [`reshape`] here rather than
// each spelling the funnel out — the disclosure obligation is identical for all
// three and stating it once is what stops the third one growing up without it.
//
// ★★★ **`reshape_annotation` and not the three wrappers**, and that is a
// deliberate reversal of the obvious call. The wrappers are one-liners that
// pass `modified: None`, so they can never stamp `/M`; this shell knows the
// time and the engine reads no clock, on purpose:
//
// > pdfcer reads no clock (determinism — the same edit on the same file
// > produces the same bytes), so the three convenience wrappers leave `/M`
// > exactly as it was and say so.
//
// A reviewer's comment whose shape changed and whose modification date did not
// is a comment that lies about when it was last touched, and §12.5.2 admits any
// string for `/M`. So this shell supplies one, in the ASN.1 form §7.9.4
// defines, and `AnnotationReshape::mod_date_written` reports whether it landed.

// ★★ **The date comes from `app::clock::pdf_date_utc`, and this paragraph is
// here because the first draft of this file did NOT.**
//
// A second civil-from-days implementation was written out in full — twelve
// lines of Howard Hinnant's algorithm — before `cargo test` surfaced a doctest
// for `app::clock::pdf_date_utc` doing the identical job, with the identical
// UTC ruling and a better failure mode. ⇒ **Two copies of a calendar are two
// calendars**, and the one nobody looks at is the one that claims 30 February.
// The duplicate was deleted rather than kept beside it.
//
// Its `None` case is the one this call site cares about: a clock before the
// Unix epoch yields no stamp, `/M` is left exactly as it was, and the
// annotation's date is unchanged rather than false. `AnnotationReshape::
// mod_date_written` reports which happened.

/// **Apply one node edit to a markup annotation**, as one undoable command.
///
/// ★★★ **Restyle a text-BEARING annotation** — a sticky note's icon and
/// colour, a stamp's colour. `EditSession::set_text_annot_style`
/// (`pdfcer-core` `edit.rs:27124`).
///
/// # ★★★ Why this is a second function beside the markup restyle, not a branch
/// # inside it
///
/// Because it is a second **verb over a second spec family**, and the engine's
/// own doc on `edit::TextAnnotStyle` (`edit.rs:15969`) is the argument:
///
/// > `MarkupStyle` reaches its annotation through
/// > `annot_author::spec_from_dict`, whose arms are the geometric family and
/// > the four text markups. **There is no `/Text` arm** … So the two verbs are
/// > not a split anyone chose for tidiness — they read through different
/// > functions because the two families are modelled by different spec types.
///
/// ⇒ A single body with an `if subtype == "Text"` inside it would put the
/// routing decision where nothing checks it. It is a `match` instead, on
/// `crate::panels::properties::markup::textannot::Reach`, decided in the panel
/// by asking each of the two engine readers in turn — and this function is only
/// ever reached down one arm of it.
///
/// # ★★ The page is `0`, and that is not a defect
///
/// `set_text_annot_style` takes an `ObjId` and nothing else — the property that
/// puts this variant in `AnnotAction` at all — so there is no page to pass.
/// [`clear_note`] and the node verbs pass `0` for the identical reason, and the
/// funnel uses the argument for its undo label and its raster invalidation
/// rather than to find anything. `EditScope::Document` (the plain
/// [`super::apply::vector_edit`], not the `_on_page` twin) is right for the
/// same reason it is right for a note: the annotation may not be on the page
/// the view is showing, and a `/Popup` companion may not be on its own.
///
/// # ★ What the trace carries, and why it is not the values
///
/// `icon_written` and `color_written` — the engine's own two booleans off
/// [`pdfcer_core::edit::TextAnnotStyleChange`] — plus how the appearance was
/// written. **Not the icon name**, and that is deliberate: the icon is
/// invisible in pdfcer's own picture by construction
/// (`annot_author::sticky_note` draws one marker for all seven), so what a
/// driven check needs is *did `/Name` change at all*, which no screenshot can
/// answer. A name in the trace would only restate what the panel already
/// displays.
///
/// # ★ The undo entry it pushes
///
/// `pdfcer_core::edit::CommandKind::SetTextAnnotStyle` — the engine's own
/// label for this command, pushed by the verb itself rather than by the funnel,
/// so an undo of a restyle is one entry and names the act rather than the
/// appearance rewrite underneath it.
///
/// ★ No disclosure list. Unlike `set_markup_style`, this verb reports no
/// `dropped` catalogue — it re-bakes from a spec the same reader hands the
/// authoring path, so there is nothing it can silently lose that this shell
/// could name.
pub(super) fn set_text_annot_style(
    doc: &mut OpenDoc,
    id: ObjId,
    style: &pdfcer_core::edit::TextAnnotStyle,
) {
    super::apply::vector_edit(doc, "set-text-annot-style", 0, 1, |session| {
        session.set_text_annot_style(id, style).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★★★ `was_foreign=` is new with `pdfcer-core` `Pass 253.5`,
                // and it is traced rather than DISCLOSED, deliberately.
                //
                // The engine's field reports that the annotation's previous
                // appearance was one pdfcer would not have drawn — a
                // designer's stream with a shadow or a gradient — and that
                // re-baking has just replaced it with pdfcer's plainer
                // rendering. That is exactly the class of thing Rule 4's
                // surviving half says must reach the operator off-canvas.
                //
                // ⚠ **It cannot happen through this shell today**, and the
                // reason is worth stating rather than discovering: the field is
                // `false` for every subtype but `/FreeText`, and the only
                // surface that raises this action —
                // `panels::properties::markup::textannot` — **declines a
                // `/FreeText` by name**. So a disclosure wired here would be a
                // sentence no operator could ever see, which is worse than none:
                // it reads as covered.
                //
                // ⇒ It goes on the trace, which is where a fact with no reader
                // belongs, and it becomes the **tripwire** for the day a
                // `/FreeText` does reach this verb: a driven run showing
                // `was_foreign=1` means the panel's decline has been lifted and
                // an operator-facing sentence is now owed. That is the same
                // posture `appearance_matrix_updated` gets one module along.
                format!(
                    "set-text-annot-style-applied id={} subtype={} icon={} colour={} \
                     was_foreign={} ap={:?}",
                    id.num,
                    change.subtype,
                    change.icon_written,
                    change.color_written,
                    u8::from(change.appearance_was_foreign),
                    change.appearance
                )
            });
            Vec::new()
        })
    });
}

// ===========================================================================
// The router — moved here from `apply` on 2026-09-05 under R2
// ===========================================================================

/// **Route one annotation verb to its body.**
///
/// Called from `apply`'s single `Action::Annot(_)` arm, which replaced eleven
/// that each destructured one variant and called one function two lines long.
/// The seam is `apply`'s own, stated in its header and drawn twice already:
/// **that file routes by family, and the family module decides.**
///
/// # ★ Why the author name is a parameter and not a read
///
/// [`crate::app::actions::annot::AnnotAction::SetNote`] carries `keep_author`
/// — a fact about the **document** the raising surface had in front of it —
/// and the name is a fact about the **operator** that only the apply scope can
/// see. A panel that carried a name would be reading preferences it is not
/// handed; a body that re-derived `keep_author` would be walking the annotation
/// a second time for something already known. So one travels on the action and
/// one travels as an argument, which is the split the field's own doc argues
/// for.
///
/// The empty string means *no name is set*, and it is filtered here rather than
/// at the call site so that every future caller of this router gets the same
/// answer to *"what does a blank author preference mean?"*.
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
        // ★ The one arm here whose body is in another family's module, and it is
        // deliberate: `app::actions::reorder` owns the `/Annots` permutation and
        // has since the tab-order panel needed it. A second implementation
        // beside `reorder_annotations` — same engine verb, same three
        // disclosures, different words — is precisely the drift that module's
        // own header is now about.
        A::Arrange { page, id, to } => crate::app::actions::reorder::arrange(doc, page, id, to),
        A::Resize {
            id,
            anchor,
            sx,
            sy,
            uniform,
            modifiers,
        } => resize(doc, id, anchor, (sx, sy), uniform, modifiers),
        // ★ Two rotation arms, not one with a kind flag: the engine refuses a
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
        // ★★★ A reply, and note what this arm does NOT do: it asks no
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
        // ★★ The **document's** `/Open`, which is a different subject from
        // whether a bubble is showing on screen — `canvas::notepopup::open`
        // owns that and raises nothing. Reached only from an explicit control;
        // the variant's docs carry the undo argument.
        A::SetOpen { id, open } => set_open(doc, id, open),
        // ★★★ The three node verbs — `Pass 255.0`, and the operator's *"I also
        // can't edit or delete nodes of a markup shape once it is drawn."*
        //
        // Three arms and not one carrying a `VertexEdit`, matching the three
        // variants: the shell's action bus does not carry the engine's enum, so
        // a fourth `VertexEdit` variant arrives as a compile error in
        // [`reshape`] rather than as a silent `..` here.
        A::MoveNode { id, index, dx, dy } => move_node(doc, id, index, dx, dy),
        A::InsertNode { id, after, at } => insert_node(doc, id, after, at),
        A::RemoveNode { id, index } => remove_node(doc, id, index),
        // ★★★ The three ink point verbs — `Pass 278.0`, O158 — carry a `(stroke,
        // point)` address and reach a different planner; [`inknodes::apply`]
        // destructures them. One arm here so this file stays under R2.
        A::MoveInkPoint { .. } | A::InsertInkPoint { .. } | A::RemoveInkPoint { .. } => {
            inknodes::apply(doc, action);
        }
        // ★★ **The only report a refused node edit produces.** The gesture
        // preflights through `reshape_annotation_preview`, so no verb is
        // reached, no funnel is entered and no `EditRefused` is recorded — if
        // this arm is removed the operator drags a corner of a triangle out of
        // the shape, releases, and the triangle is still a triangle with
        // nothing anywhere saying why. That silence is the report this whole
        // feature answers.
        //
        // ★ Recorded here rather than at the gesture because the decline store
        // is `pub(super)` inside `crate::app` and the canvas is outside that
        // boundary — the same crossing `DimensionAction::DeclineVertexEdit`
        // makes for the ce-dimension twin.
        A::DeclineNodeEdit { why } => {
            crate::app::status::decline::record_markup_node_refused(why);
        }
        // ★★★ The SECOND style verb, and the one arm here that answers a
        // different engine function from its neighbours. `set_markup_style`
        // reads through `spec_from_dict`, which has no `/Text` arm;
        // `set_text_annot_style` reads through `text_spec_from_dict`. Routing
        // between them is a `match` in the panel that raises this, not a
        // subtype string compared here.
        A::SetTextAnnotStyle { id, style } => set_text_annot_style(doc, id, &style),
    }
}
