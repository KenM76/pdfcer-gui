//! # `canvas::textedit::paint` — **what a draft looks like on the page**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-20, when the in-place editor pushed `canvas::textedit` past
//! 1,500 lines. A real seam: everything here answers one question — *what does
//! an operator see while they are composing?* — and nothing here takes a
//! keystroke, resolves a click or builds an action.
//!
//! ## The one thing to know before changing anything in here
//!
//! **The draft is drawn ONCE and measured ONCE.** The editor box's text and the
//! caret's position come from the same string, the same `FontId` and the same
//! size, in that order, a few lines apart. That is not tidiness — it is the
//! defect this file was rewritten to remove.
//!
//! The caret used to be derived from the page's own glyph advances, which was
//! right while the page's glyphs were the only thing on screen and became wrong
//! the moment a preview was drawn in a different font: the caret would sit
//! somewhere other than between the characters the operator could see, and
//! drift further with every keystroke.
//!
//! Two derivations of one position, agreeing at first and separating under use,
//! is the same class of defect as the vertex drag that tracked at `1/zoom` and
//! the snap marker that sat off by the scroll origin. This project has now met
//! it three times. **If you add anything to this file that needs to know where
//! a character is, measure it from the layout — never from the document.**
//!
//! ## Rule 4 lives here too
//!
//! An in-place editor covers applied content while it is open. It does not
//! restyle it, mark it, tint it or flag it — and the moment it closes, what
//! replaces it is `pdfcer-render`'s output with no marking of any kind. See
//! [`preview`]'s own header for the argument against D4a's ghost text and why
//! an opaque editor is a different thing from a translucent one.

use egui::{Pos2, Ui};

use super::{Anchor, Draft, Preview, read};
use crate::app::state::OpenDoc;

/// The smallest the in-place editor's text may be drawn, in points.
///
/// A 4 pt note at 25 % zoom is a box two pixels high, and an operator cannot
/// type into a line. The box grows past the run it covers rather than becoming
/// illegible — the alternative is a preview that technically exists.
const MIN_PREVIEW_PT: f32 = 11.0;

/// The largest, in points. A title at 400 % zoom would otherwise fill the canvas
/// with a single word.
const MAX_PREVIEW_PT: f32 = 40.0;

/// How much of the editor box's height the glyphs take, leaving the rest as
/// leading. A cap height is roughly 70 % of a line box, and text set at the full
/// box height sits on the edges and reads as cramped.
const PREVIEW_FILL: f32 = 0.72;

/// How far in from the editor box's left edge the text starts, in points.
///
/// Shared by the text and the caret, so a caret at index 0 sits exactly where
/// the first character does rather than a hair to one side of it.
const PREVIEW_INSET_PT: f32 = 2.0;

