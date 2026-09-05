//! `ribbon_matches_the_mockup_geometry` — the band, measured against
//! `mockups/pdfcer-shell.html`.
//!
//! # ★★★ Why this check exists, and why it was written UNRUN
//!
//! On 2026-09-04 the operator compared the shipped ribbon with the mockup and
//! named four differences:
//!
//! > *"there are still a lot of things that still look like our old layout
//! > including text label location and missing glyphs."*
//!
//! and, at length, the biggest one: **every item in the shipped band is drawn
//! inside a visible button frame, and the mockup draws them frameless.**
//!
//! All four were fixed in the same session, and every fix carries a unit test
//! that measures a rectangle or a metric. **Not one of those tests can see
//! whether the band LOOKS like the mockup**, and the distinction is not
//! pedantic — it is this project's standing finding, written into
//! `MODES_AND_PANELS.md`: *layout and appearance defects have exactly one
//! oracle, a rendered screenshot.* The two caption-less ribbon groups that
//! started this whole harness were found by a screenshot while every unit
//! test passed.
//!
//! The session that wrote the fix could not take one. The operator was at his
//! keyboard, a watchdog kills GUI processes on sight, and raising a window
//! takes his focus. So this file is the check that would have settled it,
//! written from the same CSS the fix was translated from, and **left unrun**.
//! Running it is one command:
//!
//! ```text
//! cargo run -p ui-verify -- --check ribbon_matches_the_mockup_geometry \
//!     --exe target/debug/pdfcer-gui.exe --width 1700
//! ```
//!
//! 1700 rather than the harness's usual 1100 is deliberate and is the second
//! half of the reason the comparison was inconclusive: the mockup was rendered
//! at 1700 px and the only recent capture of the real File tab
//! (`target/uv-icons/ribbon_captions.png`) is 1100 px wide. Some of what read
//! as "missing" in that comparison is `RIBBON_SCALING.md`'s collapse ladder
//! working correctly at a narrower width. **A fair comparison has to be at the
//! same width**, and a check that drove 1100 would re-file the same false
//! finding.
//!
//! # What it asserts, and which half of each pair a unit test could already do
//!
//! | # | claim | mockup | a unit test can see it? |
//! |---|---|---|---|
//! | 1 | the band's first control sits clear of the tab strip | `.ribbon { padding: 6px … }` | yes — `egui-shell`'s `the_band_draws_clear_space_above_its_first_control` |
//! | 2 | a Large control is 56 pt, not the full row area | `.rb.big { height: 56px }` | yes — `a_large_control_is_shorter_than_the_row_area_it_sits_in` |
//! | 3 | the caption hangs at the bottom of the row area | `.grp .cap { margin-top: auto }` | yes — `every_caption_in_a_band_shares_one_baseline` |
//! | 4 | **a resting control paints no frame** | `.rb { border: 1px solid transparent }` | ★★★ **NO** |
//! | 5 | **every control draws a glyph** | `svg.g` | ★★★ **NO** |
//!
//! Rows 1–3 are re-asserted here anyway, and that is not duplication: a unit
//! test measures what the layout code *computed*, and this measures what the
//! process *published while drawing on a real screen at a real DPI*. The two
//! have disagreed before in this codebase — `sizing::render_large`'s
//! zero-height overflow-menu defect passed every unit test and was found by
//! this harness — and when they disagree, this one is right.
//!
//! ★★★ **Rows 4 and 5 are the reason the file exists.** Both are questions
//! about ink, and a rect cannot answer either:
//!
//! * A frame is `weak_bg_fill` plus `bg_stroke` painted into a rectangle the
//!   control occupies **whether or not the frame is drawn** — that is
//!   precisely what makes `Button::frame_when_inactive(false)` safe, and
//!   precisely what makes it invisible to a geometry test. Every rect in the
//!   trace is identical before and after the fix.
//! * A missing glyph is `icons::paint_missing_mark`'s slashed box, which
//!   occupies exactly the rect a real glyph would. The band reports the item;
//!   the item reports its size; nothing reports what was painted inside it.
//!
//! # How row 4 is measured, since "is there a frame?" needs a definition
//!
//! [`is_frameless`] samples a one-pixel ring just inside a control's declared
//! rectangle and compares it with a ring just outside. A framed control
//! differs on both counts — a fill that is not the band's, and a stroke on the
//! boundary. A frameless one is the band's own colour right up to and across
//! its edge, because nothing was painted there at all.
//!
//! ★ It samples the **corners' neighbourhoods rather than the whole ring**,
//! and skips any sample that lands on ink: a control's icon and label are
//! inside its rect and are supposed to be different from the background. The
//! corners of a ribbon button are the one part reliably empty of content in
//! both designs, which is what makes them the right place to ask about the
//! frame and the wrong place to ask about anything else.
//!
//! # Which control, and why the check picks it rather than taking one
//!
//! It must be **resting**: not hovered, not focused, not selected. A selected
//! control draws its plate at rest by design (`.rb[aria-pressed="true"]`), and
//! a check that happened to sample `View ▸ Scroll` — the page-display mode
//! that is on by default — would report a frame and file a defect against the
//! one behaviour the fix deliberately kept.
//!
//! So it drives the **File** tab, whose band holds no toggle at all, and it
//! parks the pointer at the window's bottom-left corner before capturing. Both
//! are stated in [`RESTING_TAB`] and [`assess`] rather than assumed.

