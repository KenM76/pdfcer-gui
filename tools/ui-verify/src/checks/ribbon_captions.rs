//! `ribbon_group_captions_legible` — every ribbon group has a caption you can
//! read.
//!
//! # The defect this detects
//!
//! Two of them, sharing one region and one measurement, because they are
//! indistinguishable until you look:
//!
//! * **Illegible caption.** The same root cause as D2 — a foreground colour
//!   paired with a fill the palette did not expect. Ribbon group captions are
//!   small, low-emphasis text (`egui-shell` draws them `weak()` and
//!   `small()`), which is exactly the category a theme change damages first
//!   and a reviewer notices last.
//! * **Missing caption.** The 2026-08-08 screenshot audit found **two ribbon
//!   groups rendering with no caption at all**. Not faint — absent. That was
//!   caught by a screenshot, not by a test, and `GUI_ROADMAP.md` cites it as
//!   one of the two findings that justified building this harness.
//!
//! [`crate::checks::legibility`] reports them separately, because they have
//! different fixes: a uniform region means the caption is not being drawn and
//! the theme is a red herring. It reports a third case separately too — a
//! caption the ribbon declares at a rect outside the window — because that is
//! a layout defect and neither of the first two.
//!
//! # Why the ribbon in particular
//!
//! `RIBBON_IA.md` specifies seven tabs, each with several captioned groups.
//! The caption is what makes the ribbon navigable — it is the only text that
//! says what the row of icons beneath it is *for*. A ribbon whose captions
//! are invisible degrades to an undifferentiated field of icons, which is the
//! state the audit found parts of it in.
//!
//! It is also the surface most likely to regress silently, because captions
//! are drawn by shared chrome code: one widget-style change alters every
//! caption on all seven tabs at once, and nothing about the diff says so.
//!
//! # How it works, as of S2
//!
//! 1. Launch the binary with its diagnostic switch set, opening the fixture if
//!    one was given.
//! 2. Read the trace and collect every region the application **declared**
//!    with a `ui-rect` event.
//! 3. Keep the ones whose names follow the ribbon-caption convention
//!    ([`is_ribbon_caption`]).
//! 4. Capture the window and measure each of them against the WCAG 2.1 AA
//!    large-text floor of 3:1.
//!
//! There is no tab iteration and no calibration step, and both absences are
//! the point. The application reports where its captions are; the harness
//! measures what it reports. A ribbon that gains an eighth tab, collapses to
//! an icon rail (`MODES_AND_PANELS.md` puts that on the roadmap, and it moves
//! every caption in the window) or reflows at a narrower width changes what it
//! declares, and this check follows it with no edit here.
//!
//! # What it reports today, and why that is not a fudge
//!
//! **The ribbon does not exist yet.** `crates/egui-shell/src/ribbon/` is being
//! written as this is; nothing declares a caption region. So step 3 finds
//! nothing and the check reports SKIPPED with a reason that says so *in those
//! terms* — "the application declared no ribbon group caption regions, and
//! here are the three it did declare".
//!
//! That specific wording is load-bearing. The three verdicts available here
//! are:
//!
//! * **PASS** would be a lie: nothing was measured.
//! * **FAIL** would be worse than a lie: it would file a defect against
//!   captions nobody has written, and this codebase has already paid for one
//!   filed-then-retracted defect (see [`crate::coords`]). A check that fails
//!   on unwritten code teaches its readers to ignore it, after which its true
//!   reports get ignored too.
//! * **SKIP naming the missing subsystem** is the honest report, and it is
//!   also *actionable*: it tells the ribbon's author exactly which names to
//!   publish.
//!
//! # The one thing that must remain true for this to start working by itself
//!
//! [`is_ribbon_caption`] must recognise the names the ribbon publishes. See
//! its documentation for what it matches and why the rule is a pair of words
//! rather than an exact spelling.

use crate::checks::legibility::{self, PlannedRegion, RegionArea, TraceRegions};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::image::Image;
use crate::launch::{LaunchSpec, Session};
use crate::profile::Vocabulary;
use crate::report::CheckReport;
use crate::trace::Trace;

/// See the module documentation.
pub struct RibbonGroupCaptionsLegible;

/// The region set this check asks a profile for, when it has to fall back to
/// one. No profile currently declares it, deliberately — see
/// [`crate::profile::PDFCER_GUI`].
const SET: &str = "ribbon_group_captions";