/// **Draw the in-place editor: what you are typing, where you are typing it.**
///
/// ## ★★★ D4a's ghost text, the decision that followed it, and why that
/// ## decision was half-right
///
/// The old shell drew the draft *as text*, in an `egui` proportional font, over
/// a **translucent mask** — which `DEFECTS.md` D4a names as the second
/// contributor to "weird": *"you type in the wrong typeface at the wrong
/// widths, then it snaps to reality on Accept."*
///
/// This module's answer, until 2026-08-20, was to draw **no glyphs at all** —
/// a caret and a bracket, with this promise attached:
///
/// > *"The characters themselves are shown off-canvas, in the status bar, where
/// > `text::textedit` owns the sentence."*
///
/// **That promise was never kept.** `text::tool::text_edit_live` says *"Enter
/// commits what you have typed. Esc abandons it."* and nothing anywhere renders
/// the draft. So the operator typed into a bracket and their characters
/// appeared nowhere at all:
///
/// > *"I can edit text now, but there is no live preview of that either."*
/// > — 2026-08-20
///
/// ## Why an in-place editor BOX is not the ghost D4a condemns
///
/// The distinction is not cosmetic and it is the whole justification for
/// reversing the decision:
///
/// | | old ghost | this |
/// |---|---|---|
/// | drawn | translucent, **over** the original glyphs | **opaque**, covering them |
/// | reads as | the document, in the wrong typeface | an editor, obviously |
/// | on commit | "it snapped to reality" | the editor closed |
///
/// D4a's defect is that the ghost **imitated applied content**. Rule 4's
/// one-line test — *would a screenshot of the editing canvas differ from a
/// screenshot of the same document saved and reopened?* — caught it because the
/// old shell's canvas differed **in the one respect the operator was looking
/// at**, while claiming to be the document.
///
/// An opaque editor box differs too, and does not claim otherwise. Rule 4
/// permits exactly this by name: *"a snap indicator, a hover highlight, a
/// rubber-band … these are the cursor; they describe what is about to
/// happen."* An in-place editor **is** the cursor. What the rule forbids is
/// styling content **already applied** as though it were pending — and this
/// covers the applied content rather than restyling it.
///
/// Every program does it this way, which is the second half of the argument: a
/// spreadsheet cell, a Word table cell, a CAD attribute editor, a file-name
/// rename in Explorer. All of them cover the original with a filled box while
/// you type and reveal the result on commit. Nobody is surprised when the box
/// closes.
///
/// ## The two objections that stood, and what happened to them
///
/// **"A ghost in the wrong face is a lie about the document."** True of a
/// translucent one. An opaque box makes no claim about the document's typeface
/// because it is visibly not the document — it has an edge, a fill and UI text.
///
/// **"A ghost in the right face would need re-rasterizing the run's embedded
/// font per keystroke, and `BENCHMARK.md` says ~99 % of render cost on dense CAD
/// is resolution-independent."** Still true, and this is why the box does **not**
/// attempt the document's typeface. It costs one filled rectangle and one text
/// layout per frame.
///
/// ## The caret is measured against the text AS DRAWN
///
/// Not against the page's glyph advances, which is where it used to come from.
/// The draft is now drawn in the shell's font, so a caret placed by the
/// document's metrics would sit somewhere other than between the characters the
/// operator can see — and would drift further with every keystroke. One string,
/// one font, one size, measured once: the preview and the caret cannot disagree.
/// The editor box's own region name. See its publication in [`preview`].
pub const REGION_BOX: &str = "text-edit.box"; // ui-text-exempt: trace region name, never displayed

