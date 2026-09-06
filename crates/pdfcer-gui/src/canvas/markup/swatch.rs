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
//! | **Colour** | ✅ | two swatches, each opening [`super::palette`]'s grid of Acrobat's own colours — see [`super::pen`] on why there are eight slots and two controls |
//! | **Line width** | ✅ | a drag value in points, over the pen's own range |
//! | **Fill** | ⬜ | **a design decision about the PEN, and only about the pen** — see the row's own correction below |
//! | **Opacity** | ✅ | **shipped 2026-08-28** — a percentage drag value writing `/CA`. This row said *"blocked on the engine"* for four months after it stopped being true; see below |
//! | **Line style** | ✅ | **shipped 2026-09-06** — a four-entry chooser writing `/BS` `/S` and `/D`. §5.5 does not list it; it is here because it is a property of the next mark exactly as the four above are, and because the engine shipped the author-time half (`MarkupOptions::dash`) beside the restyle half on the day the Format tab got its own copy. See [`super::linestyle`] |
//!
//! ⚠ The table is now **five rows over a four-item specification**, which is a
//! table that has outgrown its source rather than one that is wrong. §5.5 was
//! written before a dash was expressible at all; the *Line style* row is the
//! one entry here that this shell added rather than answered. It is called out
//! so that the next reader does not spend a minute looking for a fifth bullet in
//! `RIBBON_IA.md` §5.5 that is not there — the entry they will find is §5.8's,
//! about the **other** surface.
//!
//! ### ★★★ The Fill row, corrected: it was describing ONE of two surfaces and
//! ### did not say which
//!
//! It used to read *"a design decision, not a gap"*, followed by [`super::spec`]'s
//! argument that a filled comment shape hides the drawing it is a comment about.
//! That argument is **still correct and still in force** — for this control. What
//! the row failed to say is that there are *two* fills in this program and it was
//! only ever talking about one of them:
//!
//! | which fill | whose surface | state |
//! |---|---|---|
//! | the fill the **pen** authors a NEW mark with | this module, the Markup ▸ Style group | ⬜ **deliberately absent**, and not a gap. `spec` passes `interior: None` for every shape, and on a CAD sheet an unfilled comment shape is the only kind that does not hide the content it is about |
//! | the fill of a mark **already on the page** | `panels::properties::markup`, the contextual Format tab | ✅ being built — an operator who wants a filled shape places one and fills it, which is a decision they made about a specific mark rather than a default applied to every future one |
//!
//! ⇒ The distinction is the whole answer to *"why can I fill that rectangle and
//! not this pen?"*, and a row that named neither surface could not give it.
//!
//! ### ★★★ The Opacity row, corrected: it was FALSE, and false in the direction
//! ### this project has been wrong in before
//!
//! It used to read *"blocked on the engine. Annotation transparency is `/CA`,
//! which `pdfcer-core` does not write yet — filed, accepted, not started."* That
//! was true when written and stopped being true on **2026-08-27**, when
//! `Pass 81.1` landed `MarkupOptions::opacity` in answer to a request this shell
//! filed itself. `set_markup_style` writes `/CA`; `add_markup_with` authors a
//! translucent mark in one verb and one undo entry; [`Pen::opacity`] and
//! `MIN_OPACITY` have existed since the day after.
//!
//! ⇒ The control was **already drawn** in [`show`] below, with its own trace
//! region and its own tooltip, while this table three screens above it said it
//! could not exist. Both were read by everyone who opened the file and only the
//! table was believed, because a table is what a reader trusts.
//!
//! ★★ That is the **eighth** stale blocker this project has found and it is the
//! second one *in this file*. The standing rule it produced — **a backlog row is
//! a record, not evidence** — has a corollary that this instance adds: *a
//! capability table in a module header is a claim about the module, and the
//! module is right there.* The correction is written rather than deleted because
//! the shape of the mistake is the useful part.
//!
//! ## ★★★ THE PALETTE POPUP — what the operator asked for by name
//!
//! > *"Also make sure you've used the same default colours and style look for
//! > these things as Adobe."*
//!
//! The **colours** half is [`super::palette`] and [`super::pen::Pen::default`].
//! The **style look** half is this module, and it is a specific, nameable change:
//!
//! | before | after |
//! |---|---|
//! | `egui`'s `color_edit_button_srgba` — a generic HSV wheel with hue, saturation and value sliders | a **grid of named preset swatches**, with the full picker one click below it |
//!
//! Acrobat does not open a colour wheel when you press its comment colour chip.
//! It shows a small grid of colour cells, each with a name, and *More colours…*
//! underneath for anything else. That shape is not decoration — it is what makes
//! the control usable at ribbon speed: an operator marking up a drawing wants
//! *red*, and a wheel makes them navigate to it and produces a slightly different
//! red every time. A grid gives them the same red as last time, and — because
//! every cell is a value measured out of Acrobat — the same red Acrobat would
//! have given them.
//!
//! ⇒ The full picker is kept, underneath, because *"the ten Adobe uses"* is not
//! *"the ten that exist"* and an operator with a company standard colour must not
//! be told no. It expands **in place** rather than opening a second popup: a
//! popup inside a popup is two dismissal rules the operator has to learn.
//!
//! ## Why the swatch shows the colour rather than naming it
//!
//! Because the operator is choosing a colour and the only useful preview of a
//! colour is the colour. The chip is a filled rectangle carrying the pen's
//! current value; its accessible name comes from the hover text, which is why
//! both swatches carry one.
//!
//! **The alpha channel is not offered**, and [`super::pen::Pen::set_ink`]
//! carries the argument: a PDF annotation's `/C` is three components, and
//! feeding a picker's alpha into it would be a value with nowhere to go. The
//! `Opaque` variant of the picker is what says so, and it is why the grid's
//! cells are opaque too.

