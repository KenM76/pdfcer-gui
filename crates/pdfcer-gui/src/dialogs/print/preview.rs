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
//! 5. **A hatch over what will be lost** — over the part of the overhang that
//!    actually carries ink, and over nothing else. Hatched rather than filled:
//!    a hatch means *"this will happen and has not happened yet"*, which is
//!    exactly a pre-print clip. A solid fill reads as something already done.
//!
//! ## ★★★ (5) became ink-aware on 2026-09-03 — operator request O113
//!
//! It used to hatch the **whole** overhanging region the moment
//! `Placement::clipped` was true, and that flag is a *geometric* verdict: the
//! page box exceeds the printable rectangle. On a CAD sheet printed 1:1 the
//! part that exceeds it is empty paper, so the hatch shouted about losing
//! something on every one of the operator's drawings while nothing was being
//! lost — *"the area that isn't printed is just empty border."*
//!
//! **A disclosure that is technically true and practically false is the worst
//! kind.** An operator who sees the same red band on every drawing learns to
//! ignore it, and then does not see it on the one sheet where the border really
//! does have a title block in it.
//!
//! The hatch now asks [`super::ink::InkMask`] — a downsample of the very raster
//! drawn at (4) — what is in the band, and covers the ink extent within it. No
//! ink ⇒ no hatch. [`hatch_lost_content`] holds the geometry, `super::ink`
//! holds the pixel test and the measurement behind its threshold, and
//! [`Overhang`] is how the caption is kept from contradicting the picture.
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
use crate::dialogs::print::ink;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

/// What the shown sheet's overhang turned out to contain — the fact the hatch
/// is drawn from, lifted out so the CAPTION can be drawn from the same one.
///
/// # ★★★ Why this is a return value and not something the caption re-derives
///
/// Operator request O113 makes the hatch ink-aware, and a caption that kept
/// announcing a clip over a preview showing no hatch would be the identical
/// contradiction one level up: *"this sheet will lose content"* printed above a
/// picture that visibly loses none. An operator resolving that disagreement
/// resolves it by trusting neither.
///
/// The only way the two cannot disagree is for them to be the **same
/// computation**, so [`paint`] reports what it found and [`column`] says it.
/// A caption that asked the mask a second time would be a second call site for
/// a question with a threshold in it, and the two would drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Overhang {
    /// The placement reported no clip. The page fits the printable area.
    Fits,
    /// The placement reported a clip **and the overhanging band carries ink**,
    /// so something really will be cropped. Hatched.
    Losing,
    /// ★ The placement reported a clip and the band is **blank paper** — the
    /// 1:1 CAD drawing O113 is about. Nothing hatched, and the caption says so
    /// rather than leaving the operator to wonder why the warning has no
    /// picture.
    BlankBand,
    /// The placement reported a clip and there is **no raster to ask** — the
    /// degraded state [`texture_for`] documents, where the page did not render
    /// and the preview shows a flat fill. The whole band is hatched, because a
    /// failed render must not be able to switch a warning off.
    Unknown,
}

// ★★★ `COLUMN_WIDTH_PTS` WAS HERE, AND ITS REASONING WAS THE DEFECT — 2026-09-03.
//
// It was a fixed 340 pt, and its doc comment argued the case at length:
//
// > **A CONSTANT, and that is the whole point.** The dialog body lives inside a
// > `ScrollArea::both`, and a horizontally scrollable area has no bounded width
// > to report [...] Two fixed columns break that circle — the content width is
// > a constant, so the horizontal scrollbar has something stable to measure and
// > the operator gets a scrollbar instead of a column that grows to meet it.
//
// The circle it describes is real. The conclusion is wrong, and it produced two
// separate operator-visible defects:
//
//  1. **Two scrollbars that could not be dismissed.** The fixed content width
//     was `max`'d with `available_width` at the call site precisely so a wide
//     window would not leave a dead strip — which put `available_width` back
//     into the circle the constant existed to break, and deadlocked the two
//     bars against each other. `PrintDialog::body` documents the mechanism.
//  2. **A preview that could not be made bigger.** *"the preview should be
//     adjustable size"* — widening the dialog widened the empty space and left
//     the sheet the same postage stamp, because the column could not grow.
//
// The right answer to the circle is not a constant, it is to lay the columns
// out to the width actually available and tell the `ScrollArea` nothing. It
// then shows a bar when, and only when, the content does not fit — which is
// the only thing a scrollbar should ever mean.
//
// The width now comes from `PrintDialog::preview_width`, which the splitter
// drags; the floor, the default and the options column's floor are
// `PREVIEW_MIN_WIDTH_PTS`, `PREVIEW_DEFAULT_WIDTH_PTS` and
// `OPTIONS_COLUMN_MIN_WIDTH_PTS` in the parent module. `column` takes it as a
// parameter.

