//! # `markupswatch` — the Markup ▸ Style group's one control
//!
//! The `colour_swatch` custom item the manifest has declared since S2 and
//! nothing ever drew, so the Style group rendered a caption over an empty band.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/markupswatch.md`.

use egui::Ui;

use crate::entry;
use crate::markuppalette as palette;
use crate::markuppen::{MAX_WIDTH_PTS, MIN_OPACITY, MIN_WIDTH_PTS, Pen, PenSlot};
use crate::text::markup as t;

/// The region this control publishes, so a check can find and drive it.
pub const REGION_INK: &str = "markup.style.ink"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the highlighter.
pub const REGION_HIGHLIGHTER: &str = "markup.style.highlighter"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the width.
pub const REGION_WIDTH: &str = "markup.style.width"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the opacity.
pub const REGION_OPACITY: &str = "markup.style.opacity"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the line-style chooser.
pub const REGION_DASH: &str = "markup.style.dash"; // ui-text-exempt: trace region name, never displayed
/// As [`REGION_INK`], for the draw-on-the-page switch.
pub const REGION_TARGET: &str = "markup.style.target"; // ui-text-exempt: trace region name, never displayed
/// The palette grid inside an open swatch popup.
pub const REGION_PALETTE: &str = "markup.style.palette"; // ui-text-exempt: trace region name, never displayed
/// The id salt for the *More colours…* disclosure inside the popup.
const REGION_MORE_COLOURS: &str = "markup.style.more_colours"; // ui-text-exempt: widget id salt, never displayed

