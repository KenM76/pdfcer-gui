//! # `app::status::disclosure` — the three rule-4 lines in the status bar
//!
//! Split out of [`super`] on 2026-08-26 when a third line took that file past
//! the 1,500-line ceiling (R2). The seam is a real one and was already drawn in
//! the parent's own prose, which distinguished *narration* from *disclosure*:
//!
//! > The left half carries four things, and only the first is the narrator. The
//! > others look similar and are governed by different rules.
//!
//! Everything here is **rule 4**: pdfcer did something the operator did not ask
//! for and cannot see, so it says so — off-canvas, never on the page.
//!
//! ## ★★★ Three lines, and they are INDEPENDENT
//!
//! | line | answers |
//! |---|---|
//! | [`fill_disclosure`] | what a form fill had to **infer** — an auto-size chosen, characters that could not be encoded |
//! | [`edit_disclosure`] | what a move or delete had to change about an object's **form** to express the request |
//! | [`recovered_disclosure`] | how this **file** was assembled, before anything was drawn |
//!
//! The obvious mistake, adding a third beside two, is an `else if` chain that
//! shows whichever fires first. A document opened from a damaged index, then
//! edited, with a form filled, owes the operator all three —
//! `disclosure_independence` in the parent asserts they cannot collide.
//!
//! ★★ The third is the odd one out and the reason for this module's header: the
//! first two are about **something the operator just did**, and the last is
//! about **what the file was before they touched it**. It is also the only one
//! that persists for the life of the document rather than until the next edit.

use egui::{Align, Layout, Vec2};

use super::{
    NOTES_WIDTH_FRACTION, REGION_BLEND_SPACE, REGION_CATCHING_UP, REGION_EDIT_DISCLOSURE,
    REGION_FILL_DISCLOSURE, REGION_LINE_WEIGHTS, REGION_RECOVERED, ROW_HEIGHT_PTS,
};
use crate::app::state::OpenDoc;
use crate::text::forms as t_forms;
use crate::text::status as t;

/// What the last fill **inferred**, in the bar, until the document moves on.
///
/// # Why this is not behind the disclosure triangle beside it
///
/// The render notes are *narration* — a census of what a raster contained —
/// and `DEFECTS.md` §5's complaint was their prominence: the first thing an
/// operator read was the application talking about itself. Demoting them was
/// right.
///
/// These two sentences are the opposite kind of thing. They are the surviving
/// half of rule 4: **an inference the operator cannot see still owes a
/// report.** `applied_autosize` means pdfcer chose a point size the document
/// asked it to choose; `unencodable_chars` means the operator's own typing is
/// not what the page now says. Neither is re-derivable from the saved file
/// afterwards — both look exactly like the author's decision — so a
/// disclosure the operator has to *open something* to find is a disclosure
/// that did not happen.
///
/// # Why the status bar rather than the Forms panel alone
///
/// The panel shows them, and that was sufficient while the panel was the only
/// way to fill. Canvas filling landed 2026-08-14 and broke the assumption: a
/// fill can now happen in **Read mode with the panel closed**, and Read's
/// dock does not mount Forms unless the operator put it there. The bar is the
/// one surface present in every mode.
///
/// # It retires itself
///
/// Keyed on [`OpenDoc::edit_epoch`] **after** the fill, so any later edit —
/// including an undo — moves the document past it and the sentence
/// disappears with no code remembering to clear it. That is deliberate:
/// state that must be cleared is state that will one day be shown against
/// the wrong document.
///
/// Elided at the same fraction as the notes line, whole text on hover, and
/// **it does not make the bar taller** — R128, exactly as for its neighbour.
fn fill_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(d) = crate::panels::forms::edit::last_fill_disclosure(doc.edit_epoch) else {
        return;
    };

    // Both can be true of one fill. Joined rather than shown as two lines,
    // because two lines is two rows, which is the R128 loop.
    let mut line = String::new();
    if let Some(size) = d.applied_autosize {
        line.push_str(&t_forms::forms_fill_autosize_note(&d.field, size));
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
    if line.is_empty() {
        return;
    }

    disclosure_line(ui, REGION_FILL_DISCLOSURE, &line);
}

