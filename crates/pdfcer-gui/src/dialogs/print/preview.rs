//! # `dialogs::print::preview` — a zoomable, pannable picture of the real sheet
//!
//! ## Why a picture rather than a number
//!
//! Carried across from the old shell with its reasoning intact, because the
//! reasoning is the reason this file was worth salvaging rather than
//! rewriting:
//!
//! > pdfcer diverges from Acrobat here on purpose — Acrobat clips silently
//! > when content falls outside the printable area, and pdfcer says so. That
//! > divergence is worth nothing if the GUI reduces it to a count an operator
//! > can look past.
//! >
//! > The whole reason `pdfcer-print` reads real device geometry instead of
//! > guessing a bounding box is so this can be exact. Drawing only the SHEET
//! > and not the PRINTABLE AREA would be the naive version and would show a
//! > page fitting that will not.
//!
//! And the correction that followed, kept because it records a mistake worth
//! not repeating:
//!
//! > This function computed real device geometry from the first day and still
//! > never drew the page: the "placed" rectangle was a flat fill in the
//! > surface colour. So the preview was not broken — the geometry was right —
//! > it simply answered "where will the page sit" and never "what is on it",
//! > which is the half an operator checking a margin actually needs.
//!
//! ## What is drawn, outermost first
//!
//! Four rectangles and one hatch, in this order, because each is *inside* the
//! previous and the nesting is the information:
//!
//! 1. **The sheet** — `DeviceGeometry::physical_pt`, the whole piece of paper.
//! 2. **The printable area** — inset by the driver's own unprintable margins.
//!    This, not the sheet, is what constrains the job. A preview that showed
//!    only the sheet would show pages fitting that the hardware will crop.
//! 3. **The placed page** — `Placement`'s offset and scale, applied *within*
//!    the printable area.
//! 4. **The page's real content**, rendered through the same options the
//!    spooler uses (see [`super::render_options`]) and drawn into (3).
//! 5. **A hatch over what will be lost**, when the placement reports a clip.
//!    Hatched rather than filled: a hatch means *"this will happen and has
//!    not happened yet"*, which is exactly a pre-print clip. A solid fill
//!    reads as something already done.
//!
//! ## ★ The preview owns NO scroll area, deliberately
//!
//! Zoom is Ctrl+wheel and pan is a primary-button drag. Neither competes with
//! the dialog's own [`egui::ScrollArea`]: per
//! `D:\dev\rag\egui\egui_0.35_zoom_with_keyboard_vs_app_zoom_chords.md` egui
//! splits wheel input at the input-state level, so a wheel event carrying the
//! zoom modifier surfaces as `zoom_delta()` and contributes nothing to
//! `smooth_scroll_delta()` — the two cannot fire from one gesture. A plain
//! wheel over the preview therefore belongs unambiguously to the dialog, and
//! there is no nested consumer to race it. **Scroll-to-pan was rejected for
//! exactly that reason**: it would have made the preview a scroll consumer
//! and put the question back.
//!
//! ## Colours: chrome for the diagram, pass-through for the page
//!
//! Everything this file paints *except the page bitmap* is chrome — a diagram
//! of a piece of paper — and takes its colour from [`egui::Visuals`], the
//! same discipline [`crate::canvas::overlay`] states. The page bitmap is
//! **document content** and is drawn with a white multiplier, which
//! `painter.image` treats as "draw the pixels as rendered". Any palette role
//! there would mean restyling the application restyled the operator's page.

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureId, Ui, Vec2};

use crate::app::state::OpenDoc;
use crate::dialogs::print::PrintDialog;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

/// Width of the preview column, in egui points.
///
/// # ★ A CONSTANT, and that is the whole point
///
/// The dialog body lives inside a [`egui::ScrollArea::both`], and a
/// horizontally scrollable area has no bounded width to report: anything laid
/// out from `ui.available_width()` inside one is being sized from a number
/// that the scroll area is itself deriving from the content. Two fixed
/// columns break that circle — the content width is a constant, so the
/// horizontal scrollbar has something stable to measure and the operator gets
/// a scrollbar instead of a column that grows to meet it.
pub(super) const COLUMN_WIDTH_PTS: f32 = 340.0;

/// Height of the fixed strip under the preview canvas, in egui points.
///
/// # ★ FIXED, so the canvas can never be shrunk by its own caption
///
/// The canvas height is computed as `available − this constant`. Reading the
/// strip's ACTUAL laid-out height instead would reproduce a measured feedback
/// loop exactly: the clip caption wraps to two lines on a narrow window, the
/// strip grows, the canvas shrinks, the sheet is refitted smaller — and the
/// operator watches the preview settle over several frames for no reason they
/// can see. Subtracting a constant means the strip's content cannot reach the
/// canvas at all.
///
/// The window-resize coupling is the DESIRED one and is unaffected: a taller
/// window still means a taller canvas.
const STRIP_HEIGHT_PTS: f32 = 68.0;

/// Smallest preview canvas, in egui points. Below this the sheet outline
/// stops being a picture and becomes a smudge, so the column scrolls rather
/// than shrinking further.
const CANVAS_MIN_HEIGHT_PTS: f32 = 160.0;

/// Largest preview canvas, in egui points.
///
/// A ceiling rather than a preference. `ui.available_height()` inside a
/// scroll area is a value this code does not own; clamping both ends means a
/// surprising answer from egui produces a preview that is merely the wrong
/// size rather than one that allocates a screen-sized rect.
const CANVAS_MAX_HEIGHT_PTS: f32 = 1400.0;

