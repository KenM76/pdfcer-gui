//! # `dialogs::print::preview` — a zoomable, pannable picture of the real sheet
//!
//! ## Why a picture rather than a number
//!
//! pdfcer diverges from Acrobat here on purpose: Acrobat clips silently when
//! content falls outside the printable area, and pdfcer says so. That
//! divergence is worth nothing if the GUI reduces it to a count an operator can
//! look past. The whole reason `pdfcer-print` reads real device geometry
//! instead of guessing a bounding box is so this picture can be exact — drawing
//! only the SHEET and not the PRINTABLE AREA would show a page fitting that
//! will not.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/print/preview/mod.md`.

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureId, Ui, Vec2};

use crate::app::state::OpenDoc;
use crate::dialogs::print::PrintDialog;
use crate::dialogs::print::ink;
use crate::dialogs::print::position;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

/// What the shown sheet's overhang turned out to contain — the fact the hatch
/// is drawn from, lifted out so the CAPTION can be drawn from the same one.
///
/// # Why this is a return value and not something the caption re-derives
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
    /// The placement reported a clip and the band is **blank paper** — the
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

// THE PREVIEW COLUMN'S WIDTH IS A PARAMETER, NEVER A CONSTANT IN THIS FILE.
//
// The circle that tempts one is real: the dialog body lives inside a
// `ScrollArea::both`, a horizontally scrollable area has no bounded width to
// report, so a column laid out to `available_width` inside one is measuring a
// number its own content decides. Two fixed columns appear to break that.
//
// They do not, and the two defects are operator-visible:
//
//  1. **Two scrollbars that cannot be dismissed.** A fixed content width has to
//     be `max`'d with `available_width` at the call site or a wide window
//     leaves a dead strip — which puts `available_width` straight back into the
//     circle the constant was there to break, and deadlocks the two bars
//     against each other. `PrintDialog::body` documents the mechanism.
//  2. **A preview that cannot be made bigger.** *"the preview should be
//     adjustable size"* — widening the dialog widens the empty space and leaves
//     the sheet the same postage stamp, because the column cannot grow.
//
// ⇒ The answer to the circle is to lay the columns out to the width actually
// available and tell the `ScrollArea` nothing. It then shows a bar when, and
// only when, the content does not fit — the only thing a scrollbar should ever
// mean.
//
// The width comes from `PrintDialog::preview_width`, which the splitter drags;
// the floor, the default and the options column's floor are
// `PREVIEW_MIN_WIDTH_PTS`, `PREVIEW_DEFAULT_WIDTH_PTS` and
// `OPTIONS_COLUMN_MIN_WIDTH_PTS` in [`super::layout`]. `column` takes it as a
// parameter.

/// Height of the fixed strip under the preview canvas, in egui points.
const STRIP_HEIGHT_PTS: f32 = 96.0;

/// Smallest preview canvas, in egui points. Below this the sheet outline
/// stops being a picture and becomes a smudge, so the column scrolls rather
/// than shrinking further.
const CANVAS_MIN_HEIGHT_PTS: f32 = 160.0;

/// Largest preview canvas, in egui points.
const CANVAS_MAX_HEIGHT_PTS: f32 = 1400.0;

/// The Pop-out button's region, for the driven harness.
///
/// Declared with the **visibility-gated** publisher, unlike the preview
/// column's own region next door, and the two are opposites on purpose: this
/// one exists to be clicked, so a rect the operator cannot reach is worse than
/// no rect at all; that one exists to be seen to disappear, so a rect that is
/// merely scrolled out of view must still count as present.
pub(super) const REGION_POP_OUT: &str = "print.preview.popout";

