//! `every_theme_preset_keeps_the_page_white` — the harness's two theme blind
//! spots, closed in one check.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/theme_page.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, delta, fill_of, frame_of, list,
};
use crate::checks::settings_theme::{DIALOG, THEME_PREFIX, open_the_theme_picker};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Rgb;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// **Every preset the shell ships, in the order the picker draws them.**
const PRESETS: [&str; 3] = ["quiet", "airy", "dark"];

/// The canvas's page raster — `canvas::trace::REGION_PAGE`.
const PAGE: &str = "page";

/// The scrollable viewport the sheet sits inside — `REGION_CANVAS_VIEWPORT`.
const CANVAS_VIEWPORT: &str = "canvas-viewport";

/// How far apart two presets' canvas surrounds must measure before the two are
/// **different presets** rather than one preset measured twice.
const MIN_PRESET_DISTINCTION: u16 = 4;

/// How far a layout edge must move before it counts as a **different set of
/// metrics**.
const MIN_METRIC_SHIFT_PTS: f32 = 2.0;

/// The lowest channel a **white sheet** may measure.
const PAGE_MIN_CHANNEL: u8 = 235;

/// The widest a white sheet's channels may spread before it is **tinted**.
const PAGE_MAX_SPREAD: u8 = 8;

/// How far the sheet may move between one preset and the next.
const MAX_PAGE_DRIFT: u16 = 6;

/// The thinnest strip of canvas surround worth sampling, in logical points.
const MIN_BAND_PTS: f32 = 16.0;

// ===========================================================================
// `every_theme_preset_keeps_the_page_white`
// ===========================================================================

/// **The sheet stays white, under every preset the shell ships.**
pub struct EveryThemePresetKeepsThePageWhite;

