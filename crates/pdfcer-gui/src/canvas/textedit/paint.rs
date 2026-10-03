//! # `canvas::textedit::paint` — **what a draft looks like on the page**
//!
//! A draft on an existing run is drawn in the run's own font by
//! [`super::shaped`]; everything below is the shell-font box used otherwise.
//!
//! ## Why this is its own file
//!
//!
//! ## The one thing to know before changing anything in here
//!
//! **The draft is drawn ONCE and measured ONCE.** The editor box's text and the
//! caret's position come from the same string, the same `FontId` and the same
//! size, in that order, a few lines apart. That is not tidiness — it is the
//! defect this file was rewritten to remove.
//!
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/paint.md`.

use egui::{Pos2, Ui};

use pdfcer_gui_base::text::previewfallback::PreviewFallback;

use super::{Anchor, Draft, Preview, read};
use crate::app::state::OpenDoc;

/// The smallest the editor box for NEW text may be drawn, in points.
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
    super::ime::show(ui, ctx, screen);
    // An existing run is drawn in its own font, where it is, when the engine
    // can lay it out (`shaped`). Otherwise, and for new text, an opaque
    // editor box in the shell's font: it covers the original glyphs, which
    // are baked into the page raster, and promises nothing about typeface or
    // metrics that the commit would then break.
    let reason = match super::shaped::read(ctx, &draft) {
        Some(s) if super::shaped::paint(ui, ctx, p, &draft, &s, screen) => {
            super::fallback::publish(ctx, &draft, None, 0.0);
            return;
        }
        Some(_) => Some(Some(PreviewFallback::TooLarge)),
        None => super::shaped::fallback(ctx, &draft),
    };
    let (height, font_pt) = sizes(p, &draft, page, screen);
    // Unknown until the draft's first layout; published once it is known.
    if let Some(why) = reason {
        super::fallback::publish(ctx, &draft, why, font_pt);
    }
    // ONE font binding and ONE layout, shared by the fill, the text and the
    // caret below. Two `FontId`s built separately would be two derivations of
    // one fact, and the caret would sit where a *slightly different* string
    // would have ended. See this module's header.
    let font = egui::FontId::proportional(font_pt);
    // A BOX DRAFT WRAPS; EVERY OTHER DRAFT DOES NOT.
    //
    // The operator, 2026-08-21: *"I should be able to make it multi line."*
    //
    // `Anchor::Box` is the anchor a dragged rectangle produces, and the whole
    // point of that rectangle is a **width to wrap against** — a PDF has no
    // paragraph, so each visual line is its own show operator and something has
    // to decide where the second line starts.
    //
    // The preview therefore wraps at **the box's own screen width**, which is
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
        Anchor::Box { llx, urx, .. } | Anchor::Block { llx, urx, .. } => {
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

    // THE BOX GROWS WITH WHAT IS IN IT, and it has to.
    //
    // It was `screen.shrink(1.0)` — the glyph box, exactly — until this was
    // driven and looked at. Two ways that is wrong, and the second is the
    // serious one:
    //
    // 1. **A draft longer than the run it replaces** overflows a box sized to
    //    the original, so the tail of what you are typing sits on bare page.
    // 2. **An `Anchor::Origin` draft has no glyph box at all.** `caret_box`
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
    // A BOX GROWS DOWNWARD FROM ITS TOP EDGE, and a single-line draft stays
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
        let height = height.max(laid.rect.height());
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
    // THE SELECTION IS DRAWN UNDER THE TEXT, before the galley, so the
    // characters sit ON the highlight rather than behind it. Drawing it after
    // would need a translucent fill and would tint every glyph it covers.
    //
    // This is a **cursor**, not content marking, and R8b rule 4 permits it
    // for exactly that reason: it shows what the *next keystroke* will replace
    // and it is gone the moment the draft commits. Nothing about the applied
    // document is styled here.
    // The box, published for the HARNESS as well as for the pointer
    // handlers. A driven check that wants to sweep across a draft has no other
    // way to find it: the editor is painted into the canvas rather than laid
    // out as a widget, so it appears in no layout the harness can read, and a
    // check aiming at it from the run's page coordinates would be aiming at
    // the glyphs the box is covering rather than at the box.
    crate::diag::ui_rect(REGION_BOX, body);
    // Publish the box and the galley for the pointer handlers. See
    // `canvas::textedit::hit`: this is the ONE layout, and hit-testing it is
    // the inverse of the `pos_from_cursor` the caret is drawn with.
    crate::canvas::textedit::hit::publish(
        ctx,
        crate::canvas::textedit::hit::Layout {
            body,
            body_canvas: egui::Rect::from_two_pos(p.map.to_page(body.min), p.map.to_page(body.max)),
            caret: crate::canvas::textedit::hit::Caret::Galley {
                origin: text_origin,
                galley: laid.clone(),
            },
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
        //
        //
        // This is the same class of defect as the vertex drag that tracked at
        // `1/zoom`: two derivations of one position, agreeing at first and
        // separating under use. So there is one derivation. The preview draws
        // the text; the caret measures **the same string, in the same font, at
        // the same size**, and the two cannot disagree.
        // Measured from the GALLEY THAT WAS DRAWN, not from a second
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
        // The index is a CHARACTER index and `ccursor_from_index` is what
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

/// The editor box's height and its font size, in screen points.
///
/// A draft on an existing run is set at the run's own size times the zoom,
/// unclamped, so the stand-in takes the space the saved text will. New text
/// has no size to take, and keeps a legible clamp.
fn sizes(
    p: &Preview<'_>,
    draft: &Draft,
    page: &pdfcer_core::page_tree::Page,
    screen: egui::Rect,
) -> (f32, f32) {
    if let Anchor::Run { run, .. } | Anchor::Block { run, .. } = &draft.anchor
        && let Some(size) = run_size(p.doc, *run)
        && let Some(scale) = screen_per_point(p, page)
    {
        let font = size * scale;
        return (screen.height().max(font), font);
    }
    let height = screen.height().clamp(MIN_PREVIEW_PT, MAX_PREVIEW_PT);
    (height, height * PREVIEW_FILL)
}

/// The run's largest glyph size, in PDF points.
fn run_size(doc: &OpenDoc, run: usize) -> Option<f32> {
    let text = doc.page_text()?;
    text.runs
        .get(run)?
        .glyphs
        .iter()
        .map(|g| g.size)
        .reduce(f32::max)
        .filter(|s| s.is_finite() && *s > 0.0)
}

/// Screen points per PDF point at the current zoom, rotation-independent.
fn screen_per_point(p: &Preview<'_>, page: &pdfcer_core::page_tree::Page) -> Option<f32> {
    let a = crate::viewer::pdf_space_to_canvas(Pos2::ZERO, page)?;
    let b = crate::viewer::pdf_space_to_canvas(Pos2::new(0.0, 100.0), page)?;
    Some((p.map.to_screen(b) - p.map.to_screen(a)).length() / 100.0)
}

/// The draft's box in **canvas** space, or `None` when it cannot be derived.
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
        // A box's caret box is a nominal ONE-LINE slot at the box's TOP-LEFT,
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
        // The whole paragraph, so the editor box covers every line it replaces.
        Anchor::Block {
            llx, lly, urx, ury, ..
        } => {
            #[allow(clippy::cast_possible_truncation)]
            let (lo, hi) = (
                Pos2::new(*llx as f32, *lly as f32),
                Pos2::new(*urx as f32, *ury as f32),
            );
            let lo = crate::viewer::pdf_space_to_canvas(lo, page)?;
            let hi = crate::viewer::pdf_space_to_canvas(hi, page)?;
            Some(egui::Rect::from_two_pos(lo, hi))
        }
    }
}