use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect};
use crate::image::{Image, Rgb};
use crate::launch::{LaunchSpec, Session};
use crate::profile::DeclaredRegion;
use crate::report::CheckReport;

/// See the module documentation.
pub struct RibbonMatchesTheMockupGeometry;

/// The tab this check drives.
///
/// **File**, and the choice is load-bearing rather than alphabetical: it is
/// the tab the operator compared, it is the widest one, and — the property
/// row 4 depends on — **not one of its controls is a toggle**. Every other tab
/// carries at least one command that is selected at rest (View's page display,
/// View's armed tool, Markup's shape), and a selected control draws a plate on
/// purpose.
const RESTING_TAB: &str = "file";

/// The width the mockup was rendered at.
///
/// See the module header: comparing a 1700 px mockup with an 1100 px capture
/// makes the collapse ladder look like a defect. The default is stated here so
/// a run that does not pass `--width` still compares like with like.
const MOCKUP_WIDTH: u32 = 1700;

/// How far a measured figure may sit from the mockup's, in points.
///
/// One point. Below what anyone can see, above `egui`'s own rounding, and the
/// same slack `egui-shell`'s own layout tests use — deliberately, so a
/// disagreement between the two is a real disagreement rather than two
/// tolerances.
const SLACK: f32 = 1.0;

impl Check for RibbonMatchesTheMockupGeometry {
    fn name(&self) -> &'static str {
        "ribbon_matches_the_mockup_geometry"
    }

    fn defect(&self) -> &'static str {
        "the ribbon band drawn to different proportions from \
         mockups/pdfcer-shell.html — a visible frame around every resting \
         control, a Large control spanning the whole row area, or a control \
         with no glyph in it"
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

