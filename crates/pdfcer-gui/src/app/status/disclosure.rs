//! # `app::status::disclosure` — the three rule-4 lines in the status bar
//!
//!
//! > The left half carries four things, and only the first is the narrator. The
//! > others look similar and are governed by different rules.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/disclosure.md`.

use egui::{Align, Layout, Vec2};

use super::{
    NOTES_WIDTH_FRACTION, REGION_BLEND_SPACE, REGION_CATCHING_UP, REGION_EDIT_DISCLOSURE,
    REGION_FILL_DISCLOSURE, REGION_LINE_WEIGHTS, REGION_LOAD_ANOMALIES, REGION_RECOVERED,
    REGION_SPOTS_FLATTENED, ROW_HEIGHT_PTS,
};
use crate::app::state::OpenDoc;
use crate::text::forms as t_forms;
use crate::text::status as t;

/// What the last fill **inferred**, in the bar, until the document moves on.
fn fill_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(d) = crate::panels::forms::edit::last_fill_disclosure(doc.edit_epoch) else {
        return;
    };

    // Both can be true of one fill. Joined rather than shown as two lines,
    // because two lines is two rows, which is the R128 loop.
    let mut line = String::new();
    if let Some(size) = d.applied_autosize {
        //
        // Until today this arm called `forms_fill_autosize_note` for every
        // outcome, so the operator got *"pdfcer chose 6.0 pt"* in the one case
        // where 6.0 pt **does not fit** and the text is going to overflow the
        // box. `AutoFitBound::Floor` is the engine naming exactly that — its
        // comment at the branch reads *"the one case where the returned size
        // does NOT fit the constraint that produced it"* — and this shell was
        // throwing the distinction away while `OPERATOR_REQUESTS.md` O86 told
        // the operator, under a ✅, that pdfcer reports it.
        //
        // ⚠ `None` means NO bound was decided, which is a real state and not a
        // missing one: a multiline field keeps the engine's older whole-box
        // route, and the engine declines to name a bound there because that
        // *"would report a constraint that was never evaluated"*. It takes the
        // general sentence, which is true of it.
        //
        // A missing bound is not a `Height`. That is the same mistake as
        // reading a missing texture as zero thinned strokes (`O137`), and it is
        // written here because this arm is where somebody would make it.
        use pdfcer_core::vartext::AutoFitBound as Bound;
        line.push_str(&match d.applied_autosize_bound {
            Some(Bound::Floor) => t_forms::forms_fill_autosize_overflow_note(&d.field, size),
            Some(Bound::Width) => t_forms::forms_fill_autosize_width_note(&d.field, size),
            // `Height` is the ordinary case and keeps the ordinary sentence.
            //
            // ⚠ **`AutoFitBound` IS `#[non_exhaustive]`** — checked, after a
            // first draft of `bound_token` asserted the opposite because its
            // grep read the `#[derive]` line and the attribute is on the line
            // after it. So `Some(_)` is genuinely reachable, not a formality.
            //
            // A bound this build has never met joins `Height` and `None` and
            // gets the sentence that is true of **every** auto-size — *"pdfcer
            // chose N pt; another program may choose differently"* — rather
            // than a claim about a constraint it cannot name. That is the
            // conservative direction: the general sentence under-informs, and
            // the two specific ones would be assertions about the operator's
            // document made from a value this build cannot read.
            //
            // ⇒ The thing that will notice a new variant is
            // `check-engine-api-drift`, not the compiler. Said plainly here
            // because a comment claiming otherwise is what this evening kept
            // finding.
            Some(Bound::Height) | None | Some(_) => {
                t_forms::forms_fill_autosize_note(&d.field, size)
            }
        });
    }
    if d.unencodable_chars > 0 {
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&t_forms::forms_fill_unencodable_note(
            &d.field,
            d.unencodable_chars,
        ));
    }
    if d.password_withheld.is_some() {
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&t_forms::forms_fill_password_withheld_note(&d.field));
    }
    if line.is_empty() {
        return;
    }

    disclosure_line(ui, REGION_FILL_DISCLOSURE, &line);
}

