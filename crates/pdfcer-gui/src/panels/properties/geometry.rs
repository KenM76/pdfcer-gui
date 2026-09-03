//! # `panels::properties::geometry` — **X, Y, W and H, typed rather than
//! dragged**
//!
//! ## What this closes
//!
//! `FEATURES.md`'s Phase 1 remainder, verbatim:
//!
//! > **Editable geometry** — X/Y/W/H in the Properties panel, typed rather
//! > than dragged.
//!
//! And it closes it *because the resize gesture landed first*. Every number in
//! this section is a call into machinery that already exists and is already
//! tested: a position change is [`VectorAction::MoveSelection.into()`], the same variant a
//! move drag raises, and a size change is [`crate::canvas::resizing::action`],
//! the same function the eight grips raise. **This module computes two scale
//! factors and a delta and contributes no geometry of its own.**
//!
//! That is the whole design and it is deliberate. A properties panel that
//! reimplemented "make it 40 points wide" would be a second scale
//! implementation with a second set of rounding, a second pivot convention and
//! a second answer to *what happens to line weights* — and the two would drift,
//! silently, because nothing compares them.
//!
//! ## ★ Why an operator needs this even though the grips work
//!
//! Because a grip cannot express *exactly 40.0 points*. The resize gesture is
//! excellent for "about this big" and incapable of "the same as the one above
//! it", and a drawing is full of the second kind. It is also the only route
//! for a **small** object: at fit-page zoom on an ISO A1 sheet a 6 mm symbol is
//! about four pixels across, so its eight grips overlap each other and the
//! gesture is unusable at the zoom where the operator can see the sheet.
//!
//! It is additionally the only **accessible** route to a resize. A gesture that
//! requires a sub-pixel drag is a gesture some operators cannot perform, and
//! `MODES_AND_PANELS.md` §7's accessibility line asks for a typed equivalent to
//! every direct-manipulation edit for exactly that reason.
//!
//! ## ★★ Why there is an Apply button and the fields do not commit as you type
//!
//! Because **every commit is an undo entry**, and a `DragValue` the operator
//! scrubs from 40 to 120 would raise eighty of them. That is not a theoretical
//! objection: `app::actions`' `MoveNodes` doc comment makes the identical
//! argument about looping the singular verbs — *"N undo entries for one drag,
//! and each planned against byte offsets the previous one invalidated"* — and a
//! live-committing spinner is that loop with a nicer face on it.
//!
//! So the four fields edit a **draft**, and one press turns the draft into at
//! most two commands. Two rather than one because a move and a scale are
//! different verbs in `EditSession` and this shell does not have a combined
//! one; the operator who changes only X gets exactly one entry, and the
//! operator who changes X and W gets two, in that order, which is the order
//! that makes the second one's pivot mean what the preview said.
//!
//! ## ★ Why the draft is discarded when the object changes underneath it
//!
//! The draft is stamped with `(page, object, edit epoch)`. If any of the three
//! moves — the operator selects something else, or *anything at all* edits the
//! document — the draft is dropped and re-seeded from the object's current
//! bounds.
//!
//! The epoch is the one that matters and it is the one that is easy to leave
//! out. Without it, this sequence silently destroys work: type `W = 40`, do not
//! press Apply, press `Ctrl+Z` to undo something unrelated, press Apply. The
//! draft would still hold the numbers computed against the *pre-undo* bounds,
//! and the scale factor would be `40 / (a width that no longer exists)`. The
//! object would end up some third size that the operator never typed and cannot
//! predict. Re-seeding on the epoch makes that unrepresentable rather than
//! merely unlikely.
//!
//! ## What this deliberately does NOT offer
//!
//! - **Rotation.** `EditSession` has no rotate verb, and expressing one as
//!   `move_nodes` is not the same edit: it would rotate the anchors and leave
//!   every glyph, dash pattern and line cap in the original orientation. That
//!   is a shear of the outline, not a rotation of the object.
//! - **A units picker.** Every number here is in **PDF user-space points**,
//!   which is what `move_objects` and `move_nodes` take and what the rest of
//!   this panel already shows. A millimetre field would be a conversion this
//!   module owns, and the measure tools already own a scale model with a
//!   *different* answer — a drawing at 1:50 has a page millimetre and a world
//!   millimetre and they are not the same length. Two conversions, one label.
//! - **Multi-object geometry.** The same refusal `resizing` makes and for the
//!   same reason: `move_nodes` addresses one object, and *"set both of these to
//!   40 wide"* is a different feature (align/distribute) with a different
//!   surface.