/// How the SKIP reason completes the sentence "the application declared no …".
const CONVENTION: &str = "ribbon group caption regions";

impl Check for RibbonGroupCaptionsLegible {
    fn name(&self) -> &'static str {
        "ribbon_group_captions_legible"
    }

    fn defect(&self) -> &'static str {
        "ribbon group captions rendering below a readable contrast, or not \
         rendering at all — the 2026-08-08 audit found two groups with no caption"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Does this region name declare a **ribbon group caption**?
///
/// # The rule
///
/// The name mentions the ribbon and mentions a caption. Case-insensitive,
/// separator-agnostic. Every spelling below matches:
///
/// ```text
/// ribbon.group.view.zoom.caption      egui-shell's ribbon::report::group_caption
/// ribbon-caption:view/zoom
/// ribbon-group-caption:zoom
/// ```
///
/// # Why a two-word rule rather than an exact prefix
///
/// The spelling is owned by **another crate**. `egui-shell` publishes rects
/// through a `fn(&str, Rect)` callback the application registers, and
/// `crates/egui-shell/src/ribbon/report.rs` builds the names — currently
/// `format!("ribbon.group.{tab}.{group}.caption")`, pinned there by its own
/// stability test. This harness cannot depend on that crate (it drives a
/// process, not a library), so it cannot import the constant, and hard-coding
/// one exact spelling would mean the check silently stops matching if the
/// ribbon settles on a different one — reporting SKIP forever while looking
/// like it is working.
///
/// A pair of required words is the weakest rule that cannot match anything
/// else the application declares. The regions in the trace today are `page`,
/// `central-panel`, `canvas-viewport`, `canvas-message` and `status-message`;
/// none contains either word. A future `ribbon.group.view.zoom` (the group
/// body, without its caption) is correctly excluded, because the caption is
/// the only thing this check can measure — the group body contains icons, and
/// icons are not text with a contrast floor.
///
/// # The failure mode this leaves open, stated
///
/// If the ribbon publishes captions under a name containing neither word —
/// say `chrome.band.zoom.label` — this check goes on skipping and says the
/// application declared no ribbon caption regions, which would then be a false
/// statement. The mitigation is that the SKIP reason **lists every name that
/// was declared**, so a reader who has just written the ribbon sees its names
/// in the output and the mismatch is visible in one line rather than
/// invisible.
#[must_use]
pub fn is_ribbon_caption(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("ribbon") && n.contains("caption")
}