/// The placed page's own rectangle inside the preview canvas.
///
/// Published so a driven check can start a drag INSIDE the page rather than on
/// the paper around it — the two gestures share one mouse button and differ
/// only by where the press landed, so a driver that guessed the rectangle
/// would be measuring the pan half the time.
///
/// It is the page clipped to the canvas, i.e. the part that can actually be
/// pressed, and it is absent rather than empty when there is none. See the
/// publisher in [`paint`] for why neither the page nor `ui_rect_visible` of the
/// page is the right thing to publish.
pub(super) const REGION_PAGE: &str = "print.preview.page";

/// The fraction of the canvas the fitted sheet occupies.
const FIT_MARGIN: f32 = 0.92;

/// Resolution the preview bitmap is rendered at, in DPI.
const TARGET_DPI: f32 = 150.0;

/// Ceiling on the preview bitmap's longest side, in pixels.
const MAX_SIDE_PX: f32 = 2200.0;

/// Smallest preview zoom, as a multiple of the fit scale.
const ZOOM_MIN: f32 = 0.25;
/// Largest preview zoom. See [`ZOOM_MIN`].
const ZOOM_MAX: f32 = 40.0;

/// One zoom step per button press.
const ZOOM_STEP: f32 = 1.25;

/// The egui texture name the preview bitmap is uploaded under.
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
/// ## The settings field, and the standing rule it satisfies
///
/// **A rendering knob and the key that invalidates its cache land in the same
/// commit, or the control silently does nothing.** Every setting reaching
/// [`crate::app::settings::SettingsExt::render_options`] changes these pixels,
/// so the whole `Settings` value is in the key.
///
/// A **font-environment generation is absent**, and the reason is that there is
/// nothing to key on rather than that it was forgotten: the preview rasterises
/// through `pdfcer_render::render_page_with_view`, which takes no font
/// environment at all. The operator's font folders reach *embedding*
/// ([`crate::app::fonts`]) and do not reach a render. The day a render takes
/// one, this key gains a field in the same commit — the rule above is the whole
/// point of this paragraph.
///
/// ### Why the whole `Settings` and not the fields it reads
///
/// Because a list here would be a second statement of which settings affect a
/// render — one in this key and one in `SettingsExt::render_options` — and the
/// failure mode of the two disagreeing is silent: a rendering setting added to
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
    /// is not the rendering fields spelled out.
    settings: pdfcer_core::settings::Settings,
    /// The fixed line width on paper and the page's placement scale (O233);
    /// `None` when the document's own weights print.
    lines: Option<(f64, f64)>,
}

impl PreviewKey {
    /// Build the key for one page.
    ///
    /// # Called from exactly one place, and that place is the VERDICT
    /// # cache's context
    ///
    /// [`super::verdicts::Context::preview_key`] is the sole caller, and the
    /// inversion is the point rather than plumbing.
    ///
    /// That module remembers, per sheet, whether the overhang the preview
    /// hatched turned out to be blank, so the commit button can subtract the
    /// sheets known to lose nothing. A remembered verdict is a claim about
    /// **these pixels**, and a cache key claims that equal keys would render
    /// the same image. A verdict cached under a weaker key would be a verdict
    /// about a page that has changed — and it would be confidently wrong,
    /// because its whole purpose is to take a warning away.
    ///
    /// Deriving this key **from** the verdict cache's context makes "the
    /// verdict is keyed on at least what the pixels are keyed on" a structural
    /// fact rather than a promise held by two doc comments. Constructing it
    /// here as well, from the same three values, would be the second reading of
    /// one rule that this type's `settings` field already argues against.
    pub(super) fn new(
        page: usize,
        scope: pdfcer_render::AnnotationScope,
        settings: &pdfcer_core::settings::Settings,
        lines: Option<(f64, f64)>,
    ) -> Self {
        Self {
            page,
            scope,
            settings: settings.clone(),
            lines,
        }
    }