pub fn preview(ui: &Ui, ctx: &egui::Context, p: &Preview<'_>) {
    let Some(draft) = read(ctx) else {
        return;
    };
    if draft.page != p.page_index {
        return;
    }
    let Some(page) = p.doc.pages.get(p.page_index) else {
        return;
    };
    let theme = egui_shell::theme::Theme::of(ui.ctx());
    let painter = ui.painter();
    // A 1 s blink, and a repaint request so it actually blinks on a canvas with
    // no other reason to redraw.
    let on = (ui.input(|i| i.time) * 1.6) as i64 % 2 == 0;
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(400));
    let Some(rect) = caret_box(p.doc, &draft, page) else {
        return;
    };
    let screen = egui::Rect::from_two_pos(p.map.to_screen(rect.min), p.map.to_screen(rect.max));
    // ★★★ **WHAT YOU ARE TYPING, WHERE YOU ARE TYPING IT.** 2026-08-20.
    //
    // The operator: *"I can edit text now, but there is no live preview of that
    // either."* He is right and it made the feature nearly unusable — the page
    // renders **committed** glyphs, the draft lived beside them, and nothing
    // drew it. So an operator saw the old text, a blinking caret, and no
    // evidence that their keystrokes had landed anywhere.
    //
    // # ★★ Why an in-place EDITOR BOX and not text overlaid on the page
    //
    // The tempting shape is "draw the draft where the glyphs are, in the
    // document's own font, so it looks like the finished result". Two problems,
    // and the second is fatal:
    //
    // 1. **This shell does not have the document's font.** The page is
    //    rasterised by `pdfcer-render` from embedded programs; egui's text stack
    //    has its own faces. The draft would render in a different typeface at a
    //    different width whatever we did.
    // 2. **★ The original glyphs are still underneath.** They are baked into
    //    the page raster and this shell cannot un-draw them. Text drawn on top
    //    of text is illegible, and the shorter the edit the worse it gets —
    //    changing `SHEET 1 OF 4` to `SHEET 2 OF 4` would show both `1` and `2`
    //    superimposed, which is the one character the operator is looking at.
    //
    // Masking the original needs the page's local background colour, which this
    // shell would have to *guess* — it is whatever the drawing has there, not
    // necessarily white.
    //
    // So: an **opaque editor box**, which is what every in-place editor in every
    // program already is. A spreadsheet cell, a Word table cell, a CAD attribute
    // editor, a file-name rename in Explorer — all of them cover the original
    // with a filled box while you type and reveal the result on commit. It is
    // the convention *and* the honest picture: the box says "this is a draft in
    // an editor", which is exactly what it is, and it makes no promise about
    // typeface or metrics that the commit would then break.
    //
    // # The size is the RUN's, not the UI's
    //
    // The glyph box gives the run's height, so the editor sits at the size of
    // the text it is replacing and a long draft is visibly long. Clamped to
    // something legible, because a 4 pt note at 25 % zoom is a box two pixels
    // high and the operator would be typing into a line.
    let height = screen.height().clamp(MIN_PREVIEW_PT, MAX_PREVIEW_PT);
    // ★ ONE font binding and ONE layout, shared by the fill, the text and the
    // caret below. Two `FontId`s built separately would be two derivations of
    // one fact, and the caret would sit where a *slightly different* string
    // would have ended. See this module's header.
    let font = egui::FontId::proportional(height * PREVIEW_FILL);
    // ★★★ A BOX DRAFT WRAPS; EVERY OTHER DRAFT DOES NOT.
    //
    // The operator, 2026-08-21: *"I should be able to make it multi line."*
    //
    // `Anchor::Box` is the anchor a dragged rectangle produces, and the whole
    // point of that rectangle is a **width to wrap against** — a PDF has no
    // paragraph, so each visual line is its own show operator and something has
    // to decide where the second line starts.
    //
    // ★ The preview therefore wraps at **the box's own screen width**, which is
    // the same width `add_text`'s boxed variant will wrap to. Not the same
    // *metrics* — the preview is the shell's font and the commit is the pen's,
    // which this module's header is explicit about and which is why the box is
    // opaque rather than a ghost. What it promises is *"your text will break
    // around here"*, and that promise it keeps.
    //
    // A point or run draft passes `f32::INFINITY`, which is exactly
    // `layout_no_wrap` and is written as one call so the caret below measures
    // through the same path in both cases. Two layout calls would be the two
    // derivations this module deleted `caret_x` to be rid of.
    let box_width = match &draft.anchor {
        Anchor::Box { llx, urx, .. } => {
            #[allow(clippy::cast_possible_truncation)]
            let (lo, hi) = (*llx as f32, *urx as f32);
            let a = crate::viewer::pdf_space_to_canvas(Pos2::new(lo, 0.0), page);
            let b = crate::viewer::pdf_space_to_canvas(Pos2::new(hi, 0.0), page);
            match (a, b) {
                (Some(a), Some(b)) => Some((p.map.to_screen(b).x - p.map.to_screen(a).x).abs()),
                _ => None,
            }
        }
        Anchor::Run { .. } | Anchor::Origin { .. } => None,
    };
    let wrap_at = box_width.map_or(f32::INFINITY, |w| (w - PREVIEW_INSET_PT * 2.0).max(1.0));
    let laid = painter.layout(
        draft.text.clone(),
        font.clone(),
        theme.palette.text,
        wrap_at,
    );

    // ★★ THE BOX GROWS WITH WHAT IS IN IT, and it has to.
    //
    // It was `screen.shrink(1.0)` — the glyph box, exactly — until this was
    // driven and looked at. Two ways that is wrong, and the second is the
    // serious one:
    //
    // 1. **A draft longer than the run it replaces** overflows a box sized to
    //    the original, so the tail of what you are typing sits on bare page.
    // 2. **★ An `Anchor::Origin` draft has no glyph box at all.** `caret_box`
    //    returns a nominal 6 × 14 pt for new text, so Add-text drew its
    //    characters almost entirely OUTSIDE the fill — text on the page
    //    background in the shell's font, which is exactly the translucent ghost
    //    D4a condemns, arrived at by accident.
    //
    // So the width is the greater of the run's extent and the laid-out text.
    // Every in-place editor in every program does this — a spreadsheet cell
    // editor grows as you type past the column, and it grows because the
    // alternative is text that has escaped its own control.
    let width = box_width.unwrap_or_else(|| {
        screen
            .width()
            .max(laid.rect.width() + PREVIEW_INSET_PT * 2.0)
    });
    // ★★ A BOX GROWS DOWNWARD FROM ITS TOP EDGE, and a single-line draft stays
    // centred on the run it replaces.
    //
    // Two different anchors and therefore two different rectangles, and the
    // difference is not cosmetic: `add_text`'s boxed variant is **top-anchored**
    // — the text is laid out from the top of the box downward — so a preview
    // centred on the caret slot would show the paragraph in a place the commit
    // will not put it.
    //
    // The height is the laid-out text's, floored at one line, so an empty box
    // still shows where the first character will land rather than collapsing to
    // nothing.
    let body = if box_width.is_some() {
        egui::Rect::from_min_size(
            egui::pos2(screen.left(), screen.top()),
            egui::vec2(
                width,
                (laid.rect.height() + PREVIEW_INSET_PT * 2.0).max(height),
            ),
        )
    } else {
        egui::Rect::from_min_size(
            egui::pos2(screen.left(), screen.center().y - height / 2.0),
            egui::vec2(width, height),
        )
    };
    painter.rect_filled(body, 0.0, theme.palette.surface);
    let text_origin = egui::pos2(
        body.left() + PREVIEW_INSET_PT,
        if box_width.is_some() {
            body.top() + PREVIEW_INSET_PT
        } else {
            body.center().y - laid.rect.height() / 2.0
        },
    );
    // ★★ THE SELECTION IS DRAWN UNDER THE TEXT, before the galley, so the
    // characters sit ON the highlight rather than behind it. Drawing it after
    // would need a translucent fill and would tint every glyph it covers.
    //
    // ★ This is a **cursor**, not content marking, and R8b rule 4 permits it
    // for exactly that reason: it shows what the *next keystroke* will replace
    // and it is gone the moment the draft commits. Nothing about the applied
    // document is styled here.
    // ★★ The box, published for the HARNESS as well as for the pointer
    // handlers. A driven check that wants to sweep across a draft has no other
    // way to find it: the editor is painted into the canvas rather than laid
    // out as a widget, so it appears in no layout the harness can read, and a
    // check aiming at it from the run's page coordinates would be aiming at
    // the glyphs the box is covering rather than at the box.
    crate::diag::ui_rect(REGION_BOX, body);
    // ★ Publish the box and the galley for the pointer handlers. See
    // `canvas::textedit::hit`: this is the ONE layout, and hit-testing it is
    // the inverse of the `pos_from_cursor` the caret is drawn with.
    crate::canvas::textedit::hit::publish(
        ctx,
        crate::canvas::textedit::hit::Layout {
            body,
            body_canvas: egui::Rect::from_two_pos(p.map.to_page(body.min), p.map.to_page(body.max)),
            origin: text_origin,
            galley: laid.clone(),
        },
    );
    selection(
        painter,
        &draft,
        &laid,
        text_origin,
        theme.palette.selection_fill,
    );
    painter.galley(text_origin, laid.clone(), theme.palette.text);

    // The bracket, drawn round the EDITOR rather than round the run: it is the
    // extent of what the operator is composing, which after the first keystroke
    // is no longer the extent of what they are replacing.
    painter.rect_stroke(
        body,
        0.0,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Outside,
    );
    if on {
        // ★★★ The caret is measured against the text AS DRAWN — not against
        // the page's glyph metrics. 2026-08-20, with the live preview.
        //
        // It used to come from `caret_x`, which walks the RUN's glyph advances.
        // That was right when the page's own glyphs were the only thing on
        // screen. It is wrong now: the draft is drawn in the shell's font
        // inside an editor box, so a caret placed by the document's metrics
        // would sit somewhere other than between the characters the operator
        // can actually see — and the further they typed, the further out it
        // would drift.
        //
        // This is the same class of defect as the vertex drag that tracked at
        // `1/zoom`: two derivations of one position, agreeing at first and
        // separating under use. So there is one derivation. The preview draws
        // the text; the caret measures **the same string, in the same font, at
        // the same size**, and the two cannot disagree.
        // ★★ Measured from the GALLEY THAT WAS DRAWN, not from a second
        // layout of a prefix string.
        //
        // The prefix trick was right while a draft was one line: lay out
        // `text[..caret]` and its width is the caret's x. It cannot survive
        // wrapping — a prefix laid out on its own breaks in different places
        // from the same characters inside the whole paragraph, so the caret
        // would drift a line at a time, and would be exactly wrong at the point
        // the operator was looking at.
        //
        // `Galley::pos_from_cursor` asks the drawn galley where a character
        // index is, in ITS coordinates, and answers with a rect spanning that
        // row's height. One derivation, wrapped or not, which is the rule this
        // module deleted `caret_x` to establish.
        //
        // ★ The index is a CHARACTER index and `ccursor_from_index` is what
        // takes one — the same unit `Draft::caret` is documented in. Passing a
        // byte offset would compile and would put the caret inside a multi-byte
        // character on any document with an accent in it.
        let slot = laid.pos_from_cursor(egui::text::CCursor::new(draft.caret));
        let x = text_origin.x + slot.min.x;
        painter.line_segment(
            [
                egui::Pos2::new(x, text_origin.y + slot.min.y),
                egui::Pos2::new(x, text_origin.y + slot.max.y),
            ],
            egui::Stroke::new(1.5, theme.palette.accent),
        );
    }
}