use egui::Ui;
use pdfcer_core::vector::Point;

use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::canvas::resizing;
use crate::text::panels::properties as t;

/// The `ui-rect` region this section publishes, so a driven check can find the
/// fields without knowing the panel's arrangement.
pub const REGION: &str = "properties.geometry"; // ui-text-exempt: diagnostic region name

/// The **Width** field's own region.
///
/// ★ Published per field rather than leaving a driven check to divide [`REGION`]
/// into quarters, because the row heights are the theme's and would change under
/// a UI-scale setting the operator can move. `D:/dev/rag/egui/` records the
/// general form: *a harness that computes a control's position from a container's
/// is asserting the layout it was written against, not the one that shipped.*
pub const WIDTH_REGION: &str = "properties.geometry.width"; // ui-text-exempt: diagnostic region name

/// The **Apply** button's own region.
pub const APPLY_REGION: &str = "properties.geometry.apply"; // ui-text-exempt: diagnostic region name

/// The smallest width or height the fields will accept.
///
/// A quarter point, matching `resizing::is_usable`'s own floor on the factors
/// it will act on. Below it the scale is a degenerate collapse — every node of
/// the path onto one line — which `move_nodes` would happily perform and which
/// no operator means.
pub const MIN_EXTENT_PT: f64 = 0.25;

/// The typed values, and what they were seeded from.
///
/// # Why the seed is stored and not just the values
///
/// Because *what changed* is the question this section has to answer on Apply,
/// and it cannot be answered by comparing the draft to the object's **current**
/// bounds — those are the same numbers the draft was seeded from, so an
/// operator who typed nothing and one who typed the current value back in would
/// be indistinguishable. Storing the seed makes "the operator touched this
/// field" a fact rather than an inference.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeometryDraft {
    /// What the draft describes: `(page, object index, edit epoch)`.
    ///
    /// `None` before the first seed. See the module header for why the epoch
    /// is a member and not an optimisation.
    stamp: Option<(usize, usize, u64)>,
    /// Left edge, PDF user-space points.
    pub x: f64,
    /// **Bottom** edge, PDF user-space points — Y is up in PDF space, and this
    /// field is labelled from the bottom rather than silently flipped, because
    /// a panel that showed a top-down Y would disagree with every other number
    /// in the program including the status bar's cursor readout.
    pub y: f64,
    /// Width, PDF user-space points.
    pub w: f64,
    /// Height, PDF user-space points.
    pub h: f64,
}

/// The axis-aligned bounds of a path object, in PDF user space.
///
/// Derived from the object's **anchors**, which is the same set
/// [`resizing::action`] moves — deliberately, so the number this panel shows
/// and the number the edit acts on cannot disagree. A bounding box taken from
/// any other source (the object's `/BBox`, the canvas outline, the render
/// extent) would include curve bulges, line weight or a transform this section
/// does not apply, and the operator would type 40 and measure something else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Minimum X.
    pub x0: f64,
    /// Minimum Y.
    pub y0: f64,
    /// Maximum X.
    pub x1: f64,
    /// Maximum Y.
    pub y1: f64,
}

impl Bounds {
    /// The bounds of a list of anchors, or `None` for an empty list.
    #[must_use]
    pub fn of(points: &[(usize, Point)]) -> Option<Self> {
        let (_, first) = points.first()?;
        let mut b = Self {
            x0: first.x,
            y0: first.y,
            x1: first.x,
            y1: first.y,
        };
        for (_, p) in points {
            b.x0 = b.x0.min(p.x);
            b.y0 = b.y0.min(p.y);
            b.x1 = b.x1.max(p.x);
            b.y1 = b.y1.max(p.y);
        }
        Some(b)
    }

    /// Width.
    #[must_use]
    pub fn w(self) -> f64 {
        self.x1 - self.x0
    }

    /// Height.
    #[must_use]
    pub fn h(self) -> f64 {
        self.y1 - self.y0
    }
}

