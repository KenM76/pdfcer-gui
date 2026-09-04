//! `every_theme_preset_keeps_the_page_white` — the harness's two theme blind
//! spots, closed in one check.
//!
//! # Where this came from
//!
//! `REVIEW_TRIAGE.md` row **PartC**, from the outside review of 2026-09-03:
//! *"Two harness gaps: the **Airy** preset is never driven, and no check samples
//! the **page canvas** under a theme — only the window body."*
//!
//! Both are about the same three radio buttons that
//! [`super::settings_theme`] already drives, which is why this began life
//! inside that file and moved out of it: the two checks together crossed rule
//! R2's 1,500-line ceiling, and the seam R2 forced is a real one. That module
//! asks *does choosing a theme change the program*; this one asks *and what did
//! it change that it had no business changing*. They share three clicks —
//! [`super::settings_theme::open_the_theme_picker`] — and nothing else.
//!
//! ## The Airy preset was never driven — and it is the worst one
//!
//! [`super::settings_theme::SettingsThemeTakesEffect`] clicks exactly one radio, `Dark`, because Dark
//! is the preset whose effect is unmistakable. `Quiet` is the default and gets
//! measured by being the *before* picture. **`Airy` was clicked by nothing in
//! this repository.**
//!
//! That is not a tidy gap. Airy is the preset most likely to fail a contrast
//! assertion, measured: on the two defects found by the same review — the
//! selected dock tab and the document tab's close ✕ — the luminance gaps under
//! Airy were **28.2 and 5.0**, against 45 and 18 under the presets that *were*
//! driven. Airy's panel is pure white (`#FFFFFF`) and the 27 % selection wash
//! barely darkens it, so white-on-white is five levels of luminance away. ⇒
//! **The preset most likely to fail is the one nothing drove.**
//!
//! [`EveryThemePresetKeepsThePageWhite`] drives all three.
//!
//! ## Nothing sampled the PAGE under a theme — only the window body
//!
//! Every theme oracle in this project, this file's original check included,
//! measures **chrome**: a dialog body, a rendered widget pair, a palette.
//! Nothing had ever asked what the theme did to the **sheet**.
//!
//! ★★★ **That is the single invariant a dark theme in this product must hold.**
//! pdfcer draws CAD drawings. A dark chrome is a comfort; a *tinted sheet* is an
//! unreadable drawing, because the linework's contrast is the whole content and
//! the paper is the reference the eye reads it against. `egui_shell::theme`
//! knows this and says so — `Preset::Dark`'s own doc comment is *"Dark chrome
//! against light content, as CAD tools do it"*, and its `label_backdrop` and
//! `label_text` deliberately stay dark-on-light *"because they sit over
//! CONTENT, whose colour the document decides and the theme does not."*
//!
//! A stated intention held by nothing but a comment is exactly the shape of
//! defect this suite exists for. [`EveryThemePresetKeepsThePageWhite`] measures
//! the page raster itself, under each of the three presets, and asserts it does
//! not move.
//!
//! # Everything else about how this measures
//!
//! is on [`EveryThemePresetKeepsThePageWhite`] itself, which carries the oracle
//! argument, the vacuity table and the two witnesses. The constants each carry
//! their own derivation, including the one that was **wrong on the first live
//! run and corrected against the pixels** — see [`MIN_PRESET_DISTINCTION`],
//! which is also where a finding about `Palette::content_backdrop` is recorded
//! for somebody else to act on.

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
///
/// ★ This is `egui_shell::theme::Preset::ALL` restated as strings, and the
/// restatement is the point rather than a duplication to be apologised for:
/// `ui-verify` does not compile against `egui-shell`, so the *only* way this
/// crate can know the set is to write it down — and writing it down is what
/// makes an omission visible. Until 2026-09-04 the suite named exactly one of
/// these three.
///
/// A preset added to the shell and not added here ships undriven, which is the
/// condition `Preset::ALL`'s own comment warns about in the other direction:
/// *"A preset missing here ships unverified."*
const PRESETS: [&str; 3] = ["quiet", "airy", "dark"];

/// The canvas's page raster — `canvas::trace::REGION_PAGE`.
///
/// ★ **The sheet itself, not the area around it.** Its sibling
/// [`CANVAS_VIEWPORT`] is the scroll area the sheet sits in, and that module's
/// own comment says why confusing the two is a real error rather than a
/// pedantic one: at fit-page the two rects differ by the centring margin, and a
/// check that measured one while meaning the other *"would sample the grey
/// surround"* — which is the backdrop this file deliberately samples on
/// purpose, elsewhere, for the opposite reason.
const PAGE: &str = "page";