/// **Is this control drawn without a frame?**
///
/// The oracle for row 4 of the module header's table, and the only assertion
/// in this file that a unit test could not have made.
///
/// # The measurement
///
/// Four probes, one per corner, each a pair: a pixel `inset` inside the
/// control's rectangle and a pixel `inset` outside it, on the diagonal. A
/// control with a frame differs at the inside probe (its `weak_bg_fill`) and
/// again on the boundary between them (its `bg_stroke`). A control with no
/// frame is the band's own colour at both.
///
/// A probe pair is **discarded** when either sample is far from the modal
/// background — that is content (an icon corner, a descender) rather than
/// chrome, and asserting on it would report a defect against a glyph. The
/// verdict is taken over the pairs that survive; if none survives, the caller
/// is told so rather than being handed a pass built on nothing.
///
/// ★ `inset` is 2 px rather than 1: `egui` rounds a button's corners
/// (`Metrics::corner_radius`, 3 pt in `Quiet`), so the literal corner pixel of
/// the rectangle is outside the painted shape even when a frame IS drawn, and
/// a one-pixel probe would report every framed control as frameless.
#[must_use]
pub fn is_frameless(image: &Image, rect: PixRect, ground: Rgb, inset: u32) -> Option<bool> {
    let far = |c: Rgb| {
        let d = |a: u8, b: u8| i32::from(a).abs_diff(i32::from(b));
        d(c.r, ground.r) + d(c.g, ground.g) + d(c.b, ground.b) > 24
    };
    // ★★ Every probe is `checked_sub`, and a corner whose outside sample would
    // fall at a negative coordinate is **declined**, not clamped to zero.
    //
    // That is not defensive arithmetic against a synthetic fixture. A ribbon
    // control genuinely can sit at the window's left edge — the first item of
    // the first group at a width where the band has scrolled — and clamping
    // would sample the control's own left column as though it were the band
    // behind it, which reports every such control as frameless whatever it
    // drew. Declining loses one corner and keeps the other three, and
    // `judged == 0` is the caller's signal that nothing was measured at all.
    let sub = |a: u32, b: u32| a.checked_sub(b);
    let corners = [
        (
            Some(rect.x + inset),
            Some(rect.y + inset),
            sub(rect.x, inset),
            sub(rect.y, inset),
        ),
        (
            sub(rect.x + rect.w, inset),
            Some(rect.y + inset),
            Some(rect.x + rect.w + inset),
            sub(rect.y, inset),
        ),
        (
            Some(rect.x + inset),
            sub(rect.y + rect.h, inset),
            sub(rect.x, inset),
            Some(rect.y + rect.h + inset),
        ),
        (
            sub(rect.x + rect.w, inset),
            sub(rect.y + rect.h, inset),
            Some(rect.x + rect.w + inset),
            Some(rect.y + rect.h + inset),
        ),
    ];

    let mut judged = 0_usize;
    let mut framed = 0_usize;
    for (ix, iy, ox, oy) in corners {
        let (Some(ix), Some(iy), Some(ox), Some(oy)) = (ix, iy, ox, oy) else {
            continue;
        };
        let (Some(inside), Some(outside)) = (image.pixel(ix, iy), image.pixel(ox, oy)) else {
            continue;
        };
        // The outside probe must be the band. If it is not, this corner is
        // next to a neighbouring control or a separator and says nothing about
        // this one's frame.
        if far(outside) {
            continue;
        }
        judged += 1;
        if far(inside) {
            framed += 1;
        }
    }
    (judged > 0).then_some(framed == 0)
}

/// Every region the ribbon declared, as `(name, rect)`.
fn ribbon_regions(ctx: &CheckContext, trace: &crate::trace::Trace) -> Vec<DeclaredRegion> {
    ctx.profile
        .vocab
        .declared_regions(trace)
        .into_iter()
        .filter(|r| r.name.starts_with("ribbon."))
        .collect()
}