/// Height of the fixed strip under the preview canvas, in egui points.
///
/// # ★★ It reserves THREE rows, not two, since 2026-09-03
///
/// The strip is two `horizontal` rows — the seven controls, then the zoom
/// caption. The first of those is now `horizontal_wrapped` (see [`strip`] for
/// why), so on a narrow column it becomes two rows and the strip becomes three.
///
/// Reserving for the wrapped case is what keeps the constant honest. If it
/// reserved two rows the strip would overflow the column vertically the moment
/// it wrapped, the column's content would exceed the body, and a **vertical**
/// scrollbar would appear — which is the defect this whole change removes,
/// re-entering by the other axis. The cost of reserving the third row is about
/// 28 pt of canvas height on a wide column where it is not needed, against a
/// scrollbar that cannot be dismissed on a narrow one.
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
const STRIP_HEIGHT_PTS: f32 = 96.0;

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

/// The Pop-out button's region, for the driven harness.
///
/// ★ Declared with the **visibility-gated** publisher, unlike the preview
/// column's own region next door, and the two are opposites on purpose: this
/// one exists to be clicked, so a rect the operator cannot reach is worse than
/// no rect at all; that one exists to be seen to disappear, so a rect that is
/// merely scrolled out of view must still count as present.
pub(super) const REGION_POP_OUT: &str = "print.preview.popout";

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

impl PreviewKey {
    /// Build the key for one page.
    ///
    /// # ★★★ Called from exactly one place, and that place is the VERDICT
    /// # cache's context
    ///
    /// [`super::verdicts::Context::preview_key`] is the sole caller — operator
    /// request O113, 2026-09-04. The inversion is the point rather than
    /// plumbing.
    ///
    /// That module remembers, per sheet, whether the overhang the preview
    /// hatched turned out to be blank, so the commit button can subtract the
    /// sheets known to lose nothing. A remembered verdict is a claim about
    /// **these pixels**, and this type's own doc comment states what a cache
    /// key is: *"if these are equal, re-rendering would produce the same
    /// image."* A verdict cached under a weaker key would be a verdict about a
    /// page that has changed — and it would be confidently wrong, because its
    /// whole purpose is to take a warning away.
    ///
    /// Deriving this key **from** the verdict cache's context makes "the
    /// verdict is keyed on at least what the pixels are keyed on" a structural
    /// fact rather than a promise held by two doc comments. Constructing it
    /// here as well, from the same three values, would be the second reading
    /// of one rule that this file's `settings` field already argues against.
    pub(super) fn new(
        page: usize,
        scope: pdfcer_render::AnnotationScope,
        settings: &pdfcer_core::settings::Settings,
    ) -> Self {
        Self {
            page,
            scope,
            settings: settings.clone(),
        }
    }
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
    /// The frame's cache context: the rendering inputs and the printable
    /// rectangle, built once in [`PrintDialog::show`].
    ///
    /// ★ Both caches in this dialog hang off it. The page texture's
    /// [`PreviewKey`] is built from it (see [`PreviewKey::new`]), and every
    /// remembered overhang verdict is void the moment it changes — so the
    /// verdict cache cannot be keyed on less than the pixels are.
    pub(super) context: &'a super::verdicts::Context,
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

/// **Where this preview is being drawn** — operator request O112 ask 2.
///
/// # ★★ One function, two homes, and the difference is one button
///
/// The preview is the same picture and the same arithmetic in the print
/// dialog's column and in its own OS window. What differs is a single control:
/// the column offers *"pop this out"*, and the popped window does not, because
/// the way back is its own close button. Passing that as a parameter rather
/// than writing a second draw function is the whole reason this feature is
/// cheap — and a second draw function is how the two copies of a preview come
/// to disagree about a margin.
///
/// ★ There is deliberately **no** "put it back" button in the popped window.
/// The operator's own words were *"closing the window pops it back into place
/// on the print window"*, and that is also the convention: a popped-out pane
/// docks by being closed, everywhere this pattern appears. A second control
/// that did the same thing as the title bar's X would be an invented
/// interaction beside a conventional one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Placement {
    /// Inside the print dialog, in the column beside the options.
    InDialog,
    /// In its own resizable OS window. See [`super::popout`].
    PoppedOut,
}