/// The scrollable viewport the sheet sits inside — `REGION_CANVAS_VIEWPORT`.
///
/// Needed for the **backdrop band**: the strip of canvas surround between the
/// viewport's edge and the sheet's. It is the one surface that proves a preset
/// took effect *in the same capture that measures the page*, and it is measured
/// rather than assumed — see [`MIN_PRESET_DISTINCTION`] for what it turned out
/// to be painted with, which is not the role named for the job. See
/// [`backdrop_band`].
const CANVAS_VIEWPORT: &str = "canvas-viewport";

/// How far apart two presets' canvas surrounds must measure before the two are
/// **different presets** rather than one preset measured twice.
///
/// # ★★ The derivation, and the wrong one it replaced
///
/// This constant was first written as 10, derived from `Palette::content_backdrop`
/// — the role whose own doc comment says it exists for exactly this surface:
/// *"the area behind the application's main content — deliberately its own role
/// rather than reusing `surface`, because the content must read as an object ON
/// something, and a backdrop equal to the panel makes its edge disappear."*
/// Quiet `#6E7074`, Airy `#8A8D93`, Dark `#16171A`: a closest pair of 28.
///
/// **The first live run measured 242, 249 and 36.** Those are `Palette::surface`
/// — `#F2F2F3`, `#FAFAFB`, `#24262A` — so the canvas surround is *not* painted
/// with `content_backdrop`, and a grep confirms nothing in either crate reads
/// that role at all. The number was corrected to what the application actually
/// paints; the finding was reported rather than absorbed, because it is the
/// condition that role was written to prevent and it is not this crate's to fix.
///
/// | preset | measured surround | palette role | distance from `quiet` |
/// |---|---|---|---|
/// | `quiet` | 242, 242, 243 | `surface` `#F2F2F3` | — |
/// | `airy`  | 249, 249, 250 | `surface` `#FAFAFB` | **7** |
/// | `dark`  | 36, 38, 42    | `surface` `#24262A` | **206** |
///
/// **4** sits below the closest real pair and above the reading a run in which
/// nothing happened produces — which is **0**, not a small number, because two
/// identically painted regions in a lossless capture are identical.
///
/// ★ Seven is a thinner margin than this file would choose, which is why colour
/// is not the only witness: [`MIN_METRIC_SHIFT_PTS`] gives the `quiet`↔`airy`
/// pair a second, independent one, and the two are combined with an `or`.
const MIN_PRESET_DISTINCTION: u16 = 4;

/// How far a layout edge must move before it counts as a **different set of
/// metrics**.
///
/// ★ The second witness, and it is the one that makes the Airy assertion solid.
///
/// Airy is the only preset that changes `Metrics` as well as `Palette` —
/// `control_height` 24 → 28, `gutter` 4 → 8, `panel_padding` 6 → 12,
/// `corner_radius` 3 → 6 — so the ribbon above the canvas gets taller and the
/// canvas viewport's top edge moves down with it. Measured on the first live
/// run: **143.3 → 175.0 logical points**, a shift of 31.7.
///
/// Dark inherits `quiet.metrics` verbatim (`metrics: quiet.metrics` in
/// `Theme::dark`), so that pair shifts by exactly 0.0 and is distinguished by
/// colour instead. Between them the two witnesses cover all three pairs with a
/// wide margin each, which no single one of them does.
///
/// ★★ And the first live run gave this constant a second job nobody planned.
/// Under Airy the canvas surround measured **249, 249, 250** and the sheet
/// measured **249, 249, 249** — the paper and the surface it sits on are the
/// same colour to within one level, which is the very outcome
/// `Palette::content_backdrop` was declared to prevent (*"a backdrop equal to
/// the panel makes its edge disappear"*). A colour witness cannot tell a
/// correctly-aimed backdrop sample from a mis-aimed one that landed on the
/// sheet, under that preset, at all. The layout witness can, and does.
///
/// **2.0** points: a layout that did not change reports a difference of exactly
/// zero — these are floats straight from the trace, not measurements — so the
/// floor only has to sit above float formatting, and it sits an order of
/// magnitude below the 31.7 it is looking for.
const MIN_METRIC_SHIFT_PTS: f32 = 2.0;