/// The fraction of the canvas the fitted sheet occupies.
///
/// Slightly under 1 so the sheet's own outline is not flush against the
/// canvas edge — the outline is load-bearing here (it is the paper), and an
/// outline touching the boundary reads as content continuing off-screen.
const FIT_MARGIN: f32 = 0.92;

/// Resolution the preview bitmap is rendered at, in DPI.
///
/// Chosen against what the preview is FOR — checking that fine print clears
/// the unprintable margin — rather than against the size it is first drawn
/// at. At fit the bitmap is heavily downsampled, and that headroom is what
/// lets the operator zoom in and still see type rather than a mosaic. It is
/// deliberately NOT the job's own render DPI: the job renders at up to 2400
/// DPI and a preview does not need a 500 MB pixmap to answer a margin
/// question.
const TARGET_DPI: f32 = 150.0;

/// Ceiling on the preview bitmap's longest side, in pixels.
///
/// The DPI figure alone is not a bound: an ISO A0 sheet is 3370 pt on its
/// long side, which at 150 DPI is 7020 px and 190 MB of RGBA. This clamp
/// holds the worst case near 20 MB regardless of page size, which matters
/// because large-format CAD sheets are exactly the document population this
/// project's operator prints.
///
/// **Set ABOVE the office page sizes on purpose.** The first value tried was
/// 1600, which is below US Legal (2100 px) and below US Letter's own 1650 —
/// so the "ceiling for exotic sheets" silently became the scale for every
/// ordinary document, quietly costing preview sharpness on the common case to
/// bound the rare one. 2200 leaves A4 (1754), Letter (1650) and Legal (2100)
/// at the full target DPI and binds only where it was meant to.
/// [`a_letter_page_previews_at_the_target_resolution`] is the test that caught
/// it and is what keeps the two constants in step.
const MAX_SIDE_PX: f32 = 2200.0;

/// Smallest preview zoom, as a multiple of the fit scale.
///
/// Bounded on BOTH sides because zoom is driven by a wheel: an unbounded
/// multiplier reached by a flick leaves the operator staring at one white
/// pixel with no way back except the Fit button they may not have found.
const ZOOM_MIN: f32 = 0.25;
/// Largest preview zoom. See [`ZOOM_MIN`].
const ZOOM_MAX: f32 = 40.0;

/// One zoom step per button press.
const ZOOM_STEP: f32 = 1.25;

/// The egui texture name the preview bitmap is uploaded under.
///
/// ★ **Distinct from `crate::render::raster`'s `PAGE_TEXTURE_ID`, and it has
/// to be.** egui reuses the allocation when the same name is loaded again, so
/// sharing a name with the canvas's page texture would make each surface
/// silently overwrite the other's pixels — the canvas would show the preview's
/// page at the preview's resolution, and neither would look broken enough to
/// investigate. That module's own header already names this hazard: *"The
/// moment a second live page texture exists … this must become a per-texture
/// id."* This is that second live page texture.
const PREVIEW_TEXTURE_ID: &str = "pdfcer-print-preview"; // ui-text-exempt: internal texture id, never displayed

/// What a cached preview bitmap is a picture OF.
///
/// # Every field here is something that changes the pixels
///
/// A cache key is a claim: *"if these are equal, re-rendering would produce
/// the same image."* Getting it wrong in the lax direction is the bug class
/// `RenderKey`'s staleness fields were each added to close — a control that
/// changes the render, does not change the key, and therefore silently does
/// nothing.
///
/// **Orientation is deliberately absent, and the REASON is subtle enough to
/// be worth stating.** Orientation *does* reach planning, so it does change
/// [`crate::dialogs::print::spooler::Placement::scale`] and therefore the
/// rectangle the preview draws. It still changes no pixel of **this bitmap**:
/// the texture is rasterised at [`raster_scale`], which is derived from the
/// page's own size and the preview's target DPI and never from the placement
/// — the placement scales the drawn rectangle, not the raster. Nothing here
/// rotates page content either: the driver turns the sheet, pdfcer does not
/// turn the page. So the key stays as it is, and putting orientation in it
/// would throw the cache away on every radio click for nothing.
///
/// ## ★ The settings field, added 2026-08-17 in the commit that added its control
///
/// This paragraph used to read:
///
/// > **Two fields the old shell's key carried are absent because their inputs
/// > do not exist in this crate yet**: a font-environment generation (nothing
/// > here lets an operator name a font folder) and the CMYK conversion intent
/// > (no settings surface). … this key must gain both in the same commit as
/// > those surfaces, or the preview will keep showing a page rendered under the
/// > previous choice.
///
/// The settings surface landed and the field landed with it, as instructed. It
/// covers **five** rendering settings rather than the one that note anticipated
/// — see [`crate::app::settings::SettingsExt::render_options`].
///
/// A **font-environment generation is still absent**, and still for the stated
/// reason: nothing in this build lets an operator name a font folder, so there
/// is no input to key on. `tools.font_folders` is registered with no dispatch
/// arm; the day it gains one, this key gains a field in the same commit.
///
/// ### Why the whole `Settings` and not the five fields it reads
///
/// Because listing five would be a second statement of which settings affect a
/// render — one here and one in `SettingsExt::render_options` — and the failure
/// mode of the two disagreeing is silent: a sixth rendering setting added to
/// the funnel and not to this list produces a preview that never updates, with
/// no error anywhere. Keying on the whole value cannot drift, and the cost is a
/// `String` comparison on a cache hit against a rasterisation on a miss.
///
/// This is why the type is `Clone`/`PartialEq` rather than `Copy`/`Eq`: the
/// settings carry the theme token, which is a `String`. The theme is not a
/// rendering input, but excluding it would mean naming fields again.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PreviewKey {
    /// Which document page (0-based).
    page: usize,
    /// Which annotation classes are painted.
    scope: pdfcer_render::AnnotationScope,
    /// The operator's configuration, whole — see the type's own docs on why it
    /// is not the five rendering fields spelled out.
    settings: pdfcer_core::settings::Settings,
}