/// Draw the preview column: the canvas, then the fixed strip beneath it.
pub(super) fn column(
    ui: &mut Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    column_height: f32,
    column_width: f32,
    placement: Placement,
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
    // ★ The width is passed IN, not read from a constant, because the operator
    // drags it — see `PrintDialog::splitter`. The `fit` computed below is
    // recomputed from this rect every frame, so a wider column simply shows a
    // bigger sheet with no further change; that coupling was already the
    // desired half of the feedback relationship this function documents.
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(column_width, canvas_height),
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
    let (texture, overhang) = paint(ui, inputs, dialog, shown, rect, scale);
    strip(ui, inputs, dialog, shown, rect, scale, placement);

    // ★ Read AFTER `paint`, which is what makes the sheet on screen count as
    // examined on the frame it is drawn rather than on the next one — see the
    // clip summary below for what this number is and how its wording follows
    // from it. Also read BEFORE the trace, so the trace reports the claim this
    // frame's picture was drawn beside.
    let claim = dialog
        .verdicts
        .claim(inputs.context, job, inputs.page_sizes);

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
             sheet_of={}/{} zoom={:.3} pan=({:.1},{:.1}) tex={} overhang={} claim={}:{}",
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
            // ★ `overhang=` is the ONLY headless evidence that operator request
            // O113 works, and it is here because the thing that changed is
            // something a capture cannot distinguish: a preview with no hatch
            // over a blank band and a preview with no hatch because the ink
            // test silently found nothing anywhere look IDENTICAL in a
            // screenshot. The word says which. `blank-band` on a 1:1 CAD sheet
            // is the request satisfied; `losing` on the same sheet is the
            // defect back; `unknown` means the page did not render and the
            // whole band was hatched as the honest fallback.
            match overhang {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                Overhang::Fits => "fits",
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                Overhang::Losing => "losing",
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                Overhang::BlankBand => "blank-band",
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                Overhang::Unknown => "unknown",
            },
            // ★ The job-wide claim beside the sheet-level verdict, because the
            // pair is what a driven check has to read: `overhang=blank-band`
            // says this sheet's band is empty paper, and `claim=` says what
            // that did to the number on the button. `overhang=blank-band` next
            // to `claim=geometric:1` would be the verdict landing and the
            // count not moving — a cache that silently never matched, which is
            // indistinguishable from a working cache in any capture.
            claim.trace_word(),
            claim.count(),
        )
    });

    // The count, always, for a multi-page job whose clip is on a sheet the
    // preview is not showing.
    //
    // ★★★ THE COUNT GOT BETTER — the sentence did not get vaguer. Operator
    // request O113, 2026-09-04.
    //
    // This comment used to say the line was "UNCHANGED by O113, deliberately",
    // on the grounds that `Job::clipped()` is a plan-time geometric fact which
    // is still exactly true and that softening its wording to match a picture
    // that knows more would be trading a true statement for a comfortable one.
    // **That reasoning stands, and it is the reason the fix is here rather
    // than in the wording.** What changed is not the sentence; it is that the
    // number is now corrected by verdicts the preview has *already* produced,
    // at no rendering cost, and the wording follows what the corrected number
    // can support:
    //
    //   * nothing subtracted  -> the geometric sentence, unchanged, word for
    //     word — including the case where the operator never stepped the
    //     preview at all;
    //   * every clipped sheet examined -> the same sentence, now verified;
    //   * some examined, some not -> a ceiling, which says so.
    //
    // ★ It must be the SAME claim the commit button draws, or this line and
    // the button would show two different numbers for one job — the exact
    // contradiction O113 reported, moved rather than fixed. Both call
    // `Verdicts::claim` on the same context, and `ClipClaim` owns which
    // sentence each state gets, so neither surface chooses wording of its own.
    if let Some(summary) = claim.summary(job.plans.len()) {
        ui.label(egui::RichText::new(summary).color(ui.visuals().warn_fg_color));
    }
    // ★★★ …and the sheet-level correction under it, which is what stops the
    // caption and the hatch contradicting each other — operator request O113.
    //
    // The job-wide count above says "this sheet will lose content" while the
    // picture beside it shows a page with nothing hatched. Left alone, an
    // operator resolving that disagreement resolves it by trusting neither
    // half. This line resolves it for them, and it can only be written because
    // `paint` has already answered the question for the sheet on screen — the
    // same computation the hatch was drawn from, so the two cannot drift.
    //
    // Drawn in the ordinary text colour, not the warning colour: it is the
    // sentence that takes a warning AWAY for this sheet, and colouring it as a
    // warning would put back the alarm it exists to remove.
    if overhang == Overhang::BlankBand {
        ui.label(egui::RichText::new(t::overhang_is_blank()).small());
    }
}