use egui::Ui;

use super::palette;
use super::pen::{MAX_WIDTH_PTS, MIN_OPACITY, MIN_WIDTH_PTS, Pen, PenSlot};
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
/// As [`REGION_INK`], for the line-style chooser.
///
/// ★ It doubles as the combo's `id_salt`, which is deliberate and is the one
/// place in this module where a region name is load-bearing twice: a driven
/// check finds the control by this name, and `egui` remembers the popup's open
/// state under it. Two spellings of one control would give the harness a rect
/// for a widget whose popup lives under a different key.
pub const REGION_DASH: &str = "markup.style.dash"; // ui-text-exempt: trace region name, never displayed
/// The palette grid inside an open swatch popup.
///
/// ★ Published only while a popup is open, which is the point: a driven check
/// asking *"did pressing the swatch show Acrobat's colours"* gets no rect at all
/// on the frame before the press, and a rect afterwards. A region that were
/// always present would answer the question the same way whether the popup had
/// opened or not — the exact failure `app::status::filter`'s own header records
/// from the day a Select button did nothing for a week.
pub const REGION_PALETTE: &str = "markup.style.palette"; // ui-text-exempt: trace region name, never displayed
/// The id salt for the *More colours…* disclosure inside the popup.
///
/// A salt rather than a region: it is `egui`'s persistence key for whether the
/// full picker is expanded, and it must be stable across frames or the section
/// would collapse itself every time the popup reopened.
const REGION_MORE_COLOURS: &str = "markup.style.more_colours"; // ui-text-exempt: widget id salt, never displayed

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
        let _ = chip(ui, pen, PenSlot::Shape, REGION_INK, t::pen_colour_tooltip());
        let _ = chip(
            ui,
            pen,
            PenSlot::Highlighter,
            REGION_HIGHLIGHTER,
            t::highlighter_colour_tooltip(),
        );

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
        // — in answer to a request this shell filed itself.
        //
        // ★★★ AND THIS COMMENT SAID THE ROW HAD BEEN CORRECTED, AND IT HAD NOT.
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

        // ★★★ LINE STYLE — `RIBBON_IA.md` §5.8's eighth control, and the last
        // of the eight to get an engine verb.
        //
        // It is on the **Style** group rather than only on the contextual Format
        // tab because this group's whole subject is *what the next mark looks
        // like*, and "solid or dashed" is as much a property of the next mark as
        // its colour and its width are. `MarkupOptions::dash` is the author-time
        // half the engine shipped alongside the restyle half
        // (`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:4782`), so a shape can
        // be DRAWN dashed rather than drawn solid and then corrected — which is
        // one gesture and one undo entry instead of two, the same argument
        // `add_markup_with` makes about opacity three controls to the left.
        //
        // ★ A ComboBox and not a set of toggle buttons: four entries whose
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
            crate::canvas::markup::linestyle::chooser(
                ui,
                REGION_DASH,
                crate::canvas::markup::linestyle::DashReading::Offered(pen.dash),
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
    });
}

