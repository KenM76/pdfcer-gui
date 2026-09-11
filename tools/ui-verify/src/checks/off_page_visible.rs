//! `an_object_off_the_page_is_actually_drawn` — **O23's "see" half, with a
//! screenshot as its oracle.**
//!
//! # The report
//!
//! Ken, 2026-08-21, `OPERATOR_REQUESTS.md` **O23**:
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! and, three weeks later and after the *reach* half had shipped:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! Two verbs in one sentence, and they were built separately because they
//! break separately:
//!
//! | half | check | what it proves |
//! |---|---|---|
//! | **reach** | `off_page_press` | a press in the grey becomes a gesture, so the object can be banded, selected and dragged |
//! | **see** | **this file** | the object is *painted*, so the operator knows it is there at all |
//!
//! `off_page_press`'s own header states the boundary verbatim — *"it does not
//! claim the operator can SEE the object"* — and that sentence is what this
//! check discharges.
//!
//! # ★★★ Why a screenshot, and why nothing else would do
//!
//! `D:/dev/rag/egui/` carries the rule this obeys: **layout and clipping
//! defects have exactly one oracle, and it is a rendered screenshot.** The
//! whole defect here is a *pixmap size* — `pdfcer_render::render_page` sizes
//! its pixmap to the `/CropBox`, so nothing culls the off-page square; there
//! are simply no pixels out there to put it in. Every layer above that is
//! innocent and reports success:
//!
//! * the decomposer lists the square (`render::offpage` asserts the content
//!   union includes it);
//! * the hit test finds it (`off_page_press` asserts a band takes it);
//! * the render worker returns a pixmap with no error;
//! * the canvas paints that pixmap at the right rectangle.
//!
//! ⇒ A trace-only check would have been green for the entire three weeks the
//! operator could not see his object. So this one counts **ink in the
//! capture**, and the trace line below is the *corroboration*, not the
//! assertion.
//!
//! # The two assertions, and why neither is sufficient alone
//!
//! 1. **`canvas-halo tier=halo`** — the shell decided to widen the raster, and
//!    the box it widened to reaches left of x = 0. Without this, a passing
//!    pixel test could be measuring the wrong square, a stale capture, or a
//!    window that happens to have something dark at that address.
//! 2. **Ink where the off-page square is, and paper where it is not** — the
//!    pixels. Without this, `tier=halo` would say only that the shell *asked*
//!    for a bigger raster; the engine could still have clipped it, the texture
//!    could be placed at the wrong rectangle, or the image could be drawn
//!    under the backdrop.
//!
//! ★ The **pair** is the point, and the second half of the pair is the control
//! this suite has learned to insist on: *"a uniform failure at every rung of a
//! sweep is about the probe."* A patch of the halo that is inside the widened
//! raster but outside the square must come back as **paper**. If both patches
//! read dark, the probe is aimed at something other than the page — a panel, a
//! shadow, the desktop — and this check says so rather than reporting a pass.
//!
//! # The fixture
//!
//! `fixtures/off-page-object.pdf`, shared with both sibling checks, because two
//! fixtures for one property is two chances for one of them to stop having it.
//! A 200 × 200 page, no `/CropBox`, two black filled rectangles:
//!
//! ```text
//! 0 0 0 rg
//! 40 40 60 60 re f          <- A, on the page
//! -160 100 120 40 re f      <- B, ENTIRELY left of the media box
//! ```
//!
//! So the content union is x −160…200, y 0…200 and the halo box must be that.
//! B's centre is `(−100, 120)`; the paper control is `(−100, 40)`, which is
//! 60 pt below B, still 100 pt left of the sheet, and therefore inside the
//! widened raster and outside every mark in the file.
//!
//! # ★★ Why `view.zoom_actual` and not fit-page
//!
//! The same reason `off_page_press` gives, and it was measured there: fit-page
//! on a 200 × 200 fixture in a maximised window puts the sheet at roughly
//! 3.8 px per point, so x = −100 lands about 381 px left of the sheet where
//! only ~243 px of viewport exists, and the conversion refuses — correctly —
//! and the check SKIPS **silently, because a SKIP is not red**. At 100 % the
//! sheet is ~200 px wide in a ~1250 px viewport and the whole halo fits with
//! room to spare. 100 % is also a property of the DOCUMENT rather than of the
//! window, so this check's geometry no longer varies with the screen it runs
//! on.
//!
//! # Every way this reports SKIP
//!
//! No binary, no diagnostic channel, no `canvas-viewport` region, not enough
//! grey on screen to reach x = −100, or a capture that could not be taken.
//! **Not** "the halo never engaged" and **not** "there was no ink" — those are
//! failures, and they are the two this check exists to find.
//!
//! ## ⚠ And one deliberate non-skip: `known=false`
//!
//! `canvas-halo known=` reports whether the page had been decomposed when the
//! tier was decided. `OpenDoc::content_bounds_if_known` peeks and never
//! builds — a build costs 469 ms on the operator's own drawing and the canvas
//! runs every frame — so on the first frame of any document the answer is
//! `false` and there is no halo. `render::settle` builds it immediately after,
//! so by the time this check reads anything it must be `true`.
//!
//! ⇒ `known=false` in the **last** line is therefore a real failure with a
//! precise cause (the settle-time build is not running), not a timing wobble,
//! and it is reported as one.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Single-page display, then **100 %**. See the module header — fit-page puts
/// the aim outside the viewport and the check skips without saying so.
const INVOKE: &str = "view.page_single,view.zoom_actual";