/// Paint the sheet, the printable area, the placed page and the clip hatch.
///
/// Returns the texture that was drawn — `None` when the page would not render,
/// which is the degraded-but-honest state described on [`texture_for`] — and
/// **what the overhang turned out to hold**, which is what the caption is then
/// written from. See [`Overhang`] for why the second half is returned rather
/// than recomputed.
fn paint(
    ui: &Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    shown: usize,
    rect: Rect,
    scale: f32,
) -> (Option<TextureId>, Overhang) {
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
    //
    // A missing plan or page size means the sheet the stepper is on does not
    // exist, which is a state one frame of a page-range edit can pass through.
    // The sheet and printable rectangles are already drawn, which is the
    // honest picture; there is no page to place and therefore no clip to
    // report, so the overhang is `Fits` rather than a warning about nothing.
    let (Some(&plan), Some(&size)) = (
        job.plans.get(shown),
        job.plans
            .get(shown)
            .and_then(|p| inputs.page_sizes.get(p.index)),
    ) else {
        return (None, Overhang::Fits);
    };

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
    //
    // ★ `plan.placement.clipped` is the cheap GEOMETRIC gate and is kept as
    // one: it is a plan-time fact needing no raster, so it costs nothing and it
    // short-circuits every sheet that fits. Everything past it asks the
    // narrower question O113 is about — is anything actually THERE.
    if !plan.placement.clipped {
        return (texture, Overhang::Fits);
    }
    // ★ The mask is re-borrowed here rather than returned from `texture_for`
    // because it is 64 KiB (see `ink::CELLS_LONG_SIDE`) and cloning it once a
    // frame to satisfy a borrow would cost more than the hatch. The mutable
    // borrow `texture_for` took has ended by this line, so an immutable
    // reborrow is free.
    //
    // Matched against the texture that was actually DRAWN, not merely taken
    // from the cache: if those two could differ, the mask would be describing a
    // different page from the one on screen, which is the exact staleness the
    // shared cache tuple exists to prevent. Asserting it here as well costs one
    // comparison and makes the invariant checkable at the point it is relied
    // on.
    let mask = dialog
        .preview_texture
        .as_ref()
        .filter(|(_, tex, _)| Some(tex.id()) == texture)
        .map(|(_, _, mask)| mask);
    let overhang = hatch_lost_content(&painter, placed, printable, mask, visuals.warn_fg_color);
    // ★★★ REMEMBERED HERE, at the single point where the ink question was
    // actually asked — operator request O113, 2026-09-04.
    //
    // The commit button's count is the geometric count minus the sheets known
    // blank, and this is where a sheet becomes known. It is recorded from the
    // value `hatch_lost_content` just returned — the same computation the
    // hatch above was drawn from — and never re-derived: a second call site
    // for a question with a pixel threshold in it is how the button and the
    // picture would come to disagree again, one level further down.
    //
    // Recorded only past the `plan.placement.clipped` gate, so the map holds
    // exactly what the count consults: what the ink test found in the overhang
    // of a sheet that has one. A sheet that fits is not a sheet anybody needs
    // a verdict about.
    dialog
        .verdicts
        .remember(inputs.context, &plan, inputs.page_sizes, overhang);
    (texture, overhang)
}