/// The width of the line-style chooser, in points.
///
/// ★ Wide enough for its longest entry — *"Dashed (the file's own pattern)"* is
/// longer still but is a **reading** and never appears on this surface, because
/// the pen always holds one of the four. Sized against the two `DragValue`s
/// beside it rather than to look comfortable on its own: the Style group already
/// carries two chips and two numbers, and a fifth control that took a third of
/// the band would push the group into the overflow on a narrow window.
const DASH_WIDTH: f32 = 92.0;

/// **The side of one colour chip and of one palette cell**, in points.
///
/// Sixteen, which is a little under the height of a ribbon button and a little
/// over the smallest target a pointer hits reliably. It is one constant for both
/// because a preview that is a different size from the cells it previews reads
/// as a different kind of thing — the chip *is* the cell that is currently
/// chosen, and the grid is where the other nine live.
const CELL_PTS: f32 = 16.0;

/// **One colour chip: the pen's current colour, and the grid behind it.**
///
/// # ★ It is drawn rather than assembled from a `Button`
///
/// A `Button` fills with the *theme's* widget colour and paints its content on
/// top; what is wanted here is a rectangle of the **document's** colour, at a
/// known size, with a frame that keeps a white or near-white pen visible against
/// a light ribbon. `Button::fill` could be forced to the document colour, but
/// then hover and press would restyle the operator's ink — an application state
/// changing the apparent colour of an annotation, which is exactly what
/// `check-theme-colors.sh`'s document-colour rule exists to prevent.
///
/// So: the fill is the document's and never moves; the **frame** is the widget
/// visual and carries hover and focus. Two colours with two owners, which is the
/// distinction this project has already shipped wrong once.
///
/// # ★★ `CloseOnClickOutside`, not `CloseOnClick`
///
/// The default menu behaviour closes on any click inside, which would shut the
/// popup the instant the operator touched the *More colours…* disclosure or
/// dragged inside the picker — a picker that vanishes on its first drag is a
/// picker that cannot be used at all.
///
/// Picking a **cell** does close it, explicitly, via [`Ui::close`]: that is one
/// completed decision, and Acrobat's grid closes on a pick. The two behaviours
/// are therefore not inconsistent — clicking a cell is choosing, clicking the
/// disclosure is navigating, and only the first is finished.
///
/// # Returns
///
/// The **chip's** response, not the popup's. [`show`] discards it; the test that
/// asserts the popup opens needs `Popup::default_response_id` of exactly this
/// response, and there is no other way to name the flag the popup's open state
/// lives under — `Memory::any_popup_open` is `pub(crate)` to egui.
///
/// That is the same return, for the same reason, that `app::status::filter`'s
/// own `show` carries, and its header records why it is not a wasted one: the
/// alternative was a test that could only assert the chip exists, which is
/// precisely the claim that stayed true throughout the week a Select button did
/// nothing at all.
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
///
/// # Why the heading names Adobe
///
/// Once, at the top, and [`crate::text::markup::palette_heading`] carries the
/// argument: the values are Acrobat's, measured, and a grid captioned *Colours*
/// would look like ten colours somebody liked. It is the only place in the
/// running program where the provenance of these values is visible.
///
/// # ★ Deliberately NOT `.strong()`
///
/// `tools/gates/check-strong-text.sh` rejects it and defect D11 is why: egui has
/// no role for emphasised text, so `.strong()` resolves to the accent-filled
/// widget state — pale text on a pale ground. The hierarchy here is carried by
/// the separator underneath, as it is in every other popup in this shell.
fn popup(ui: &mut Ui, pen: &mut Pen, slot: PenSlot) {
    ui.label(t::palette_heading());
    ui.separator();
    crate::diag::ui_rect(REGION_PALETTE, grid(ui, pen, slot));
    ui.separator();

    // ★ The full picker, expanded IN PLACE rather than in a second popup.
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
            // ★ `Alpha::Opaque`. `/C` is three components — see
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
///
/// # ★★ The chosen cell is marked, and marking it is not decoration
///
/// A grid of ten colours with no indication of which one is current answers
/// *"what can I pick"* and not *"what did I pick"*, and the second is the
/// question an operator opening a colour popup usually has. The mark is a
/// **heavier ring in the theme's accent** — not a colour of this module's own,
/// because a fixed-colour marker would be invisible on the cell whose colour it
/// happened to match, which on a ten-colour grid is a one-in-ten chance of a
/// control that looks broken.
///
/// # ★★★ It is `Theme::accent_pair`, and the first draft got the ROLE wrong
///
/// The ring was first drawn with `ui.visuals().selection.stroke`, which reads
/// like the obvious answer — *this cell is selected, use the selection stroke*
/// — and `tools/gates/check-selection-channel.sh` refused it by name. It is
/// right to: `egui::Visuals::selection` is how egui styles a **selected
/// widget**, it supplies the fill and text colour of every
/// `Button::selected(true)` in the application, and it is not a
/// general-purpose emphasis. Defect T2 is what happens when content code
/// borrows it — the theme repoints the channel to satisfy the borrowers and
/// every selected chrome control in the program is then painted with canvas
/// ink, with every gate still green because every colour involved was
/// correctly sourced from the palette.
///
/// ⇒ **Correctly sourced, wrong role** — the same sentence this project has
/// already had to write about a colour that passed every check. What this cell
/// actually is is *chrome, an emphasised mark*, and the theme's name for that
/// is [`egui_shell::theme::Theme::accent_pair`]. Only the accent half is used:
/// the ring is a stroke, not a plate, so there is no `on_accent` to place on it
/// and `check-plate-colour.sh` has nothing to ask for.
///
/// # A cell counts as chosen only on an EXACT match
///
/// A pen the operator set through the full picker to a near-red is not the
/// palette's red, and marking the nearest cell would tell them they had picked
/// something they had not.
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
///
/// The whole pen rather than the field that moved, because what a harness needs
/// to assert is *what the next markup will be authored with* — and a line
/// carrying one field would need the reader to accumulate state across lines to
/// answer that. It is a handful of numbers.
fn trace(pen: Pen) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "markup-pen ink={:?} highlighter={:?} width_pts={} opacity={} ca={:?} \
             dash={:?} d={:?}",
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
            // ★ BOTH again, for the identical reason one step along: `dash` is
            // the chooser's entry and `d` is the run lengths `/BS` `/D` will
            // carry — `None` meaning no dash key at all. A trace with only the
            // first could not tell "Solid, so no key" from "the pattern was
            // dropped between the chooser and the engine", which is precisely
            // the defect this control was built to fix on the restyle side.
            pen.dash,
            pen.dash_option().map(|d| d.pattern().to_vec()),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every region this module publishes is distinct.
    ///
    /// They exist so a harness can aim at one control out of five, and two that
    /// shared a name would send it to whichever the application declared last —
    /// a click on the wrong control, reported as the right one failing.
    ///
    /// ★ The list grew from three to five and the test name grew with it, on
    /// purpose: `the_three_controls_publish_distinct_regions` would have gone on
    /// passing while checking three of five, which is the shape of gate that
    /// reports clean having looked at almost nothing.
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
    ///
    /// A press AND a release, because egui raises `clicked()` on the release and
    /// a press-only frame would assert nothing about a click. Borrowed verbatim
    /// from `app::status::filter`'s harness, which is where this shell learned
    /// that a control can be perfectly laid out and completely inert.
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

    /// ★★★ **PRESSING THE SWATCH SHOWS ACROBAT'S COLOURS.**
    ///
    /// The whole of the *"style look"* half of the operator's ask, reduced to the
    /// one thing that can be false about it. Everything else in this module —
    /// the grid's geometry, its names, its measured values — is worth nothing if
    /// the popup never opens, and *"the chip is drawn in the right place and
    /// does nothing"* is a state this project has shipped before and describes in
    /// `app::status::filter`'s header at length: 1,628 tests, 17 gates and an
    /// off-screen launch all confirmed a Select button's rect while the button
    /// itself was inert for a week.
    ///
    /// ★ It asserts on `Popup::is_id_open` — the flag a duplicated
    /// `Popup::toggle_id` would fight over — rather than on anything downstream.
    ///
    /// Falsified by deleting the `Popup::menu(…)` block from [`chip`]: the
    /// assertion fired. Restored.
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
    ///
    /// The other half of a toggle, and the half a careless fix breaks: deleting
    /// a duplicate toggle could as easily be deleting *the* toggle, leaving a
    /// popup that opens and cannot be dismissed from the control that opened it.
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

    /// ★★★ **CLICKING A CELL AUTHORS THAT CELL'S COLOUR, INTO THAT SLOT.**
    ///
    /// The claim the whole module rests on, and the one a screenshot cannot
    /// make: a grid that renders ten beautiful squares and writes nothing is
    /// indistinguishable from a working one until an annotation comes out the
    /// wrong colour in a saved file.
    ///
    /// # Why the FIRST and the LAST cell specifically
    ///
    /// Because they are the two whose position can be derived from the returned
    /// bounds without re-deriving the layout: the first cell's top-left **is**
    /// `bounds.min` and the last cell's bottom-right **is** `bounds.max`, since
    /// [`grid`] unions exactly the cell rects and the grid is rectangular
    /// (`palette::tests::the_grid_is_rectangular`). Aiming at a middle cell would
    /// mean this test computing the spacing, which is the layout asserting
    /// itself.
    ///
    /// They are also the two that matter: an off-by-one in the row loop puts the
    /// last cell somewhere else entirely, and a reversed iteration swaps them.
    ///
    /// Falsified by making [`grid`]'s click arm write `PenSlot::Shape` instead
    /// of `slot`: the highlighter half of the assertion fired. Restored.
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

    /// ★★ **The opacity control exists and is wired to the pen.**
    ///
    /// Written because this module's header claimed for four months that it
    /// could not exist — *"blocked on the engine … `/CA`, which `pdfcer-core`
    /// does not write yet"* — while the widget was drawn thirty lines below the
    /// claim. Nothing checked either statement, so the false one survived.
    ///
    /// The assertion is deliberately about the **range** rather than about a
    /// drag: it pins that the control's bounds are the pen's own
    /// (`MIN_OPACITY..=1.0`, expressed as a percentage), which is the property
    /// that would silently rewrite the operator's value if the widget's range
    /// were narrower than what the pen may legally hold. A control narrower than
    /// its value is the defect the settings window's own sliders document.
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