impl GeometryDraft {
    /// Seed the draft from `bounds` if it does not already describe
    /// `(page, object, epoch)`.
    ///
    /// Idempotent within a stamp, which is what lets it be called every frame:
    /// the operator's typing survives redraws and is discarded exactly when the
    /// thing it describes stops being the thing it described.
    pub fn sync(&mut self, page: usize, object: usize, epoch: u64, bounds: Bounds) {
        if self.stamp == Some((page, object, epoch)) {
            return;
        }
        self.stamp = Some((page, object, epoch));
        self.x = bounds.x0;
        self.y = bounds.y0;
        self.w = bounds.w();
        self.h = bounds.h();
    }

    /// Whether anything was typed — i.e. whether Apply has work to do.
    ///
    /// Compared against the seed, not against the document. See the struct's
    /// own note for why that distinction is load-bearing.
    #[must_use]
    pub fn differs_from(&self, bounds: Bounds) -> bool {
        !near(self.x, bounds.x0)
            || !near(self.y, bounds.y0)
            || !near(self.w, bounds.w())
            || !near(self.h, bounds.h())
    }

    /// Whether the typed extents are large enough to act on.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.w >= MIN_EXTENT_PT && self.h >= MIN_EXTENT_PT
    }
}

/// Two values that are the same number to within a tenth of a point.
///
/// ★ A tolerance rather than `==`, because these values make a round trip
/// through an `f64` spinner and back, and `40.0` typed into a field that was
/// seeded with `39.999999999999996` is the operator changing nothing. Without
/// it, merely selecting an object and pressing Apply would raise a move of
/// 4 × 10⁻¹⁵ points — a real undo entry, a real content-stream rewrite, and a
/// real cache invalidation, for an edit with no effect at any zoom.
///
/// A tenth of a point is about 35 µm on paper: finer than any plotter this
/// operator's drawings are printed on, and coarser than every float artefact.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.1
}

/// What Apply should raise, given a draft and the bounds it was seeded from.
///
/// Returns the commands **in the order they must run**: the move first, then
/// the scale. That order is not cosmetic — the scale's pivot is the object's
/// bottom-left corner *as the operator sees it after the move*, so computing
/// the scale against pre-move bounds and applying it after the move would put
/// the object somewhere neither number described.
///
/// # Why this is a pure function taking `Bounds` rather than a method on `Ui`
///
/// So it can be tested without a document, a provider or an `egui::Context`.
/// The four cases that matter — position only, size only, both, neither — are
/// four assertions here and would each be a driven check otherwise.
#[must_use]
pub fn plan(draft: &GeometryDraft, bounds: Bounds) -> Plan {
    let dx = draft.x - bounds.x0;
    let dy = draft.y - bounds.y0;
    let moved = !near(draft.x, bounds.x0) || !near(draft.y, bounds.y0);

    // ★ A zero-extent axis cannot be scaled and is not an error: a horizontal
    // line has no height, and asking for `h_new / 0` is how a NaN reaches
    // `move_nodes`. The factor is 1 — leave that axis alone — which is also
    // what the operator means, because a field showing `0.0` for a flat line is
    // describing a fact rather than offering an edit.
    let sx = if bounds.w() > f64::EPSILON {
        draft.w / bounds.w()
    } else {
        1.0
    };
    let sy = if bounds.h() > f64::EPSILON {
        draft.h / bounds.h()
    } else {
        1.0
    };
    let scaled = !near(draft.w, bounds.w()) || !near(draft.h, bounds.h());

    Plan {
        translate: moved.then_some((dx, dy)),
        // The pivot is the bottom-left of the box *after* the move, so the
        // corner the operator pinned with X and Y is the corner that stays.
        scale: scaled.then_some((
            Point {
                x: draft.x,
                y: draft.y,
            },
            (sx as f32, sy as f32),
        )),
    }
}

/// What one press of Apply amounts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    /// `(dx, dy)` in PDF points, when the position changed.
    pub translate: Option<(f64, f64)>,
    /// `(pivot, (sx, sy))`, when the size changed.
    pub scale: Option<(Point, (f32, f32))>,
}

impl Plan {
    /// Whether the plan would do anything.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.translate.is_none() && self.scale.is_none()
    }
}

