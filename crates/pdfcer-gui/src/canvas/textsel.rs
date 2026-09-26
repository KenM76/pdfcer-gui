//! # `canvas::textsel` — selecting text on the page, and copying what was selected
//!
//! The operator's rule for Read mode is *"the document shouldn't allow editing
//! and should allow only selecting of objects that acrobat reader would
//! allow."* `app::modes::capability` owns the first half — Read refuses every
//! content gesture. This module is the second half, and it is a widening rather
//! than a narrowing: **Reader allows text selection**, so a Read mode that
//! refused it would be less than "only what Reader allows", not a cautious
//! reading of it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textsel.md`.

use std::collections::HashMap;

use egui::{Pos2, Rect};
use pdfcer_core::annot_author::Quad;
use pdfcer_core::page_tree::{Page, Rect as PdfRect};
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};
use pdfcer_core::text_extract::PageText;

use bands::{Accum, Band};

/// The two keyboard verbs and the clipboard write. See §9.
pub mod clipboard;

/// **Who owns the primary button** — the mode gate and the whole argument for
/// it. See §3.
pub mod gate;

/// **How a selection's glyph cells become the boxes it paints and marks** — the
/// accumulation half of §5, in the two frames §8 made necessary.
use pdfcer_gui_base::textselbands as bands;

/// **The rotated-text page §8's rules are tested on** — test-only.
#[cfg(test)]
pub mod fixture;

pub use clipboard::{TextKey, apply_key, copy, pending_key};

/// Re-exported flat, so every call site still writes `textsel::takes_the_press`
/// and nothing outside `canvas/` learns that the module was split. The same
/// contract [`clipboard`]'s re-export above honours, for the same reason.
pub use gate::takes_the_press;

/// How far above the baseline a glyph's box reaches, as a fraction of its
/// effective font size.
const GLYPH_ASCENT: f32 = 0.85;

/// How far below the baseline a glyph's box reaches, as a fraction of its
/// effective font size. The descender half of [`GLYPH_ASCENT`]'s pairing.
const GLYPH_DESCENT: f32 = 0.22;

/// **A range of characters on one page, and everything derived from it.**
#[derive(Debug, Clone, PartialEq)]
pub struct TextSelection {
    /// Which page the range is on. A selection is single-page — module header
    /// §4 — so this is a fact about the whole value rather than about one end.
    pub page: usize,
    /// Where the gesture started. Held so a drag or a Shift+click can extend
    /// **from** it: the anchor is the end the operator is not moving, and
    /// re-deriving it from the quads would be impossible once the focus has
    /// crossed it.
    anchor: TextPosition,
    /// Where the pointer is now. The end a drag moves.
    focus: TextPosition,
    /// The [`crate::app::state::OpenDoc::edit_epoch`] the positions above were
    /// resolved against. See the module header §7 — this is the whole of the
    /// staleness mechanism.
    epoch: u64,
    /// The selected glyphs' boxes, **in canvas space**, one per line of the
    /// selection.
    ///
    /// Canvas space (Y-down, page top-left, `/Rotate` applied) rather than PDF
    /// user space, and projected once here rather than per frame, for the
    /// reason `crate::find::Hit::canvas` gives for doing the same: page
    /// geometry cannot change while a document is open, so the answer is
    /// constant for the life of the selection, and the paint path becomes a
    /// projection with no PDF concepts in it at all.
    ///
    /// One box per line rather than one per glyph — a hundred adjacent
    /// rectangles paint as one band anyway, and merging them is what lets a
    /// selection over a paragraph cost four boxes instead of four hundred.
    pub quads: Vec<Rect>,
    /// **The same boxes, in PDF user space** — ready to become a text
    /// markup's `/QuadPoints`.
    ///
    /// One entry per entry of [`Self::quads`], in the same order, from the same
    /// accumulation in [`resolve`]. Not a conversion *of* that field and not a
    /// second walk: the walk produces one `Vec` of PDF-space rectangles and both
    /// of these are built from it, which is what makes *"what is highlighted is
    /// what is marked"* true by construction rather than by two functions
    /// agreeing. Module header §5.1 carries the argument, including why
    /// inverting the canvas projection at the authoring site is the wrong answer
    /// on a rotated page.
    ///
    /// `Quad` rather than `Rect` because that is the type
    /// [`pdfcer_core::annot_author::MarkupSpec::TextMarkup`] takes, and building
    /// it here — once, from the rectangle the glyphs actually produced — leaves
    /// the authoring site with nothing geometric to decide.
    pub page_quads: Vec<Quad>,
    /// **Exactly the characters those boxes cover**, ready for the clipboard.
    ///
    /// Includes the engine's derived word spaces and line breaks, because they
    /// are runs in their own right and the walk passes straight through them —
    /// which is what makes a copied paragraph read as a paragraph rather than
    /// as one unbroken word.
    pub text: String,
}