/// Hatch **only the parts of `placed` that fall outside `printable` AND carry
/// ink**.
///
/// # ★★★ Operator request O113 — what changed here and why
///
/// > *"can you make it so the red pattern you put over the page if it is going
/// > to print beyond the printable borders is only over the areas that extend
/// > beyond the printable page? Our drawing get drawn 1:1 and the area that
/// > isn't printed is just empty border."*
///
/// This function used to hatch the whole overhanging region the moment
/// `Placement::clipped` was true. That flag is a *geometric* verdict — the page
/// box exceeds the printable rectangle — and on a CAD sheet printed 1:1 the
/// part that exceeds it is empty paper. The hatch shouted about losing
/// something on every drawing, and nothing was being lost, which is a
/// disclosure that is technically true and practically false. An operator who
/// sees the same red band on every 1:1 drawing learns to ignore it, and then
/// does not see it on the one sheet where the border really does have a title
/// block in it.
///
/// It now asks [`ink::InkMask`] what is actually in the band, and hatches the
/// **ink extent within it**. No ink in the band ⇒ **no hatch at all**, which
/// is the whole request.
///
/// # Only the right and bottom overhangs, and that is not an omission
///
/// Carried from the original, because it is still true: a placement offsets the
/// page *into* the printable area from its top-left corner, so content is lost
/// off the far edges. Hatching all four would draw a warning over paper that
/// will print.
///
/// # ★★ The two bands are now DISJOINT, which fixes a second over-hatch
///
/// The old code took `right_band.union(bottom_band)`, and `Rect::union` is a
/// **bounding box**, not a set union: the union of a tall strip on the right
/// and a wide strip along the bottom is a rectangle that also covers the region
/// which is neither right of nor below the printable area — paper that prints
/// perfectly. That was a second, smaller instance of the same defect O113
/// reports, hiding inside the first.
///
/// The bottom band is therefore cut at `printable.max.x`, so the two bands meet
/// without overlapping. Disjoint also means the shared bottom-right corner is
/// hatched once rather than twice, so its lines are the same weight as
/// everywhere else instead of reading as a darker patch.
///
/// # ★ What happens when there is no mask
///
/// `mask` is `None` when the page did not render — the same degraded state
/// [`texture_for`] documents, in which the preview shows a flat fill instead of
/// the page. In that state the honest answer to *"is anything in the band?"* is
/// **"unknown"**, and the disclosure falls back to the old behaviour: hatch the
/// whole band. Silence would be the wrong failure direction here. A missing
/// render must not be able to turn a warning off.
/// # Returns
///
/// What the band turned out to hold, so the caption can be written from the
/// same computation the hatch was — see [`Overhang`].
fn hatch_lost_content(
    painter: &egui::Painter,
    placed: Rect,
    printable: Rect,
    mask: Option<&ink::InkMask>,
    colour: Color32,
) -> Overhang {
    let (lost, overhang) = lost_regions(placed, printable, mask);
    for region in lost {
        hatch(painter, region, colour);
    }
    overhang
}

/// **What is actually lost, and what to call it** — the whole of operator
/// request O113's decision, with no painter in it.
///
/// # ★★★ Pure on purpose, because this is the pair that must not disagree
///
/// It returns the rectangles to hatch *and* the [`Overhang`] the caption is
/// written from, from **one** computation. That is the only structural
/// guarantee that the picture and the sentence agree: they are not two readings
/// of the same data, they are two halves of one answer. Splitting it out from
/// [`hatch_lost_content`] also makes that agreement **testable with no GUI at
/// all** — see [`tests::a_blank_overhang_hatches_nothing_and_says_so`], which
/// asserts the empty list and the `BlankBand` verdict together, and
/// [`tests::an_inked_overhang_hatches_only_the_ink_and_says_so`], which asserts
/// the hatched rectangle really is a small part of the band.
///
/// The returned rectangles are in screen points and are ready to draw; the
/// caller does no further arithmetic on them.
fn lost_regions(
    placed: Rect,
    printable: Rect,
    mask: Option<&ink::InkMask>,
) -> (Vec<Rect>, Overhang) {
    // The two disjoint overhangs, in SCREEN points.
    //
    // Right: everything of the page past the printable area's right edge, full
    // height. Bottom: everything past its bottom edge, cut at that same right
    // edge so the corner belongs to exactly one band.
    let bands = [
        placed.intersect(Rect::everything_right_of(printable.max.x)),
        placed
            .intersect(Rect::everything_below(printable.max.y))
            .intersect(Rect::everything_left_of(printable.max.x)),
    ];
    let Some(mask) = mask else {
        // No raster to ask. Disclose the whole of both bands rather than
        // nothing: "unknown" must not present as "nothing is lost".
        let whole: Vec<Rect> = bands.into_iter().filter(|b| b.is_positive()).collect();
        // A clipped placement with no positive band is a geometric
        // contradiction the arithmetic can produce at a degenerate scale.
        // Report it as `Fits` rather than as an unexplained warning with no
        // picture under it.
        let verdict = if whole.is_empty() {
            Overhang::Fits
        } else {
            Overhang::Unknown
        };
        return (whole, verdict);
    };

    let mut lost = Vec::new();
    for band in bands {
        if !band.is_positive() {
            continue;
        }
        // The band, expressed as a fraction of the page, handed to the mask;
        // the ink extent inside it, brought back to screen points. `placed` is
        // the page's own rectangle on screen, so it is exactly the frame that
        // converts between the two — and it already carries the zoom, the pan
        // and the placement scale, which is why nothing here has to know about
        // any of them.
        //
        // ★ THE REQUEST, in one `else`: no ink in the band means the band is
        // empty paper, so nothing is lost and nothing is drawn.
        let Some(extent) = mask.ink_extent(normalised_in(band, placed)) else {
            continue;
        };
        let region = denormalised_in(extent, placed);
        if region.is_positive() {
            lost.push(region);
        }
    }
    let verdict = if lost.is_empty() {
        Overhang::BlankBand
    } else {
        Overhang::Losing
    };
    (lost, verdict)
}