// ★★ `caret_x` was DELETED on 2026-08-20, with the live preview, and the reason
// is worth keeping.
//
// It derived the caret's position from the RUN's own glyph advances — exact
// while `caret <= glyphs.len()` and extrapolated beyond it, with a doc comment
// explaining the approximation honestly.
//
// The live preview made it wrong rather than approximate. The draft is now drawn
// in the shell's font inside an in-place editor box, so a caret placed by the
// DOCUMENT's metrics would sit somewhere other than between the characters the
// operator can see — and would drift further with every keystroke. Two
// derivations of one position, agreeing at first and separating under use, which
// is the same class of defect as the vertex drag that tracked at `1/zoom`.
//
// So there is one derivation now: the preview draws the string, and the caret
// measures the same string in the same font at the same size, inline above.
// Removed rather than left unused, because a plausible-looking helper is
// something a later hand reaches for.

/// The draft's box in **canvas** space, or `None` when it cannot be derived.
///
/// For an existing run this is the union of its glyph boxes; for a new-text
/// origin it is a nominal one-line box at the click. Both are converted through
/// [`crate::viewer::pdf_space_to_canvas`], the inverse of the bridge
/// [`resolve_run`] uses, so the caret lands on the glyphs it was resolved from.
/// **Highlight what is selected**, one rectangle per run of characters that
/// share a row.
///
/// # ★★ Why it is measured character by character rather than from two
/// # endpoints
///
/// Because a selection can wrap. Two endpoint rectangles describe a selection
/// on one row and say nothing useful about one spanning three — the middle rows
/// are not between the two x-coordinates in any sense a painter can use, and
/// reconstructing them means asking the galley for its row geometry, which is
/// a second derivation of what `pos_from_cursor` already answers.
///
/// So each character's own slot is asked for, and adjacent slots on the same
/// row are merged into one rectangle. The cost is O(n) in the SELECTION's
/// length, per frame — and a draft is one show operator, so n is tens of
/// characters. That is the same trade `Draft::caret` makes for character
/// indices, made again for the same reason.
///
/// ★ A character's right edge is taken from **the next slot's left edge**,
/// not from its own `max.x`. The two differ where a row ends: the last
/// character of a wrapped row has a next slot on the row BELOW, which is how
/// the row break is detected at all.
fn selection(
    painter: &egui::Painter,
    draft: &crate::canvas::textedit::Draft,
    laid: &std::sync::Arc<egui::Galley>,
    origin: egui::Pos2,
    fill: egui::Color32,
) {
    let Some((from, to)) = crate::canvas::textedit::caret::range(draft.mark, draft.caret) else {
        return;
    };
    let slot = |i: usize| laid.pos_from_cursor(egui::text::CCursor::new(i));
    let mut run: Option<egui::Rect> = None;
    for i in from..to {
        let here = slot(i);
        let next = slot(i + 1);
        // Same row when the tops agree. The next slot is on the row below at a
        // wrap, and its own `min.x` is then meaningless as a right edge.
        let wraps = (next.min.y - here.min.y).abs() > 0.5;
        let right = if wraps { here.max.x } else { next.min.x };
        let cell = egui::Rect::from_min_max(
            egui::pos2(origin.x + here.min.x, origin.y + here.min.y),
            egui::pos2(origin.x + right, origin.y + here.max.y),
        );
        run = match run {
            Some(open) if !wraps && (open.top() - cell.top()).abs() < 0.5 => Some(open.union(cell)),
            Some(open) => {
                painter.rect_filled(open.union(cell), 0.0, fill);
                None
            }
            None => Some(cell),
        };
    }
    if let Some(open) = run {
        painter.rect_filled(open, 0.0, fill);
    }
}