/// The lowest channel a **white sheet** may measure.
///
/// A page raster with white paper measures 255 on every channel; a JPEG-ish
/// off-white scan or a page whose producer filled it with `0.98 g` measures in
/// the high 240s. **235** admits both and excludes anything a *theme* could
/// plausibly do — the palette's lightest chrome surface (`airy`'s `#FFFFFF`
/// panel) is white, and its darkest (`dark`'s `#16171A`) is 22, so there is no
/// near-miss to worry about.
///
/// ★ This is the ABSOLUTE half of the page assertion and it is the weaker half.
/// The one that carries the argument is [`MAX_PAGE_DRIFT`]: *the paper is the
/// document's colour and the theme has no vote on it*, which is a claim about
/// **movement** and needs no opinion about what colour the fixture's paper is.
const PAGE_MIN_CHANNEL: u8 = 235;

/// The widest a white sheet's channels may spread before it is **tinted**.
///
/// A neutral white has equal channels. A theme that tinted the sheet would
/// almost certainly tint it toward its own hue rather than merely darken it —
/// every preset in this shell is built around a blue accent — so an unequal
/// R/G/B on the paper is the specific fingerprint of the defect. 8 is one
/// quantisation bucket, i.e. the sampler's own noise floor.
const PAGE_MAX_SPREAD: u8 = 8;

/// How far the sheet may move between one preset and the next.
///
/// **Zero is the expected reading**, for the reason [`MAX_REVERT_DRIFT`] gives:
/// the page raster is rendered by `pdfcer-core` from the document's own content
/// and the preset is not an input to it, so the same pixels are painted. 6 is
/// one quantisation bucket, and it is far below any tint worth the name — the
/// mock-up that prompted this check keeps its page at `#FFFFFF` while its
/// chrome goes to `#16171A`, a distance of 233.
const MAX_PAGE_DRIFT: u16 = 6;

/// The thinnest strip of canvas surround worth sampling, in logical points.
///
/// Below this the band is mostly the sheet's own drop shadow and antialiased
/// edge rather than the backdrop, and the dominant colour it reports would be a
/// blend of the two — a measurement that moves for reasons that have nothing to
/// do with the preset. 16 points is comfortably more than any edge treatment in
/// this shell and comfortably less than the ≈ 140-point margin a fit-page view
/// leaves above an A-size sheet in a maximised window.
///
/// A run with no band this thick **SKIPS**: see [`backdrop_band`].
const MIN_BAND_PTS: f32 = 16.0;

// ===========================================================================
// `every_theme_preset_keeps_the_page_white` — `REVIEW_TRIAGE.md` PartC
// ===========================================================================