impl TextSelection {
    /// Whether this selection still describes the revision it was made
    /// against.
    #[must_use]
    pub fn live(&self, epoch: u64) -> bool {
        self.epoch == epoch
    }

    /// **Which runs of the page's extraction this selection covers**, low
    /// to high, or nothing when the revision has moved.
    #[must_use]
    pub fn runs(&self, epoch: u64) -> Vec<usize> {
        if !self.live(epoch) {
            return Vec::new();
        }
        let (start, end) = ordered(self.anchor, self.focus);
        (start.run..=end.run).collect()
    }

    /// The quads to paint on `page`, or nothing at all.
    #[must_use]
    pub fn highlights(&self, page: usize, epoch: u64) -> &[Rect] {
        if self.page == page && self.live(epoch) {
            &self.quads
        } else {
            &[]
        }
    }

    /// A selection built from nothing but a page, a revision and a list of
    /// page-space boxes — for the tests of the modules that **consume** one.
    #[cfg(test)]
    #[must_use]
    pub fn for_test(page: usize, epoch: u64, page_quads: Vec<Quad>) -> Self {
        let quads = page_quads
            .iter()
            .map(|q| {
                Rect::from_min_max(
                    Pos2::new(q.ll.0 as f32, q.ll.1 as f32),
                    Pos2::new(q.ur.0 as f32, q.ur.1 as f32),
                )
            })
            .collect();
        Self {
            page,
            anchor: TextPosition::new(0, 0),
            focus: TextPosition::new(0, 0),
            epoch,
            quads,
            page_quads,
            // Not read by anything this constructor exists for; a copy is what
            // `resolve` produces from real runs, and inventing plausible prose
            // here would make a test look like it was about the text when it is
            // about the geometry.
            text: String::new(),
        }
    }

    /// **The quads a text markup would be authored from**, or nothing at all.
    #[must_use]
    pub fn marks(&self, epoch: u64) -> &[Quad] {
        if self.live(epoch) {
            &self.page_quads
        } else {
            &[]
        }
    }

    /// How many characters are selected. For the trace line and for tests.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the selection covers nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The facts about one page that every entry point below needs.
#[derive(Clone, Copy)]
pub struct PageContext<'a> {
    /// The page's extracted text, from
    /// [`crate::app::state::OpenDoc::page_text`]. **The** extraction — see
    /// that method's docs on why there is only one.
    pub text: &'a PageText,
    /// The page itself, for the PDF-user-space → canvas projection.
    pub page: &'a Page,
    /// Which page this is, in the session's page space.
    pub index: usize,
    /// The revision the extraction describes, stamped onto any selection this
    /// produces. Module header §7.
    pub epoch: u64,
}

/// **Run one frame of a sweep gesture against the open document**, and trace it.
#[must_use]
pub fn sweep(
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    from: Pos2,
    to: Pos2,
    phase: crate::canvas::gesture::Phase,
) -> Option<TextSelection> {
    let selection =
        if let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index)) {
            // THE page's extraction, from `OpenDoc::page_text` — never a fresh
            // one built here. A second extraction with its own options would
            // segment lines differently from the one the page was drawn and
            // cached from, so a sweep would select against text that does not
            // match what is on screen.
            let ctx = PageContext {
                text: &page_text,
                page,
                index: page_index,
                epoch: doc.edit_epoch,
            };
            drag(&ctx, from, to)
        } else {
            None
        };
    crate::canvas::trace::text_selection(
        page_index,
        selection.as_ref(),
        if matches!(phase, crate::canvas::gesture::Phase::Complete) {
            "drag" // ui-text-exempt: trace token, never displayed
        } else {
            "sweep" // ui-text-exempt: trace token, never displayed
        },
    );
    selection
}

