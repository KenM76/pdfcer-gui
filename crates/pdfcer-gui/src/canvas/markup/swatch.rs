//! # `canvas::markup::swatch` — the Markup ▸ Style group's one control
//!
//! The `colour_swatch` custom item the manifest has declared since S2 and
//! nothing ever drew, so the Style group rendered a caption over an empty band.
//!
//! ## Why it is a Custom item and not three commands
//!
//! `egui-shell`'s `Item::Custom` is the extension point for a control that is
//! *not a button* — its own documentation names *"a split button with a
//! gallery"* — and this is three of those. A `Command` item can only render as
//! a button, and a button cannot ask *which colour* any more than the Recent
//! item's button could ask *which document*. The manifest declares
//! `Item::custom("colour_swatch")` and the application supplies the renderer,
//! exactly as it already does for Recent.
//!
//! That is also why this is not three registered commands. A command is a verb
//! the operator invokes and the shell dispatches; setting a pen colour is not
//! a verb, it has no undo, it raises no `Action`, and giving it a handler token
//! would put a no-op through the dispatch `match` for every click of a colour
//! picker.
//!
//! ## ★ What it sets, and the one thing it deliberately does not
//!
//! `RIBBON_IA.md` §5.5's Style group is *"Colour · Line width · Fill ·
//! Opacity"*. Two of the four ship here and two do not, and the two absences
//! are different in kind:
//!
//! | control | state | why |
//! |---|---|---|
//! | **Colour** | ✅ | two swatches — see [`super::pen`] on why there are two |
//! | **Line width** | ✅ | a drag value in points, over the pen's own range |
//! | **Fill** | ⬜ | **a design decision, not a gap.** `spec` passes `interior: None` for every shape with a note that says why: *"a filled comment shape hides the drawing it is a comment about, which on a CAD sheet is the whole content under it."* Offering it would need that decision reversed by the operator, not by this module |
//! | **Opacity** | ⬜ | **blocked on the engine.** Annotation transparency is `/CA`, which `pdfcer-core` does not write yet — filed, accepted, not started. A slider here would be an affordance for something that cannot happen, which the no-placeholders rule forbids, and it would be the *worst* kind: the mark would be authored fully opaque and the operator would have no way to tell the setting had been ignored |
//!
//! The opacity row is the one worth being careful about. It is absent rather
//! than greyed because greying is reserved for *temporarily* unavailable — no
//! document, empty undo stack — and no setting in this build makes `/CA`
//! reachable.
//!
//! ## Why the swatch shows the colour rather than naming it
//!
//! Because the operator is choosing a colour and the only useful preview of a
//! colour is the colour. `egui`'s `color_edit_button_srgba` is exactly that: a
//! button filled with the current value that opens a picker. Its accessible
//! name comes from the hover text, which is why both swatches carry one.
//!
//! **The alpha channel is not offered**, and [`super::pen::Pen::set_ink`]
//! carries the argument: a PDF annotation's `/C` is three components, and
//! feeding a picker's alpha into it would be a value with nowhere to go. The
//! `Opaque` variant of the picker is what says so.

use egui::Ui;

use super::pen::{MAX_WIDTH_PTS, MIN_OPACITY, MIN_WIDTH_PTS, Pen};
use crate::text::markup as t;

/// The region this control publishes, so a check can find and drive it.
///
/// One per part rather than one for the group: a harness proving that a colour
/// can be *changed* has to click the swatch, and a rect covering all three
/// controls would give it the wrong target two times in three.
pub const REGION_INK: &str = "markup.style.ink"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the highlighter.
pub const REGION_HIGHLIGHTER: &str = "markup.style.highlighter"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the width.
pub const REGION_WIDTH: &str = "markup.style.width"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the opacity.
pub const REGION_OPACITY: &str = "markup.style.opacity"; // ui-text-exempt: trace region name, never displayed