/// Express `part` as a fraction of `whole`: 0..1 page space, the coordinate
/// system [`ink::InkMask`] speaks.
///
/// A degenerate `whole` — a page placed at zero scale, which a nonsense
/// `/MediaBox` can produce — yields a rectangle the mask rejects rather than a
/// `NaN` that would propagate into the hatch geometry.
fn normalised_in(part: Rect, whole: Rect) -> Rect {
    if whole.width() <= 0.0 || whole.height() <= 0.0 {
        return Rect::NOTHING;
    }
    Rect::from_min_max(
        egui::pos2(
            (part.min.x - whole.min.x) / whole.width(),
            (part.min.y - whole.min.y) / whole.height(),
        ),
        egui::pos2(
            (part.max.x - whole.min.x) / whole.width(),
            (part.max.y - whole.min.y) / whole.height(),
        ),
    )
}

/// The inverse of [`normalised_in`]: 0..1 page space back to screen points.
fn denormalised_in(fraction: Rect, whole: Rect) -> Rect {
    Rect::from_min_max(
        egui::pos2(
            whole.min.x + fraction.min.x * whole.width(),
            whole.min.y + fraction.min.y * whole.height(),
        ),
        egui::pos2(
            whole.min.x + fraction.max.x * whole.width(),
            whole.min.y + fraction.max.y * whole.height(),
        ),
    )
}

