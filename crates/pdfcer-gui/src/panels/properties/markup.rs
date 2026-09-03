//! # `panels::properties::markup` — restyling a markup that is already on the
//! page
//!
//! ## What this closes
//!
//! `FEATURES.md`'s Phase 1 row *"Format tab contents — colour, width, style,
//! opacity for a **placed** markup"*, and the row `pdfcer`'s own capability
//! register carried as ⬜ with a note this project wrote:
//!
//! > **`set_markup_style` shipped in the engine on 2026-08-18 and has zero GUI
//! > callers.** It appears only in doc comments.
//!
//! It was, until this landed, **the largest engine capability with no route
//! from this GUI**. Both blockers `shell::manifest::format`'s header recorded
//! are discharged — the verb landed 2026-08-18, and annotations became
//! selectable the same day — so what remained was work rather than a block, and
//! the operator's instruction of 2026-08-19 was to do the work.
//!
//! ## ★ Why the PANEL and not the Format tab
//!
//! `RIBBON_IA.md` §5.8 settles it and the wording is the operator's own
//! decision of 2026-08-12:
//!
//! > The division of labour: the **tab** carries what a user changes *while
//! > working* — colour, width, style, align, delete. The **panel** carries
//! > everything, including the read-only facts … The panel is also where the
//! > **editable geometry** lives.
//!
//! So the panel is where the complete set goes, and it is also the cheaper
//! surface by a wide margin: a ribbon band cannot hold a colour picker or a
//! slider without a new `Item::custom` kind and a renderer for it, which is
//! shell work in a crate that must never learn what a PDF is. The tab's slice
//! is a later, smaller job that reads the same actions.
//!
//! ## ★★ Every control is `None` unless the operator touched it
//!
//! `MarkupStyle`'s own doc comment is the rule and the reason:
//!
//! > Every field is `None` by default … That shape is deliberate: a Format tab
//! > whose colour picker also had to restate the current width would overwrite
//! > whatever the operator had set from the other control.
//!
//! So this section raises **one action per control that changed**, carrying one
//! field, and never a whole style struct assembled from what the widgets happen
//! to show. The failure that prevents is specific: two controls drawn from the
//! same annotation, one of them stale by a frame, and a colour change that
//! silently reverts a width the operator set a moment earlier.
//!
//! ## What is deliberately absent
//!
//! - **Fill (`/IC`).** `canvas::markup::spec` authors `interior: None` on
//!   purpose — *"a filled comment shape hides the drawing it is a comment
//!   about, which on a CAD sheet is the whole content under it"* — and
//!   `NO_SURFACE.md` records that reversing it is the operator's call, not
//!   this module's. A control here would make the decision by offering it.
//! - **Line endings (`/LE`).** They are meaningful for `/Line` alone, and the
//!   one `/Line` an operator of this application places is an arrow whose
//!   endings are what makes it an arrow. A control that could turn an arrow
//!   into a plain line belongs with a *kind* change, which nothing here does.
//! - **A ce dimension.** [`super::dimension`] owns those, through
//!   `set_dimension_style` — a different verb with a different model, and
//!   `AnnotKind` carries the distinction **in the type** so this section's
//!   guard is a `match` the compiler checks. Restyling a ce dimension as
//!   ordinary markup regenerates it as a bare line with its label and witness
//!   lines gone.

use egui::Ui;
use pdfcer_core::annot_author::Color;
use pdfcer_core::edit::{MarkupStyle, StyleEdit};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::selection::annot::AnnotKind;
use crate::text::panels::properties as t;

/// The region this section publishes.
pub const REGION: &str = "properties.markup"; // ui-text-exempt: trace region name, never displayed

/// The narrowest border this shell offers.
///
/// Zero is excluded and it is a decision rather than an oversight: §8.4.3.2
/// gives `0` a defined meaning — *the thinnest line the device can render* —
/// which on a 600 dpi plot is a hairline and on screen at 25 % is invisible.
/// An operator who wants a mark they cannot see has the visibility toggle;
/// what they must not get is a mark whose weight depends on the output device
/// without being told.
const MIN_WIDTH_PT: f64 = 0.25;