/// Draw the Style group's controls, editing `pen` in place.
///
/// # It edits in place and raises nothing
///
/// No `Action`, no `HandlerToken`, no return value. The funnel's invariant is
/// that no code path runs from a widget to a **document**, and this touches no
/// document: it sets the pen the *next* gesture will use, which is application
/// state with no undo log to order against and nothing to alias. The same
/// argument `crate::dialogs::print` makes about spooling, one size down.
///
/// # Horizontal, and narrow on purpose
///
/// A ribbon group is a band about 70 points tall, and three stacked rows would
/// not fit. More usefully: these three are read together — *what colour, how
/// thick* — so a row is what an operator scans, and the ribbon's own group
/// caption underneath says which group they are in.
pub fn show(ui: &mut Ui, pen: &mut Pen) {
    ui.horizontal(|ui| {
        // ★ The two swatches are ADJACENT and labelled, rather than one swatch
        // that changes meaning with the armed tool.
        //
        // A single swatch would have to answer "which pen am I setting?" from
        // the armed tool, which means the control silently changes what it
        // edits as the operator moves along the Shapes row — and worse, edits
        // *nothing they can see* when no tool is armed. Two controls that each
        // always mean one thing is the version an operator can predict.
        let mut ink = pen.ink_color32();
        let ink_response = ui
            .color_edit_button_srgba(&mut ink)
            .on_hover_text(t::pen_colour_tooltip());
        crate::diag::ui_rect(REGION_INK, ink_response.rect);
        if ink != pen.ink_color32() {
            pen.set_ink(ink);
            trace(*pen);
        }

        let mut highlighter = pen.highlighter_color32();
        let hl_response = ui
            .color_edit_button_srgba(&mut highlighter)
            .on_hover_text(t::highlighter_colour_tooltip());
        crate::diag::ui_rect(REGION_HIGHLIGHTER, hl_response.rect);
        if highlighter != pen.highlighter_color32() {
            pen.set_highlighter(highlighter);
            trace(*pen);
        }

        // ★ A `DragValue`, not a slider.
        //
        // The useful range is 0.25–12 pt and an operator authoring a comment on
        // a drawing usually has a specific width in mind — 0.5 to match the
        // drawing's own linework, 2 to sit above it — rather than a value they
        // want to explore. A drag value takes a typed number, which a slider
        // cannot, and costs a quarter of the ribbon width.
        //
        // The range is the PEN's, not a local literal, for the same reason the
        // settings window's sliders take the store's: a control narrower than
        // what the value may legally hold silently rewrites it.
        let before = pen.width_pts;
        let width_response = ui
            .add(
                egui::DragValue::new(&mut pen.width_pts)
                    .speed(0.1)
                    .range(MIN_WIDTH_PTS..=MAX_WIDTH_PTS)
                    .suffix(t::width_suffix()),
            )
            .on_hover_text(t::pen_width_tooltip());
        crate::diag::ui_rect(REGION_WIDTH, width_response.rect);
        if (pen.width_pts - before).abs() > f64::EPSILON {
            trace(*pen);
        }

        // ★★★ OPACITY, and it shipped four months after the row above it said
        // it could not.
        //
        // This module's header carried a table row reading *"blocked on the
        // engine … `/CA`, which `pdfcer-core` does not write yet — filed,
        // accepted, not started"*. It was true when written and stopped being
        // true on 2026-08-27, when `Pass 81.1` landed `MarkupOptions::opacity`
        // — in answer to a request this shell filed itself. The row was
        // corrected on 2026-08-28 rather than deleted, because the SHAPE of the
        // mistake is the useful part: **a blocker's reason is prose, and no test
        // can check prose.** This is the seventh stale blocker this project has
        // found, and the standing rule that produced the check is *a backlog row
        // is a record, not evidence*.
        //
        // ★★ A percentage at the control, a fraction in the file. `/CA` is
        // `0.0`–`1.0` (§12.5.2 Table 164) and every program that offers this
        // says 40%, so the conversion happens here and nowhere else — one
        // place, so a second call site cannot write 40.0 into a key whose legal
        // maximum is 1.0. The engine **refuses** that rather than clamping it,
        // which is the correct behaviour and not one an operator should ever
        // see the result of.
        let before = pen.opacity;
        let mut percent = pen.opacity * 100.0;
        let opacity_response = ui
            .add(
                egui::DragValue::new(&mut percent)
                    .speed(1.0)
                    .range((MIN_OPACITY * 100.0)..=100.0)
                    .suffix(t::opacity_suffix()),
            )
            .on_hover_text(t::pen_opacity_tooltip());
        crate::diag::ui_rect(REGION_OPACITY, opacity_response.rect);
        pen.opacity = (percent / 100.0).clamp(MIN_OPACITY, 1.0);
        if (pen.opacity - before).abs() > f64::EPSILON {
            trace(*pen);
        }
    });
}

/// One trace line per change, carrying the whole pen.
///
/// The whole pen rather than the field that moved, because what a harness needs
/// to assert is *what the next markup will be authored with* — and a line
/// carrying one field would need the reader to accumulate state across lines to
/// answer that. It is a handful of numbers.
fn trace(pen: Pen) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "markup-pen ink={:?} highlighter={:?} width_pts={} opacity={} ca={:?}",
            pen.ink,
            pen.highlighter,
            pen.width_pts,
            pen.opacity,
            // ★ BOTH, because they answer different questions and only the
            // second is a fact about the file: `opacity` is what the control
            // holds, and `ca` is whether a `/CA` key will be written at all.
            // A trace carrying only the first cannot distinguish "opaque, so no
            // key" from "the option was dropped on the way to the engine",
            // which is exactly the failure a driven check exists to catch.
            pen.opacity_option(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three regions are distinct.
    ///
    /// They exist so a harness can aim at one control out of three, and two
    /// that shared a name would send it to whichever the application declared
    /// last — a click on the wrong control, reported as the right one failing.
    #[test]
    fn the_three_controls_publish_distinct_regions() {
        let names = [REGION_INK, REGION_HIGHLIGHTER, REGION_WIDTH];
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                assert_ne!(names[i], names[j]);
            }
        }
    }
}