/// Turn one run's trace into the regions this check is about.
///
/// Split out of [`assess`] so that the claim *"this check starts asserting on
/// its own the day the ribbon declares its captions"* is **testable without a
/// ribbon**. A test can hand this function a trace containing the lines the
/// ribbon will emit and observe that the whole chain — parse, match, convert
/// to capture pixels, resolve — produces a trace-sourced plan. If that claim
/// were only exercised by running the real ribbon, it would be untested for
/// exactly as long as it matters.
///
/// `frame` is the live window's measured geometry; it supplies the DPI scale
/// that turns the application's logical rects into pixels of the capture. See
/// [`WindowFrame::logical_to_capture_pixels`] for why no origin term appears.
fn ribbon_regions(vocab: &Vocabulary, trace: &Trace, frame: &WindowFrame) -> TraceRegions {
    let declared = vocab.declared_regions(trace);
    TraceRegions {
        matched: declared
            .iter()
            .filter(|r| is_ribbon_caption(&r.name))
            .map(|r| PlannedRegion {
                name: r.name.clone(),
                area: RegionArea::Pixels(frame.logical_to_capture_pixels(r.rect)),
            })
            .collect(),
        declared: declared.iter().map(|r| r.name.clone()).collect(),
        convention: CONVENTION,
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // Offline mode first: it needs no binary, so a run that has an image and
    // no application still produces a verdict. A PNG cannot declare regions,
    // so no trace is consulted and `resolve_set` is told so explicitly rather
    // than being left to imply something about a trace nobody read.
    if let Some(path) = &ctx.image {
        report.note(format!("asserting against the image {}", path.display()));
        let plan =
            legibility::resolve_set(ctx.profile, SET, Some(path), None).map_err(Error::new)?;
        let image = Image::load_png(path)?;
        return Ok(legibility::assess(
            &image,
            &plan,
            ctx.contrast_threshold,
            report,
        ));
    }

    // --- live mode ---------------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive and no --image to assert against. Pass --exe, or build the \
             profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("ribbon_captions.trace.txt"));
    spec.pdf = ctx.pdf.clone();
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // The ribbon is chrome: it is laid out on the first frame and does not
    // wait for a document. The settle is for the raster of whatever document
    // WAS given, because a window captured mid-raster is a window whose
    // captions are drawn over a placeholder rather than over the panel fill.
    session.settle(24);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process. Without the trace this check has no way to learn where any caption is. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    for reject in trace.rejected_steps() {
        report.note(format!(
            "the application REJECTED a script step: {}",
            reject.raw
        ));
    }

    // --- what did the application say about its own layout? ----------------
    let frame = session.frame()?;
    report.note(format!(
        "window client area {}x{} px at desktop ({}, {}), DPI scale {:.2}",
        frame.client_size.0,
        frame.client_size.1,
        frame.client_origin.0,
        frame.client_origin.1,
        frame.scale
    ));

    let trace_regions = ribbon_regions(&ctx.profile.vocab, &trace, &frame);
    report.note(trace_regions.summary());
    report.note(format!(
        "{} of them follow the ribbon-caption naming convention",
        trace_regions.matched.len()
    ));

    // Either the application declared captions and this resolves to them, or
    // it did not and this is the SKIP that names the missing subsystem. The
    // profile-fraction fallback is consulted in between and finds nothing,
    // which is intended: see `crate::profile::PDFCER_GUI`.
    let plan = legibility::resolve_set(ctx.profile, SET, None, Some(&trace_regions))
        .map_err(Error::new)?;

    // Only now does the harness need the screen. Ordering matters: a run with
    // --no-input against a ribbon-less binary should report "there is no
    // ribbon", which is a fact about the application, rather than "input is
    // disabled", which is a fact about the invocation and tells the reader
    // nothing they did not already type.
    if !ctx.allow_input {
        return Err(Error::new(format!(
            "input is disabled (--no-input), and capturing the window requires raising it to \
             the front — which takes the operator's focus. {} caption region(s) were declared \
             and could have been measured; re-run without --no-input to measure them. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
            plan.regions.len()
        )));
    }

    let shot = ctx.out("ribbon_captions.png");
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);
    Ok(legibility::assess(
        &image,
        &plan,
        ctx.contrast_threshold,
        report,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spelling `egui-shell`'s `ribbon::report::group_caption` builds
    /// today. This is the one that has to work on the day the ribbon lands,
    /// with no edit to this crate.
    #[test]
    fn the_shells_own_caption_name_is_recognised() {
        assert!(is_ribbon_caption("ribbon.group.view.zoom.caption"));
    }

    /// …and the other plausible spellings, because the exact one is owned by
    /// another crate and is not this crate's to pin.
    #[test]
    fn the_other_plausible_caption_spellings_are_recognised_too() {
        assert!(is_ribbon_caption("ribbon-caption:view/zoom"));
        assert!(is_ribbon_caption("ribbon-group-caption:zoom"));
        assert!(is_ribbon_caption("RIBBON.GROUP.FILE.CAPTION"));
    }

    /// Every region the application declares today. Matching any of these
    /// would aim a caption check at a panel and report a contrast for it.
    #[test]
    fn nothing_the_application_declares_today_is_mistaken_for_a_caption() {
        for name in [
            "page",
            "central-panel",
            "canvas-viewport",
            "canvas-message",
            "status-message",
        ] {
            assert!(!is_ribbon_caption(name), "{name} is not a ribbon caption");
        }
    }

    /// A group's own rect is not its caption. The group body is icons, and an
    /// icon has no contrast floor to assert against.
    #[test]
    fn a_group_body_is_not_a_caption() {
        assert!(!is_ribbon_caption("ribbon.group.view.zoom"));
        assert!(!is_ribbon_caption("ribbon.tab.view"));
        assert!(!is_ribbon_caption("ribbon.modes"));
    }

    /// A 1100×800 window at 100%, matching the harness's own runs.
    fn frame() -> WindowFrame {
        WindowFrame {
            client_origin: (100, 200),
            client_size: (1100, 800),
            scale: 1.0,
        }
    }

    /// **The claim this whole file rests on**, tested without a ribbon: given
    /// a trace that declares caption regions the way the ribbon will, the
    /// check resolves to them, from the trace, with no harness change and no
    /// calibration.
    ///
    /// The trace below is today's real capture with three ribbon lines added
    /// in the shape `egui-shell`'s `ribbon::report::group_caption` builds.
    #[test]
    fn the_day_the_ribbon_declares_its_captions_this_check_starts_asserting() {
        let trace = Trace::parse(
            "pdfcer-diag start argv1=None\n\
             pdfcer-diag ui-rect name=central-panel rect=[[8.0 8.0] - [1092.0 792.0]]\n\
             pdfcer-diag ui-rect name=page rect=[[16.0 22.8] - [1084.0 777.2]]\n\
             pdfcer-diag ui-rect name=ribbon.group.view.zoom.caption rect=[[20.0 84.0] - [96.0 98.0]]\n\
             pdfcer-diag ui-rect name=ribbon.group.view.pages.caption rect=[[104.0 84.0] - [188.0 98.0]]\n\
             pdfcer-diag ui-rect name=ribbon.group.view.zoom rect=[[20.0 30.0] - [96.0 84.0]]",
            "pdfcer-diag",
        );
        let regions = ribbon_regions(&Vocabulary::pdfcer_gui(), &trace, &frame());

        assert_eq!(regions.declared.len(), 5);
        let matched: Vec<&str> = regions.matched.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            matched,
            [
                "ribbon.group.view.pages.caption",
                "ribbon.group.view.zoom.caption"
            ],
            "the two captions, and NOT the group body next to them"
        );

        let plan = legibility::resolve_set(&crate::profile::PDFCER_GUI, SET, None, Some(&regions))
            .expect("the trace supplies the regions");
        assert_eq!(
            plan.source,
            legibility::RegionSource::Trace,
            "a declared rect must outrank anything hard-coded; see legibility's module docs"
        );
        match plan.regions[0].area {
            RegionArea::Pixels(px) => {
                assert_eq!(px, crate::geom::PixRect::new(104, 84, 84, 14));
            }
            RegionArea::Fraction(_) => panic!("a traced region is not a fraction"),
        }
    }

    /// And it asserts rather than passing vacuously: a caption drawn in the D2
    /// pairing (near-white on light grey) must FAIL, and the same region drawn
    /// legibly must PASS. Without both halves this is a check that cannot
    /// fail, which is indistinguishable from no check at all.
    #[test]
    fn a_declared_caption_is_measured_and_can_fail() {
        use crate::image::{Image, Rgb};

        // A 200x40 capture: the top half is the caption's declared region.
        fn capture(fg: Rgb) -> Image {
            let bg = Rgb::new(232, 232, 232);
            let mut bgra = Vec::new();
            for i in 0..200u32 * 40 {
                // 15% coverage inside the caption band, flat fill below it.
                let c = if i < 200 * 20 && i % 7 == 0 { fg } else { bg };
                bgra.extend_from_slice(&[c.b, c.g, c.r, 0xFF]);
            }
            Image::from_bgra(200, 40, bgra).unwrap()
        }

        let plan = legibility::RegionPlan {
            set_name: SET.to_owned(),
            source: legibility::RegionSource::Trace,
            provenance: "synthetic".to_owned(),
            regions: vec![PlannedRegion {
                name: "ribbon.group.view.zoom.caption".to_owned(),
                area: RegionArea::Pixels(crate::geom::PixRect::new(0, 0, 200, 20)),
            }],
        };

        let mut report = CheckReport::new("t", "t");
        let d2 = legibility::assess(
            &capture(Rgb::new(250, 250, 250)),
            &plan,
            crate::pixels::AA_LARGE,
            &mut report,
        );
        assert!(
            d2.is_some_and(|r| r.contains("below the 3.0:1 floor")),
            "a near-white caption on light grey must be reported as illegible"
        );

        let mut report = CheckReport::new("t", "t");
        assert!(
            legibility::assess(
                &capture(Rgb::new(32, 32, 32)),
                &plan,
                crate::pixels::AA_LARGE,
                &mut report,
            )
            .is_none(),
            "a dark caption on the same fill must pass — a check that fails everything is \
             no better than one that passes everything"
        );
    }
}