/// The widest.
///
/// Beyond about twelve points a border stops reading as a border and starts
/// reading as a filled shape, which is the thing this shell deliberately does
/// not author. The same ceiling `canvas::markup::pen` uses, for the same
/// reason and from the same argument.
const MAX_WIDTH_PT: f64 = 12.0;

/// **Draw the selected markup's style controls, or nothing.**
///
/// Returns whether it drew, so [`super::body`] knows the panel is already
/// saying something — the same contract [`super::dimension::section`] has, and
/// for the same reason: *"nothing is selected"* under a section describing the
/// thing that is selected would be the panel contradicting itself.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    // ★ Markup only. A ce dimension is `super::dimension`'s, and the
    // distinction is in the type rather than in a string comparison so that
    // routing one to the wrong verb is a compile error. Restyling a ce
    // dimension through `set_markup_style` regenerates it as a bare line with
    // its label and witness lines gone.
    if selection.target.kind != AnnotKind::Markup {
        return false;
    }
    // Cloned rather than borrowed: `AnnotTarget` carries the `/Subtype` as an
    // owned `String`, so it is not `Copy`, and the three rows below each need
    // the page and the id. One clone per frame of one small record is cheaper
    // than threading a borrow through a section that also reads the session.
    let target = selection.target.clone();

    crate::diag::ui_rect(REGION, ui.max_rect());
    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships
    // renders it legibly on a panel.
    ui.label(t::markup_heading());
    ui.label(
        egui::RichText::new(t::markup_subtype(&target.subtype))
            .small()
            .weak(),
    );

    // ★★ **Locked is R9's "temporarily unavailable", so it GREYS with a reason
    // rather than vanishing.**
    //
    // §12.5.3 Table 165 bit 8 says a locked annotation's properties "shall not
    // be changed by the user interface", and the engine refuses
    // `set_markup_style` for one by name. That is a property of *this*
    // annotation rather than of this build — click a different mark and the
    // controls work — which is exactly the case R9 reserves greying for, and
    // exactly the case where making the controls absent would read as pdfcer
    // being unable to restyle anything.
    if target.locked {
        ui.label(egui::RichText::new(t::markup_locked()).small().weak());
        ui.separator();
        return true;
    }

    // ★★ Read from the SESSION every frame, never from a cache, and read
    // through the SAME function the selection was made with.
    //
    // The verb this section raises rewrites the very values it displays, and an
    // action is applied *after* the frame that raised it — so a cached copy
    // would be stale for exactly the frame the operator is looking at, which is
    // the frame they judge the result on.
    //
    // `page_annotations` is `canvas::selection::annot::selectable_on`'s own
    // source, so a mark this section can restyle is by construction a mark the
    // canvas could select. A second reader — a `/Annots` walk of this module's
    // own — would eventually disagree about which annotations exist, and the
    // symptom would be controls drawn for a selection that no verb could name.
    let current = Current::read(doc, target.id);

    colour_row(ui, current, &target, actions);
    width_row(ui, current, &target, actions);
    opacity_row(ui, current, &target, actions);

    ui.label(egui::RichText::new(t::markup_note()).small().weak());
    ui.separator();
    true
}

/// What the selected mark's dictionary currently says, in the three terms this
/// section can change.
///
/// # ★★ Why it is read through `spec_from_dict` and not from `annot::Annotation`
///
/// `pdfcer_core::annot::Annotation` is the **reader's** view — id, subtype,
/// rect, flags, `/CA`, appearance — and it deliberately carries no `/C` and no
/// `/BS /W`, because nothing that renders a page needs them: the picture comes
/// from the baked `/AP`.
///
/// `annot_author::spec_from_dict` is the **author's** view, and it exists for
/// exactly this: *"so an existing annotation can be restyled by regenerating
/// its appearance from its own declared geometry"*. Reading through it means
/// the values these controls show are the values `set_markup_style` will read
/// when it plans — one derivation, not two.
///
/// ★ Its refusals are `None` here rather than an error, and that is honest
/// rather than lax. `SpecReadError`'s own doc says every variant is *"a refusal
/// to guess"* — geometry that is missing, or is not something pdfcer models. A
/// mark like that can still be **given** a colour; what cannot be done is show
/// the one it has, so the swatch falls back to its default and offers no Clear.
/// Nothing is destroyed by touching nothing.
#[derive(Debug, Clone, Copy, Default)]
struct Current {
    /// `/C`, if it is a colour a swatch can show without converting.
    colour: Option<[u8; 3]>,
    /// `/BS` `/W`, the border width in points.
    width: Option<f64>,
    /// `/CA`, the constant opacity.
    alpha: Option<f64>,
}

