//! `line_weights` — **the CAD "line weights off" display mode, driven: the
//! button exists, the drawing actually gets thinner, and the screen says it is
//! not what will print.**
//!
//! `OPERATOR_REQUESTS.md` **O137**, 2026-09-05, in his words:
//!
//! > *"awhile ago you told me you removed the button to show all lines without
//! > their thickness — thin lines or something like cad has. The button never
//! > worked but I do want that display option!"*
//!
//! Both halves of that were correct. `view.thin_lines` was registered, drawn on
//! the View tab, and **inert**; it was unregistered on 2026-08-17 because
//! `RenderOptions` had no field behind it. The engine shipped
//! `stroke_display: StrokeDisplay { Actual, Hairline }` (`Pass 254.0`,
//! `8f9fb3e`) the day this shell asked, so the control is back — and *"the
//! button never worked"* is precisely the sentence this check exists to prevent
//! being true a second time.
//!
//! # ★★★ Why a driven check when five unit tests already pass
//!
//! Because every one of them is upstream of the screen.
//!
//! | test | proves | cannot see |
//! |---|---|---|
//! | `settings::only_the_canvas_worker_sets_stroke_display` | no export can carry it | whether anything carries it |
//! | `settings::every_export_path_renders_real_widths_…` | the request carries it, the funnel does not | whether a button raises the request |
//! | `worker::the_render_key_moves_when_line_weights_are_turned_off` | the cache cannot serve a stale raster | whether a raster is asked for |
//! | `render::hairline::line_weights_off_puts_less_ink_…` | the ENGINE really thins the drawing | whether the shell ever asks it to |
//! | `commands::mapping::every_chrome_toggle_has_a_registered_command` | the id exists | whether the button is reachable |
//!
//! The gap they leave is exactly the shape of the original defect: a registered
//! command, a correct handler, an invalidated cache — and **a press that never
//! arrives**, because the item is not on the band, or is in an overflow nothing
//! opens, or is drawn in a mode this document is not in. Only driving the real
//! window answers that.
//!
//! # ★★★ The pixel assertion is SIGNED, and that is the whole point
//!
//! The two display conventions routinely confused with each other are
//! **opposites**:
//!
//! | | | precedent |
//! |---|---|---|
//! | **line weights OFF ← what he asked for** | every stroke capped at one device pixel | AutoCAD `LWDISPLAY` off |
//! | enhance thin lines | sub-pixel strokes bumped **up** to one pixel | Acrobat's preference of that name |
//!
//! *One makes thick things thin; the other makes thin things thick.* A check
//! asserting only *"the canvas changed"* would pass on a build that shipped the
//! wrong one, and shipping the wrong one is worse than shipping nothing —
//! it looks like the feature working while doing the reverse. So the assertion
//! is **strictly less ink after the press**, counted as dark pixels inside the
//! canvas rect.
//!
//! # ★★ The zoom is a PRECONDITION, not decoration
//!
//! At page-fit a CAD sheet's strokes are already at or under one device pixel,
//! the engine's §8.4.3.2 floor has them there, and a *ceiling* at one device
//! pixel has nothing to cap — so the two pictures are identical and the check
//! would fail against a perfectly good build. That is measured, not assumed:
//! `render::hairline::the_two_modes_are_identical_where_there_is_nothing_to_cap`
//! pins it at scale 1.0 in a unit test.
//!
//! ⇒ So the run **zooms in first**, to the 200–400 % band he named as where he
//! actually reads a title block, and asserts it got there before pressing
//! anything. A run that could not zoom has not built its own precondition and
//! is a SKIP, not a pass.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | open, and read the View tab | the `line_weights` item exists, and the disclosure has **never** appeared |
//! | B | Ctrl+wheel in to ≥ [`MIN_ZOOM`] | the zoom is reached and a raster catches up |
//! | C | capture the canvas | some ink to lose |
//! | D | press the item | `view-chrome LineWeights on=false` |
//! | E | capture again | **strictly less** ink; the `status-group:line-weights` line appears |
//! | F | press it again | the disclosure goes away and the ink comes back |
//!
//! ★ Phase F is not symmetry for its own sake. A mode that cannot be left is a
//! mode that follows the operator into the next thing he does, and the
//! disclosure that says *"this is not what will print"* becoming permanent
//! would be the worst version of that.
//!
//! ⬜ **NOT RUN.** Written 2026-09-05 with another track holding the machine's
//! pointer. Two driven runs at once corrupt each other, so this check has never
//! been executed against a real window; every claim in this header about what
//! the application does is derived from its source and from unit tests, and the
//! run is owed.