impl Check for EveryThemePresetKeepsThePageWhite {
    fn name(&self) -> &'static str {
        "every_theme_preset_keeps_the_page_white"
    }

    fn defect(&self) -> &'static str {
        "a theme tints the SHEET — a CAD drawing on grey paper loses the contrast its linework \
         was drawn with, and nothing in the suite has ever sampled the page under a theme, nor \
         driven the Airy preset at all"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match page_stays_white(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// What one preset measured.
#[derive(Clone, Copy, Debug)]
struct Reading {
    /// The preset's settings-file token, as clicked.
    preset: &'static str,
    /// The dominant colour of the page raster — the paper.
    page: Rgb,
    /// The dominant colour of the canvas surround beside it.
    backdrop: Rgb,
    /// The canvas viewport's top edge, in logical points — the **second**
    /// witness that a preset installed. See [`MIN_METRIC_SHIFT_PTS`]: Airy is
    /// roomier, so the ribbon above the canvas grows and this moves down with
    /// it, which distinguishes the one pair whose colours are close.
    viewport_top: f32,
    /// What share of the page region agreed on [`Self::page`]. Reported so a
    /// reader can tell *"white paper with linework on it"* (0.8–0.99) from *"a
    /// region that is mostly something else"* (0.4), which is what a mis-aimed
    /// rect looks like — and which is otherwise indistinguishable from a
    /// verdict.
    page_share: f64,
}

/// **The strip of canvas surround beside the sheet**, or `None` when there is
/// none worth sampling.
fn backdrop_band(viewport: LRect, page: LRect) -> Option<LRect> {
    // (rect, thickness) for the strip on each side of the sheet.
    let candidates = [
        (
            LRect::new(
                Pt::new(page.min.x, viewport.min.y),
                Pt::new(page.max.x, page.min.y),
            ),
            page.min.y - viewport.min.y,
        ),
        (
            LRect::new(
                Pt::new(page.min.x, page.max.y),
                Pt::new(page.max.x, viewport.max.y),
            ),
            viewport.max.y - page.max.y,
        ),
        (
            LRect::new(
                Pt::new(viewport.min.x, page.min.y),
                Pt::new(page.min.x, page.max.y),
            ),
            page.min.x - viewport.min.x,
        ),
        (
            LRect::new(
                Pt::new(page.max.x, page.min.y),
                Pt::new(viewport.max.x, page.max.y),
            ),
            viewport.max.x - page.max.x,
        ),
    ];
    // The FIRST strict maximum, not the last, and the candidate order above
    // is therefore load-bearing: above, below, left, right. A fit-page view in
    // a maximised window leaves the two flanks EXACTLY equal — 585.5 points
    // each, measured — and `Iterator::max_by` would hand back the later one.
    // The left flank is the better tie-break because a vertical scrollbar, when
    // there is one, is on the right; the 20 % inset below already clears it,
    // and a deterministic choice is worth more than a second line of defence
    // that only matters when the first has failed.
    let mut best: Option<(LRect, f32)> = None;
    for (rect, thickness) in candidates {
        if best.is_none_or(|(_, t)| thickness > t) {
            best = Some((rect, thickness));
        }
    }
    let (band, thickness) = best.expect("the candidate array is not empty"); // ui-text-exempt: panic message, never displayed
    if thickness < MIN_BAND_PTS || band.width() < MIN_BAND_PTS || band.height() < MIN_BAND_PTS {
        return None;
    }
    let dx = band.width() * 0.2;
    let dy = band.height() * 0.2;
    Some(LRect::new(
        Pt::new(band.min.x + dx, band.min.y + dy),
        Pt::new(band.max.x - dx, band.max.y - dy),
    ))
}

/// Is this colour white paper?
fn is_white_paper(c: Rgb) -> bool {
    let lo = c.r.min(c.g).min(c.b);
    let hi = c.r.max(c.g).max(c.b);
    lo >= PAGE_MIN_CHANNEL && hi - lo <= PAGE_MAX_SPREAD
}

/// The body of [`EveryThemePresetKeepsThePageWhite`].
#[allow(
    clippy::too_many_lines,
    reason = "one linear scripted sequence; splitting it would hide the order the steps must happen in"
)] // ui-text-exempt: lint justification, never displayed
fn page_stays_white(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // A DOCUMENT IS THE SUBJECT HERE, unlike its sibling. That check launches
    // with nothing open on purpose, because `file.settings` is
    // application-scoped; this one is about the SHEET, and there is no sheet
    // without a document. The two live in one file and disagree about the
    // fixture for a reason each states.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check measures the colour of a rendered page, so it needs a document \
             whose first page is white paper. SKIPPED rather than passed — there is no page to \
             measure and therefore nothing has been learned.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is six clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("theme_page.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());

    // Maximised for the same reason its sibling is — `file.settings` is in
    // the File tab's LAST group and lives in the ribbon's overflow at the
    // window's opening width, where a control publishes no rect. It also gives
    // the fit-page view a generous margin, which is the backdrop this check
    // samples.
    session.maximize();
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // THE SHEET MUST BE ON SCREEN BEFORE THE WINDOW THAT WILL COVER IT IS
    // OPENED. A document that failed to render publishes `canvas-message`
    // instead of `page`, and every reading below would then be taken off an
    // explanatory sentence on a grey field — a confident colour about the wrong
    // surface, which this project has filed six times.
    if declared(&trace, ui_rect, PAGE).is_none() {
        return Err(Error::new(format!(
            "the canvas declared no `{PAGE}` region, so nothing rendered and there is no sheet \
             to measure. Regions the canvas did declare: {}. If `canvas-message` is among them \
             the document did not open — check the fixture at {}. SKIPPED.",
            list(&declared_names(&trace, ui_rect, "canvas")),
            pdf.display()
        )));
    }

    if let Some(failure) = open_the_theme_picker(&session, &driver, ui_rect, &trace)? {
        return Ok(Some(failure));
    }

    // --- the three presets, each measured on its own capture ----------------
    let mut readings: Vec<Reading> = Vec::new();
    for preset in PRESETS {
        match measure_preset(ctx, &session, &driver, ui_rect, preset, report)? {
            Ok(reading) => readings.push(reading),
            Err(failure) => return Ok(Some(failure)),
        }
    }

    for r in &readings {
        report.note(format!(
            "{}: sheet {:?} (share {:.2}), surround {:?}, canvas top {:.1}",
            r.preset, r.page, r.page_share, r.backdrop, r.viewport_top
        ));
    }

    // --- 1. the run is not vacuous ------------------------------------------
    //
    // ASKED FIRST, AND IT IS THE WHOLE HONESTY OF THIS CHECK. "The page
    // stayed white" is true of a build in which nothing happened at all, so the
    // surround has to be shown to have moved before a page reading means
    // anything. Every pair, not just one: three presets that all measured the
    // same are three clicks none of which installed a palette.
    for (i, a) in readings.iter().enumerate() {
        for b in readings.iter().skip(i + 1) {
            let moved = delta(a.backdrop, b.backdrop);
            let shifted = (a.viewport_top - b.viewport_top).abs();
            // EITHER witness suffices, and neither is redundant: `quiet` and
            // `dark` share their metrics exactly and are separated by 206
            // levels of colour, while `quiet` and `airy` are 7 levels apart and
            // separated by 31.7 points of layout. One preset measured twice
            // moves by 0 on both.
            if moved < MIN_PRESET_DISTINCTION && shifted < MIN_METRIC_SHIFT_PTS {
                return Err(Error::new(format!(
                    "`{}` and `{}` PAINTED AND LAID OUT THE SAME — surround {:?} against {:?} (a \
                     distance of {moved}, floor {MIN_PRESET_DISTINCTION}) and canvas top {:.1} \
                     against {:.1} (a shift of {shifted:.1}, floor {MIN_METRIC_SHIFT_PTS}). \
                     Either the click did not land or the preset was never installed. Nothing is \
                     claimed about the page: a sheet that stayed white while the theme stayed \
                     put has proved nothing. SKIPPED, and the diagnosis lives in \
                     `settings_theme_takes_effect`, which measures exactly this and says which \
                     of the two it is.",
                    a.preset, b.preset, a.backdrop, b.backdrop, a.viewport_top, b.viewport_top
                )));
            }
        }
    }

    // --- 2. the fixture is white paper --------------------------------------
    //
    // Read off the FIRST preset, which is `quiet` — a light theme, which
    // cannot be the thing that darkened a sheet. So a non-white reading here is
    // a fact about the document, not about the build, and the honest outcome is
    // a SKIP naming the file. Asserting it as a failure would file a defect
    // against pdfcer for a PDF whose author filled the page grey.
    let Some(first) = readings.first() else {
        return Err(Error::new(
            "no preset was measured, so there is nothing to compare. SKIPPED.",
        ));
    };
    if !is_white_paper(first.page) {
        return Err(Error::new(format!(
            "under the `{}` preset — a LIGHT theme, which cannot have darkened anything — the \
             first page of {} measured {:?}, which is not white paper (floor {PAGE_MIN_CHANNEL} \
             per channel, spread {PAGE_MAX_SPREAD}). That is a property of the fixture, so this \
             check cannot measure its invariant on it and SKIPS rather than filing a defect \
             against the build. Pass a --pdf whose first page is white.",
            first.preset,
            pdf.display(),
            first.page
        )));
    }

    // --- 3. the verdict -----------------------------------------------------
    for r in &readings {
        if !is_white_paper(r.page) {
            return Ok(Some(format!(
                "★★★ THE `{}` PRESET TINTS THE SHEET. The page measured {:?} under it and {:?} \
                 under `{}` — while the canvas surround beside it went {:?} → {:?}, so the theme \
                 demonstrably reached the pixel next to the page and then went one region too \
                 far. \
                 \
                 A tinted sheet is an unreadable drawing: the linework is the whole content and \
                 the paper is what the eye measures its contrast against. `egui_shell::theme` \
                 states the rule it is breaking — `Preset::Dark` is *\"dark chrome against light \
                 content, as CAD tools do it\"*, and its label roles stay light-plated \
                 *\"because they sit over CONTENT, whose colour the document decides and the \
                 theme does not.\"* The fix is in the theme or in whatever tints the raster, \
                 never in this check's floor.",
                r.preset, r.page, first.page, first.preset, first.backdrop, r.backdrop
            )));
        }
        let moved = delta(r.page, first.page);
        if moved > MAX_PAGE_DRIFT {
            return Ok(Some(format!(
                "★★ THE SHEET MOVED WITH THE THEME. It measured {:?} under `{}` and {:?} under \
                 `{}` — {moved} apart, against a tolerance of {MAX_PAGE_DRIFT}. Both readings \
                 are still light enough to look like paper, which is what makes this the \
                 dangerous shape of the defect: it will not be reported, it will be lived with. \
                 The page raster is rendered by `pdfcer-core` from the document's own content \
                 and the preset is not an input to it, so the expected distance is ZERO.",
                first.page, first.preset, r.page, r.preset
            )));
        }
    }

    Ok(None)
}

/// **Click one preset's radio and measure the two surfaces**, from one capture
/// of the application's own window.
fn measure_preset(
    ctx: &CheckContext,
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    preset: &'static str,
    report: &mut CheckReport,
) -> Result<std::result::Result<Reading, String>> {
    let region = format!("{THEME_PREFIX}{preset}");
    let trace = session.trace()?;
    let Some(radio) = declared(&trace, ui_rect, &region) else {
        // A FAILURE, not a skip, and this is the row that closes the Airy
        // hole. The window is open and publishing; a preset the shell ships and
        // the picker does not offer is a preset an operator cannot choose.
        return Ok(Err(format!(
            "the Settings window is open and publishes its regions, but there is no `{region}` \
             — the picker does not offer the `{preset}` preset. Regions declared under the theme \
             namespace: {}. `egui_shell::theme::Preset::ALL` ships three, and a preset an \
             operator cannot select is a preset that ships unverified.",
            list(&declared_names(&trace, ui_rect, THEME_PREFIX))
        )));
    };
    let dialog_frame = frame_of(session, &trace, ui_rect, DIALOG)?;
    driver.click_at(dialog_frame.declared_center(radio))?;
    // Generous, and for the reason its sibling states: the theme is installed
    // at the TOP of the next frame and `Theme::apply` rewrites both of egui's
    // styles. Airy additionally re-lays the whole shell out, and the canvas
    // re-rasterises the page at the new metrics — so this settle covers a
    // re-render, not just a repaint.
    session.settle(30);

    // THE APPLICATION'S OWN WINDOW, and `frame_to_png` raises it — which
    // puts the Settings dialog behind it, exactly as intended. A screen grab
    // reads the COMPOSITED desktop, so a capture taken with the dialog in front
    // would sample the dialog's panel through the page's rectangle and report a
    // confident colour about the wrong surface. The next iteration's click
    // brings the dialog back by itself: `Driver::click_at` raises the smallest
    // window of the process containing the point, which is the dialog whatever
    // the z-order is.
    let app_frame = session.frame()?;
    let path = ctx.out(&format!("theme_page.{preset}.png"));
    let image = crate::capture::frame_to_png(session, &app_frame, &path)?;
    report.artifact(path);

    let trace = session.trace()?;
    let Some(page) = declared(&trace, ui_rect, PAGE) else {
        return Err(Error::new(format!(
            "the canvas stopped declaring `{PAGE}` after `{preset}` was chosen, so there is no \
             sheet to sample. SKIPPED."
        )));
    };
    let Some(viewport) = declared(&trace, ui_rect, CANVAS_VIEWPORT) else {
        return Err(Error::new(format!(
            "the canvas declared `{PAGE}` and no `{CANVAS_VIEWPORT}`, so the surround beside the \
             sheet cannot be located and the run would have no witness that `{preset}` installed \
             anything. SKIPPED."
        )));
    };
    let Some(band) = backdrop_band(viewport, page) else {
        return Err(Error::new(format!(
            "the sheet fills its viewport under `{preset}` — page {page:?} in viewport \
             {viewport:?} leaves no strip of surround {MIN_BAND_PTS} points thick — so there is \
             nowhere to read the backdrop, and without it a white page proves nothing. SKIPPED. \
             Run with the window maximised and the view at fit-page."
        )));
    };

    let report_at = crate::pixels::contrast_at(&image, app_frame.logical_to_capture_pixels(page));
    if report_at.sampled == 0 {
        return Err(Error::new(format!(
            "the page region {page:?} did not map onto the application's capture under \
             `{preset}`. A harness coordinate failure, not a verdict on the build."
        )));
    }
    let Some(backdrop) = fill_of(&image, &app_frame, band) else {
        return Err(Error::new(format!(
            "the surround band {band:?} did not map onto the application's capture under \
             `{preset}`. A harness coordinate failure, not a verdict on the build."
        )));
    };

    Ok(Ok(Reading {
        preset,
        page: report_at.background,
        backdrop,
        page_share: report_at.background_share,
        viewport_top: viewport.min.y,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Corners for a rectangle, so the cases below read as geometry.
    fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> LRect {
        LRect::new(Pt::new(x0, y0), Pt::new(x1, y1))
    }

    /// **The widest side wins, and this test is the bug it was written
    /// after.**
    ///
    #[test]
    fn the_thickest_side_is_the_one_sampled_even_when_it_is_a_flank() {
        let viewport = r(288.0, 143.3, 3112.0, 1327.0);
        let page = r(873.5, 153.7, 2526.5, 1321.4);
        let band = backdrop_band(viewport, page).expect("585 points of flank is a band");
        // Left flank: x from the viewport's left edge to the sheet's, inset a
        // fifth at each end. It must be beside the sheet, not on it.
        assert!(
            band.max.x <= page.min.x,
            "the LEFT flank is the tie-break; it must not overlap the sheet {page:?}: {band:?}"
        );
        assert!(
            band.width() > MIN_BAND_PTS,
            "the band must be thicker than the floor: {band:?}"
        );
    }

    /// A sheet zoomed to fill its viewport leaves no surround, and the honest
    /// answer is `None` — which the caller turns into a SKIP. Returning the
    /// sheet's own drop shadow instead would produce a number, and a number is
    /// what a caller cannot tell from a measurement.
    #[test]
    fn a_sheet_that_fills_its_viewport_has_no_band() {
        let viewport = r(288.0, 143.3, 1000.0, 800.0);
        let page = r(290.0, 145.0, 998.0, 798.0);
        assert!(backdrop_band(viewport, page).is_none());
    }

    /// Both halves of the paper test, and they catch different defects: the
    /// floor catches a sheet that was darkened, the spread catches one that was
    /// tinted at full brightness. A theme built around a blue accent does the
    /// second, and a brightness-only oracle would pass it.
    #[test]
    fn white_paper_is_bright_and_neutral_and_a_tint_is_neither() {
        assert!(is_white_paper(Rgb::new(255, 255, 255)));
        assert!(
            is_white_paper(Rgb::new(249, 249, 249)),
            "the measured sheet"
        );
        assert!(
            !is_white_paper(Rgb::new(228, 230, 236)),
            "darkened AND tinted"
        );
        assert!(
            !is_white_paper(Rgb::new(200, 200, 200)),
            "neutral but darkened"
        );
        assert!(
            !is_white_paper(Rgb::new(240, 245, 255)),
            "bright but tinted blue"
        );
    }
}