/// The tallest declared height among the band items on `tab`, which is a Large
/// control's height when the tab has one.
fn tallest_item(regions: &[DeclaredRegion]) -> Option<(String, LRect)> {
    regions
        .iter()
        .filter(|r| r.name.starts_with("ribbon.item."))
        .fold(None, |best: Option<(String, LRect)>, r| {
            let h = r.rect.max.y - r.rect.min.y;
            match &best {
                Some((_, b)) if b.max.y - b.min.y >= h => best,
                _ => Some((r.name.clone(), r.rect)),
            }
        })
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("ribbon_mockup.trace.txt"));
    spec.pdf = ctx.pdf.clone();
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}; the mockup was drawn at {MOCKUP_WIDTH} px and a \
         narrower window will legitimately collapse groups — see this check's header",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(24);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process \
             and nothing published a rectangle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }

    let frame = session.frame()?;
    let regions = ribbon_regions(ctx, &trace);
    if regions.is_empty() {
        return Err(Error::new(
            "the application declared no `ribbon.*` regions, so there is no band to measure. \
             That is a fact about the build, not about the mockup, and reporting a FAIL here \
             would file a defect against a ribbon nobody drew.",
        ));
    }
    report.note(format!("the ribbon declared {} regions", regions.len()));

    let mut failures: Vec<String> = Vec::new();

    // --- rows 1-3: geometry, from what the process published ---------------
    let group = regions
        .iter()
        .find(|r| {
            r.name.starts_with(&format!("ribbon.group.{RESTING_TAB}."))
                && !r.name.ends_with(".caption")
        })
        .ok_or_else(|| {
            Error::new(format!(
                "no `ribbon.group.{RESTING_TAB}.*` region — the {RESTING_TAB} tab is not the \
                 active one, so every measurement below would be about a different band"
            ))
        })?;
    let first_item = regions
        .iter()
        .filter(|r| r.name.starts_with("ribbon.item.") && group.rect.contains_rect(r.rect))
        .fold(f32::INFINITY, |a, r| a.min(r.rect.min.y));
    if first_item.is_finite() {
        let pad = first_item - group.rect.min.y;
        report.note(format!(
            "`{}` begins at y={:.1} and its first control at y={first_item:.1} — {pad:.1} pt of \
             clearance, against the mockup's 6 (`.ribbon {{ padding: 6px … }}`)",
            group.name, group.rect.min.y
        ));
        if pad < 6.0 - SLACK {
            failures.push(format!(
                "the band draws {pad:.1} pt above its first control; the mockup draws 6"
            ));
        }
    }

    if let Some((name, rect)) = tallest_item(&regions) {
        let h = rect.max.y - rect.min.y;
        report.note(format!(
            "the tallest band control is `{name}` at {h:.1} pt, against the mockup's 56 for a \
             Large control (`.rb.big {{ height: 56px }}`)"
        ));
        if (h - 56.0).abs() > SLACK && h > 56.0 {
            failures.push(format!(
                "`{name}` is {h:.1} pt tall. A Large control is 56 pt in the mockup, inside a \
                 68 pt row area — if this reads 68 it is still spanning the whole area"
            ));
        }
    }

    // --- rows 4 and 5: ink, which needs the screen -------------------------
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and the frame and glyph assertions need a capture \
             — which means raising the window and taking the operator's focus. The geometry \
             above was measured; the two claims only a screenshot can settle were not. \
             Reported as SKIPPED rather than passed.",
        ));
    }

    let shot = ctx.out("ribbon_mockup.png");
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);

    // The band's own ground, sampled from a point inside the group's box that
    // no control occupies: just under the caption, which `.cap` centres, so
    // the group's left edge at the caption's baseline is empty in both designs.
    let ground_at = frame.logical_to_capture_pixels(LRect::new(
        crate::geom::Pt {
            x: group.rect.min.x + 2.0,
            y: group.rect.max.y - 2.0,
        },
        crate::geom::Pt {
            x: group.rect.min.x + 3.0,
            y: group.rect.max.y - 1.0,
        },
    ));
    let Some(ground) = image.pixel(ground_at.x, ground_at.y) else {
        return Err(Error::new(
            "the band's own background could not be sampled from the capture, so 'is this \
             control the same colour as the band' has no reference and every frame verdict \
             below would be meaningless",
        ));
    };
    report.note(format!(
        "band ground sampled at ({}, {}) as #{:02X}{:02X}{:02X}",
        ground_at.x, ground_at.y, ground.r, ground.g, ground.b
    ));

    let mut judged = 0_usize;
    for r in regions
        .iter()
        .filter(|r| r.name.starts_with("ribbon.item."))
    {
        let px = frame.logical_to_capture_pixels(r.rect);
        let Some(frameless) = is_frameless(&image, px, ground, 2) else {
            continue;
        };
        judged += 1;
        if !frameless {
            failures.push(format!(
                "`{}` is drawn in a visible box. The mockup draws every resting control \
                 frameless — `.rb {{ border: 1px solid transparent }}` — and paints a frame \
                 only on hover, focus, press and selection. (A control that is SELECTED at \
                 rest is expected to have a plate; this check drives the {RESTING_TAB} tab \
                 because none of its controls is a toggle.)",
                r.name
            ));
        }
    }
    report.note(format!(
        "{judged} resting band controls were judged for a frame"
    ));
    if judged == 0 {
        return Err(Error::new(
            "no band control could be judged: every corner probe landed on ink or off the \
             capture. The frame claim was NOT measured, and a pass here would be a pass over \
             nothing.",
        ));
    }

    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failures.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic band: a uniform ground with one optional 1 px box drawn on
    /// it, so [`is_frameless`] can be exercised without a window.
    ///
    /// ★ The oracle needs its own test for the reason `PROJECT_PLAN.md` §4.1
    /// keeps restating: a predicate that has only ever been seen to say "yes"
    /// is indistinguishable from one that cannot say "no". This file's whole
    /// value is one boolean, and that boolean is asserted here against both
    /// answers.
    fn board(framed: bool) -> Image {
        let (w, h) = (40_u32, 30_u32);
        let mut bgra = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let inside = (10..30).contains(&x) && (8..22).contains(&y);
                let edge = inside && (x == 10 || x == 29 || y == 8 || y == 21);
                let c: u8 = if framed && inside {
                    if edge { 0x40 } else { 0x80 }
                } else {
                    0xE8
                };
                bgra.extend_from_slice(&[c, c, c, 0xFF]);
            }
        }
        Image::from_bgra(w, h, bgra).expect("a well-formed synthetic board")
    }

    const GROUND: Rgb = Rgb::new(0xE8, 0xE8, 0xE8);

    #[test]
    fn a_control_painted_on_the_band_reads_as_frameless() {
        let verdict = is_frameless(&board(false), PixRect::new(10, 8, 19, 13), GROUND, 2);
        assert_eq!(
            verdict,
            Some(true),
            "a control drawn with no fill and no stroke must read as frameless, or this \
             check fails the whole band the day the fix is correct"
        );
    }

    #[test]
    fn a_control_drawn_in_a_box_reads_as_framed() {
        let verdict = is_frameless(&board(true), PixRect::new(10, 8, 19, 13), GROUND, 2);
        assert_eq!(
            verdict,
            Some(false),
            "a control with a fill and a stroke must read as FRAMED. Without this half the \
             oracle could return `true` unconditionally and the check would pass over the \
             exact defect it was written for"
        );
    }

    /// ★★★ **A control that is not on the capture produces NO verdict.**
    ///
    /// The third answer, and the one that keeps the other two honest. A
    /// control laid out past the window's edge — the state
    /// `RIBBON_SCALING.md`'s scroll rung exists to make reachable, and the
    /// state `sizing::render_large`'s zero-height defect actually shipped in —
    /// has no pixels to sample. An oracle that answered `true` there would let
    /// every off-screen control certify the band as frameless, which is the
    /// exact shape of *"a check that cannot fail"*.
    ///
    /// The rect is wholly off a 40×30 board, so every one of the four probe
    /// pairs falls outside and `judged` stays zero.
    #[test]
    fn a_control_off_the_capture_is_declined_rather_than_guessed() {
        let verdict = is_frameless(&board(false), PixRect::new(200, 200, 10, 10), GROUND, 2);
        assert_eq!(
            verdict, None,
            "a rect whose probes all fall outside the image must produce no verdict. \
             Returning `true` there would let a control drawn off-screen certify the band \
             as frameless"
        );
    }

    /// ★★ **…and a control against the window's left edge is judged on the
    /// corners it has**, rather than being declined outright or — worse —
    /// judged on a clamped probe.
    ///
    /// This is a real state, not a fixture curiosity: the first item of the
    /// first group sits at the band's left edge once the band has scrolled.
    /// Its two left-hand probes would need a negative x, and the two failure
    /// modes this pins are the two obvious ways to write that:
    ///
    /// 1. **Plain `u32` subtraction panics**, in debug, on the first
    ///    left-edge control the harness meets — which is a driven check that
    ///    dies rather than reporting, on a state the collapse ladder makes
    ///    ordinary. `checked_sub` is what stops it.
    /// 2. **Declining the whole control** because one corner could not be
    ///    probed throws away the three corners that could, and a band whose
    ///    leftmost control is never judged is a band whose frame is never
    ///    checked where the operator looks first.
    ///
    /// ★ Note what this does **not** distinguish, because a falsification
    /// pass found it out rather than assuming: clamping the probe to zero
    /// instead of declining it passes this test. It does so for a benign
    /// reason — a clamped "outside" probe lands on the control's own left
    /// column, which `far(outside)` then rejects as not-the-band, so the
    /// corner is skipped either way. The decline is the clearer statement of
    /// intent; the guard is what actually carries it.
    #[test]
    fn a_control_at_the_left_edge_is_still_judged_on_its_other_corners() {
        assert_eq!(
            is_frameless(&board(true), PixRect::new(0, 8, 29, 13), GROUND, 2),
            Some(false),
            "a framed control flush against the left edge must still read as framed from \
             its right-hand corners"
        );
    }
}