/// What the last **vector edit** disclosed, in the bar, until the document
/// moves on.
fn edit_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(d) = crate::app::actions::last_edit_disclosure(doc.edit_epoch) else {
        return;
    };
    disclosure_line(
        ui,
        REGION_EDIT_DISCLOSURE,
        &t::edit_disclosure_line(&d.notes),
    );
}

/// **The picture is behind the document, and it has been long enough to say so.**
fn catching_up(ui: &mut egui::Ui, doc: &OpenDoc) {
    if !doc.page_is_catching_up() {
        return;
    }
    disclosure_line(ui, REGION_CATCHING_UP, t::page_catching_up());
}

/// Draw one disclosure sentence into the bar's single row, and publish its
/// rect.
pub(super) fn disclosure_line(ui: &mut egui::Ui, region: &str, line: &str) {
    let width = (ui.available_width() * NOTES_WIDTH_FRACTION).max(0.0);
    let rect = ui
        .allocate_ui_with_layout(
            Vec2::new(width, ROW_HEIGHT_PTS),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.add(egui::Label::new(egui::RichText::new(line).small()).truncate())
                    .on_hover_text(line.to_owned());
            },
        )
        .response
        .rect;
    crate::diag::ui_rect(region, rect);
}

/// **This file's index was damaged and pdfcer rebuilt it** — the only line here
/// that is about the FILE rather than about something the operator just did.
fn recovered_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    if doc.session.document().recovery().is_none() {
        return;
    }
    disclosure_line(ui, REGION_RECOVERED, t::recovered_status_line());
}

/// **This file contradicted itself, and pdfcer decided rather than
/// refusing** — engine `Pass 283.0`, decision 145, wired 2026-09-09.
fn load_anomalies_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(line) = super::anomalies::status_line(doc.session.document().load_anomalies()) else {
        return;
    };
    disclosure_line(ui, REGION_LOAD_ANOMALIES, &line);
}

/// **The page's colours are approximate at this zoom**, because the raster
/// grew past the size the engine will composite in CMYK.
fn blend_space_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(texture) = doc.page_texture.as_ref() else {
        return;
    };
    // `cmyk_buffer_refused`, not `blends_in_wrong_space`. The first says
    // *the correct buffer was not available*, which is true of the whole page
    // and is what changes with zoom. The second counts the blends that then
    // happened in the wrong space, and is zero on a page whose transparency is
    // outside the rendered region — so keying on it would go quiet exactly
    // where the operator scrolled away from the affected patch and back.
    if texture.diagnostics.cmyk_buffer_refused == 0 {
        return;
    }
    disclosure_line(ui, REGION_BLEND_SPACE, &t::blend_space_status_line());
}

/// **Some of the page's spot inks are drawn as process colour**: the page names
/// more than the colorant buffer holds, so the extras paint through their tint
/// transforms and overprint and blending treat them as CMYK. A property of the
/// page, so it follows the texture like [`blend_space_disclosure`].
fn spots_flattened_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(texture) = doc.page_texture.as_ref() else {
        return;
    };
    let inks = texture.diagnostics.cmyk_spots_flattened;
    if inks == 0 {
        return;
    }
    disclosure_line(
        ui,
        REGION_SPOTS_FLATTENED,
        &t::spots_flattened_status_line(inks),
    );
}