/// What the last **vector edit** disclosed, in the bar, until the document
/// moves on.
///
/// # What it says, and who wrote it
///
/// Every vector verb — the three move verbs and Delete — returns a list of
/// operator-facing sentences alongside its success, non-empty when the surgery
/// had to change an operator's *form* to express the request: an `re` rectangle
/// rewritten as four lines so one corner could move independently, an
/// implicitly-started subpath's `m` materialised, a curve dropped along with
/// the point it ran into. **The drawing is unchanged and the bytes are not
/// recoverable by reversing the gesture** — dragging the corner back does not
/// restore the rectangle form — which is precisely the condition rule 4 exists
/// for: pdfcer inferred a representation, and the operator would otherwise
/// learn it from a diff.
///
/// The sentences are `pdfcer-core`'s own and are passed through verbatim; this
/// module contributes the framing, and only the framing. See
/// [`crate::text::status::edit_disclosure_line`].
///
/// # Why it is here rather than only in the trace
///
/// It *was* only in the trace. `crate::app::actions::vector_edit`'s header
/// named that as the outstanding half in as many words — *"a disclosure that
/// only ever reaches `PDFCER_DIAG` has been recorded and not disclosed"* — and
/// this function is the half it was waiting for. The trace is unchanged and
/// still carries the full list; what has changed is that an operator who is
/// not running with `PDFCER_DIAG` set can now read it, which is every operator.
///
/// # Why the status bar rather than a panel or the canvas
///
/// Two constraints, and together they leave one surface. Rule 4 puts a
/// disclosure **off-canvas** — the one-line test is whether a screenshot of the
/// editing canvas would differ from a screenshot of the same document saved and
/// reopened, and a note drawn over the page would make it differ. And the
/// gesture that raises one is a **canvas drag**, available in Edit and Review
/// with any panel arrangement including none, so a panel could not be relied on
/// to be mounted. The bar is the one surface present in every mode.
///
/// # It retires itself, and it cannot collide with its neighbour
///
/// Keyed on [`OpenDoc::edit_epoch`] **after** the edit, exactly as
/// [`fill_disclosure`] is: any later edit — including an undo — moves the
/// document past it and the sentence disappears with no code remembering to
/// clear it. One edit bumps the epoch once and records at most one kind of
/// disclosure, so the fill line and this one can never both be live for the
/// same revision; see
/// [`crate::app::actions::last_edit_disclosure`]'s ★ section.
///
/// **It does not make the bar taller** — R128, asserted by
/// [`tests::the_bar_is_exactly_as_tall_open_as_closed`].
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
///
/// `OPERATOR_REQUESTS.md` **O63**, and the piece that makes the request's own
/// words — *"live preview for everything we do"* — true rather than
/// aspirational.
///
/// # ★★★ Why this is the general answer and the drawn preview is not
///
/// `canvas::shapes` draws a real preview, exactly, at pointer speed — and only
/// where the shell holds the geometry: a path being moved, resized, rotated or
/// node-edited. That is a large share of canvas work and **none** of the rest of
/// the program. There is no shape to slide when the operator changes a fill
/// colour, presses Bold, deletes a run of text, marks a redaction or rotates a
/// page.
///
/// For those the shell cannot draw the answer, and it cannot get one from the
/// renderer either: `BENCHMARK.md` measures a **two-pixel** region render at
/// 691 ms on the operator's own drawing, because ~99 % of render cost is
/// content-stream interpretation rather than fill. There is no arrangement of
/// the existing renderer that produces a correct picture inside a second.
///
/// ⇒ So the honest general answer is not a worse picture. It is **saying that
/// the picture is not the answer yet** — the third of the three options the
/// operator chose between, and the only one with no failure mode. It applies to
/// every edit in the program, including the ones a drawn preview will never
/// reach.
///
/// # It is a STATE, not an event, and that changes two things
///
/// Every other line in this file is keyed on [`OpenDoc::edit_epoch`] and
/// retires when the document moves past it. This one is live for as long as its
/// condition holds and stops the moment the raster lands — so it needs no
/// retirement rule at all, and it can appear for one edit and not the next
/// depending only on how hard the page was to draw.
///
/// ★ And it is **silent under 400 ms** ([`OpenDoc::page_is_catching_up`]),
/// because the picture is behind after every edit and a line that flashed on
/// each one would be noise that costs every other sentence this bar carries.
///
/// **It does not make the bar taller** — R128, the same constraint every line
/// here is under and for the same reason: it arrives without the operator
/// asking for anything, and a bar that grew on its own would re-fit the page at
/// the moment a gesture completed.
fn catching_up(ui: &mut egui::Ui, doc: &OpenDoc) {
    if !doc.page_is_catching_up() {
        return;
    }
    disclosure_line(ui, REGION_CATCHING_UP, t::page_catching_up());
}