/// ★★★ **The sheet stays white, under every preset the shell ships.**
///
/// # The invariant, and why it is the one that matters
///
/// pdfcer draws CAD drawings. A dark chrome is a preference; a **tinted sheet
/// is an unreadable drawing**, because the linework carries all of the content
/// and the paper is the reference the eye measures it against. Grey a drawing
/// sheet by fifteen levels and every hairline on it loses the contrast it was
/// drawn with — and unlike a chrome regression nobody files it as a bug,
/// because a drawing that is merely *hard* to read still looks like a drawing.
///
/// The shell already believes this. `Preset::Dark`'s own doc comment is *"Dark
/// chrome against light content, as CAD tools do it"*, and it keeps
/// `label_backdrop` and `label_text` light-plated with a stated reason —
/// *"because they sit over CONTENT, whose colour the document decides and the
/// theme does not."* The outside review of the Board mock-up on 2026-09-04 made
/// the same observation from the other side: *"the dark board keeps the PAGE
/// WHITE. That is the single invariant a dark theme in this product must hold…
/// It is also, precisely, the check the test review said nobody has written."*
///
/// So: a belief stated in three places and enforced in none. That is the exact
/// shape of every defect this suite exists for — `REVIEW_TRIAGE.md` §7's own
/// summary of the review is *"a rule cited in a comment near the code, and not
/// enforced by a mechanism inside it."*
///
/// # What it measures, and why two things rather than one
///
/// Per preset, from **one capture of the application's own window**:
///
/// | sample | region | the claim |
/// |---|---|---|
/// | the sheet | `page` | it did not move, and it is white |
/// | the surround | a band of [`CANVAS_VIEWPORT`] outside `page` | it DID move |
///
/// ★★ The second is not decoration; it is what stops the first being vacuous.
/// *"The page stayed white"* is trivially true of a build in which the click
/// never landed, the radio does nothing, the theme is not installed, or the
/// window never opened. A check asserting only the page would pass on all four
/// and report a property it had never exercised. So each preset must be shown
/// to have **changed the surround** before its page reading is admitted as
/// evidence, and a run where it did not is a SKIP naming
/// [`SettingsThemeTakesEffect`] as the place that diagnosis lives.
///
/// ★ And the surround is the right witness rather than a convenient one: it is
/// the pixel **immediately adjacent to the sheet**, in the same capture. A theme
/// that reached the page would have had to reach it through there. Sampling the
/// dialog instead would prove the theme changed *somewhere*; this proves it
/// changed at the page's own edge and the page did not follow.
///
/// ★★ Measuring it also found something nobody had looked for.
/// `Palette::content_backdrop` exists precisely for this surface and says so —
/// *"deliberately its own role rather than reusing `surface`, because the
/// content must read as an object ON something, and a backdrop equal to the
/// panel makes its edge disappear"* — and the surround measures `surface` under
/// all three presets. **Nothing in either crate reads `content_backdrop` at
/// all.** So the sheet sits on the same colour as the panels and its edge is
/// exactly as invisible as that comment predicts. This crate reports that and
/// does not assert on it: it is a finding about the theme, not about the
/// invariant under test, and the fix is a call site in another crate.
///
/// # ★★ It drives all three presets, and Airy is the point
///
/// [`SettingsThemeTakesEffect`] clicks Dark and nothing else. **Airy had never
/// been clicked by anything in this repository** — and it is the preset most
/// likely to be wrong, measured: the two contrast defects the 2026-09-04 review
/// found had luminance gaps of 28.2 and 5.0 under Airy against 45 and 18 under
/// the presets that were driven, because Airy's panel is pure white and a 27 %
/// wash barely darkens it. The preset nothing drove is the preset most likely
/// to fail.
///
/// Each preset must also measure **distinct from the others**
/// ([`MIN_PRESET_DISTINCTION`]), which is a real assertion in its own right: a
/// radio that is drawn, publishes a rect, accepts a click and selects a preset
/// whose palette is never installed is `DEFECTS.md` D10 confined to one preset,
/// and Dark-only coverage structurally cannot see it.
///
/// # What would make this vacuous, and what is done about each
///
/// | vacuity | guard |
/// |---|---|
/// | the fixture's page is not white paper | measured under the light preset FIRST and SKIPPED, naming the file |
/// | the clicks never landed | the surround must move per preset, else SKIP |
/// | the sheet fills the viewport, so there is no surround | [`backdrop_band`] returns `None` and the check SKIPS |
/// | a capture of the wrong window | the page is read from the APPLICATION's frame, re-raised per preset |
/// | a stale rect after Airy re-flows the layout | every rect is re-read from the trace after every click |
///
/// That last row is not hypothetical. Airy is the one preset that changes
/// **metrics** as well as colours — `control_height` 24 → 28, `panel_padding`
/// 6 → 12, `gutter` 4 → 8 — so the ribbon grows, the docks resize and the canvas
/// moves. A check that computed the page rect once and reused it would, under
/// Airy and only under Airy, sample a rectangle the sheet had since slid out
/// of. It would report a tinted page, confidently, about a build that is fine —
/// the exact failure this project's own rule warns of: **ask what a failing
/// pixel check SAMPLED before asking what is broken.** Six false defect reports
/// were filed here from one wrong page index.
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
///
/// # Why a band rather than the viewport
///
/// Because the viewport CONTAINS the page, so its dominant colour at any
/// ordinary zoom is the paper — the very thing this reading must be independent
/// of. The band is the part of the viewport the sheet is not on, which is the
/// surface the shell paints its canvas surround with.
///
/// # The geometry: all FOUR sides, and the thickest wins
///
/// The first draft of this function looked above and below the sheet only, and
/// its first live run SKIPPED — correctly, and usefully. A **maximised** window
/// on a landscape CAD sheet fits the page to the window's HEIGHT, so the strips
/// above and below it were 10.4 and 5.6 points of drop shadow, while the strips
/// to its left and right were **585 points each**. A check that only knew about
/// two of the four sides would have skipped on every run in the layout it was
/// written for, and a skip nobody reads is a check that has quietly stopped
/// running.
///
/// So all four candidates are considered and the **thickest** is taken —
/// thickness being the dimension across the strip, which is the one that decides
/// whether the sample is backdrop or edge treatment.
///
/// Each candidate is clipped along its long axis to the sheet's own extent
/// rather than the viewport's: that keeps the sample clear of the viewport's
/// corners, where two strips meet, and it means a strip never contains the
/// ruler, the corner box or a scrollbar track.
///
/// # The inset, on both axes
///
/// The middle 60 % in each direction. The outer fifths hold the sheet's drop
/// shadow at one end and the viewport's own boundary at the other; a dominant
/// colour taken across either is a blend that drifts for reasons unrelated to
/// the preset. On the 585-point strips above, a fifth is 117 points — far more
/// than any scrollbar or edge treatment this shell draws.
///
/// # `None`, and why it is a SKIP rather than a fallback
///
/// When no strip reaches [`MIN_BAND_PTS`] there is genuinely no backdrop on
/// screen: the sheet fills its viewport, which happens at any zoom past fit.
/// The alternative to skipping is sampling the sheet's own shadow and calling it
/// the backdrop, which would produce a *number* — and a number is what a caller
/// cannot tell from a measurement.
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
    // ★ The FIRST strict maximum, not the last, and the candidate order above
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
///
/// Both halves are needed and they catch different failures: the channel floor
/// catches a sheet that was **darkened**, the spread catches one that was
/// **tinted**. A theme built around a blue accent would do the second, and a
/// check that looked only at brightness would let a faintly blue sheet through
/// at full luminance.
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
    // ★ A DOCUMENT IS THE SUBJECT HERE, unlike its sibling. That check launches
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

    // ★ Maximised for the same reason its sibling is — `file.settings` is in
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

    // ★★ THE SHEET MUST BE ON SCREEN BEFORE THE WINDOW THAT WILL COVER IT IS
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
    // ★★★ ASKED FIRST, AND IT IS THE WHOLE HONESTY OF THIS CHECK. "The page
    // stayed white" is true of a build in which nothing happened at all, so the
    // surround has to be shown to have moved before a page reading means
    // anything. Every pair, not just one: three presets that all measured the
    // same are three clicks none of which installed a palette.
    for (i, a) in readings.iter().enumerate() {
        for b in readings.iter().skip(i + 1) {
            let moved = delta(a.backdrop, b.backdrop);
            let shifted = (a.viewport_top - b.viewport_top).abs();
            // ★ EITHER witness suffices, and neither is redundant: `quiet` and
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
    // ★ Read off the FIRST preset, which is `quiet` — a light theme, which
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
///
/// The outer `Result` is the SKIP channel; the inner one is FAIL (`Err`) versus
/// a measurement (`Ok`).
///
/// # ★★ Every rect is re-read after the click, and none is carried in
///
/// `airy` changes the shell's METRICS as well as its colours — `control_height`
/// 24 → 28, `panel_padding` 6 → 12, `gutter` 4 → 8 — so the ribbon is taller,
/// the docks are wider and the canvas has moved by the time the capture is
/// taken. A page rect read before the click is a rectangle the sheet has slid
/// out of, and sampling it would report a tinted page about a build that is
/// fine. This is the single most likely way for this check to produce a false
/// defect report, which is why the re-read is not an optimisation to fold away.
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
        // ★ A FAILURE, not a skip, and this is the row that closes the Airy
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
    // ★ Generous, and for the reason its sibling states: the theme is installed
    // at the TOP of the next frame and `Theme::apply` rewrites both of egui's
    // styles. Airy additionally re-lays the whole shell out, and the canvas
    // re-rasterises the page at the new metrics — so this settle covers a
    // re-render, not just a repaint.
    session.settle(30);

    // ★★ THE APPLICATION'S OWN WINDOW, and `frame_to_png` raises it — which
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

    /// ★★ **The widest side wins, and this test is the bug it was written
    /// after.**
    ///
    /// The first draft looked above and below the sheet only, and its first
    /// live run SKIPPED on the layout it was written for. These are the real
    /// numbers from that run: a maximised window fits a landscape sheet to the
    /// window's HEIGHT, leaving 10.4 points of margin above it and 585 to each
    /// side. A band chooser that only knows about two of the four sides finds
    /// nothing thick enough and the check stops running — silently, because a
    /// SKIP is not red.
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