/// **The canvas is deliberately not showing what will print** —
/// `OPERATOR_REQUESTS.md` **O137**, and the line that makes the whole feature
/// safe to ship.
///
/// # What it is disclosing
///
/// `view.line_weights` is off, so `pdfcer-render` is capping every stroke's
/// device width at one pixel (`RenderOptions::stroke_display =
/// StrokeDisplay::Hairline`, engine `Pass 254.0`) — the CAD "line weights off"
/// convention the operator asked for by name. The document is untouched;
/// printing, print preview and every export render the real widths.
///
/// # Why it exists, and why "he asked for it" is not an answer
///
/// The three lines above are rule 4's usual shape: pdfcer inferred something
/// the operator cannot see. This one is not — he pressed a button and got what
/// the button promised. The obligation comes from somewhere else, and it is
/// worth stating exactly, because the tempting conclusion is that a requested
/// display mode owes nothing:
///
/// **The canvas's standing claim is that what is drawn is what will be saved
/// and printed.** Every other feature in this program is built to keep that
/// claim (`canvas::form_marks`' wash argues its own exemption at length on
/// precisely this ground — it is a *control's* affordance, not content). This
/// mode suspends the claim, on purpose, for the page content itself. A
/// suspended claim is stated, or the next surprising thing the operator sees is
/// a plot that does not match his screen.
///
/// ⇒ And the surprise is realistic rather than theoretical: this is a
/// **reading** aid, so it is on precisely while he is absorbed in reading, for
/// as long as he likes, across documents and sheets. There is no gesture to
/// remember it by and no mark on the page. Nothing else in the program persists
/// a divergence like that.
///
/// # It is a STATE, like [`catching_up`], not an event
///
/// Live for exactly as long as the toggle is off. No `edit_epoch` key, nothing
/// to clear, and no way for it to be shown against a document it is not true
/// of — it reads the same `doc.view` the renderer was handed. The two state
/// lines can be live together (a slow page, hairline on) and that is bounded
/// the same way every other pair here is; see [`disclosure_line`].
///
/// Drawn through the shared [`disclosure_line`] so it inherits all four R128
/// defences at once — bounded width, fixed row height, truncation rather than
/// wrapping, and the whole sentence on hover. **It does not make the bar
/// taller**, which for a line that can be up for an hour is not a nicety: a bar
/// that grew would re-fit the page underneath it.
///
/// Off-canvas, never a badge on the page — the same constraint
/// [`edit_disclosure`] argues, and here it is doubly binding, because a mark
/// drawn over the drawing to say *"this drawing is being drawn unfaithfully"*
/// would itself be an unfaithful mark on the drawing.
fn line_weights_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    if doc.view.line_weights {
        return;
    }
    //
    // `Diagnostics::strokes_hairlined` counts strokes this render actually
    // **thinned**. Zero means the mode reached the renderer and had nothing to
    // do here, which from a chair is the identical screenshot to *the setting
    // is broken*. This shell asked the engine for that count precisely because
    // the two are indistinguishable without it, and shipping the toggle without
    // consuming it would leave criterion 3 of our own request unbuilt.
    //
    // ⚠ **A MISSING TEXTURE IS NOT A ZERO.** Before the first raster lands —
    // and on a page whose content streams will not decode — `page_texture` is
    // `None`, and there is no count either way. Saying *"nothing was thick
    // enough to thin"* there would be reporting an absence of evidence as
    // evidence of absence, on the frame where the operator is most likely to be
    // looking. The general sentence is the honest fallback: it is true whatever
    // the count turns out to be.
    let line = match doc.page_texture.as_ref() {
        Some(t) if t.diagnostics.strokes_hairlined == 0 => t::line_weights_no_effect(),
        _ => t::line_weights_off(),
    };
    disclosure_line(ui, REGION_LINE_WEIGHTS, line);
}

/// Draw all of them, in the order the parent expects.
pub(super) fn all(ui: &mut egui::Ui, doc: &OpenDoc) {
    // First, and the order is the argument: the other three describe what an
    // edit DID, and this one describes whether the operator is looking at the
    // result yet. Reading "the picture is still being drawn" after a sentence
    // about what was drawn puts the two in the wrong causal order.
    catching_up(ui, doc);
    fill_disclosure(ui, doc);
    edit_disclosure(ui, doc);
    recovered_disclosure(ui, doc);
    // Immediately after its nearest relative, and before the two render-state
    // lines. Both of these are about **how this file was assembled before
    // anything was drawn**, so they belong adjacent; and the index question
    // comes first because a rebuilt index is the more sweeping fact — it says
    // pdfcer had to find the objects at all, where this says what one of the
    // objects it found said twice.
    load_anomalies_disclosure(ui, doc);
    blend_space_disclosure(ui, doc);
    spots_flattened_disclosure(ui, doc);
    // LAST, and the position is the argument. Every line above is about
    // something that HAPPENED — a fill, an edit, how the file was assembled, a
    // buffer that would not fit. This one is about a stance the operator is
    // holding, which outlives all of them; putting a durable state ahead of the
    // transient events would push a sentence he has already read in front of
    // the one he has not.
    //
    // It is also the line most likely to be up at the same time as another,
    // because it can be up for an hour — so it is the one that should yield
    // rightmost when the bar runs short, and last is where that happens.
    line_weights_disclosure(ui, doc);
}