impl Current {
    /// Read it out of the session, this frame.
    fn read(doc: &OpenDoc, id: pdfcer_core::object::ObjId) -> Self {
        use pdfcer_core::annot_author::{MarkupSpec, spec_from_dict};
        use pdfcer_core::object::Object;

        let graph = doc.session.graph();
        let Some(Object::Dict(dict)) = doc.session.value(id) else {
            return Self::default();
        };
        // `/CA` straight off the dictionary rather than through the spec: it is
        // not part of `MarkupSpec` at all — the engine's own note says it
        // composites the annotation onto the page rather than affecting what
        // the appearance draws, which is why `set_markup_style` applies it to
        // the dictionary directly.
        // ★ `ObjectGraph::resolve` comes from the TRAIT, so it has to be in
        // scope. Reaching for the inherent method — there is none — is the
        // error a reader will hit first, and importing the trait beside the use
        // is what makes the call read as what it is: an indirect reference
        // followed through the session's overlay rather than through the base
        // file, so an unsaved edit is visible.
        use pdfcer_core::graph::ObjectGraph;
        let alpha = dict
            .get(b"CA")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_number);

        let (colour, width) = match spec_from_dict(&graph, dict) {
            Ok(spec) => match spec {
                MarkupSpec::Square {
                    border,
                    border_width,
                    ..
                }
                | MarkupSpec::Circle {
                    border,
                    border_width,
                    ..
                } => (border.and_then(|c| rgb_of(Some(&c))), Some(border_width)),
                MarkupSpec::Polygon { border, width, .. }
                | MarkupSpec::Cloud { border, width, .. } => {
                    (border.and_then(|c| rgb_of(Some(&c))), Some(width))
                }
                MarkupSpec::Line { color, width, .. }
                | MarkupSpec::PolyLine { color, width, .. }
                | MarkupSpec::Ink { color, width, .. } => (rgb_of(Some(&color)), Some(width)),
                // A text markup has a colour and no border at all — its shape is
                // `/QuadPoints` and there is nothing to stroke. The width row
                // still draws, with the engine's own default showing, because
                // `set_markup_style` accepts a width for it and simply has
                // nothing to apply it to; offering the control and having it do
                // nothing would be the inert control this project forbids, so
                // `width_row` asks this value and hides itself when it is
                // `None`.
                MarkupSpec::TextMarkup { color, .. } => (rgb_of(Some(&color)), None),
                // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does
                // not know the shape of gets no readback and no Clear, which is
                // the same answer a refused parse gets and for the same reason.
                _ => (None, None),
            },
            Err(_) => (None, None),
        };
        Self {
            colour,
            width,
            alpha,
        }
    }
}