/// Draw diagonal hatching across `area`.
///
/// Split out of [`hatch_lost_content`] when that function gained the ink test,
/// so the geometry question (*what is lost?*) and the drawing question (*what
/// does a hatch look like?*) stopped sharing a body. The lines run at 45° and
/// are clamped to the rectangle at both ends, which is what lets a caller hatch
/// several small regions without any of them bleeding into the paper between.
fn hatch(painter: &egui::Painter, area: Rect, colour: Color32) {
    let step = 6.0;
    let mut x = area.min.x;
    while x < area.max.x + area.height() {
        painter.line_segment(
            [
                egui::pos2(x.min(area.max.x), area.min.y),
                egui::pos2((x - area.height()).max(area.min.x), area.max.y),
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
    placement: Placement,
) {
    let sheets = inputs.job.plans.len();
    // ★★★ `horizontal_wrapped`, NOT `horizontal` — 2026-09-03, and this was the
    // last cause of the operator's "two scroll bars that won't go away".
    //
    // Measured: `print-strip natural_w=379.9 column_w=340.0`. This row of seven
    // controls has ALWAYS been wider than the column it sits in — 40 pt wider
    // at the default width — and `ui.horizontal` does not care: it lays out
    // past the end and reports a `min_rect` that wide. That overflow became the
    // body's content width, and the body's content width is what raises the
    // horizontal scrollbar.
    //
    // It was invisible for as long as the body forced its content to a fixed
    // 764 pt, because the strip's 380 fitted inside that and merely spilled
    // across the divider into the options column's space. It is visible in the
    // very first capture of this defect, where the button row runs past the
    // separator.
    //
    // ★★ Wrapped rather than given a wider minimum, and that choice is the
    // point. A minimum would be a **constant asserting how wide seven buttons
    // are**, and that depends on the theme preset's font size and button
    // padding, and on the label text — so it would be correct in one preset and
    // wrong in another, which is the same class of defect as the hard-coded
    // item gap this file's caller was just corrected for. A wrapped row is
    // bounded by its available width **by construction**: there is no number to
    // get wrong, and no preset in which it can overflow.
    //
    // The cost is that the row becomes two rows on a narrow column. That is
    // paid for in `STRIP_HEIGHT_PTS`, which reserves the space for it.
    let probe = ui.horizontal_wrapped(|ui| {
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
        // ★★★ POP OUT — O112 ask 2, and it is the LAST control in the row.
        //
        // Last because the row is `horizontal_wrapped`: on a narrow column the
        // row becomes two, and the control that wraps first should be the one
        // the operator reaches for least. Stepping sheets and zooming are what
        // a preview is for; moving it to another window is a once-per-session
        // act.
        //
        // ★ It is a button and not a checkbox, and not a toggle that stays
        // pressed. The window IS the state — while it is open the operator can
        // see it, and while it is closed there is nothing to un-toggle. A
        // latching control here would be a second place the truth lives, and
        // the two would disagree the moment the window was closed from its own
        // title bar, which is the documented way back.
        //
        // ★ Absent — not greyed — in the popped window itself. See
        // [`Placement`]: there is nothing there for it to do, and R9's own
        // distinction is that greying is for *temporarily* unavailable.
        if placement == Placement::InDialog {
            let popout = ui
                .button(t::preview_pop_out())
                .on_hover_text(t::preview_pop_out_tooltip());
            // Visibility-gated, because a driven check clicks this. An ungated
            // rect publishes the control's *content* position, which inside a
            // scroll area can be hundreds of points outside the window — the
            // harness then aims the real pointer at nothing, presses nothing,
            // and reports the feature as inert. That has happened in this
            // project before and is written up on `dialogs::formfield`'s
            // rotation row.
            crate::diag::ui_rect_visible(REGION_POP_OUT, popout.rect, ui.clip_rect());
            if popout.clicked() {
                dialog.preview_popped = true;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "print-preview-popped state=out".to_owned()
                });
            }
        }
    });
    // ★ THE REGRESSION TEST FOR THE LAST CAUSE OF THE TWO-SCROLLBAR DEFECT,
    // reported from inside the process because it cannot be seen from outside.
    //
    // `laid_w` must never exceed `column_w`. When it did — measured at
    // `laid_w=379.9 column_w=340.0` on 2026-09-03 — the overflow propagated
    // into the body's content width and raised a horizontal scrollbar that no
    // amount of resizing could dismiss, because the strip's width did not
    // depend on the window's.
    //
    // A driven check asserts the inequality rather than a value: the strip's
    // width is a function of the theme's font and button padding and of the
    // label text, so any constant here would be a claim that decays. The
    // relationship is what matters and it is what is asserted.
    crate::diag::trace(|| {
        format!(
            "print-strip laid_w={:.1} column_w={:.1}",
            probe.response.rect.width(),
            rect.width()
        )
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
    // ★ From the frame's context, not built here — see [`PreviewKey::new`].
    // The verdict cache's validity is defined by that context, so a key
    // derived from it cannot be stronger than the one the verdicts are held
    // under, which is the property that stops a remembered "the overhang is
    // blank" outliving the raster it was measured from.
    let key = inputs.context.preview_key(page);
    if let Some((cached, texture, _)) = &dialog.preview_texture
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
    // ★ The ink mask is built HERE, from the same pixmap, on the same miss —
    // operator request O113. Once per raster and never per frame: it is a pure
    // function of these bytes, so recomputing it while the operator pans would
    // be re-deriving an answer that cannot have changed. See the
    // `preview_texture` field's own docs for why it shares this tuple and this
    // key rather than living in a cache of its own.
    let mask = ink::InkMask::from_rgba_premultiplied(
        rendered.pixmap.width(),
        rendered.pixmap.height(),
        rendered.pixmap.data(),
    );
    let texture = upload(ctx, &rendered.pixmap);
    let id = texture.id();
    dialog.preview_texture = Some((key, texture, mask));
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
#[path = "preview_tests.rs"]
mod tests;