/// **Re-resolve a selection against the revision that just replaced it.**
///
/// What makes a restyle repeatable without re-sweeping. Pressing Bold is an
/// edit; an edit bumps [`crate::app::state::OpenDoc::edit_epoch`];
/// [`TextSelection::live`] then answers `false`, the wash vanishes and
/// [`TextSelection::runs`] returns an empty list. Without this, pressing Italic
/// straight afterwards restyles nothing and the operator has to sweep the same
/// words again between every pair of presses (`OPERATOR_REQUESTS.md` O198).
///
/// # THE STALENESS RULE IS NOT RELAXED. THE GEOMETRY IS REBUILT.
///
/// Module header §7 rejects two wrong answers, and this is neither of them.
/// It does **not** re-stamp the old selection with a new epoch — that is
/// "draw the old geometry anyway", and a restyle that changed a point size
/// moves every glyph after it, so the stored quads would wash the wrong pixels.
/// It does not re-resolve on a timer or per frame either. It re-runs the FULL
/// resolution — a fresh extraction, a fresh range, fresh quads, fresh text —
/// from the two positions the operator's own gesture set, at exactly one
/// moment: immediately after an edit that claimed not to change the text.
///
/// # THE GUARD, AND WHY IT IS THE COVERED CHARACTERS
///
/// A restyle changes how text looks and never what it says. So the covered
/// string is an invariant the caller can check, and this function checks it:
/// **if the re-resolved selection does not cover character-for-character what
/// the old one covered, `None` is returned and the selection is dropped.**
///
/// That is what makes this safe against the thing module header §7 is really
/// afraid of — a `(run, byte)` position naming different glyphs after the run
/// indices renumber. If the engine ever splits, merges or re-orders runs on a
/// restyle, the covered text moves and this declines. The shell does not have
/// to know whether `format_text` renumbers; it measures.
///
/// It is deliberately NOT a check that the run ORDINALS are unchanged. A
/// producer that re-emits a title block as two operators instead of three has
/// renumbered nothing the operator can see, and the characters are the thing
/// the operator swept.
///
/// # Returns
///
/// `None` when the positions no longer resolve to anything, when they resolve
/// to different characters, or when the page carries no extractable text. The
/// caller assigns the result, so `None` means "drop the selection" — reached by
/// measurement rather than by assumption.
#[must_use]
pub fn reresolve(ctx: &PageContext<'_>, previous: &TextSelection) -> Option<TextSelection> {
    if previous.page != ctx.index {
        return None;
    }
    let model = model(ctx);
    let renewed = resolve(&model, ctx, previous.anchor, previous.focus)?;
    (renewed.text == previous.text).then_some(renewed)
}

/// **Update the selection from a drag** — press at `from`, pointer now at `to`,
/// both in canvas space.
pub fn drag(ctx: &PageContext<'_>, from: Pos2, to: Pos2) -> Option<TextSelection> {
    let model = model(ctx);
    let anchor = hit(&model, ctx, from)?;
    // A pointer that has run off the end of the text does not cancel the
    // sweep; it stops extending it. See [`clamp_to_text`].
    let focus = match hit(&model, ctx, to) {
        Some(focus) => focus,
        None => clamp_to_text(&model, ctx, from, to)?,
    };
    resolve(&model, ctx, anchor, focus)
}

/// How many points along `from`..`to` are tried before the clamp gives up,
/// and how many halvings sharpen the one that answered.
const CLAMP_SCAN: u32 = 32;
/// The reciprocal, stated rather than divided, so the sample positions are
/// exact f32 and no cast appears in the scan.
const CLAMP_STEP: f32 = 1.0 / 32.0;
const CLAMP_SHARPEN: u32 = 8;

/// **The furthest point along `from`..`to` that still lands in text.**
fn clamp_to_text(
    model: &EditableTextModel<'_>,
    ctx: &PageContext<'_>,
    from: Pos2,
    to: Pos2,
) -> Option<TextPosition> {
    let at = |t: f32| from + (to - from) * t;
    // Backwards from the pointer. `t = 1.0` is known to fail — it is the
    // caller's own miss — so the scan starts one step in.
    let mut hit_t = None;
    let mut t = 1.0 - CLAMP_STEP;
    for _ in 1..CLAMP_SCAN {
        if hit(model, ctx, at(t)).is_some() {
            hit_t = Some(t);
            break;
        }
        t -= CLAMP_STEP;
    }
    let mut lo = hit_t?;
    // Sharpen towards the far edge of the text, so the clamped caret sits at
    // the end of the run rather than up to one scan step short of it.
    let mut hi = (lo + CLAMP_STEP).min(1.0);
    let mut best = hit(model, ctx, at(lo))?;
    for _ in 0..CLAMP_SHARPEN {
        let mid = f32::midpoint(lo, hi);
        match hit(model, ctx, at(mid)) {
            Some(pos) => {
                best = pos;
                lo = mid;
            }
            None => hi = mid,
        }
    }
    Some(best)
}