fn caret_box(
    doc: &OpenDoc,
    draft: &Draft,
    page: &pdfcer_core::page_tree::Page,
) -> Option<egui::Rect> {
    match &draft.anchor {
        Anchor::Run { run, .. } => {
            let text = doc.page_text()?;
            let r = text.runs.get(*run)?;
            let mut acc: Option<egui::Rect> = None;
            for g in &r.glyphs {
                let lo =
                    crate::viewer::pdf_space_to_canvas(Pos2::new(g.x, g.y + g.size * -0.25), page)?;
                let hi = crate::viewer::pdf_space_to_canvas(
                    Pos2::new(g.x + g.advance, g.y + g.size * 0.9),
                    page,
                )?;
                let b = egui::Rect::from_two_pos(lo, hi);
                acc = Some(acc.map_or(b, |a| a.union(b)));
            }
            acc
        }
        Anchor::Origin { x, y } => {
            #[allow(clippy::cast_possible_truncation)]
            let (x, y) = (*x as f32, *y as f32);
            let lo = crate::viewer::pdf_space_to_canvas(Pos2::new(x, y - 3.0), page)?;
            let hi = crate::viewer::pdf_space_to_canvas(Pos2::new(x + 6.0, y + 11.0), page)?;
            Some(egui::Rect::from_two_pos(lo, hi))
        }
        // ★ A box's caret box is a nominal ONE-LINE slot at the box's TOP-LEFT,
        // not the whole rectangle.
        //
        // Because that is where the first character will land: `add_text`'s
        // boxed variant is top-anchored, so the text grows down from the top
        // edge. Drawing the caret as the full box would be drawing the
        // *container* and calling it a cursor — and would put a blinking bar
        // the height of a paragraph on the page before a single letter existed.
        //
        // The box itself IS drawn, separately and as a rubber-band outline, by
        // `preview` — which is the honest division: the outline says *"your
        // text will live in here"* and the caret says *"and the next keystroke
        // goes here."*
        Anchor::Box { llx, ury, .. } => {
            #[allow(clippy::cast_possible_truncation)]
            let (x, y) = (*llx as f32, *ury as f32);
            let lo = crate::viewer::pdf_space_to_canvas(Pos2::new(x, y - 14.0), page)?;
            let hi = crate::viewer::pdf_space_to_canvas(Pos2::new(x + 6.0, y), page)?;
            Some(egui::Rect::from_two_pos(lo, hi))
        }
    }
}