/// Everything the preview needs that is NOT the dialog's own state.
///
/// # Why a struct rather than five parameters
///
/// The preview reads from two different places — the open document and the
/// planned job — and grouping them makes the borrow situation legible: the
/// caller holds `&mut PrintDialog` and these are reads of *disjoint* values,
/// which is the only reason the call compiles at all.
pub(super) struct Inputs<'a> {
    /// The open document, for its pages and its edited view.
    pub(super) doc: &'a OpenDoc,
    /// The job, as planned against real device geometry — the sheet, the
    /// resolution verdict, and one entry per sheet in send order.
    pub(super) job: &'a Job,
    /// Page sizes in **document order**, indexed by
    /// [`crate::dialogs::print::spooler::PagePlan::index`] and never by a
    /// position in the plan list.
    pub(super) page_sizes: &'a [(f64, f64)],
}

/// The preview zoom and pan after multiplying the zoom by `step` while
/// holding the screen point `at` still.
///
/// # ★ The anchor term, derived rather than tuned
///
/// The sheet is drawn at `origin(z) = centre − sheet·fit·z/2 + pan`. Holding
/// the screen point `at` fixed across a zoom from `z0` to `z1 = k·z0`
/// requires `at − (at − origin(z0))·k = origin(z1)`. Substituting both
/// origins collapses every `sheet` and `fit` term:
///
/// ```text
/// pan1 = (at − centre)·(1 − k) + k·pan0
/// ```
///
/// That the page geometry drops out is what makes this correct for a sheet of
/// any size, and it is why the anchor is computed rather than arrived at by
/// nudging the pan until it looked right. Without the anchor, zooming in on
/// the bottom-left corner of a sheet walks it off the canvas and the operator
/// has to hunt it back with a drag.
///
/// A button click passes `at == centre`, which degenerates to
/// `pan1 = k·pan0` — the sheet grows about the middle of the canvas, which is
/// where the operator is looking when they press a button rather than
/// pointing at something.
///
/// # `k` is the EFFECTIVE ratio, after clamping
///
/// Using `step` for the anchor term instead would displace the sheet on a
/// zoom the clamp refused: at maximum zoom, Ctrl+wheel would stop magnifying
/// but keep sliding the page sideways, which reads as the preview drifting on
/// its own.
///
/// A non-finite or non-positive `step` returns the inputs unchanged. egui's
/// `zoom_delta()` is well-behaved, but this is the function a future gesture
/// source would also call, and a `NaN` reaching the pan poisons every
/// subsequent frame's arithmetic with no way back except closing the dialog.
fn zoomed_view(zoom: f32, pan: Vec2, step: f32, at: Pos2, centre: Pos2) -> (f32, Vec2) {
    if !step.is_finite() || step <= 0.0 || !zoom.is_finite() || zoom <= 0.0 {
        return (zoom, pan);
    }
    let after = (zoom * step).clamp(ZOOM_MIN, ZOOM_MAX);
    let k = after / zoom;
    (after, (at - centre) * (1.0 - k) + k * pan)
}

/// The pt-to-pixel scale a preview bitmap is rendered at.
///
/// # Two bounds, and the second one is the load-bearing one
///
/// [`TARGET_DPI`] alone would be a scale, not a bound: it says how finely to
/// render a point and says nothing about how many points there are. An ANSI E
/// sheet is 2448 × 3168 pt, which at 150 DPI is 5100 × 6600 px and 134 MB of
/// RGBA for a picture drawn 300 pt wide. [`MAX_SIDE_PX`] holds that near
/// 15 MB, and it binds on exactly the large-format documents this project's
/// operator prints while leaving every office page size at full resolution.
///
/// The result depends only on the page's own size, so it is fully determined
/// by [`PreviewKey::page`] and does not need to be a key field of its own.
fn raster_scale(page_pt: (f64, f64)) -> f32 {
    let dpi_scale = TARGET_DPI / 72.0;
    let longest = page_pt.0.max(page_pt.1) as f32;
    if !longest.is_finite() || longest <= 0.0 {
        // A degenerate `/MediaBox`. The renderer has its own guards; this
        // only has to avoid handing it a division by zero.
        return dpi_scale;
    }
    dpi_scale.min(MAX_SIDE_PX / longest)
}