/// **Update the selection from a click.**
#[must_use]
pub fn click(
    ctx: &PageContext<'_>,
    current: Option<&TextSelection>,
    point: Pos2,
    shift: bool,
    double: bool,
    triple: bool,
) -> Option<TextSelection> {
    let model = model(ctx);
    let at = hit(&model, ctx, point)?;
    if triple {
        // `line_range_at` answers `None` for a position on a run carrying no
        // clustered glyph, which `hit_test` does not produce — but a `None`
        // here must clear rather than fall through to the word case, or an
        // emphatic gesture would quietly become a weaker one.
        let (start, end) = model.line_range_at(at)?;
        return resolve(&model, ctx, start, end);
    }
    if double {
        let (start, end) = model.word_range_at(at);
        return resolve(&model, ctx, start, end);
    }
    if shift && let Some(current) = current.filter(|c| c.page == ctx.index && c.live(ctx.epoch)) {
        return resolve(&model, ctx, current.anchor, at);
    }
    // A plain click collapses the range onto one caret slot, which covers no
    // glyphs, so `resolve` answers `None` and the caller clears. Expressed as a
    // degenerate range rather than as an early `return None` on purpose: there
    // is one rule for what a range means, and "a click is an empty range" is a
    // statement in that rule rather than an exception beside it.
    resolve(&model, ctx, at, at)
}

/// **Select every character on the page** — Ctrl+A. Module header §1.3.
#[must_use]
pub fn select_all(ctx: &PageContext<'_>) -> Option<TextSelection> {
    let model = model(ctx);
    let last = ctx.text.runs.len().checked_sub(1)?;
    let end = ctx.text.runs.get(last)?.text.len();
    resolve(
        &model,
        ctx,
        TextPosition::new(0, 0),
        TextPosition::new(last, end),
    )
}

/// Build the derived line/column/block structure over `ctx.text`.
fn model<'a>(ctx: &PageContext<'a>) -> EditableTextModel<'a> {
    EditableTextModel::recognize(ctx.text, &BlockRecognitionOptions::default())
}

/// **Is `canvas` inside the box of any text run on this page?**
#[must_use]
pub fn word_at(ctx: &PageContext<'_>, canvas: Pos2) -> Option<()> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    let (x, y) = (f64::from(pdf.x), f64::from(pdf.y));
    ctx.text
        .runs
        .iter()
        .filter_map(|run| run.bbox)
        .any(|b| x >= b.llx && x <= b.urx && y >= b.lly && y <= b.ury)
        .then_some(())
}

/// Where a canvas-space point lands in the page's text.
fn hit(model: &EditableTextModel<'_>, ctx: &PageContext<'_>, canvas: Pos2) -> Option<TextPosition> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    // ONE call, and no shell-side rotated-band pass in front of it.
    // `EditableTextModel::hit_test` projects the point onto the line, so a press
    // in the middle of a 90° letter lands on that letter.
    //
    // Do not reintroduce a private band here as a fallback: a shell copy of a
    // rule the engine owns keeps compiling and keeps returning something
    // plausible long after the two have diverged, and the symptom — a sweep one
    // letter short, or a sweep that selects nothing — looks like a gesture bug
    // rather than a duplicated rule.
    //
    // The nearest-line fallback inside `hit_test` is bounded at one
    // line-height and that bound is load-bearing: it is what makes this a
    // presence test. `clamp_to_text` handles the overshoot the bound creates.
    model.hit_test(f64::from(pdf.x), f64::from(pdf.y))
}