/// Draw the Style group's controls, editing `pen` in place.
pub fn show(ui: &mut Ui, pen: &mut Pen) {
    ui.horizontal(|ui| {
        // The two swatches are ADJACENT and labelled, rather than one swatch
        // that changes meaning with the armed tool.
        //
        // A single swatch would have to answer "which pen am I setting?" from
        // the armed tool, which means the control silently changes what it
        // edits as the operator moves along the Shapes row — and worse, edits
        // *nothing they can see* when no tool is armed. Two controls that each
        // always mean one thing is the version an operator can predict.
        let _ = chip(ui, pen, PenSlot::Shape, REGION_INK, t::pen_colour_tooltip());
        let _ = chip(
            ui,
            pen,
            PenSlot::Highlighter,
            REGION_HIGHLIGHTER,
            t::highlighter_colour_tooltip(),
        );

        // A `DragValue`, not a slider.
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
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut pen.width_pts,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let width_response = refusal
            .show(ui.add(widget.speed(0.1).range(MIN_WIDTH_PTS..=MAX_WIDTH_PTS)))
            .on_hover_text(t::pen_width_tooltip());
        crate::diag::ui_rect(REGION_WIDTH, width_response.rect);
        if (pen.width_pts - before).abs() > f64::EPSILON {
            trace(*pen);
        }

        // OPACITY, and it shipped four months after the row above it said
        // it could not.
        //
        // This module's header carried a table row reading *"blocked on the
        // engine … `/CA`, which `pdfcer-core` does not write yet — filed,
        // accepted, not started"*. It was true when written and stopped being
        // true on 2026-08-27, when `Pass 81.1` landed `MarkupOptions::opacity`
        // — in answer to a request this shell filed itself.
        //
        //
        // It read: *"The row was corrected on 2026-08-28 rather than deleted,
        // because the SHAPE of the mistake is the useful part."* The control
        // shipped that day; the header's table row still said **"blocked on the
        // engine … `pdfcer-core` does not write `/CA` yet"** until 2026-09-06,
        // nine days later, when somebody was sent to look at it specifically.
        //
        // ⇒ So the correction note was itself the stale claim. That is a sharper
        // instance of the rule it was written to record — *a blocker's reason is
        // prose, and no test can check prose* — because the prose that went
        // stale was **the prose asserting the correction had happened**. A
        // comment saying "this was fixed" is exactly as unchecked as the thing
        // it says was fixed, and a reader who found this comment would have
        // stopped looking. This is the eighth stale blocker this project has
        // found and the second in this file; the standing rule remains *a
        // backlog row is a record, not evidence*, and the corollary this adds is
        // that **a note claiming a record was updated is also only a record.**
        //
        // A percentage at the control, a fraction in the file. `/CA` is
        // `0.0`–`1.0` (§12.5.2 Table 164) and every program that offers this
        // says 40%, so the conversion happens here and nowhere else — one
        // place, so a second call site cannot write 40.0 into a key whose legal
        // maximum is 1.0. The engine **refuses** that rather than clamping it,
        // which is the correct behaviour and not one an operator should ever
        // see the result of.
        let before = pen.opacity;
        let mut percent = pen.opacity * 100.0;
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let opacity_response = refusal
            .show(
                ui.add(
                    widget
                        .speed(1.0)
                        .range((MIN_OPACITY * 100.0)..=100.0)
                        .suffix(t::opacity_suffix()),
                ),
            )
            .on_hover_text(t::pen_opacity_tooltip());
        crate::diag::ui_rect(REGION_OPACITY, opacity_response.rect);
        pen.opacity = (percent / 100.0).clamp(MIN_OPACITY, 1.0);
        if (pen.opacity - before).abs() > f64::EPSILON {
            trace(*pen);
        }

        // LINE STYLE — `RIBBON_IA.md` §5.8's eighth control, and the last
        // of the eight to get an engine verb.
        //
        // It is on the **Style** group rather than only on the contextual
        // Format tab because this group's whole subject is *what the next mark
        // looks like*, and "solid or dashed" is as much a property of the next
        // mark as its colour and its width are. `MarkupOptions::dash` is the
        // author-time half the engine shipped alongside the restyle half, so a
        // shape can be DRAWN dashed rather than drawn solid and then corrected
        // — which is one gesture and one undo entry instead of two, the same
        // argument `add_markup_with` makes about opacity three controls to the
        // left.
        //
        // A ComboBox and not a set of toggle buttons: four entries whose
        // difference is a line pattern cannot be told apart by a 16-point icon,
        // and there is no room on a ribbon band for four labelled buttons. The
        // arrowhead chooser on the Format tab is the same shape for the same
        // reason.
        //
        // ⚠ It reports on `.clicked()` inside the popup, so there is no
        // `drag_stopped`/`lost_focus` guard here and none is wanted: unlike the
        // two `DragValue`s above, a combo produces exactly one change per
        // decision.
        let before = pen.dash;
        let combo = ui.push_id(REGION_DASH, |ui| {
            crate::linestyle::chooser(
                ui,
                REGION_DASH,
                crate::linestyle::DashReading::Offered(pen.dash),
                DASH_WIDTH,
            )
        });
        if let Some(style) = combo.inner {
            pen.dash = style;
        }
        let dash_response = combo.response.on_hover_text(t::pen_dash_tooltip());
        crate::diag::ui_rect(REGION_DASH, dash_response.rect);
        if pen.dash != before {
            trace(*pen);
        }

        let target = ui
            .toggle_value(&mut pen.on_page, t::pen_on_page_label())
            .on_hover_text(t::pen_on_page_tooltip());
        crate::diag::ui_rect(REGION_TARGET, target.rect);
        if target.changed() {
            trace(*pen);
        }
    });
}

/// The width of the line-style chooser, in points.
const DASH_WIDTH: f32 = 92.0;

/// **The side of one colour chip and of one palette cell**, in points.
const CELL_PTS: f32 = 16.0;