/// Draw one disclosure sentence into the bar's single row, and publish its
/// rect.
///
/// The shared body of [`fill_disclosure`] and [`edit_disclosure`], written once
/// for the reason `crate::app::actions::vector_edit` is written once: the
/// R128 defence here is not one rule but four small ones that only work
/// together — a **bounded** sub-region so a long sentence cannot push the
/// navigation controls off the right of the bar, a **fixed** row height,
/// `truncate()` rather than wrapping (wrapping is how a one-row bar becomes a
/// two-row bar, which is the feedback loop with extra steps), and the full text
/// on **hover** so eliding defers rather than loses. Two hand-written copies
/// would be two chances to omit one of the four, and the omission would show up
/// as a page that re-fits itself at the moment an operator finishes a gesture.
///
/// `region` is published so `ui-verify` can assert the sentence is on screen
/// and legible rather than merely constructed — which, for a disclosure, is the
/// whole of the requirement.
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
///
/// # ★★★ Why it is in the status bar as well as in Properties
///
/// Operator ruling, 2026-08-26: *"disclose it."*
///
/// Properties already carries the detail — how many objects were recovered, how
/// many were defined more than once, how many needed repairing. But **a
/// disclosure the operator has to go looking for is half a disclosure**, and
/// this is the one fact that changes how much they should trust what is on
/// screen. A rebuilt index is a *best reading of damaged bytes*: where an object
/// was defined twice pdfcer had to pick one, and on a drawing a wrong pick is a
/// line in the wrong place on a page that renders perfectly.
///
/// # ★★ How it avoids being the nagging the old shell was criticised for
///
/// 1. **Off-canvas.** A line in the status bar, never a badge on the page. The
///    document is not in doubt as *drawn*; what is in doubt is how it was
///    *assembled*, and marking the page would be a second rendering path for
///    content that is fine — decision 059's whole subject.
/// 2. **It only appears for a file that was actually rebuilt**, which is rare. A
///    healthy document shows nothing; verified by opening one.
/// 3. **It states the fact and stops.** No icon, no colour alarm, no modal at
///    open. One sentence, and the operator decides whether it matters to the job
///    in front of them.
///
/// ★ The counters stay in Properties. The status bar answers *"is there
/// something I should know?"*; the panel answers *"what exactly?"* — and a line
/// long enough to carry three numbers would push the zoom and page controls off
/// a narrow window.
fn recovered_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    if doc.session.document().recovery().is_none() {
        return;
    }
    disclosure_line(ui, REGION_RECOVERED, t::recovered_status_line());
}