/// Draw the section, returning whether it drew anything.
///
/// Returns `false` — and draws **nothing**, per R9 — whenever the selection is
/// not a single path object on the current page. That covers no selection, an
/// annotation, several objects, a text run and an image, and in every one of
/// those cases the correct surface is silence rather than four greyed spinners:
/// the fields are not *temporarily* unavailable, they have no subject.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut GeometryDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // An annotation's geometry is its `/Rect`, which no verb in this build
    // rewrites — see `FEATURES.md`'s Format-tab row. Showing editable X/Y/W/H
    // over one would be a control that accepts a value and discards it.
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return false;
    };
    let object = *object;
    let Some(provider) = doc.page_objects() else {
        return false;
    };
    let points = provider.object_node_points(object);
    let Some(bounds) = Bounds::of(&points) else {
        return false;
    };
    // The `Ref` into the document's object cache is released before anything is
    // drawn — the same short-borrow discipline `object_section` states, and it
    // matters more here because `apply` will want `&mut OpenDoc` this frame.
    drop(provider);

    draft.sync(page, object, doc.edit_epoch, bounds);

    // ★★★ `ui_rect_visible`, not `ui_rect` — 2026-08-26, and the same lesson
    // `dialogs::settings::widgets::group` learned for its headings.
    //
    // The Properties panel is a `ScrollArea`, and in an ordinary dock layout
    // this section is taller than the slot it gets. A rect published for a
    // control that is scrolled out of sight does not merely mislead a reader:
    // a driven check **clicks** it, and the click lands on whatever is
    // genuinely at those coordinates.
    //
    // That is not hypothetical. `ui-verify geometry_fields_resize_a_shape`
    // reported *"THE WIDTH FIELD WAS SCRUBBED BY 80 PIXELS AND APPLY COMMITTED
    // NOTHING AND DECLINED NOTHING"* — a report that reads as a dead button and
    // was filed as one. The trace said otherwise:
    //
    //     properties.geometry        [[786.0 591.7] - [1100.0 762.0]]
    //     properties.geometry.apply  [[786.0 776.7] - [ 835.0 804.7]]
    //
    // Apply was **14 points below the panel's viewport**, the click went to
    // empty canvas, and nothing was pressed. The button was never broken.
    //
    // ★ Publishing only what is visible turns that false failure into an honest
    // SKIP naming the real condition — the panel is shorter than its content.
    // An absent region is a much better lie-free answer than a present one that
    // cannot be clicked.
    //
    // ★ And it is the section's REAL extent, not `ui.max_rect()`. That was the
    // available space when the section started drawing — which in a scroll area
    // is the remaining viewport — so it published a rect ending at 762 while
    // its own Apply button laid out at 776. A region that does not contain its
    // own controls is not a region.
    // No `.strong()` — R84 / DEFECTS.md D11.
    ui.label(t::geometry_heading());
    ui.label(egui::RichText::new(t::geometry_units_note()).small().weak());

    field(ui, t::geometry_x(), &mut draft.x, None);
    field(ui, t::geometry_y(), &mut draft.y, None);
    field(ui, t::geometry_w(), &mut draft.w, Some(WIDTH_REGION));
    field(ui, t::geometry_h(), &mut draft.h, None);

    let changed = draft.differs_from(bounds);
    let usable = draft.is_usable();
    // ★ The draft AND what was inferred from it, on the trace channel, because
    // the three ways this section fails are indistinguishable from outside:
    // Apply greyed because the draft was wiped, Apply greyed because the scrub
    // never landed, and Apply live but `plan` returning nothing. A line saying
    // only "Apply was greyed" would have sent the first driven run of this
    // feature looking in the wrong one of the three.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "geometry-draft x={:.2} y={:.2} w={:.2} h={:.2} bw={:.2} bh={:.2} \
             changed={changed} usable={usable}",
            draft.x,
            draft.y,
            draft.w,
            draft.h,
            bounds.w(),
            bounds.h()
        )
    });

    // ★ Greying, not hiding, and this is the case R9 reserves it for: Apply is
    // *temporarily* unavailable — type a different number and it works — which
    // is exactly the distinction the rule draws against a capability that is
    // absent. Both reasons are on the hover, because a dead button with no
    // explanation is the defect this project keeps finding.
    let enabled = changed && usable;
    let response = ui
        .add_enabled(enabled, egui::Button::new(t::geometry_apply()))
        .on_disabled_hover_text(if usable {
            t::geometry_nothing_typed()
        } else {
            t::geometry_too_small()
        });
    crate::diag::ui_rect_visible(APPLY_REGION, response.rect, ui.clip_rect());

    if response.clicked() {
        let plan = plan(draft, bounds);
        if let Some((dx, dy)) = plan.translate {
            actions.push(
                VectorAction::MoveSelection {
                    page,
                    objects: vec![object],
                    dx,
                    dy,
                }
                .into(),
            );
        }
        if let Some((pivot, factors)) = plan.scale {
            // ★ Routed through `resizing::action` rather than assembled here,
            // so the six refusals — not a path, no nodes, no object model — are
            // asked once and answered the same way for a typed edit as for a
            // dragged one. A second construction of `MoveNodes` in this file
            // would be a second place for "can this object be scaled?" to have
            // an opinion.
            let planned = resizing::action(
                &doc.selection,
                page,
                doc.page_objects().as_deref(),
                pivot,
                factors,
            );
            match planned {
                Ok(action) => actions.push(action),
                // Off-canvas, in words, through the SAME `decline` the grips
                // use — so a typed refusal and a dragged one produce one
                // sentence from one place. See `resizing::decline` for why a
                // refusal is recorded against epoch zero.
                Err(refusal) => resizing::decline(refusal),
            }
        }
    }

    ui.separator();
    // ★ Published HERE, at the end, and that placement is the fix.
    //
    // `ui.min_rect()` after the section has drawn is what it actually occupies.
    // Before it draws, `min_rect` is empty and `max_rect` is the space
    // *available* — which in a scroll area is the remaining viewport, and which
    // is what this used to publish: a rect ending at 762 while its own Apply
    // button laid out at 776. A region that does not contain its own controls
    // is not a region, and a check dividing it into quarters to find a field
    // would have been dividing the wrong box.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    true
}