/// Draw the preview column: the canvas, then the fixed strip beneath it.
pub(super) fn column(
    ui: &mut Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    column_height: f32,
) {
    let job = inputs.job;
    if job.plans.is_empty() {
        ui.label(t::no_pages_selected());
        return;
    }
    // Which sheet the preview shows. The stepper walks the SELECTED pages,
    // not the document's, because a preview of a page the job does not
    // include would be answering a question nobody asked.
    let shown = dialog.preview_page.min(job.plans.len() - 1);

    // The canvas takes whatever the column has, MINUS a constant. See
    // `STRIP_HEIGHT_PTS` for why the constant rather than the strip's
    // measured height, and `CANVAS_MAX_HEIGHT_PTS` for why the result is
    // clamped at both ends.
    let canvas_height =
        (column_height - STRIP_HEIGHT_PTS).clamp(CANVAS_MIN_HEIGHT_PTS, CANVAS_MAX_HEIGHT_PTS);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(COLUMN_WIDTH_PTS, canvas_height),
        // `click_and_drag` rather than `drag`: a click that does not move must
        // still mark the canvas hovered-and-interacted, which is what gates
        // the Ctrl+wheel read below.
        Sense::click_and_drag(),
    );

    // Fit the SHEET into the preview box, preserving aspect. Recomputed every
    // frame from the CURRENT rect on purpose: a taller window should show a
    // bigger sheet, and that coupling is the desired half of the feedback
    // hazard `STRIP_HEIGHT_PTS` describes, not the hazardous half — `rect` is
    // derived from a constant, so nothing the strip draws can feed back into
    // it.
    let sheet = job.device.physical_pt;
    let fit = (rect.width() / sheet.0 as f32).min(rect.height() / sheet.1 as f32) * FIT_MARGIN;

    // ---- zoom and pan, before anything is drawn from them ------------------
    //
    // Ctrl+wheel, gated on hover so it cannot steal the gesture from a sibling
    // control. Zoom is anchored on the POINTER: without the anchor term,
    // zooming in on the bottom-left corner of a sheet walks it off screen and
    // the operator has to hunt it back with a drag.
    if response.hovered() {
        let step = ui.input(|i| i.zoom_delta());
        if (step - 1.0).abs() > f32::EPSILON {
            let at = response.hover_pos().unwrap_or_else(|| rect.center());
            zoom_by(dialog, step, at, rect.center());
        }
    }
    // ★ `dragged_by(Primary)`, never bare `dragged()`. Per
    // `D:\dev\rag\egui\egui_response_drag_predicates_are_button_agnostic.md`
    // the unqualified predicate fires for middle and right drags too, which
    // would silently claim the right-drag this preview may later want for a
    // context menu.
    if response.dragged_by(egui::PointerButton::Primary) {
        dialog.preview_pan += response.drag_delta();
    }

    let scale = fit * dialog.preview_zoom;
    // Rendered before the strip is laid out, because the strip's Actual-size
    // button needs the scale this frame settled on.
    let texture = paint(ui, inputs, dialog, shown, rect, scale);
    strip(ui, inputs, dialog, shown, rect, scale);

    // The canvas rectangle and the two geometry rectangles, in one line.
    //
    // ★ `sheet=` and `printable=` are here because they are the only honest
    // evidence that the Orientation radio reaches the geometry. The radio
    // changes no pixel of the page bitmap (see `PreviewKey`) and turns a
    // rectangle whose aspect a screenshot can suggest but not measure. A trace
    // of the two rectangles is what lets a harness assert the turn rather than
    // photograph it. `tex=0` with a plan present is the regression that would
    // put the flat-fill fallback back without anything else looking wrong.
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-preview canvas=[{:.0},{:.0} {:.0}x{:.0}] fit={fit:.4} scale={scale:.4} \
             device_dpi={:?} sheet={:.0}x{:.0} printable={:.0}x{:.0} margin={:.0},{:.0} \
             sheet_of={}/{} zoom={:.3} pan=({:.1},{:.1}) tex={}",
            rect.min.x,
            rect.min.y,
            rect.width(),
            rect.height(),
            job.device.dpi,
            job.device.physical_pt.0,
            job.device.physical_pt.1,
            job.device.printable_pt.0,
            job.device.printable_pt.1,
            job.device.offset_pt.0,
            job.device.offset_pt.1,
            shown + 1,
            job.plans.len(),
            dialog.preview_zoom,
            dialog.preview_pan.x,
            dialog.preview_pan.y,
            u8::from(texture.is_some()),
        )
    });

    // The count, always, for a multi-page job whose clip is on a sheet the
    // preview is not showing.
    let clipped = job.clipped();
    if clipped > 0 {
        ui.label(
            egui::RichText::new(t::clip_summary(clipped, job.plans.len()))
                .color(ui.visuals().warn_fg_color),
        );
    }
}