/// ★★★ **The page's colours are approximate at this zoom**, because the raster
/// grew past the size the engine will composite in CMYK.
///
/// # What the operator sees without this, and why it reads as a bug
///
/// Reported 2026-08-26: *"seems I get different results depending on Zoom
/// level. The [shading] boxes … on zoom out the colors between our
/// rendering and the references don't match, but they do when I am zoomed in.
/// up to 474% they are mismatched, but at 579% they match."*
///
/// Measured the same day, and his bracket contains the answer exactly.
/// `pdfcer-render` composites a page with transparency in a **subtractive CMYK
/// buffer** — the correct space for it — and that buffer has a documented
/// ceiling of 256 MiB at 20 B/px, i.e. **13,421,772 pixels**. Past it the
/// renderer falls back to compositing in sRGB and counts that it did
/// (`cmyk_buffer_refused`, `blends_in_wrong_space`).
///
/// On an A4 page that ceiling is crossed at **zoom 534 %**, dead centre of the
/// 474–579 % band he bracketed. Either side of it the same page renders
/// different colours: measured on the conformance suite's composite page, up to
/// **16 levels out of
/// 255** in the transparency patches, by box-averaging every pixel of both
/// renders into a common grid so that resampling could not masquerade as the
/// effect.
///
/// # ★★ Why this is a disclosure and not just a fix
///
/// It is *both*, and the fix is not ours to make alone. The engine's ceiling is
/// deliberate — `ARCHITECTURE.md` §10 forbids an untrusted-input-sized
/// allocation without one — and its fallback is honest. What is wrong is where
/// **this shell** stops asking for whole-page rasters: `render::strategy`
/// switches to the region tier at `MAX_PIXMAP_EDGE`, which for A4 is zoom
/// 2071 %. Between 534 % and 2071 % the GUI asks for a raster the engine cannot
/// composite properly, and that four-times band of zoom is entirely this
/// shell's choice.
///
/// Measured, so the fix is known to work: a **region** render below the ceiling
/// composites in CMYK at any zoom (`--region 0,60,596,260 --scale 8` →
/// `cmyk_buffer=1`). The buffer is sized to the region, not to the page. So the
/// repair is for `strategy::for_page` to respect the pixel ceiling as well as
/// the edge ceiling — which needs `MAX_CMYK_BUFFER_BYTES` to be public, and it
/// is `pub(crate)` today. That is filed as an engine request rather than
/// guessed at with a hardcoded 13,421,772, which would be a measured limit
/// copied into a second place to rot.
///
/// Until then the operator is **told**, which is rule 4's surviving half doing
/// exactly its job: this is an inference the operator cannot see — a screenshot
/// of the page says nothing about which space it was composited in — so it owes
/// an off-canvas report. Nothing is marked on the canvas.
///
/// ★ It names **zooming out** as the remedy, because that is the one that
/// works, is instant, and is the opposite of what an operator chasing a colour
/// difference would try.
fn blend_space_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(texture) = doc.page_texture.as_ref() else {
        return;
    };
    // ★ `cmyk_buffer_refused`, not `blends_in_wrong_space`. The first says
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

/// ★★★ **The canvas is deliberately not showing what will print** —
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
/// # ★★★ Why it exists, and why "he asked for it" is not an answer
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
/// ⇒ ★★ And the surprise is realistic rather than theoretical: this is a
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
/// ★ Drawn through the shared [`disclosure_line`] so it inherits all four R128
/// defences at once — bounded width, fixed row height, truncation rather than
/// wrapping, and the whole sentence on hover. **It does not make the bar
/// taller**, which for a line that can be up for an hour is not a nicety: a bar
/// that grew would re-fit the page underneath it.
///
/// ★ Off-canvas, never a badge on the page — the same constraint
/// [`edit_disclosure`] argues, and here it is doubly binding, because a mark
/// drawn over the drawing to say *"this drawing is being drawn unfaithfully"*
/// would itself be an unfaithful mark on the drawing.
fn line_weights_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    if doc.view.line_weights {
        return;
    }
    disclosure_line(ui, REGION_LINE_WEIGHTS, t::line_weights_off());
}

/// Draw all of them, in the order the parent expects.
pub(super) fn all(ui: &mut egui::Ui, doc: &OpenDoc) {
    // ★ First, and the order is the argument: the other three describe what an
    // edit DID, and this one describes whether the operator is looking at the
    // result yet. Reading "the picture is still being drawn" after a sentence
    // about what was drawn puts the two in the wrong causal order.
    catching_up(ui, doc);
    fill_disclosure(ui, doc);
    edit_disclosure(ui, doc);
    recovered_disclosure(ui, doc);
    blend_space_disclosure(ui, doc);
    // ★★★ LAST, and the position is the argument. Every line above is about
    // something that HAPPENED — a fill, an edit, how the file was assembled, a
    // buffer that would not fit. This one is about a stance the operator is
    // holding, which outlives all of them; putting a durable state ahead of the
    // transient events would push a sentence he has already read in front of
    // the one he has not.
    //
    // ★ It is also the line most likely to be up at the same time as another,
    // because it can be up for an hour — so it is the one that should yield
    // rightmost when the bar runs short, and last is where that happens.
    line_weights_disclosure(ui, doc);
}