/// The tier trace this check reads.
const HALO: &str = "canvas-halo"; // ui-text-exempt: a trace event name, never displayed

/// The value of `tier=` that means the raster was widened.
const TIER_HALO: &str = "halo"; // ui-text-exempt: a trace field value, never displayed

/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The scroll area the sheet is drawn inside. Every point this check samples is
/// outside the page's own rect by construction, so this is the only bound an
/// off-page conversion can be made against.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// The fixture, relative to the workspace root. Shared with both siblings.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// Its page.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// **The centre of the off-page square**, in page points. Ink is expected here.
const INK_AT: (f64, f64) = (-100.0, 120.0);

/// **Paper, off the page** — inside the widened raster, 60 pt below the square,
/// 100 pt left of the sheet. Paper is expected here.
///
/// ★ This is the control, and the module header says why a check that samples
/// only its subject cannot tell a working feature from a mis-aimed probe.
const PAPER_AT: (f64, f64) = (-100.0, 40.0);

/// Half the width and half the height of each sampled patch, in page points.
///
/// The square is 120 × 40 pt, so ±8 pt about its centre stays 52 pt clear of
/// its left and right edges and 12 pt clear of its top and bottom. The paper
/// patch is the same size so that the two counts are directly comparable — a
/// ratio between differently sized samples is a number nobody can read.
const PATCH_PT: f64 = 8.0;

/// A pixel at or below this in every channel is ink.
///
/// The fixture fills with `0 0 0 rg`, so true ink is `#000000`; the slack is
/// for the capture's colour management and for antialiasing at the patch's
/// edge, which cannot reach the middle 16 × 16 pt of a 120 × 40 pt rectangle.
const INK: u8 = 96;

/// At least this fraction of the ink patch must actually be dark.
///
/// Deliberately not 1.0: the patch is converted through the window frame's
/// scale and rounded, so its outermost row can land a pixel outside the square
/// on a fractional-DPI display. Deliberately not 0.1 either — a tenth of a
/// patch is what a scroll bar or a tooltip edge could contribute.
const INK_FRACTION: f64 = 0.90;

/// At most this fraction of the paper patch may be dark.
///
/// See [`PAPER_AT`]. Anything above this and the probe is not measuring the
/// page.
const PAPER_INK_FRACTION: f64 = 0.05;

/// See the module documentation.
pub struct AnObjectOffThePageIsActuallyDrawn;

impl Check for AnObjectOffThePageIsActuallyDrawn {
    fn name(&self) -> &'static str {
        "an_object_off_the_page_is_actually_drawn"
    }

    fn defect(&self) -> &'static str {
        "an object placed past the edge of the sheet is never painted, because every raster is \
         sized to the crop box — so the operator can scroll out to where it is, select it and \
         drag it, while looking at empty grey"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The fraction of `patch` that is ink, and how many pixels that was measured