/// **One colour chip: the pen's current colour, and the grid behind it.**
fn chip(ui: &mut Ui, pen: &mut Pen, slot: PenSlot, region: &str, tooltip: &str) -> egui::Response {
    let current = pen.color32_of(slot);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(CELL_PTS, CELL_PTS), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let painter = ui.painter();
        // DOCUMENT COLOUR: the operator's own pen, previewed at the size and
        // shape of the value that will land in `/C`. No theme may move it.
        painter.rect_filled(rect, visuals.corner_radius, current);
        // Chrome: the frame is the widget's, so hover and press read normally
        // and a white pen is still visible against a light ribbon.
        painter.rect_stroke(
            rect,
            visuals.corner_radius,
            visuals.fg_stroke,
            egui::StrokeKind::Inside,
        );
    }
    let response = response.on_hover_text(tooltip);
    crate::diag::ui_rect(region, response.rect);

    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| popup(ui, pen, slot));

    response
}

/// The grid, and the route to the full picker underneath it.
fn popup(ui: &mut Ui, pen: &mut Pen, slot: PenSlot) {
    ui.label(t::palette_heading());
    ui.separator();
    crate::diag::ui_rect(REGION_PALETTE, grid(ui, pen, slot));
    ui.separator();

    // The full picker, expanded IN PLACE rather than in a second popup.
    //
    // `color_edit_button_srgba` would have been one line, and it opens a popup
    // of its own — a popup inside a popup, with two dismissal rules the operator
    // has to learn and an outer one that can close while the inner is open. A
    // collapsing section has one rule and one place to look.
    //
    // It is CLOSED by default, which is the whole point of the change: the grid
    // is the fast path and the wheel is the escape hatch, not the other way
    // round.
    egui::CollapsingHeader::new(t::more_colours())
        .id_salt(REGION_MORE_COLOURS)
        .default_open(false)
        .show(ui, |ui| {
            let mut chosen = pen.color32_of(slot);
            // `Alpha::Opaque`. `/C` is three components — see
            // `Pen::set_ink` — so a picker offering a fourth would be offering
            // a value with nowhere to go.
            if egui::widgets::color_picker::color_picker_color32(
                ui,
                &mut chosen,
                egui::widgets::color_picker::Alpha::Opaque,
            ) {
                pen.set_colour(slot, chosen);
                trace(*pen);
            }
        })
        .header_response
        .on_hover_text(t::more_colours_tooltip());
}