/// **Which way the text under `canvas` runs**, in CANVAS space, as an angle
/// in degrees from the horizontal — or `None` where the pointer is not over
/// rotated text.
#[must_use]
pub fn tilt_at(ctx: &PageContext<'_>, canvas: Pos2) -> Option<f32> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    // The engine's own answer: `Line::direction` is the unit vector every
    // glyph on that line shares, sourced from the §9.4.4 text rendering matrix
    // rather than recovered from glyph origins. A shell-side census over origins
    // can come up empty on exactly the sparse rotated stamp it would be written
    // for, which is why the direction is read and never inferred.
    //
    // Containment, not nearest-line — see the doc above. `Line::bbox` is
    // computed in the line's own frame, so for a 90° line it is the tall narrow
    // box the ink actually occupies.
    let model = model(ctx);
    let dir = model
        .lines()
        .iter()
        .find(|line| {
            let (x, y) = (f64::from(pdf.x), f64::from(pdf.y));
            line.bbox.llx <= x && x <= line.bbox.urx && line.bbox.lly <= y && y <= line.bbox.ury
        })
        .map(|line| line.direction)
        // Horizontal text needs no tilt, and answering `0.0` for it would make
        // every ordinary page pay for a bitmap the cursor already has.
        .filter(|dir| (dir.1).abs() > f32::EPSILON || dir.0 < 0.0)?;
    // One point on the direction and one a short way along it. The length is
    // arbitrary — only the difference is used — but it is a whole point rather
    // than an epsilon so the subtraction below is nowhere near `f32`
    // cancellation at page coordinates.
    let from = crate::viewer::pdf_space_to_canvas(egui::pos2(pdf.x, pdf.y), ctx.page)?;
    let to =
        crate::viewer::pdf_space_to_canvas(egui::pos2(pdf.x + dir.0, pdf.y + dir.1), ctx.page)?;
    let step = to - from;
    if step.length_sq() < f32::EPSILON {
        return None;
    }
    Some(step.y.atan2(step.x).to_degrees())
}

/// **Does this line run in a direction the page-axis box would get wrong?**
fn is_rotated(model: &EditableTextModel<'_>, line: usize) -> bool {
    model.lines().get(line).is_some_and(|line| {
        let (dx, dy) = line.direction;
        dy.abs() > f32::EPSILON || dx < 0.0
    })
}