/// The border colour.
///
/// ★ A **swatch plus a Clear**, not a swatch alone. `StyleEdit` has two arms
/// and they mean different things in the file: `Set` writes `/C`, and `Clear`
/// removes it, restoring the standard's default. A control that could only set
/// would make `/C` a one-way door — once an operator gave a mark a colour there
/// would be no way back to the file's own, and the difference is visible in
/// another viewer even when it is not visible here.
fn colour_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.colour;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_colour_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Set(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    ))),
                    ..MarkupStyle::default()
                },
            });
        }
        // Absent when there is nothing to clear, rather than greyed: a Clear
        // beside a mark that has no `/C` is a control whose only possible
        // effect is an undo entry the operator did not earn.
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// The border width.
///
/// ⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`**, and
/// the engine says so in its own doc: the rectangle is derived from the
/// geometry plus a margin that contains the stroke and any arrowheads, so a
/// wider pen needs a bigger box. That is disclosed in [`t::markup_note`]
/// rather than here, because it is true of the section and not of this control
/// alone.
fn width_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ★ ABSENT rather than greyed when the mark has no border to widen — a
    // highlight is `/QuadPoints` and has nothing to stroke. R9: an unavailable
    // capability renders nothing. A greyed spinner here would be pdfcer
    // implying that a highlight could have a line width if only something were
    // different, and nothing is.
    let Some(mut width) = current.width else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(t::markup_width_label());
        let response = ui.add(
            egui::DragValue::new(&mut width)
                .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                .speed(0.1)
                .suffix(t::markup_width_suffix()),
        );
        // ★ `drag_stopped` and `lost_focus`, not `changed`. A `DragValue` reports
        // a change on every pixel of a drag, and each one here is a
        // content-stream rewrite plus an undo entry — so a single drag across
        // the control would leave forty entries on the stack and re-plan the
        // annotation forty times. The colour swatch above needs no such guard:
        // it opens a popup and reports once, on the operator's pick.
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    width: Some(width),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// The constant opacity, `/CA`.
///
/// ★★ **This is the control `NO_SURFACE.md` recorded as "blocked on the engine"
/// for weeks, and the blocker was false.** `set_markup_style` has taken an
/// opacity since it shipped and writes `/CA` clamped to `0.0..=1.0`; the row
/// that said otherwise was a claim about a repository this project does not
/// build, and it could not fail a test. See `NO_SURFACE.md` §1b.
///
/// Shown as a **percentage**, because that is the unit every other application
/// an operator has used states opacity in, and `/CA`'s own `0.0..=1.0` is a
/// file-format detail they should never meet.
fn opacity_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.alpha;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut percent = (existing.unwrap_or(1.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    ui.horizontal(|ui| {
        ui.label(t::markup_opacity_label());
        let response = ui.add(
            egui::DragValue::new(&mut percent)
                .range(0..=100)
                .speed(1.0)
                .suffix(t::markup_opacity_suffix()),
        );
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Set(f64::from(percent) / 100.0)),
                    ..MarkupStyle::default()
                },
            });
        }
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// An annotation's `/C` as sRGB bytes, if it has one this control can show.
///
/// ★ `None` for anything that is not RGB, and that is honest rather than
/// lossy: §12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, and a
/// swatch showing a CMYK mark's *converted* colour would be a control whose
/// readback is a conversion the operator never asked for — pick it up, put it
/// down unchanged, and the file now says something different. Those marks get
/// the default swatch and no Clear, so nothing is destroyed by touching
/// nothing.
fn rgb_of(color: Option<&Color>) -> Option<[u8; 3]> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    match color? {
        Color::Rgb(r, g, b) => Some([
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]),
        Color::Gray(v) => {
            let g = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            Some([g, g, g])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grey resolves to an equal-component swatch, and RGB round-trips.
    ///
    /// Grey is included rather than refused because it is **lossless** in both
    /// directions — `Gray(v)` and `Rgb(v, v, v)` are the same ink — where CMYK
    /// is not, which is the distinction `rgb_of`'s own docs draw.
    #[test]
    fn a_swatch_shows_only_colours_it_can_show_without_converting() {
        assert_eq!(rgb_of(Some(&Color::Rgb(1.0, 0.0, 0.0))), Some([255, 0, 0]));
        assert_eq!(rgb_of(Some(&Color::Gray(0.0))), Some([0, 0, 0]));
        assert_eq!(rgb_of(Some(&Color::Gray(1.0))), Some([255, 255, 255]));
        assert_eq!(rgb_of(None), None);
    }

    /// ★ The width range is the same one the markup pen offers.
    ///
    /// Two ranges for one quantity would let an operator author a 2 pt mark and
    /// then be unable to set 2 pt on it — or, worse, set a width here the pen
    /// could not have produced, so a document would carry marks the shell
    /// cannot make.
    #[test]
    fn the_width_range_matches_the_pen_that_authors() {
        assert!((MIN_WIDTH_PT - crate::canvas::markup::pen::MIN_WIDTH_PTS).abs() < f64::EPSILON);
        assert!((MAX_WIDTH_PT - crate::canvas::markup::pen::MAX_WIDTH_PTS).abs() < f64::EPSILON);
    }
}