/// over.
///
/// Returns `None` when the patch has no area in the capture at all — which is
/// a finding, never "no ink": `WindowFrame::logical_to_capture_pixels` clamps
/// to the capture, so a zero-area result means the address is off screen.
fn ink_fraction(image: &Image, patch: crate::geom::PixRect) -> Option<(f64, u64)> {
    let total = u64::from(patch.w) * u64::from(patch.h);
    if total == 0 {
        return None;
    }
    let dark = image
        .pixels_in(patch)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count() as u64;
    #[allow(
        clippy::cast_precision_loss,
        reason = "a patch is a few hundred pixels; f64 is exact far past that" // ui-text-exempt: clippy lint justification, never displayed
    )]
    Some((dark as f64 / total as f64, total))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check has no bound to \
             convert an off-page point against.",
            ctx.profile.name
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax, \
             shared with `off_page_marquee` and `off_page_press`.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-page-visible.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
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
    session.settle(40);
    // Maximising widens the grey the off-page square has to fit into.
    session.maximize();
    // ★ A long settle, on purpose. The halo cannot appear on the first frame:
    // the page must be decomposed before the shell knows where its ink reaches,
    // and that build happens in `render::settle` AFTER the picture is asked
    // for. Then the widened raster itself has to be rendered and uploaded. The
    // sequence is page → decomposition → halo raster, and this check reads the
    // end of it.
    session.settle(60);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen and there is nothing to be off \
             the edge of. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }

    // --- assertion 1: the shell decided to widen the raster -------------------
    let tiers: Vec<String> = trace
        .events(HALO)
        .map(|l| {
            format!(
                "{}({})",
                l.get("tier").unwrap_or("?"),
                l.get("known").unwrap_or("?")
            )
        })
        .collect();
    let Some(last) = trace.events(HALO).last() else {
        return Err(Error::new(format!(
            "the application emitted no `{HALO}` line at all, so it is not this build's tier \
             decision that is wrong — the trace does not carry one. Either this binary predates \
             `canvas::trace::halo` (2026-09-10) or the canvas never laid out a page. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("tier decisions this run: {}", list(&tiers)));

    if last.get("known") != Some("true") {
        return Ok(Some(format!(
            "★★★ THE PAGE WAS NEVER DECOMPOSED, so the shell could not know where its ink \
             reaches: `{}`.\n\n\
             `canvas::present` peeks at the content bounds through \
             `OpenDoc::content_bounds_if_known` and NEVER builds them — a decomposition costs \
             469 ms on the operator's own drawing and the canvas runs every frame. The build is \
             `OpenDoc::ensure_content_bounds`, called once per frame from \
             `render::settle::settle_and_rasterize`. `known=false` on the last line means that \
             call is gone, or that it is failing on this fixture.\n\n\
             ★ Not a timing wobble: this check settles 100 frames' worth after launch, and the \
             build happens on the frame after the first render is requested. Trace: {}.",
            last.raw,
            session.trace_path().display()
        )));
    }
    if last.get("tier") != Some(TIER_HALO) {
        return Ok(Some(format!(
            "★★★ THE RASTER WAS NEVER WIDENED: `{}`.\n\n\
             The page decomposed (`known=true`) and the shell still asked for the crop box. On \
             this fixture the content union is x −160…200 against a 200 × 200 media box, so \
             `render::halo::region` has a 160 pt overhang to find — far above its 1 pt \
             tolerance.\n\n\
             ★ Suspect, in order: `PageFrame::crop()` being compared against a content box in \
             the wrong space (the union is PDF user space, y-up, and so is the crop box — a \
             canvas-space content box would make the overhang vanish on an upright page and \
             invert on a rotated one); the pixmap-ceiling guard declining at this zoom, which \
             would be wrong at 100 % on a 360 pt box; or the halo branch never being reached \
             because `strategy::for_page` answered `Region`. Tier decisions: {}. Trace: {}.",
            last.raw,
            list(&tiers),
            session.trace_path().display()
        )));
    }
    report.note(format!("the shell widened the raster: `{}`", last.raw));

    // --- assertion 2: the pixels ---------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let frame = session.frame()?;
    let viewport = declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{VIEWPORT_REGION}` region, so this check has no bound \
             to convert an off-page point against. It cannot fall back to the page's own rect: \
             every point sampled here is outside that by construction."
        ))
    })?;

    // Both patches through the off-page conversion, and both as RECTANGLES —
    // the corners are converted, not the centre plus a guessed pixel radius,
    // so the patch is the right size at any window scale.
    let patch_of = |at: (f64, f64)| -> Result<crate::geom::PixRect> {
        let lo = mapping
            .doc_to_window_off_page(DocPoint::new(0, at.0 - PATCH_PT, at.1 + PATCH_PT), viewport)?;
        let hi = mapping
            .doc_to_window_off_page(DocPoint::new(0, at.0 + PATCH_PT, at.1 - PATCH_PT), viewport)?;
        Ok(frame.logical_to_capture_pixels(LRect::new(
            Pt::new(lo.x().min(hi.x()), lo.y().min(hi.y())),
            Pt::new(lo.x().max(hi.x()), lo.y().max(hi.y())),
        )))
    };
    let ink_patch = patch_of(INK_AT)?;
    let paper_patch = patch_of(PAPER_AT)?;

    let shot = ctx.out("off-page-visible.png");
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot.clone());

    let Some((ink, ink_px)) = ink_fraction(&image, ink_patch) else {
        return Err(Error::new(format!(
            "the ink patch clipped to zero area in the capture ({ink_patch:?} of a {} x {} \
             image), so nothing was measured. The conversion accepted the point, so this is a \
             capture-geometry disagreement rather than an application defect — suspect the \
             window frame's scale. Screenshot: {}.",
            image.width(),
            image.height(),
            shot.display()
        )));
    };
    let Some((paper, paper_px)) = ink_fraction(&image, paper_patch) else {
        return Err(Error::new(format!(
            "the PAPER control patch clipped to zero area ({paper_patch:?}), so the measurement \
             has no control and this check refuses to report on the ink patch alone. \
             Screenshot: {}.",
            shot.display()
        )));
    };
    report.note(format!(
        "ink patch {ink:.3} over {ink_px} px at ({:.0}, {:.0}) pt; paper control {paper:.3} over \
         {paper_px} px at ({:.0}, {:.0}) pt",
        INK_AT.0, INK_AT.1, PAPER_AT.0, PAPER_AT.1
    ));

    // ★★★ THE CONTROL FIRST. If the paper patch is dark, the probe is not
    // looking at the page and nothing said about the ink patch would mean
    // anything — including a pass.
    if paper > PAPER_INK_FRACTION {
        return Ok(Some(format!(
            "★★★ THE PROBE IS NOT MEASURING THE PAGE: the CONTROL patch is {paper:.3} dark, \
             where at most {PAPER_INK_FRACTION:.2} is allowed. That patch is at ({:.0}, {:.0}) \
             pt — 60 pt below the off-page square and 100 pt left of the sheet — and the \
             fixture has no mark anywhere near it.\n\n\
             ⇒ Read this as a finding about the HARNESS before reading it as one about the \
             feature. Something dark is at that address: a panel, a dropped shadow, a context \
             menu, or the desktop showing through because the window is not where \
             `WindowFrame` thinks it is. The ink patch measured {ink:.3}, and whatever that \
             number is it is not evidence. Screenshot: {}.",
            PAPER_AT.0,
            PAPER_AT.1,
            shot.display()
        )));
    }

    if ink < INK_FRACTION {
        return Ok(Some(format!(
            "★★★ THE OFF-PAGE OBJECT IS NOT PAINTED: only {ink:.3} of the patch at ({:.0}, \
             {:.0}) pt is ink, where at least {INK_FRACTION:.2} is required. The control patch \
             is clean at {paper:.3}, so the probe IS looking at the page — this is the feature, \
             not the harness.\n\n\
             ★★ The shell asked for the widened box (`{}`), so the decision is right and the \
             failure is downstream of it. Three candidates, in order:\n\
             • the engine clipped the region to the crop box after all — `render::offpage` \
             asserts it does not, and if that module now fails too the finding belongs in the \
             request channel, not here;\n\
             • the texture is placed at the wrong rectangle — `render::region::region_on_screen` \
             maps the region back to screen, and a halo reaches LEFT of the page's own rect \
             where every previous region was inside it;\n\
             • the widened raster is being drawn UNDER the whole-page backdrop \
             (`canvas::backdrop::paint`), which is painted at the page's rect and would leave \
             the overhang uncovered.\n\n\
             Screenshot: {}.",
            INK_AT.0,
            INK_AT.1,
            last.raw,
            shot.display()
        )));
    }

    report.note(format!(
        "★★★ the square that lies ENTIRELY off the left edge of the sheet is on the screen: \
         {ink:.3} of a patch at ({:.0}, {:.0}) pt is ink, against {paper:.3} at the paper \
         control 60 pt below it. That is O23's second verb — the operator can SEE what he put \
         past the page edge, not merely reach it",
        INK_AT.0, INK_AT.1
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{INK_AT, INK_FRACTION, PAPER_AT, PAPER_INK_FRACTION, PATCH_PT};

    /// The fixture's off-page square `(left, bottom, right, top)`, transcribed
    /// from its own content stream — `-160 100 120 40 re f`.
    const SQUARE_B: (f64, f64, f64, f64) = (-160.0, 100.0, -40.0, 140.0);

    /// The fixture's on-page square — `40 40 60 60 re f`.
    const SQUARE_A: (f64, f64, f64, f64) = (40.0, 40.0, 100.0, 100.0);

    /// The fixture's media box, which is also its crop box: it declares none.
    const MEDIA: (f64, f64, f64, f64) = (0.0, 0.0, 200.0, 200.0);

    /// ★★★ The ink patch is wholly inside the off-page square, with margin on
    /// every side. If this ever stops holding, the check measures antialiasing
    /// and its threshold becomes a coin toss.
    #[test]
    fn the_ink_patch_is_inside_the_off_page_square() {
        assert!(INK_AT.0 - PATCH_PT > SQUARE_B.0, "left margin");
        assert!(INK_AT.0 + PATCH_PT < SQUARE_B.2, "right margin");
        assert!(INK_AT.1 - PATCH_PT > SQUARE_B.1, "bottom margin");
        assert!(INK_AT.1 + PATCH_PT < SQUARE_B.3, "top margin");
    }

    /// ★★ …and that square really is off the page, which is the entire premise.
    #[test]
    fn the_off_page_square_is_entirely_off_the_page() {
        assert!(
            SQUARE_B.2 < MEDIA.0,
            "square B's right edge {} must be left of the media box's left edge {}",
            SQUARE_B.2,
            MEDIA.0
        );
    }

    /// ★★★ The control patch touches NEITHER square, and is off the page. A
    /// control that could overlap a mark would fire the harness-defect branch
    /// on a perfectly good build.
    #[test]
    fn the_paper_patch_touches_no_mark_in_the_fixture() {
        let disjoint = |s: (f64, f64, f64, f64)| {
            PAPER_AT.0 + PATCH_PT < s.0
                || PAPER_AT.0 - PATCH_PT > s.2
                || PAPER_AT.1 + PATCH_PT < s.1
                || PAPER_AT.1 - PATCH_PT > s.3
        };
        assert!(disjoint(SQUARE_A), "the control patch overlaps square A");
        assert!(disjoint(SQUARE_B), "the control patch overlaps square B");
        assert!(
            PAPER_AT.0 + PATCH_PT < MEDIA.0,
            "the control patch must be off the sheet too — it is measuring the WIDENED raster, \
             and a patch on the paper would pass even with the halo switched off"
        );
    }

    /// ★ The control patch is inside the halo box, or it would be sampling the
    /// canvas's grey rather than the raster's paper — which is a different
    /// colour and a different claim.
    #[test]
    fn the_paper_patch_is_inside_the_widened_raster() {
        let halo_llx = SQUARE_B.0.min(MEDIA.0);
        let halo_lly = SQUARE_B.1.min(MEDIA.1);
        assert!(PAPER_AT.0 - PATCH_PT > halo_llx, "left of the halo box");
        assert!(PAPER_AT.1 - PATCH_PT > halo_lly, "below the halo box");
    }

    /// The two thresholds cannot both be satisfied by one uniform image, which
    /// is what makes the pair an oracle rather than two opinions.
    #[test]
    fn the_thresholds_are_mutually_exclusive() {
        // A const block, at clippy's insistence and to its credit: this pair is
        // knowable without running anything, so it is checked when the file
        // compiles rather than when the suite happens to be run.
        const {
            assert!(
                PAPER_INK_FRACTION < INK_FRACTION,
                "a uniform capture must fail one of the two"
            );
        }
    }
}