    /// Set the fixed line width, if any, on a render at `scale` pixels per
    /// page point.
    pub(super) fn apply_lines(&self, options: &mut pdfcer_render::RenderOptions, scale: f64) {
        if let Some((pt, placement)) = self.lines {
            super::lines::apply(options, Some(pt), scale, placement);
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
    /// Both caches in this dialog hang off it. The page texture's
    /// [`PreviewKey`] is built from it (see [`PreviewKey::new`]), and every
    /// remembered overhang verdict is void the moment it changes — so the
    /// verdict cache cannot be keyed on less than the pixels are.
    pub(super) context: &'a super::verdicts::Context,
}

/// The preview zoom and pan after multiplying the zoom by `step` while
/// holding the screen point `at` still.
///
/// # The anchor term, derived rather than tuned
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
fn raster_scale(page_pt: (f64, f64)) -> f32 {
    let dpi_scale = crate::units::scale_from_dpi(f64::from(TARGET_DPI)) as f32;
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
/// # One function, two homes, and the difference is one button
///
/// The preview is the same picture and the same arithmetic in the print
/// dialog's column and in its own OS window. What differs is a single control:
/// the column offers *"pop this out"*, and the popped window does not, because
/// the way back is its own close button. Passing that as a parameter rather
/// than writing a second draw function is the whole reason this feature is
/// cheap — and a second draw function is how the two copies of a preview come
/// to disagree about a margin.
///
/// There is deliberately **no** "put it back" button in the popped window.
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
    // The width is passed IN, not read from a constant, because the operator
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
    let scale = fit * dialog.preview_zoom;

    // ---- what a primary drag takes hold of — operator request O208 ----
    //
    // The page, if the drag started on it; the paper otherwise. Both gestures
    // are a primary drag on the same rectangle, so the only thing that can
    // separate them is WHERE the press landed.
    //
    // Latched at `drag_started_by`, and read from `press_origin` rather
    // than `interact_pointer_pos`. Two findings in `D:\dev\rag\egui\`
    // bind this: `drag_started` fires only after egui's drag threshold, by
    // which time the interact position has moved off the pixel the operator
    // actually pressed, and `press_origin` is the one value that stays put for
    // the whole press. Re-deciding per frame would be worse still — the
    // page moves under the pointer while it is being dragged, so the same
    // gesture would reclassify itself the moment it left the page and finish by
    // panning the view.
    //
    // `*_by(Primary)`, never a bare predicate: per
    // `egui_response_drag_predicates_are_button_agnostic.md` the unqualified
    // form fires for middle and right drags too, which would silently claim the
    // right-drag this preview may later want for a context menu.
    // A poster sheet is not draggable: its position is the imposition's.
    let page_rect = job
        .plans
        .get(shown)
        .filter(|plan| plan.tile.is_none())
        .and_then(|plan| {
            inputs.page_sizes.get(plan.index).map(|&size| {
                let (_, printable) = frames(job, rect, dialog.preview_pan, scale);
                placed_rect(printable, plan.placement, size, scale)
            })
        });
    if response.drag_started_by(egui::PointerButton::Primary) {
        let origin = ui.input(|i| i.pointer.press_origin());
        dialog.preview_grab = match (origin, page_rect) {
            (Some(at), Some(page)) if page.contains(at) => position::Grab::Page,
            _ => position::Grab::Paper,
        };
        // The canvas is already `FOCUSABLE` — `Sense::click_and_drag()` is
        // `CLICK | FOCUSABLE | DRAG` — but nothing had ever asked for the
        // focus. The arrow-key nudge below is gated on having it, so the
        // gesture that moves the page is also what makes the keys that move it
        // live.
        response.request_focus();
    }
    if response.drag_stopped_by(egui::PointerButton::Primary) {
        dialog.preview_grab = position::Grab::Nothing;
    }
    if response.dragged_by(egui::PointerButton::Primary) {
        let delta = response.drag_delta();
        match dialog.preview_grab {
            // Screen points back to paper points by dividing out the one scale
            // the whole preview is drawn at, so the page follows the pointer at
            // any zoom. No sign flip: the placement offsets are fed to the
            // device context as its own `dest_x`/`dest_y`, which is the same
            // right-and-down sense as the screen.
            position::Grab::Page => {
                if let Some(plan) = job.plans.get(shown) {
                    dialog.page_positions.nudge(
                        plan.index,
                        f64::from(delta.x / scale),
                        f64::from(delta.y / scale),
                    );
                    // The job is planned at the top of `PrintDialog::show`, so
                    // a position changed here reaches the placement on the NEXT
                    // frame. Without this the last frame of a drag would never
                    // be drawn, and the page would appear to stop a pointer-move
                    // short of where it was let go.
                    ui.ctx().request_repaint();
                }
            }
            position::Grab::Paper | position::Grab::Nothing => dialog.preview_pan += delta,
        }
    }
    // The keyboard route to the same thing — O206's clause: a gesture
    // offered with a pointer is offered with the keys as well. It reads the
    // focus rather than the hover so a nudge cannot be delivered to a page the
    // operator is merely passing over.
    if response.has_focus()
        && let (Some((dx, dy)), Some(plan)) = (position::arrow_nudge(ui), job.plans.get(shown))
        && plan.tile.is_none()
    {
        dialog.page_positions.nudge(plan.index, dx, dy);
        ui.ctx().request_repaint();
    }
    // Rendered before the strip is laid out, because the strip's Actual-size
    // button needs the scale this frame settled on.
    let (texture, overhang, edges) = paint(ui, inputs, dialog, shown, rect, scale);
    strip(ui, inputs, dialog, shown, rect, scale, placement);

    // Read AFTER `paint`, which is what makes the sheet on screen count as
    // examined on the frame it is drawn rather than on the next one — see the
    // clip summary below for what this number is and how its wording follows
    // from it. Also read BEFORE the trace, so the trace reports the claim this
    // frame's picture was drawn beside.
    let claim = dialog
        .verdicts
        .claim(inputs.context, job, inputs.page_sizes);

    // The DOCUMENT page the shown sheet is, which is what a position is keyed
    // on. `shown` walks the job; see `PagePlan::index` for why the two are not
    // the same number. `0` only when the job plans no pages, which returned
    // above.
    let page_index = job.plans.get(shown).map_or(0, |plan| plan.index);

    // The canvas rectangle and the two geometry rectangles, in one line.
    //
    // `sheet=` and `printable=` are here because they are the only honest
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
             sheet_of={}/{} zoom={:.3} pan=({:.1},{:.1}) pos={:.2},{:.2} grab={} \
             moved={} tex={} overhang={} edges={} claim={}:{}",
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
            // `pos=` is the ONLY headless evidence that operator
            // request O208 works, and it has to be here rather than inferred
            // from `overhang=` because the two are independent: a page dragged
            // across a blank border changes its position and changes neither
            // the hatch nor the verdict. In PAPER POINTS, from the operator's
            // displacement rather than from the placement, so a driven check
            // reads the quantity the controls set and not the sum of that and
            // whatever `place_page` chose — which would make `(0,0)` mean
            // two different things.
            //
            // `grab=` beside it is what separates the two gestures that share
            // one button. A drag that pans when it should have moved the page
            // leaves `pos=` unchanged and `pan=` changed, and without this word
            // that is indistinguishable from a drag that never reached the
            // canvas at all.
            dialog.page_positions.of(page_index).dx_pt,
            dialog.page_positions.of(page_index).dy_pt,
            match dialog.preview_grab {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                position::Grab::Nothing => "none",
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                position::Grab::Page => "page",
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                position::Grab::Paper => "paper",
            },
            dialog.page_positions.moved_count(),
            u8::from(texture.is_some()),
            // `overhang=` is the ONLY headless evidence that operator request
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
            // `edges=` is the headless evidence for the four-edge hatch, and
            // it is SEPARATE from `overhang=` because the two answer different
            // questions. `overhang=` says what the ink test found; `edges=`
            // says which of the page's four sides hang past the printable area
            // at all, which is the geometry the widened hatch draws from.
            //
            // A four-character word in [`geometry::EDGE_LETTERS`]' order, not a
            // count, because a count cannot tell a page hanging off the LEFT
            // from one hanging off the right, and that distinction is the whole
            // of what changed. An unmoved oversized page reads `.r.b`; the same
            // page dragged up and left reads `lrtb`.
            edges
                .iter()
                .zip(geometry::EDGE_LETTERS)
                .map(|(&over, letter)| if over { letter } else { '.' })
                .collect::<String>(),
            // The job-wide claim beside the sheet-level verdict, because the
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
    // THE COUNT GETS BETTER; THE SENTENCE DOES NOT GET VAGUER — operator
    // request O113.
    //
    // `Job::clipped()` is a plan-time geometric fact and is exactly true.
    // Softening its wording to match a picture that knows more would trade a
    // true statement for a comfortable one, so the correction belongs in the
    // NUMBER, not in the prose: the count is corrected by verdicts the preview
    // has *already* produced, at no rendering cost, and the wording follows
    // what the corrected number can support:
    //
    //   * nothing subtracted  -> the geometric sentence, unchanged, word for
    //     word — including the case where the operator never stepped the
    //     preview at all;
    //   * every clipped sheet examined -> the same sentence, now verified;
    //   * some examined, some not -> a ceiling, which says so.
    //
    // It must be the SAME claim the commit button draws, or this line and
    // the button would show two different numbers for one job — the exact
    // contradiction O113 reported, moved rather than fixed. Both call
    // `Verdicts::claim` on the same context, and `ClipClaim` owns which
    // sentence each state gets, so neither surface chooses wording of its own.
    if let Some(summary) = claim.summary(job.plans.len()) {
        ui.label(egui::RichText::new(summary).color(ui.visuals().warn_fg_color));
    }
    // …and the sheet-level correction under it, which is what stops the
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
fn paint(
    ui: &Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    shown: usize,
    rect: Rect,
    scale: f32,
) -> (Option<TextureId>, Overhang, [bool; 4]) {
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let job = inputs.job;

    let (sheet_rect, printable) = frames(job, rect, dialog.preview_pan, scale);
    painter.rect_filled(sheet_rect, 2.0, visuals.extreme_bg_color);
    painter.rect_stroke(
        sheet_rect,
        2.0,
        Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        StrokeKind::Middle,
    );

    // The printable area, inset by the driver's own unprintable margins —
    // the rectangle that actually constrains the job, and the reason the
    // preview is worth having at all. Computed by [`frames`] above, with the
    // sheet, because the two share an origin.
    painter.rect_stroke(
        printable,
        0.0,
        Stroke::new(1.0, visuals.weak_text_color()),
        StrokeKind::Middle,
    );

    // `page_sizes` is indexed by `plan.index`, NOT by `shown`.
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
        return (None, Overhang::Fits, [false; 4]);
    };

    let placed = placed_rect(printable, plan.placement, size, scale);
    // The drag target, published — and it is the GRABBABLE part of the page,
    // not the page.
    //
    // Neither obvious choice works here, and both fail silently:
    //
    // * `placed` itself is wrong because at actual size on a large-format sheet
    //   the page is several times the canvas, so its centre is off screen —
    //   sometimes outside the window entirely. A driver handed that point
    //   presses the desktop.
    // * `ui_rect_visible(.., placed, rect)` is wrong in the opposite direction:
    //   it refuses to publish anything below 60 % visible, which is exactly the
    //   oversized-page case the position controls exist for. The region would
    //   be absent precisely when it is needed, and an absent region reads as a
    //   missing control.
    //
    // The intersection is right on both counts: its centre is inside the page
    // AND inside the canvas by construction, and it is empty only when the page
    // really has been panned off the canvas and cannot be grabbed at all.
    // A poster sheet shows only its tile: the page is clipped to the tile's
    // content, and the band's marks and label are drawn as the sheet has them.
    let canvas = plan.tile.map_or(rect, |tile| {
        super::poster::paint_band(
            &painter,
            printable,
            &tile,
            scale,
            &super::poster::document_name(inputs.doc),
        );
        super::poster::on_screen(printable, tile.content_pt, scale).intersect(rect)
    });
    let painter = painter.with_clip_rect(canvas);
    let grabbable = placed.intersect(rect);
    if grabbable.is_positive() && plan.tile.is_none() {
        crate::diag::ui_rect(REGION_PAGE, grabbable);
    }

    // The rendered page, if one is available. The fallback is a flat fill — a
    // preview showing the right rectangle and no content is degraded but
    // honest; one showing a stale page would be wrong.
    let texture = texture_for(ui.ctx(), inputs, dialog, plan.index, plan.placement.scale);
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
    if texture.is_some() {
        let frame = detail::Frame {
            doc: inputs.doc,
            key: inputs.context.preview_key(plan.index, plan.placement.scale),
            page: plan.index,
            size,
            placed,
            canvas,
            base_scale: raster_scale(size),
            print_scale: (f64::from(job.resolution.dpi) / 72.0 * plan.placement.scale) as f32,
            scope: dialog.scope,
        };
        dialog.preview_detail.frame(&painter, &frame);
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
    // `plan.placement.clipped` is the cheap GEOMETRIC gate and is kept as
    // one: it is a plan-time fact needing no raster, so it costs nothing and it
    // short-circuits every sheet that fits. Everything past it asks the
    // narrower question O113 is about — is anything actually THERE.
    if !plan.placement.clipped {
        return (texture, Overhang::Fits, [false; 4]);
    }
    // The mask is re-borrowed here rather than returned from `texture_for`
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
    // REMEMBERED HERE, at the single point where the ink question is actually
    // asked — operator request O113.
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
    (texture, overhang, overhang_edges(placed, printable))
}

/// The fixed strip: the sheet stepper on the left, the zoom controls on the
/// right, and the magnification readout beneath.
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
    // `horizontal_wrapped`, NOT `horizontal` — this row is the operator's
    // "two scroll bars that won't go away" if it is allowed to overflow.
    //
    // Measured: `print-strip natural_w=379.9 column_w=340.0`. Seven controls
    // are wider than the column at the default width, and `ui.horizontal` does
    // not care: it lays out past the end and reports a `min_rect` that wide.
    // That overflow becomes the body's content width, and the body's content
    // width is what raises the horizontal scrollbar. A body that forced its
    // content to a fixed width would hide it, not fix it — the row would merely
    // spill across the divider into the options column's space.
    //
    // Wrapped rather than given a wider minimum, and that choice is the point.
    // A minimum would be a **constant asserting how wide seven buttons are**,
    // which depends on the theme preset's font size and button padding and on
    // the label text — so it would be correct in one preset and wrong in
    // another. A wrapped row is bounded by its available width **by
    // construction**: there is no number to get wrong, and no preset in which
    // it can overflow.
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
        // POP OUT — O112 ask 2, and it is the LAST control in the row.
        //
        // Last because the row is `horizontal_wrapped`: on a narrow column the
        // row becomes two, and the control that wraps first should be the one
        // the operator reaches for least. Stepping sheets and zooming are what
        // a preview is for; moving it to another window is a once-per-session
        // act.
        //
        // It is a button and not a checkbox, and not a toggle that stays
        // pressed. The window IS the state — while it is open the operator can
        // see it, and while it is closed there is nothing to un-toggle. A
        // latching control here would be a second place the truth lives, and
        // the two would disagree the moment the window was closed from its own
        // title bar, which is the documented way back.
        //
        // Absent — not greyed — in the popped window itself. See
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
            // and reports the feature as inert. `dialogs::formfield`'s rotation
            // row documents the same failure.
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
    // THE REGRESSION TEST FOR THE TWO-SCROLLBAR DEFECT, reported from inside
    // the process because it cannot be seen from outside.
    //
    // `laid_w` must never exceed `column_w`. Where it does — measured at
    // `laid_w=379.9 column_w=340.0` with the row unwrapped — the overflow
    // propagates into the body's content width and raises a horizontal
    // scrollbar that no amount of resizing dismisses, because the strip's width
    // does not depend on the window's.
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
    // `horizontal_wrapped`, not `horizontal`, for the reason the strip probe
    // above documents: a plain `ui.horizontal` lays out past the end of its
    // column and reports a `min_rect` that wide, which propagates outward as
    // the body's content width and raises a horizontal scrollbar the operator
    // cannot dismiss. Measured: this row laid out 402.5 pt inside a 340 pt
    // column and took the body to 822.5 pt against a 776 pt viewport.
    ui.horizontal_wrapped(|ui| {
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
fn zoom_by(dialog: &mut PrintDialog, step: f32, at: Pos2, centre: Pos2) {
    let (zoom, pan) = zoomed_view(dialog.preview_zoom, dialog.preview_pan, step, at, centre);
    dialog.preview_zoom = zoom;
    dialog.preview_pan = pan;
}

/// The texture for `page`, rendering and caching it if needed.
fn texture_for(
    ctx: &egui::Context,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    page: usize,
    placement: f64,
) -> Option<TextureId> {
    // From the frame's context, not built here — see [`PreviewKey::new`].
    // The verdict cache's validity is defined by that context, so a key
    // derived from it cannot be stronger than the one the verdicts are held
    // under, which is the property that stops a remembered "the overhang is
    // blank" outliving the raster it was measured from.
    let key = inputs.context.preview_key(page, placement);
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
    let mut options = super::commit::render_options(dialog.scope, &inputs.doc.settings);
    key.apply_lines(&mut options, f64::from(raster_scale(size)));
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
    // The ink mask is built HERE, from the same pixmap, on the same miss —
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
fn upload(ctx: &egui::Context, pixmap: &pdfcer_render::tiny_skia::Pixmap) -> TextureHandle {
    let image = premultiplied_image(pixmap.width(), pixmap.height(), pixmap.data());
    // LINEAR, matching the canvas: the preview is drawn at whatever the fit
    // and the operator's zoom produce, which is almost never 1:1 with the
    // bitmap, and nearest-neighbour sampling of a downsampled page is a mess
    // of aliased hairlines exactly where the operator is looking for hairlines.
    // Recorded so that a frame in which the preview AND the canvas both
    // uploaded is not mistaken for a frame in which only the canvas did — see
    // `crate::render::pressure`, whose refusal to guess is only worth
    // something if the census of uploads is complete.
    crate::render::pressure::record_other(
        ctx,
        crate::render::pressure::Surface::Preview,
        pixmap.width(),
        pixmap.height(),
    );
    ctx.load_texture(PREVIEW_TEXTURE_ID, image, egui::TextureOptions::LINEAR)
}

/// Read `tiny-skia`'s bytes as what they are: **premultiplied** RGBA8.
///
/// Split out from [`upload`] so the one thing here that can be silently wrong
/// is the one thing that is unit-testable without an `egui::Context`.
fn premultiplied_image(width: u32, height: u32, data: &[u8]) -> egui::ColorImage {
    egui::ColorImage::from_rgba_premultiplied([width as usize, height as usize], data)
}

/// **The preview's arithmetic** — the three rectangles and the hatch
/// verdict, split out at R2's ceiling when O208 widened the hatch to four
/// edges. Its header carries the seam and what it must not learn.
mod geometry;

/// The zoomed-in preview, rendered sharp up to the print resolution.
pub(super) mod detail;

// Glob re-exported rather than named one by one. A hand-written list here is a
// list inside the mechanism whose whole purpose is to make the split invisible,
// and this project's standing lesson is that such a list is exactly where the
// next addition goes missing — silently, with every gate green.
pub(super) use geometry::*;

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