use crate::checks::driving::{declared, declared_names, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::sys::vk::CONTROL as VK_CONTROL;
use crate::trace::Trace;

/// The canvas viewport's published region name.
const CANVAS_REGION: &str = "canvas-viewport";
/// The ribbon item this check presses.
const ITEM: &str = "ribbon.item.view.line_weights";
/// The status-bar disclosure's published region.
const DISCLOSURE: &str = "status-group:line-weights";
/// `view-chrome LineWeights on=…` — the application's own report of the toggle.
const HAIRLINE_EVENT: &str = "canvas-hairline";
const CHROME_EVENT: &str = "view-chrome";

/// The zoom the run must reach before the assertion means anything.
///
/// 250 %, inside the 200–400 % band the operator named. Below roughly 150 % a
/// default-width (1.0 unit) stroke is under two device pixels and capping it at
/// one moves too little ink to distinguish from antialiasing noise.
const MIN_ZOOM: f64 = 2.5;
/// Wheel notches per batch while climbing, and how many batches before giving
/// up.
const BATCH: usize = 3;
/// Enough batches to climb from fit to 250 % on any page this shell opens.
const MAX_BATCHES: usize = 30;
/// A pixel this dark or darker counts as ink — see
/// `pdfcer_gui::render::hairline`, which uses the same threshold for the same
/// reason.
const INK: u8 = 128;
/// How much of the ink must go.
///
/// **3 %**, where the unit measurement on `a1-titleblock.pdf` at scale 4 is
/// **17.7 %** (484,078 → 398,578 dark pixels). Deliberately far under it: most
/// of a title block's ink is text and fills, which this mode correctly does not
/// touch, so the share that *can* move depends on the document. What is being
/// asserted is the direction and the reality of the change, not a rendering
/// constant.
const MIN_INK_DROP: f64 = 0.03;

/// The operator's line-weights display mode, end to end.
pub struct LineWeightsOffThinsTheDrawingAndSaysSo;

impl Check for LineWeightsOffThinsTheDrawingAndSaysSo {
    fn name(&self) -> &'static str {
        "line_weights"
    }

    fn defect(&self) -> &'static str {
        "the control that draws every line one pixel wide is on the ribbon and does nothing — \
         which is the operator's own report about its predecessor — or it changes the picture \
         in the WRONG DIRECTION (thickening thin strokes, Acrobat's \"enhance thin lines\", the \
         opposite convention), or it changes the canvas without saying that the canvas no \
         longer shows what will print"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The live zoom, as the canvas published it.
fn zoom_now(trace: &Trace, canvas_event: &str) -> Option<f64> {
    trace
        .last(canvas_event)
        .and_then(|l| l.get("zoom"))
        .and_then(|v| v.parse().ok())
}

/// Whether the disclosure has **ever** been declared.
///
/// ★★ `ui_rect` is a **change log**, not a per-frame census — a widget that
/// stops being drawn publishes nothing. So "has it ever appeared" is the
/// question a change log can answer honestly, and "is it on screen now" is not.
/// Phase A therefore asks *never*, and phase F asks *not since the second
/// press*, which is the same question scoped to a suffix of the trace.
fn ever_declared(trace: &Trace, ui_rect: &str) -> bool {
    trace
        .events(ui_rect)
        .any(|l| l.get("name") == Some(DISCLOSURE))
}

/// The most recent `view-chrome LineWeights on=…` the application reported.
fn line_weights_state(trace: &Trace) -> Option<bool> {
    trace
        .events(CHROME_EVENT)
        .filter(|l| l.raw.contains("LineWeights"))
        .last()
        .and_then(|l| l.get("on"))
        .map(|v| v == "true")
}

/// Dark pixels inside `region` of `image`.
fn ink_in(image: &Image, region: crate::geom::PixRect) -> u64 {
    image
        .pixels_in(region)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count() as u64
}

/// **Where the densest patch of ink sits inside `canvas`**, as fractions of the
/// canvas rect, or `None` when the canvas carries no ink at all.
///
/// # ★★★ Why the climb has to be aimed, and what NOT aiming it cost
///
/// `Ctrl+wheel` zooms **about the pointer**, so whatever is under the cursor
/// stays under it for the whole climb. Aiming at the canvas's geometric centre
/// therefore does not zoom into the drawing — it zooms into whatever happens to
/// be in the middle of the sheet, and on a CAD sheet that is **blank paper**:
/// the linework is a border and a title block around the edges.
///
/// This check SKIPPED on `fixtures/a1-titleblock.pdf` for exactly that reason,
/// and its skip message named the repair without performing it — *"aim the view
/// at content before climbing"*. **A SKIP is not red**, so it had been telling
/// nobody anything for as long as it had been running.
///
/// # ★★ Why a patch and not the ink's bounding box
///
/// Measured on that fixture at scale 0.25: the ink spans **24–2356 pt
/// horizontally and 28–1660 vertically** — very nearly the whole A1 sheet — and
/// the centre of that box is the blank middle. A bounding box says where the
/// drawing *is*; it does not say where the drawing is *dense*, and those are
/// opposite answers on a sheet whose content is a frame.
///
/// # The method, and why it is deliberately coarse
///
/// Divide the canvas into a `GRID × GRID` lattice, count dark pixels per cell,
/// return the fullest cell's centre as fractions. Eight cells per axis is
/// enough to separate a title block from a border on any sheet size, and coarse
/// on purpose: a fine grid finds a single thick line and aims at a spot that
/// leaves the viewport as soon as the zoom climbs, while a coarse one finds a
/// *region* that stays populated all the way up.
///
/// ★ It answers the cell's **centre**, not the darkest pixel, for the same
/// reason — the exact pixel of a stroke is a knife edge at 400 %, and one
/// rounding in the pointer's position falls off it.
///
/// ★★ Fractions rather than a screen point, so the caller converts through
/// `Frame::declared_at` like every other aim in this harness. `coords`' rule is
/// that a coordinate is **produced by a conversion and never assembled**, and
/// returning pixels here would be assembling one two conversions away from the
/// rect it belongs to.
fn densest_ink(image: &Image, canvas: crate::geom::PixRect) -> Option<(f32, f32)> {
    /// Cells per axis. See the note above on why this is coarse.
    const GRID: u32 = 8;
    if canvas.w < GRID || canvas.h < GRID {
        return None;
    }
    let (cw, ch) = (canvas.w / GRID, canvas.h / GRID);
    let mut best: Option<(u64, u32, u32)> = None;
    for gy in 0..GRID {
        for gx in 0..GRID {
            let cell = crate::geom::PixRect {
                x: canvas.x + gx * cw,
                y: canvas.y + gy * ch,
                w: cw,
                h: ch,
            };
            let ink = ink_in(image, cell);
            if ink > 0 && best.is_none_or(|(most, _, _)| ink > most) {
                best = Some((ink, gx, gy));
            }
        }
    }
    best.map(|(_, gx, gy)| {
        #[allow(clippy::cast_precision_loss)]
        let f = |g: u32| (g as f32 + 0.5) / GRID as f32;
        (f(gx), f(gy))
    })
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Pass a drawing whose STROKES are where the zoom lands. Measured \
             2026-09-05: `D:/Dev/pdfTests/ncored-benchmark-cad-drawing.pdf` reaches 433 % \
             with 57,980 ink pixels and PASSES - 82.7 % of the ink goes. \
             `fixtures/a1-titleblock.pdf` does NOT: its title block is in one corner, the \
             zoom is anchored at the centre, and the climb lands on blank paper (136 ink \
             pixels of 286,528). That is a fact about where the ink SITS, not about the \
             fixture's quality. On a page of text this mode correctly changes nothing, \
             because only stroked paths are capped.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses a ribbon item and Ctrl+wheels \
             the canvas. Reported as SKIPPED rather than passed.",
        ));
    }
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("line_weights.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process.",
            vocab.start_event
        )));
    }

    // --- A: the disclosure must be absent before anything is pressed -------
    //
    // Not decoration. A line that is always on says nothing, and this one says
    // "the canvas is not showing what will print" — permanently true would be a
    // permanent lie on every document he opens.
    if ever_declared(&trace, ui_rect) {
        return Ok(Some(format!(
            "the `{DISCLOSURE}` disclosure is showing on a freshly opened document, before \
             anything was pressed. Line weights ship ON — `ViewState::default()` sets \
             `line_weights: true` — so the canvas IS showing what will print and saying \
             otherwise is a standing falsehood. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("no line-weights disclosure on open, which is correct");

    // --- B: climb into the band where the mode has something to cap --------
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    let canvas = declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;

    // ★★★ **AIMED AT MEASURED INK, not at the canvas's centre** — 2026-09-07.
    //
    // `Ctrl+wheel` zooms about the pointer, so this one point decides what the
    // whole climb lands on. It used to be `frame.declared_center(canvas)`, and
    // on a CAD sheet that is blank paper: the linework is a border and a title
    // block round the edges. This check reported **136 ink pixels of 286,528**
    // and SKIPPED — correctly, and invisibly, because a SKIP is not red.
    //
    // ⇒ One capture at the opening zoom, [`densest_ink`] over it, and the aim
    // is a fact about this document rather than an assumption about where
    // drawings put their content. See that function for why a *patch* and not
    // the ink's bounding box.
    let fit_shot = ctx.out("line_weights.fit.png");
    let at_fit = crate::capture::window_to_png(&session, &fit_shot)?;
    report.artifact(fit_shot);
    let canvas_px = frame.logical_to_capture_pixels(canvas);
    let at = match densest_ink(&at_fit, canvas_px) {
        Some((fx, fy)) => {
            report.note(format!(
                "aimed the zoom at the densest ink in view ({:.0} %, {:.0} % across the canvas) \
                 rather than at its centre — on a CAD sheet the centre is blank paper",
                fx * 100.0,
                fy * 100.0
            ));
            frame.declared_at(canvas, fx, fy)
        }
        None => {
            // ★ Not an error, and it must not become one: a page really can be
            // blank at the opening zoom, and the ink floor further down is the
            // assertion that owns that case and words it properly. Falling back
            // to the centre keeps this function's failure mode identical to
            // what it was before the aim existed.
            report.note(
                "no ink anywhere in the canvas at the opening zoom, so the climb is aimed at \
                 the centre as it always was — the ink floor below will report what that means",
            );
            frame.declared_center(canvas)
        }
    };

    let mut batches = 0;
    loop {
        let now = zoom_now(&session.trace()?, vocab.canvas_event).unwrap_or(0.0);
        if now >= MIN_ZOOM || batches >= MAX_BATCHES {
            break;
        }
        driver.scroll_at_held(at, &[VK_CONTROL], 1, BATCH)?;
        session.settle(6);
        batches += 1;
    }
    // The zoom and the raster are two clocks. Give the worker time to catch up
    // before any pixel is read — the same trap `blend_space` records, where a
    // check asserted against a texture from three zoom levels ago.
    session.settle(50);
    let trace = session.trace()?;
    let reached = zoom_now(&trace, vocab.canvas_event).unwrap_or(0.0);
    report.note(format!(
        "climbed to zoom {:.0}% in {batches} batch(es)",
        reached * 100.0
    ));
    if reached < MIN_ZOOM {
        return Err(Error::new(format!(
            "only reached {:.0}%, short of the {:.0}% this check needs. Below that a \
             default-width stroke is already about one device pixel and a CEILING at one \
             device pixel has nothing to cap — the two pictures would be identical and the \
             assertion would fail against a correct build. SKIPPED rather than failed: the \
             wheel may not be reaching the canvas, which is `zoom_gallery`'s report to make.",
            reached * 100.0,
            MIN_ZOOM * 100.0
        )));
    }

    // --- C: how much ink is on the page with real widths -------------------
    let frame = session.frame()?;
    let canvas = declared(&session.trace()?, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}` after the climb")))?;
    let px = frame.logical_to_capture_pixels(canvas);
    let before_path = ctx.out("line_weights.before.png");
    let before = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);
    let ink_before = ink_in(&before, px);
    report.note(format!(
        "{ink_before} ink pixels in the canvas with real widths"
    ));
    // ★★★ **A FLOOR, NOT A ZERO CHECK — raised 2026-09-05, the first time this
    // check was ever run.**
    //
    // The guard used to be `ink_before == 0`. On its first run it measured
    // **138 ink pixels in a canvas of roughly 300,000** — four hundredths of one
    // per cent — because climbing to 394 % on an A1 sheet lands the view on
    // **blank paper**: the title block is in a corner and the zoom is anchored
    // at the centre. 138 is above zero, so the guard let it through, and the
    // check then reported *"the drawing lost 100.0 % of its ink"* and
    // *"turning line weights back on did not restore the drawing"*.
    //
    // ⇒ Both sentences were true of the pixels and **false about the
    // application.** They are a statement about 138 pixels of nothing, and the
    // screenshot settles it in one look.
    //
    // ★★ This is the third instance of the same shape today, and the rule it
    // earns is: **a probe whose baseline has no dynamic range cannot produce a
    // verdict.** `mouse_work_survives_every_render_tier` failed at every rung on
    // his own drawing until it was re-aimed from blank paper to an object; the
    // right-edge assertion in `master_detail` read six antialiased pixels as
    // clipped text. Every one of them was a measurement of a region that had no
    // subject in it.
    //
    // The floor is a **proportion of what was sampled**, not an absolute, so it
    // survives a different window size and a different display scale. Half a
    // per cent is far below any drawing the operator would call one and far
    // above the stray page-edge line this run found.
    //
    // ⚠ SKIP and not FAIL, deliberately. A blank canvas says nothing about
    // whether line weights work; reporting it red would be *"the application is
    // broken for the check's own reason"*, which this project has now written
    // down three times in one day.
    const MIN_INK_SHARE: f64 = 0.005;
    let sampled = (px.w as f64) * (px.h as f64);
    #[allow(clippy::cast_precision_loss)]
    let share = if sampled > 0.0 {
        ink_before as f64 / sampled
    } else {
        0.0
    };
    if share < MIN_INK_SHARE {
        return Err(Error::new(format!(
            "the canvas carries {ink_before} ink pixel(s) of {sampled:.0} sampled \
             ({:.3} %), which is below the {:.1} % floor a measurement of this mode \
             needs. Almost certainly the zoom climb has landed on BLANK PAPER — on \
             an A1 sheet the drawn content is in one corner and the zoom is anchored \
             at the centre. Aim the view at content before climbing (the same repair \
             `mouse_work_survives_every_render_tier` needed on 2026-09-05), or pass a \
             fixture whose ink is where the zoom lands. SKIPPED, not failed: a blank \
             canvas says nothing about whether line weights work.",
            share * 100.0,
            MIN_INK_SHARE * 100.0,
        )));
    }

    // --- D: press it -------------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)?.ok_or_else(|| {
        Error::new(format!(
            "no `{ITEM}` on the View tab or in its overflow. Items declared: {}. \
             ★★★ THIS IS THE ORIGINAL DEFECT'S SHAPE: the command can be registered, handled \
             and cache-invalidated, with five unit tests green, and still be unreachable — \
             which from the operator's chair is the button doing nothing. It belongs in \
             View ▸ Display beside Rulers, Grid and Guides.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(40);

    let trace = session.trace()?;
    match line_weights_state(&trace) {
        None => {
            return Ok(Some(format!(
                "the `{ITEM}` item was clicked and the application never traced a \
                 `{CHROME_EVENT}` line naming `LineWeights`. The press reached no handler — \
                 `Action::ToggleViewChrome` traces unconditionally — so either the item's \
                 rect is not where it is drawn or the command id maps to nothing. Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(true) => {
            return Ok(Some(format!(
                "after one press the application still reports `LineWeights on=true`. Either \
                 the press landed twice or the toggle wrote the value it read. Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(false) => {
            report.note("the application reports line weights OFF");
        }
    }

    // --- E: the picture must actually get thinner, and the bar must say so --
    //
    // ★★ The SAME `px` rectangle as phase C, deliberately re-used rather than
    // re-read. Two captures compared pixel for pixel are only comparable if
    // they name the same region — and the ribbon tab click above cannot move
    // the canvas, so re-deriving it here would introduce a way for the two
    // counts to be of different areas with nothing reporting it.
    let after_path = ctx.out("line_weights.after.png");
    let after = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);
    let ink_after = ink_in(&after, px);
    report.note(format!("{ink_after} ink pixels with line weights off"));

    if ink_after >= ink_before {
        let direction = if ink_after > ink_before {
            "MORE ink than before. That is the OPPOSITE convention — Acrobat's \"enhance thin \
             lines\", which thickens sub-pixel strokes. He asked for AutoCAD's `LWDISPLAY` \
             off, which THINS fat ones, and shipping the opposite is worse than shipping \
             nothing because it looks like the feature working"
        } else {
            "exactly as much ink as before. The state changed, the trace says so, and the \
             picture did not — which is the operator's own words about this control's \
             predecessor: \"the button never worked\". Look first at whether the raster cache \
             served a stale texture (`RenderKey` must carry the stroke display) and second at \
             whether `render_on_worker` assigns `options.stroke_display`"
        };
        return Ok(Some(format!(
            "at zoom {:.0}% the canvas has {ink_after} dark pixels with line weights off \
             against {ink_before} with them on — {direction}. Captures are beside the trace at \
             {}.",
            reached * 100.0,
            session.trace_path().display()
        )));
    }

    let dropped = (ink_before - ink_after) as f64 / ink_before as f64;
    report.note(format!(
        "the drawing lost {:.1} % of its ink",
        dropped * 100.0
    ));

    // ★★★ **THE ENGINE'S OWN COUNT IS THE PRIMARY ORACLE** — 2026-09-07.
    //
    // The ink ratio below used to be the only one, and it failed a **working
    // build**: on `a1-titleblock.pdf` at 394 % the drawing lost 1.39 %, under
    // the floor, because the strokes in the aimed region were already close to
    // one device pixel and the §8.4.3.2 floor had nothing left to take. The
    // measurement was calibrated on a different drawing.
    //
    // ⇒ **Never widen a tolerance when the measurement runs out — read a better
    // instrument.** `Diagnostics::strokes_hairlined` (`Pass 254.1`) is the
    // count of strokes this raster actually THINNED, published on the
    // `canvas-hairline` line, and it separates the two states the pixel ratio
    // cannot:
    //
    // | strokes thinned | ink moved | verdict |
    // |---|---|---|
    // | **0** | — | **SKIP** — the mode reached the renderer and had nothing to cap in this view. Not a defect, and it is what the operator is told too |
    // | **> 0** | down | **PASS** — the mode did work and put less ink down |
    // | **> 0** | up or level | **FAIL** — handled above; the wrong convention, or a stale raster |
    //
    // ★ This shell asked the engine for that count precisely because *"an
    // operator could switch it on, see no change, and have no way to tell 'this
    // drawing has no strokes thin enough to matter' from 'the setting is
    // broken'"*. The harness had the identical problem and is fixed the
    // identical way.
    let thinned = trace
        .events(HAIRLINE_EVENT)
        .last()
        .and_then(|l| l.get("thinned"))
        .map(std::borrow::ToOwned::to_owned);
    match thinned.as_deref() {
        Some("0") => {
            return Err(Error::new(format!(
                "the mode is on and the engine thinned **0 strokes** in this view: \
                 `{HAIRLINE_EVENT} thinned=0`. That is not a defect — every stroke here is \
                 already at or below one device pixel, so the ceiling is a no-op and the page \
                 renders byte-identically either way. The application says the same thing to \
                 the operator (`line_weights_no_effect`).\n\
                 SKIPPED rather than passed, because a mode with nothing to do proves nothing \
                 about a mode that has something to do. Aim at heavier linework or drop the \
                 zoom, where the strokes are wider in device pixels. Trace: {}.",
                session.trace_path().display()
            )));
        }
        None => {
            report.note(format!(
                "the application published no `{HAIRLINE_EVENT}` line, so the ink ratio below \
                 is the only oracle this run has — that is how this check worked before \
                 2026-09-07 and it is why it once failed a working build"
            ));
        }
        Some(n) => {
            report.note(format!(
                "★★ the engine reports {n} stroke(s) thinned, so the mode reached the renderer \
                 and did work — which is what the ink ratio below can only infer"
            ));
        }
    }

    // ★ The ink ratio is kept as a SECOND opinion rather than deleted, and the
    // floor is only applied when the count could not be read. With the count in
    // hand the pixels answer a different and weaker question — *did the picture
    // visibly change* — and a build that thinned strokes and drew them anyway
    // would satisfy the count and fail here, which is worth catching.
    if thinned.is_none() && dropped < MIN_INK_DROP {
        return Ok(Some(format!(
            "the canvas lost only {:.2} % of its ink ({ink_before} -> {ink_after}) at zoom \
             {:.0}%, and no `{HAIRLINE_EVENT}` line was published to say whether that is \
             because nothing needed thinning. The unit measurement on a1-titleblock.pdf at \
             scale 4 is 17.7 %. Trace: {}.",
            dropped * 100.0,
            reached * 100.0,
            session.trace_path().display()
        )));
    }

    let trace = session.trace()?;
    if !ever_declared(&trace, ui_rect) {
        return Ok(Some(format!(
            "the canvas is drawing every stroke one pixel wide and the `{DISCLOSURE}` line \
             never appeared. The screen now deliberately does not show what will print, and \
             nothing says so — an operator three sheets deeper has no way to know why a plot \
             does not match his screen. `app::status::disclosure::line_weights_disclosure` is \
             the surface; check it is reached from `disclosure::all` and that the region name \
             still matches. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the status bar discloses that this is not what will print");

    // --- F: and it can be turned back off ----------------------------------
    let mark = session.trace()?.events(ui_rect).count();
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(40);
    let trace = session.trace()?;
    if line_weights_state(&trace) != Some(true) {
        return Ok(Some(format!(
            "a second press did not put line weights back on — the application reports \
             {:?}. A reading aid that cannot be left is one that follows the operator into \
             everything he does next. Trace: {}.",
            line_weights_state(&trace),
            session.trace_path().display()
        )));
    }
    let after_path = ctx.out("line_weights.restored.png");
    let restored = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);
    let ink_restored = ink_in(&restored, px);
    report.note(format!("{ink_restored} ink pixels once turned back on"));
    if ink_restored <= ink_after {
        return Ok(Some(format!(
            "turning line weights back on did not restore the drawing: {ink_after} dark pixels \
             with the mode on, {ink_restored} after switching it off again, against \
             {ink_before} originally. The mode is one-way, which means the raster cache is not \
             keyed on it in both directions. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // ★ The disclosure must have STOPPED. A change log cannot say "absent now",
    // so the question is scoped: nothing published it after the second press.
    let after_second = session
        .trace()?
        .events(ui_rect)
        .skip(mark)
        .any(|l| l.get("name") == Some(DISCLOSURE));
    if after_second {
        return Ok(Some(format!(
            "line weights are back on and the `{DISCLOSURE}` line is still being published. \
             The canvas shows what will print again, so the disclosure is now a falsehood — \
             and one that will sit there for the rest of the session. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the disclosure retired itself when the mode was switched off");

    Ok(None)
}