/// **The one derivation** — module header §5.
fn resolve(
    model: &EditableTextModel<'_>,
    ctx: &PageContext<'_>,
    anchor: TextPosition,
    focus: TextPosition,
) -> Option<TextSelection> {
    let covered = model.resolve_range(anchor, focus);
    if covered.is_empty() {
        return None;
    }

    // Which line the engine clustered each glyph onto. Built from
    // `model.lines()` rather than by re-clustering on baseline y: the engine's
    // lines already account for the backward-jump split that separates two
    // columns sharing a baseline (module header §4), and a box drawn from a
    // second clustering would span a column gap the copy does not.
    let mut line_of: HashMap<(usize, usize), usize> = HashMap::new();
    for (index, line) in model.lines().iter().enumerate() {
        for gref in &line.glyphs {
            line_of.insert((gref.run, gref.glyph), index);
        }
    }

    // The boxes, in the order their lines are first met, so a selection's quads
    // are in the same content order as its text. `Vec` rather than a map keyed
    // on the line index: the count is one per line of the selection, so a linear
    // scan is cheaper than hashing, and the order is the point.
    let mut boxes: Vec<(Band, Accum)> = Vec::new();
    for gref in &covered {
        let Some(glyph) = model.glyph(*gref) else {
            continue;
        };
        // Which frame this glyph's cell is measured in, decided on the
        // ENGINE's `Line::direction`. A glyph on a rotated line is banded with
        // its own line, in that line's axes; every other glyph keeps the
        // engine's line and page axes. The two never mix, because a `Band`
        // carries which it is.
        //
        // A glyph the line clustering did not claim still has to be drawn, or a
        // selection would silently highlight less than it copies. `Band::Loose`
        // keyed per glyph gives those a box each — visibly correct, and rare
        // enough that the cost is not worth a second clustering rule.
        let band = match line_of.get(&(gref.run, gref.glyph)) {
            Some(&line) if is_rotated(model, line) => Band::Rotated(line),
            Some(&line) => Band::Engine(line),
            None => Band::Loose(gref.run, gref.glyph),
        };
        let cell = match band {
            // The engine's own approximation of a glyph box, with the ascent
            // and descent fractions chosen in the module header §5.
            Band::Engine(_) | Band::Loose(..) => {
                let (x0, x1) = (glyph.x, glyph.x + glyph.advance);
                Accum::Page(PdfRect::from_corners(
                    f64::from(x0.min(x1)),
                    f64::from(glyph.y - glyph.size * GLYPH_DESCENT),
                    f64::from(x0.max(x1)),
                    f64::from(glyph.y + glyph.size * GLYPH_ASCENT),
                ))
            }
            // The same box, in the line's frame: one advance along the writing
            // direction, and the same ascender/descender span across it. For a
            // horizontal direction this reduces to the expression above, which
            // is the check that it is a generalisation and not a second rule.
            Band::Rotated(line) => {
                let dir = model.lines()[line].direction;
                Accum::Frame {
                    dir,
                    origin: (glyph.x, glyph.y),
                    along: (0.0, glyph.advance),
                    perp: (-glyph.size * GLYPH_DESCENT, glyph.size * GLYPH_ASCENT),
                }
            }
        };
        match boxes.iter_mut().find(|(k, _)| *k == band && k.merges()) {
            Some((_, accum)) => accum.absorb(&cell),
            None => boxes.push((band, cell)),
        }
    }

    // The characters, from the runs themselves. `get(..)` rather than indexing:
    // core guarantees a `TextPosition`'s offset is on a glyph boundary and
    // therefore on a UTF-8 boundary, and a stale position that is not must
    // contribute nothing rather than panic in the middle of a drag.
    let (start, end) = ordered(anchor, focus);
    let mut text = String::new();
    for index in start.run..=end.run.min(ctx.text.runs.len().saturating_sub(1)) {
        let Some(run) = ctx.text.runs.get(index) else {
            break;
        };
        let lo = if index == start.run {
            start.byte_offset
        } else {
            0
        };
        let hi = if index == end.run {
            end.byte_offset
        } else {
            run.text.len()
        };
        // Every run is copied verbatim, including the extraction's derived
        // word spaces and line breaks. There is deliberately no filter here for
        // spurious breaks inside a rotated line: the extraction resolves a
        // baseline step into the line's own frame, so it emits none. A filter
        // against something that is no longer produced is a filter that will one
        // day remove something real.
        if let Some(slice) = run.text.get(lo..hi) {
            text.push_str(slice);
        }
    }

    // The projection into canvas space, through `find::reveal::quad_to_canvas`
    // — the SAME function Find projects its hits with. Reusing it rather than
    // mapping two corners here is what makes a selection box and a find box over
    // the same word land in the same place on a rotated page: it maps all four
    // corners and bounds them, because `/Rotate 90` sends the `ul`/`lr` pair to
    // two corners that are no longer the extremes.
    //
    // **Both spaces are kept, and they are pushed in the same iteration** —
    // module header §5.1. A box whose projection declines contributes to
    // *neither*: the two vectors are index-aligned by construction, and a
    // `filter_map` on one with a plain `map` on the other would let the wash and
    // the authored mark describe different sets of glyphs, in the direction
    // nobody would notice (the mark is in the file; the wash is gone by the next
    // frame).
    let mut quads: Vec<Rect> = Vec::with_capacity(boxes.len());
    let mut page_quads: Vec<Quad> = Vec::with_capacity(boxes.len());
    for (_, accum) in boxes {
        let quad = accum.quad();
        if let Some(canvas) = crate::find::reveal::quad_to_canvas(&quad, ctx.page) {
            quads.push(canvas);
            page_quads.push(quad);
        }
    }
    if quads.is_empty() {
        return None;
    }

    Some(TextSelection {
        page: ctx.index,
        anchor,
        focus,
        epoch: ctx.epoch,
        quads,
        page_quads,
        text,
    })
}

/// The two positions in content order.
fn ordered(a: TextPosition, b: TextPosition) -> (TextPosition, TextPosition) {
    if (a.run, a.byte_offset) <= (b.run, b.byte_offset) {
        (a, b)
    } else {
        (b, a)
    }
}

/// **Answer the text selection's own two chords, Ctrl+A and Ctrl+C.**
pub fn keys(
    ctx: &egui::Context,
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    active_tool: crate::canvas::tool::CanvasTool,
    caps: crate::app::modes::Capabilities,
    selection: &mut Option<TextSelection>,
) {
    if let Some(key) = pending_key(ctx)
        && takes_the_press(active_tool, caps)
        && let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index))
    {
        let text_ctx = PageContext {
            text: &page_text,
            page,
            index: page_index,
            epoch: doc.edit_epoch,
        };
        apply_key(ctx, &text_ctx, key, selection);
    }
}

#[cfg(test)]
mod tests;