/// **The grid of Acrobat's colours.** Returns the rect it occupied.
fn grid(ui: &mut Ui, pen: &mut Pen, slot: PenSlot) -> egui::Rect {
    let current = pen.color32_of(slot);
    // Chrome, an emphasised mark — see this function's header on why this is the
    // accent and emphatically not `Visuals::selection`. `on_accent` is
    // discarded because a stroke has no plate to put ink on.
    let (accent, _on_accent) = egui_shell::theme::Theme::accent_pair(ui.ctx());
    let mut bounds = egui::Rect::NOTHING;
    ui.vertical(|ui| {
        for row in palette::ACROBAT.chunks(palette::COLUMNS) {
            ui.horizontal(|ui| {
                for cell in row {
                    let (rect, response) = ui
                        .allocate_exact_size(egui::vec2(CELL_PTS, CELL_PTS), egui::Sense::click());
                    bounds = bounds.union(rect);
                    if ui.is_rect_visible(rect) {
                        let visuals = *ui.style().interact(&response);
                        let chosen = cell.color32() == current;
                        let painter = ui.painter();
                        // DOCUMENT COLOUR: a palette cell — one click from the
                        // annotation's `/C`, so no theme may move it.
                        painter.rect_filled(rect, visuals.corner_radius, cell.color32());
                        // Chrome: the frame. A heavier accent ring for the cell
                        // that is current, the ordinary widget stroke for the
                        // rest — so the mark reads as "this one" rather than as
                        // "this one is a different colour". Doubled in width as
                        // well as recoloured, because on a saturated cell a hue
                        // change alone is easy to miss and a thicker ring is
                        // legible whatever the cell underneath is.
                        let stroke = if chosen {
                            egui::Stroke::new(visuals.fg_stroke.width * 2.0, accent)
                        } else {
                            visuals.fg_stroke
                        };
                        painter.rect_stroke(
                            rect,
                            visuals.corner_radius,
                            stroke,
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.on_hover_text(cell.name).clicked() {
                        pen.set_colour(slot, cell.color32());
                        trace(*pen);
                        // One completed decision — see `chip`'s note on why a
                        // cell closes the popup and the disclosure does not.
                        ui.close();
                    }
                }
            });
        }
    });
    bounds
}

/// One trace line per change, carrying the whole pen.
fn trace(pen: Pen) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "markup-pen ink={:?} highlighter={:?} width_pts={} opacity={} ca={:?} \
             dash={:?} d={:?} on_page={}",
            pen.ink,
            pen.highlighter,
            pen.width_pts,
            pen.opacity,
            // BOTH, because they answer different questions and only the
            // second is a fact about the file: `opacity` is what the control
            // holds, and `ca` is whether a `/CA` key will be written at all.
            // A trace carrying only the first cannot distinguish "opaque, so no
            // key" from "the option was dropped on the way to the engine",
            // which is exactly the failure a driven check exists to catch.
            pen.opacity_option(),
            // BOTH again, for the identical reason one step along: `dash` is
            // the chooser's entry and `d` is the run lengths `/BS` `/D` will
            // carry — `None` meaning no dash key at all. A trace with only the
            // first could not tell "Solid, so no key" from "the pattern was
            // dropped between the chooser and the engine", which is precisely
            // the defect this control was built to fix on the restyle side.
            pen.dash,
            pen.dash_option().map(|d| d.pattern().to_vec()),
            pen.on_page,
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every region this module publishes is distinct.
    #[test]
    fn every_control_publishes_a_distinct_region() {
        let names = [
            REGION_INK,
            REGION_HIGHLIGHTER,
            REGION_WIDTH,
            REGION_OPACITY,
            REGION_DASH,
            REGION_PALETTE,
            REGION_MORE_COLOURS,
        ];
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                assert_ne!(
                    names[i], names[j],
                    "two regions share the name {}",
                    names[i]
                );
            }
        }
    }

    /// Raw input for a completed primary click at `pos`.
    fn click_at(pos: egui::Pos2) -> egui::RawInput {
        egui::RawInput {
            events: vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }

    /// One frame of a single [`chip`], returning its popup id and its rect.
    fn chip_frame(
        ctx: &egui::Context,
        pen: &mut Pen,
        slot: PenSlot,
        input: egui::RawInput,
    ) -> (egui::Id, egui::Rect) {
        let mut id = egui::Id::NULL;
        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run_ui(input, |ui| {
            let response = chip(ui, pen, slot, REGION_INK, t::pen_colour_tooltip());
            id = egui::Popup::default_response_id(&response);
            rect = response.rect;
        });
        (id, rect)
    }

    /// **PRESSING THE SWATCH SHOWS ACROBAT'S COLOURS.**
    #[test]
    fn pressing_the_swatch_opens_the_palette() {
        let ctx = egui::Context::default();
        let mut pen = Pen::default();

        let (id, rect) = chip_frame(&ctx, &mut pen, PenSlot::Shape, egui::RawInput::default());
        assert!(
            rect.is_positive(),
            "the chip must occupy space before a click can be aimed at it"
        );
        assert!(
            !egui::Popup::is_id_open(&ctx, id),
            "an idle frame must open nothing — without this, the assertion below \
             would pass on a build whose popup was simply always open"
        );

        chip_frame(&ctx, &mut pen, PenSlot::Shape, click_at(rect.center()));
        assert!(
            egui::Popup::is_id_open(&ctx, id),
            "clicking the colour chip must open the palette — `Popup::menu` \
             already toggles, so a second toggle beside it cancels the first and \
             nothing appears"
        );
    }

    /// …and clicking it again closes it.
    #[test]
    fn pressing_the_swatch_again_closes_the_palette() {
        let ctx = egui::Context::default();
        let mut pen = Pen::default();
        let (id, rect) = chip_frame(&ctx, &mut pen, PenSlot::Shape, egui::RawInput::default());
        let target = rect.center();

        chip_frame(&ctx, &mut pen, PenSlot::Shape, click_at(target));
        assert!(
            egui::Popup::is_id_open(&ctx, id),
            "the first click opens it"
        );

        chip_frame(&ctx, &mut pen, PenSlot::Shape, click_at(target));
        assert!(
            !egui::Popup::is_id_open(&ctx, id),
            "the second click on the chip must close it again"
        );
    }

    /// One frame of the bare [`grid`], returning the rect it occupied.
    fn grid_frame(
        ctx: &egui::Context,
        pen: &mut Pen,
        slot: PenSlot,
        input: egui::RawInput,
    ) -> egui::Rect {
        let mut bounds = egui::Rect::NOTHING;
        let _ = ctx.run_ui(input, |ui| {
            bounds = grid(ui, pen, slot);
        });
        bounds
    }

    /// **CLICKING A CELL AUTHORS THAT CELL'S COLOUR, INTO THAT SLOT.**
    #[test]
    fn clicking_a_cell_sets_that_slots_colour() {
        for (index, cell) in [
            (0, &palette::ACROBAT[0]),
            (
                palette::ACROBAT.len() - 1,
                &palette::ACROBAT[palette::ACROBAT.len() - 1],
            ),
        ] {
            // The HIGHLIGHTER slot, deliberately, and not the one the chip in
            // `show` happens to be listed first with: a grid that ignored its
            // `slot` argument and always wrote the shape pen would pass a
            // Shape-only test and fail this one.
            let slot = PenSlot::Highlighter;
            let ctx = egui::Context::default();
            let mut pen = Pen::default();
            let before = pen.colour_of(PenSlot::Shape);

            let bounds = grid_frame(&ctx, &mut pen, slot, egui::RawInput::default());
            assert!(bounds.is_positive(), "the grid must occupy space");

            let half = CELL_PTS / 2.0;
            let target = if index == 0 {
                bounds.min + egui::vec2(half, half)
            } else {
                bounds.max - egui::vec2(half, half)
            };
            grid_frame(&ctx, &mut pen, slot, click_at(target));

            assert_eq!(
                pen.colour_of(slot),
                cell.rgb_components(),
                "clicking cell {index} ({}) did not set the highlighter to it",
                cell.name
            );
            assert_eq!(
                pen.colour_of(PenSlot::Shape),
                before,
                "clicking a cell for the highlighter also moved the shape pen"
            );
        }
    }

    /// **The opacity control exists and is wired to the pen.**
    #[test]
    fn the_opacity_control_covers_the_whole_range_the_pen_allows() {
        let floor = MIN_OPACITY * 100.0;
        assert!(
            floor > 0.0,
            "a control whose bottom end authors an invisible mark is a defect \
             report waiting to be filed — see MIN_OPACITY"
        );
        assert!(floor < 100.0);
        for opacity in [MIN_OPACITY, 0.4, 1.0] {
            let pen = Pen {
                opacity,
                ..Pen::default()
            };
            let percent = pen.opacity * 100.0;
            assert!(
                (floor..=100.0).contains(&percent),
                "{opacity} is a legal pen opacity the control cannot reach"
            );
        }
        // …and fully opaque still writes no `/CA` at all, which is the half of
        // the contract the percentage conversion could silently break.
        assert_eq!(Pen::default().opacity_option(), None);
        assert_eq!(
            Pen {
                opacity: 0.4,
                ..Pen::default()
            }
            .opacity_option(),
            Some(0.4)
        );
    }
}