/// One labelled numeric field.
///
/// `DragValue` rather than a `TextEdit`, because it accepts both — an operator
/// can scrub it *or* click and type an exact number — and the scrubbing costs
/// nothing here precisely because the fields edit a draft. On a live-committing
/// surface a scrubbable field would be the eighty-undo-entries problem the
/// module header describes; on a drafted one it is a free second input method.
fn field(ui: &mut Ui, label: &str, value: &mut f64, region: Option<&'static str>) {
    ui.horizontal(|ui| {
        ui.label(label);
        let response = ui.add(egui::DragValue::new(value).speed(SPEED).fixed_decimals(2));
        if let Some(region) = region {
            // Visible-clipped, for the reason the section's own publication
            // gives: this panel scrolls, and a field a harness can see but an
            // operator cannot is a coordinate that scrubs whatever is behind
            // it.
            crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        }
    });
}

/// PDF points per screen pixel of horizontal scrub.
///
/// Half a point, so a hundred-pixel drag spans fifty points — about the range a
/// draughtsman adjusts a symbol by — and so the value visibly moves on a slow
/// drag rather than jumping. A driven check depends on this being **exact**:
/// scrubbing `n` pixels changes the field by `n × SPEED`, which is what lets the
/// harness assert the number it expects instead of merely that something
/// changed.
const SPEED: f64 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(v: &[(f64, f64)]) -> Vec<(usize, Point)> {
        v.iter()
            .enumerate()
            .map(|(i, (x, y))| (i, Point { x: *x, y: *y }))
            .collect()
    }

    #[test]
    fn bounds_come_from_the_anchors() {
        let b = Bounds::of(&pts(&[(10.0, 20.0), (50.0, 20.0), (50.0, 80.0)])).unwrap();
        assert!((b.x0 - 10.0).abs() < 1e-9);
        assert!((b.y0 - 20.0).abs() < 1e-9);
        assert!((b.w() - 40.0).abs() < 1e-9);
        assert!((b.h() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn no_anchors_means_no_bounds() {
        assert!(Bounds::of(&[]).is_none());
    }

    /// ★★ **Selecting an object and pressing Apply raises nothing.**
    ///
    /// The float round trip through the spinner is why this needs a test rather
    /// than being obvious: a seed of `39.999999999999996` and a typed `40.0`
    /// are different `f64`s and the same edit. Without the tolerance this would
    /// raise a move of 4 × 10⁻¹⁵ points — a real undo entry and a real
    /// content-stream rewrite for a change no zoom can show.
    #[test]
    fn an_untouched_draft_plans_nothing() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 3, 7, b);
        assert!(!d.differs_from(b));
        assert!(plan(&d, b).is_empty());

        // The same number arriving with float dust on it is still no change.
        // Computed rather than written as a literal, because a literal with
        // that many digits is rounded back to 40.0 by the parser and the test
        // would assert nothing.
        d.w = (0.1_f64 + 0.2) * 100.0 + 10.000_000_000_000_004;
        assert!(plan(&d, b).is_empty());
    }

    /// Position only → one move, no scale.
    #[test]
    fn moving_it_plans_a_move_and_no_scale() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 0, 0, b);
        d.x = 110.0;
        let p = plan(&d, b);
        assert_eq!(p.translate, Some((100.0, 0.0)));
        assert!(p.scale.is_none());
    }

    /// ★ Size only → one scale, pivoted on the corner the operator did NOT
    /// touch, so the object grows to the right and upward rather than about its
    /// middle. That is what a properties panel means by "X, Y, W, H": X and Y
    /// name a corner, and changing W moves the *other* edge.
    #[test]
    fn resizing_it_pivots_on_the_stated_corner() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 0, 0, b);
        d.w = 80.0; // double the width
        let p = plan(&d, b);
        assert!(p.translate.is_none());
        let (pivot, (sx, sy)) = p.scale.unwrap();
        assert!((pivot.x - 10.0).abs() < 1e-9, "pivot is the stated X");
        assert!((pivot.y - 20.0).abs() < 1e-9, "pivot is the stated Y");
        assert!((sx - 2.0).abs() < 1e-6);
        assert!((sy - 1.0).abs() < 1e-6, "the untouched axis is not scaled");
    }

    /// ★★ **A flat object does not produce a NaN.**
    ///
    /// A horizontal line has zero height, and the obvious implementation
    /// computes `h_new / 0`. `move_nodes` would accept the resulting NaN
    /// coordinates and write them into the content stream, producing a page
    /// that no viewer — including this one — can render.
    #[test]
    fn a_zero_height_object_scales_only_the_axis_it_has() {
        let b = Bounds {
            x0: 0.0,
            y0: 50.0,
            x1: 100.0,
            y1: 50.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 0, 0, b);
        d.w = 200.0;
        let (_, (sx, sy)) = plan(&d, b).scale.unwrap();
        assert!((sx - 2.0).abs() < 1e-6);
        assert!(sy.is_finite() && (sy - 1.0).abs() < 1e-6);
    }

    /// ★ **The draft is discarded when the document changes underneath it**,
    /// which is the sequence in the module header: type a width, undo something
    /// unrelated, press Apply. Without the epoch in the stamp the factor would
    /// be computed against bounds that no longer exist.
    #[test]
    fn a_new_epoch_reseeds_the_draft() {
        let b = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 40.0,
            y1: 40.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 0, 1, b);
        d.w = 400.0;
        // Same page, same object, but the document moved on.
        let after = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 10.0,
        };
        d.sync(0, 0, 2, after);
        assert!(
            (d.w - 10.0).abs() < 1e-9,
            "the typed 400 must not survive an edit it was not computed against"
        );
    }

    /// Selecting a different object reseeds too.
    #[test]
    fn a_new_object_reseeds_the_draft() {
        let b = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 40.0,
            y1: 40.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, 0, 1, b);
        d.x = 999.0;
        d.sync(0, 1, 1, b);
        assert!((d.x - 0.0).abs() < 1e-9);
    }

    /// A collapse is refused before it can reach `move_nodes`.
    #[test]
    fn a_degenerate_extent_is_not_usable() {
        let mut d = GeometryDraft {
            w: 0.0,
            h: 40.0,
            ..Default::default()
        };
        assert!(!d.is_usable());
        d.w = MIN_EXTENT_PT;
        assert!(d.is_usable());
    }
}