/// Paint the sheet, the printable area, the placed page and the clip hatch.
///
/// Returns the texture that was drawn, or `None` when the page would not
/// render — which is the degraded-but-honest state described on
/// [`texture_for`].
fn paint(
    ui: &Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    shown: usize,
    rect: Rect,
    scale: f32,
) -> Option<TextureId> {
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let job = inputs.job;
    let sheet = job.device.physical_pt;

    let sheet_px = egui::vec2(sheet.0 as f32 * scale, sheet.1 as f32 * scale);
    let origin = rect.center() - sheet_px / 2.0 + dialog.preview_pan;
    let sheet_rect = Rect::from_min_size(origin, sheet_px);
    painter.rect_filled(sheet_rect, 2.0, visuals.extreme_bg_color);
    painter.rect_stroke(
        sheet_rect,
        2.0,
        Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        StrokeKind::Middle,
    );

    // The printable area, inset by the driver's own unprintable margins. This
    // is the rectangle that actually constrains the job — and the reason the
    // preview is worth having at all.
    let printable = Rect::from_min_size(
        origin
            + egui::vec2(
                job.device.offset_pt.0 as f32 * scale,
                job.device.offset_pt.1 as f32 * scale,
            ),
        egui::vec2(
            job.device.printable_pt.0 as f32 * scale,
            job.device.printable_pt.1 as f32 * scale,
        ),
    );
    painter.rect_stroke(
        printable,
        0.0,
        Stroke::new(1.0, visuals.weak_text_color()),
        StrokeKind::Middle,
    );

    // ★ `page_sizes` is indexed by `plan.index`, NOT by `shown`.
    //
    // `shown` walks the JOB (which may be a custom range, odd/even filtered,
    // or reversed) and `page_sizes` is in document order, so the two coincide
    // only for a whole-document forward job. Indexing by `shown` draws the
    // placed rectangle at the size of a page the job may not even contain,
    // which on a document mixing sheet sizes is a preview that reports a clip
    // that will not happen or misses one that will.
    let plan = *job.plans.get(shown)?;
    let size = *inputs.page_sizes.get(plan.index)?;

    let placed = Rect::from_min_size(
        printable.min
            + egui::vec2(
                plan.placement.offset_x_pt as f32 * scale,
                plan.placement.offset_y_pt as f32 * scale,
            ),
        egui::vec2(
            (size.0 * plan.placement.scale) as f32 * scale,
            (size.1 * plan.placement.scale) as f32 * scale,
        ),
    );

    // The rendered page, if one is available. The fallback is a flat fill — a
    // preview showing the right rectangle and no content is degraded but
    // honest; one showing a stale page would be wrong.
    let texture = texture_for(ui.ctx(), inputs, dialog, plan.index);
    if let Some(texture) = texture {
        painter.image(
            texture,
            placed,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            // NOT A THEME COLOUR: a pass-through tint for an already-rendered
            // page bitmap. `painter.image` MULTIPLIES the texture by this
            // value, so white means "draw the pixels as rendered". These are
            // document content, not chrome — restyling the application must
            // not restyle the operator's page, and any palette role here would
            // do exactly that.
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(placed, 0.0, visuals.panel_fill);
    }
    painter.rect_stroke(
        placed,
        0.0,
        Stroke::new(1.0, visuals.weak_text_color()),
        StrokeKind::Middle,
    );

    // What will be lost, hatched. A hatch means "this will happen and has not
    // happened yet", which is exactly a pre-print clip. A solid fill would
    // read as something already done.
    if plan.placement.clipped {
        hatch_lost_content(&painter, placed, printable, visuals.warn_fg_color);
    }
    texture
}

/// Hatch the part of `placed` that falls outside `printable`.
///
/// Only the right and bottom overhangs are hatched, and that is not an
/// omission: a placement offsets the page *into* the printable area from its
/// top-left corner, so content is lost off the far edges. Hatching all four
/// would draw a warning over paper that will print.
fn hatch_lost_content(painter: &egui::Painter, placed: Rect, printable: Rect, colour: Color32) {
    let lost = placed
        .intersect(Rect::everything_right_of(printable.max.x))
        .union(placed.intersect(Rect::everything_below(printable.max.y)));
    if !lost.is_positive() {
        return;
    }
    let step = 6.0;
    let mut x = lost.min.x;
    while x < lost.max.x + lost.height() {
        painter.line_segment(
            [
                egui::pos2(x.min(lost.max.x), lost.min.y),
                egui::pos2((x - lost.height()).max(lost.min.x), lost.max.y),
            ],
            Stroke::new(1.0, colour),
        );
        x += step;
    }
}

/// The fixed strip: the sheet stepper on the left, the zoom controls on the
/// right, and the magnification readout beneath.
///
/// They share a row because they are both "what am I looking at" controls and
/// because two rows plus the clip caption would not fit the fixed strip — and
/// the strip's height is fixed for the feedback-loop reason on
/// [`STRIP_HEIGHT_PTS`], so the layout has to live inside it rather than the
/// other way round.
fn strip(
    ui: &mut Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    shown: usize,
    rect: Rect,
    scale: f32,
) {
    let sheets = inputs.job.plans.len();
    ui.horizontal(|ui| {
        if ui.button(t::preview_previous()).clicked() {
            dialog.preview_page = shown.saturating_sub(1);
            // A different sheet is a different picture, so the zoom and pan
            // the operator chose for the last one no longer mean anything —
            // on a differently-sized sheet they would put the new page off
            // screen. Reset rather than carry.
            reset_view(dialog);
        }
        ui.label(t::preview_position(shown + 1, sheets));
        if ui.button(t::preview_next()).clicked() && shown + 1 < sheets {
            dialog.preview_page = shown + 1;
            reset_view(dialog);
        }
        ui.separator();
        if ui
            .button(t::preview_zoom_fit())
            .on_hover_text(t::preview_zoom_fit_tooltip())
            .clicked()
        {
            reset_view(dialog);
        }
        // Buttons as well as the wheel gesture, and kept even though
        // Ctrl+wheel exists: the commonest reason to zoom a print preview is
        // checking that fine print clears the margin, which is a deliberate
        // look at a known amount of magnification, not a scrub. A gesture is
        // faster and a button is findable.
        if ui.button(t::preview_zoom_out()).clicked() {
            zoom_by(dialog, 1.0 / ZOOM_STEP, rect.center(), rect.center());
        }
        if ui.button(t::preview_zoom_in()).clicked() {
            zoom_by(dialog, ZOOM_STEP, rect.center(), rect.center());
        }
        // Actual size means one PDF point drawn as one egui point, so the
        // multiplier that gets there is `1 / scale` — the number the
        // percentage readout will then show as 100%.
        if ui
            .button(t::preview_zoom_actual())
            .on_hover_text(t::preview_zoom_actual_tooltip())
            .clicked()
            && scale > 0.0
        {
            zoom_by(dialog, 1.0 / scale, rect.center(), rect.center());
        }
    });
    ui.horizontal(|ui| {
        // The scale as a percentage of ACTUAL size, not of the fit — see
        // `text::print::preview_zoom_percent` for why a number that changes
        // when the window is dragged would be useless. Clamped and rounded
        // before the cast: `ZOOM_MIN` and `ZOOM_MAX` bound the multiplier and
        // `fit` is a ratio of two positive lengths, so the product cannot be
        // negative or large enough to saturate — but the clamp is written
        // rather than argued, because a degenerate `physical_pt` of zero would
        // make `fit` infinite and a cast of infinity is a silent zero.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped and rounded on the line above" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let percent = (scale * 100.0).round().clamp(0.0, 100_000.0) as u32;
        ui.label(
            egui::RichText::new(t::preview_zoom_percent(percent))
                .small()
                .weak(),
        );
        ui.label(egui::RichText::new(t::preview_pan_hint()).small().weak());
    });
}

/// Put the preview back to fit, centred.
///
/// Two fields, one place. The Fit button and both stepper buttons need
/// exactly this, and three copies of `zoom = 1.0; pan = ZERO` is how a fourth
/// caller ends up resetting only one of them.
pub(super) fn reset_view(dialog: &mut PrintDialog) {
    dialog.preview_zoom = 1.0;
    dialog.preview_pan = Vec2::ZERO;
}

/// Multiply the preview zoom by `step`, keeping the point `at` still.
///
/// A two-field assignment over [`zoomed_view`], which holds the arithmetic and
/// is where it is tested — `PrintDialog` carries a spooler's worth of device
/// state that a test of the anchor term has no business constructing.
fn zoom_by(dialog: &mut PrintDialog, step: f32, at: Pos2, centre: Pos2) {
    let (zoom, pan) = zoomed_view(dialog.preview_zoom, dialog.preview_pan, step, at, centre);
    dialog.preview_zoom = zoom;
    dialog.preview_pan = pan;
}

/// The texture for `page`, rendering and caching it if needed.
///
/// Returns the id rather than the handle so the caller holds no borrow of
/// `dialog` past the call — the alternative is a `&TextureHandle` living
/// across the rest of a function that also wants `dialog` mutably, which
/// compiles only by accident of statement ordering.
///
/// # Never re-rendered on a frame where nothing changed
///
/// Same discipline as the printer enumeration in
/// [`crate::dialogs::print::PrintDialog::open`] and as `RenderKey`'s
/// staleness fields: a preview that re-rasterised sixty times a second would
/// make an open dialog cost more than the print. [`PreviewKey`] carries every
/// input that can change the pixels; see its docs for why orientation is not
/// one of them, and for the two fields it must gain when their controls land.
///
/// A failed render clears the cache and returns `None`, which drops the
/// preview back to the flat fill. **It is not reported as an error**: the same
/// failure will be reported honestly, once, by the spool attempt, and a
/// preview that turns into an error banner while the operator is still
/// choosing a page range is noise in front of a decision they have not made
/// yet.
fn texture_for(
    ctx: &egui::Context,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    page: usize,
) -> Option<TextureId> {
    let key = PreviewKey {
        page,
        scope: dialog.scope,
        settings: inputs.doc.settings.clone(),
    };
    if let Some((cached, texture)) = &dialog.preview_texture
        && *cached == key
    {
        return Some(texture.id());
    }
    let page_obj = inputs.doc.pages.get(page)?;
    let size = *inputs.page_sizes.get(page)?;
    // The SAME builder the spooler calls. See `super::render_options` for the
    // choices it encodes and why a second copy of them here would defeat the
    // preview's entire purpose.
    let options = super::render_options(dialog.scope, &inputs.doc.settings);
    // `session.view()`, NOT `session.document()` — the view composes the
    // overlay and the staging buffer, so unsaved edits are what the operator
    // is about to print.
    let view = inputs.doc.session.view();
    let rendered =
        pdfcer_render::render_page_with_view(&view, page_obj, raster_scale(size), &options).ok();
    let Some(rendered) = rendered else {
        dialog.preview_texture = None;
        return None;
    };
    let texture = upload(ctx, &rendered.pixmap);
    let id = texture.id();
    dialog.preview_texture = Some((key, texture));
    Some(id)
}

/// Upload a rendered pixmap as the preview's own texture.
///
/// # ★ This is a SECOND premultiplied-alpha call site, and that is a defect
/// # this module cannot fix from here
///
/// [`crate::render::raster`]'s header is explicit that the convention is
/// enforced *"by there being **one** function, not by review: both
/// `ColorImage` constructors accept the bytes without complaint, and the
/// wrong one silently darkens every antialiased glyph edge."* This is a
/// second one, and it exists only because that module's public helper
/// (`texture_from_pixels`) takes a `RenderedPixels` — a worker result carrying
/// a `RenderKey` — and uploads under a *single fixed texture name* shared with
/// the canvas.
///
/// The old shell did not have this problem: it had
/// `raster::texture_from_pixmap(ctx, name, &pixmap)`, and `SALVAGE.md` records
/// that helper as *"left behind — it exists for the print preview (S5)"*.
/// Restoring it and deleting this function is the correct fix, and it is a
/// change to `render/raster.rs` rather than to this file.
///
/// Until then the convention is held by this doc comment and by the assertion
/// in [`the_preview_upload_reads_pixels_as_premultiplied`], which is the same
/// fixture `render::raster`'s own test uses — so the two cannot drift without
/// one of them failing.
fn upload(ctx: &egui::Context, pixmap: &pdfcer_render::tiny_skia::Pixmap) -> TextureHandle {
    let image = premultiplied_image(pixmap.width(), pixmap.height(), pixmap.data());
    // LINEAR, matching the canvas: the preview is drawn at whatever the fit
    // and the operator's zoom produce, which is almost never 1:1 with the
    // bitmap, and nearest-neighbour sampling of a downsampled page is a mess
    // of aliased hairlines exactly where the operator is looking for hairlines.
    ctx.load_texture(PREVIEW_TEXTURE_ID, image, egui::TextureOptions::LINEAR)
}

/// Read `tiny-skia`'s bytes as what they are: **premultiplied** RGBA8.
///
/// Split out from [`upload`] so the one thing here that can be silently wrong
/// is the one thing that is unit-testable without an `egui::Context`.
fn premultiplied_image(width: u32, height: u32, data: &[u8]) -> egui::ColorImage {
    egui::ColorImage::from_rgba_premultiplied([width as usize, height as usize], data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The screen position a sheet point lands at, for a given view.
    ///
    /// Mirrors [`paint`]'s own `origin` computation so the anchor tests below
    /// assert the property that matters — "this point did not move" — rather
    /// than re-stating the formula they are meant to be checking.
    fn on_screen(
        sheet_pt: Vec2,
        fit: f32,
        zoom: f32,
        pan: Vec2,
        centre: Pos2,
        point_in_sheet: Vec2,
    ) -> Pos2 {
        let s = fit * zoom;
        let origin = centre - (sheet_pt * s) / 2.0 + pan;
        origin + point_in_sheet * s
    }

    /// ★ The point under the pointer does not move when you zoom on it.
    ///
    /// This is the whole reason the anchor term exists, and it is the one
    /// property a reader can check without re-deriving the algebra. Asserted
    /// on an OFF-CENTRE point, because every wrong version of this formula —
    /// including simply omitting the term — is correct at the centre.
    #[test]
    fn ctrl_wheel_zoom_holds_the_point_under_the_pointer_still() {
        // US Letter, fitted into a 340 x 400 canvas at the same margin factor
        // the preview uses.
        let sheet = egui::vec2(612.0, 792.0);
        let fit = (340.0_f32 / sheet.x).min(400.0 / sheet.y) * FIT_MARGIN;
        let centre = egui::pos2(170.0, 200.0);
        // A point near the sheet's bottom-right, which is where an operator
        // checking a margin actually looks.
        let target_in_sheet = egui::vec2(560.0, 730.0);

        let (zoom0, pan0) = (1.0_f32, Vec2::ZERO);
        let at = on_screen(sheet, fit, zoom0, pan0, centre, target_in_sheet);

        let (zoom1, pan1) = zoomed_view(zoom0, pan0, 2.5, at, centre);
        let after = on_screen(sheet, fit, zoom1, pan1, centre, target_in_sheet);

        assert!(
            (after - at).length() < 0.001,
            "the anchored point moved from {at:?} to {after:?} — without the \
             (at - centre)(1 - k) term, zooming in on a corner walks the sheet \
             off the canvas"
        );
        assert!(
            (zoom1 - 2.5).abs() < 1e-6,
            "the zoom itself must still be applied; got {zoom1}"
        );
    }

    /// A button press anchors on the canvas centre, which is the degenerate
    /// case `pan1 = k * pan0` — the sheet grows about the middle rather than
    /// about wherever the pointer happened to be resting.
    #[test]
    fn a_button_zoom_scales_the_existing_pan_about_the_centre() {
        let centre = egui::pos2(170.0, 200.0);
        let (zoom, pan) = zoomed_view(2.0, egui::vec2(30.0, -12.0), 1.25, centre, centre);
        assert!((zoom - 2.5).abs() < 1e-6);
        assert!((pan.x - 37.5).abs() < 1e-4, "pan.x was {}", pan.x);
        assert!((pan.y + 15.0).abs() < 1e-4, "pan.y was {}", pan.y);
    }

    /// ★ A zoom the clamp refuses must not pan either.
    ///
    /// The bug this pins is subtle and would look like a hardware fault: at
    /// maximum zoom the wheel stops magnifying but keeps sliding the sheet
    /// sideways, so the preview appears to drift on its own. It comes from
    /// using the REQUESTED step for the anchor term instead of the effective,
    /// post-clamp ratio.
    #[test]
    fn a_refused_zoom_leaves_the_pan_exactly_where_it_was() {
        let pan = egui::vec2(21.0, -8.0);
        let (zoom, after) = zoomed_view(
            ZOOM_MAX,
            pan,
            4.0,
            egui::pos2(300.0, 40.0),
            egui::pos2(170.0, 200.0),
        );
        assert!((zoom - ZOOM_MAX).abs() < 1e-6, "clamped at the ceiling");
        assert!(
            (after - pan).length() < 1e-4,
            "a refused zoom moved the sheet from {pan:?} to {after:?}"
        );

        // The same at the floor.
        let (zoom, after) = zoomed_view(
            ZOOM_MIN,
            pan,
            0.1,
            egui::pos2(300.0, 40.0),
            egui::pos2(170.0, 200.0),
        );
        assert!((zoom - ZOOM_MIN).abs() < 1e-6);
        assert!((after - pan).length() < 1e-4);
    }

    /// A hostile or degenerate step is a no-op rather than a `NaN` that
    /// poisons every later frame's pan arithmetic.
    #[test]
    fn a_non_finite_or_negative_step_changes_nothing() {
        let pan = egui::vec2(3.0, 4.0);
        let centre = egui::pos2(0.0, 0.0);
        for step in [f32::NAN, f32::INFINITY, 0.0, -1.5] {
            let (zoom, after) = zoomed_view(2.0, pan, step, egui::pos2(10.0, 10.0), centre);
            assert!(
                (zoom - 2.0).abs() < 1e-6 && (after - pan).length() < 1e-6,
                "step {step} must be ignored, got zoom {zoom} pan {after:?}"
            );
        }
    }

    /// An ordinary page renders at the target DPI — the pixel ceiling does not
    /// bind, and must not quietly downgrade every normal preview.
    #[test]
    fn a_letter_page_previews_at_the_target_resolution() {
        let scale = raster_scale((612.0, 792.0));
        assert!(
            (scale - TARGET_DPI / 72.0).abs() < 1e-6,
            "a Letter page must not be capped; got {scale}"
        );
    }

    /// ★ A large-format sheet is capped by PIXELS, not by DPI.
    ///
    /// The bound that matters. An ANSI E sheet at the target DPI would be
    /// 5100 x 6600 px and about 134 MB of RGBA for a picture drawn 300 pt
    /// wide — and CAD sheets are exactly the population this project's
    /// operator prints, so this is the common case, not the exotic one.
    #[test]
    fn a_large_format_sheet_is_capped_by_pixels() {
        let sheet = (2448.0, 3168.0); // ANSI E, 34 x 44 inches.
        let scale = raster_scale(sheet);
        let longest = sheet.0.max(sheet.1) as f32 * scale;
        assert!(
            longest <= MAX_SIDE_PX + 0.5,
            "the long side rendered to {longest} px, over the {MAX_SIDE_PX} ceiling"
        );
        assert!(
            scale < TARGET_DPI / 72.0,
            "the cap must actually bind on this size; got {scale}"
        );
    }

    /// ★ Where the pixel ceiling starts to bind, asserted from both sides.
    ///
    /// The regression the ceiling's own doc comment records is a value chosen
    /// too low: 1600 px silently downgraded Letter, Legal and A4 — the common
    /// case — in order to bound the rare one. Asserting only that those three
    /// are uncapped would let the constant drift *upward* unnoticed instead,
    /// so the boundary is pinned from both directions.
    ///
    /// **A3 is on the capped side, and that is correct rather than a
    /// near-miss.** Its long edge is 1191 pt, which at the target DPI is
    /// 2481 px — past the 2200 ceiling. A3 is a drafting sheet, not an office
    /// page, so it belongs with the large-format population this bound exists
    /// for; US Legal at 2100 px is the largest size that clears it. If either
    /// constant moves, this test says which side of the line each size landed
    /// on rather than merely that something changed.
    #[test]
    fn the_pixel_ceiling_binds_above_the_office_sizes() {
        for (name, size) in [
            ("A4", (595.0, 842.0)),
            ("Letter", (612.0, 792.0)),
            ("Legal", (612.0, 1008.0)),
        ] {
            let scale = raster_scale(size);
            assert!(
                (scale - TARGET_DPI / 72.0).abs() < 1e-6,
                "{name} was capped to {scale}; the ceiling is meant to leave every \
                 office page size at the full target DPI"
            );
        }
        for (name, size) in [("A3", (842.0, 1191.0)), ("ANSI E", (2448.0, 3168.0))] {
            let scale = raster_scale(size);
            assert!(
                scale < TARGET_DPI / 72.0,
                "{name} was NOT capped ({scale}); the ceiling is meant to bind on \
                 drafting and large-format sheets, which is where the memory goes"
            );
        }
    }

    /// A degenerate `/MediaBox` must not divide by zero. Real files carry
    /// them — the renderer has its own guards, and this only has to hand it a
    /// finite number.
    #[test]
    fn a_zero_sized_page_yields_a_finite_scale() {
        let scale = raster_scale((0.0, 0.0));
        assert!(scale.is_finite() && scale > 0.0, "got {scale}");
    }

    /// ★ The preview reads pixels as premultiplied, exactly as the canvas does.
    ///
    /// The same fixture `render::raster`'s own test uses — a half-transparent
    /// red pixel stored the way `tiny-skia` stores it (`R·A, G·A, B·A, A`).
    /// Read as *unmultiplied*, epaint would take the red channel at face value
    /// and re-multiply it, yielding `r = 64`; read as premultiplied it
    /// round-trips.
    ///
    /// This test exists because [`upload`] is a **second** call site for a
    /// convention that module says must have exactly one. Until that is fixed
    /// there, this is what stops the two drifting silently — and the failure
    /// mode being defended against is not a crash but every antialiased glyph
    /// edge in the preview quietly darkening.
    #[test]
    fn the_preview_upload_reads_pixels_as_premultiplied() {
        let image = premultiplied_image(1, 1, &[128, 0, 0, 128]);
        assert_eq!(image.size, [1, 1]);
        let px = image.pixels[0];
        assert_eq!((px.r(), px.g(), px.b(), px.a()), (128, 0, 0, 128));
    }
}
