//! `ribbon_matches_the_mockup_geometry` — the band, measured against
//! `mockups/pdfcer-shell.html`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ribbon_mockup.md`.

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
const RESTING_TAB: &str = "file";

/// The width the mockup was rendered at.
const MOCKUP_WIDTH: u32 = 1700;

/// How far a measured figure may sit from the mockup's, in points.
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
/// `inset` is 2 px rather than 1: `egui` rounds a button's corners
/// (`Metrics::corner_radius`, 3 pt in `Quiet`), so the literal corner pixel of
/// the rectangle is outside the painted shape even when a frame IS drawn, and
/// a one-pixel probe would report every framed control as frameless.
///
/// # `ground` is the load-bearing argument, and a wrong one makes this
/// # function measure NOTHING while looking exactly like a working oracle
///
/// Every verdict here is a comparison against `ground`, and both branches use
/// it: `far(outside)` **discards** the pair, `far(inside)` **convicts** it. So
/// a `ground` sampled from anything that is not the band makes `far` true
/// almost everywhere, every pair is discarded, `judged` reaches zero and the
/// function returns `None` for the entire band.
///
/// That happened on 2026-09-05 and it is written up at the caller, where the
/// sampling point lives: the reference was taken from inside a **collapsed
/// group's plate**, `#E8E8EA` against the band's `#F2F2F3` — a channel-sum
/// distance of **29 against this function's threshold of 24.** Ten points of
/// grey, and the check went from PASS to *"0 resting band controls were judged
/// for a frame"*.
///
/// ⇒ **The `None` return is what made that visible at all**, and it is worth
/// keeping for that reason alone. A version answering `true` when nothing could
/// be measured would have reported the band frameless — the very claim the
/// check exists to establish — on a run that had measured no pixels.
///
/// A hypothesis that was **falsified by driving**, recorded because the
/// reasoning was plausible and wrong: when the band began stacking controls
/// into columns one point apart, the diagonal outside probe looked certain to
/// land on the neighbour above. It does not — the diagonal steps sideways as
/// well as up, out of the control's own x range and into the group's padding,
/// which is band. Reverted to the diagonal after a driven run with the fixed
/// `ground` judged **11** controls either way. *A layout change is not
/// automatically the cause of a probe that stopped measuring; find the
/// reference first.*
#[must_use]
pub fn is_frameless(image: &Image, rect: PixRect, ground: Rgb, inset: u32) -> Option<bool> {
    let far = |c: Rgb| {
        let d = |a: u8, b: u8| i32::from(a).abs_diff(i32::from(b));
        d(c.r, ground.r) + d(c.g, ground.g) + d(c.b, ground.b) > 24
    };
    // Every probe is `checked_sub`, and a corner whose outside sample would
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

    //
    // A group the collapse ladder folded publishes
    // `ribbon.group.<tab>.<id>.collapsed`: a captioned BUTTON, with a plate
    // painted under it and **no items inside it at all**. It is a `ribbon.group`
    // region by name and it is not a band group by nature, and this `find` took
    // the first match.
    //
    // Both things below then went quietly wrong, in different ways:
    //
    // * The 6 pt clearance assertion found no items inside the rect, so
    //   `first_item` was infinite and the whole check simply **did not run** —
    //   no note, no failure, no skip. An assertion that vanishes when its
    //   subject is absent is the shape this harness exists to remove.
    // * The band ground was sampled from inside the collapsed group's **plate**
    //   — `#E8E8EA` against the band's own `#F2F2F3`, a channel-sum distance of
    //   **29 against a `far` threshold of 24.** Every real band pixel was then
    //   "not the background", every probe pair was discarded, and the check
    //   reported *"0 resting band controls were judged for a frame"* — a pixel
    //   oracle measuring nothing, on a run that had PASSED the sweep before.
    //
    // ⇒ **Ten points of grey.** The failure is not that the reference was
    // wrong by a lot; it is that a reference sampled from *whatever region
    // happened to be first* has no reason to be the band at all.
    let group = regions
        .iter()
        .find(|r| {
            r.name.starts_with(&format!("ribbon.group.{RESTING_TAB}."))
                && !r.name.ends_with(".caption")
                && !r.name.ends_with(".collapsed")
        })
        .ok_or_else(|| {
            Error::new(format!(
                "no OPEN `ribbon.group.{RESTING_TAB}.*` region — either the {RESTING_TAB} tab \
                 is not the active one, so every measurement below would be about a different \
                 band, or every group on it is collapsed, in which case there are no resting \
                 controls to judge"
            ))
        })?;
    report.note(format!("measuring against `{}`", group.name));
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

    /// **A control that is not on the capture produces NO verdict.**
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

    /// **…and a control against the window's left edge is judged on the
    /// corners it has**, rather than being declined outright or — worse —
    /// judged on a clamped probe.
    #[test]
    fn a_control_at_the_left_edge_is_still_judged_on_its_other_corners() {
        assert_eq!(
            is_frameless(&board(true), PixRect::new(0, 8, 29, 13), GROUND, 2),
            Some(false),
            "a framed control flush against the left edge must still read as framed from \
             its right-hand corners"
        );
    }

    /// **A `ground` that is not the band's colour makes this REFUSE, and
    /// the margin is ten points of grey.**
    #[test]
    fn a_ground_taken_from_a_collapsed_groups_plate_produces_no_verdict() {
        // The two colours measured off the capture on the day, and their
        // distance in this function's own metric. `board(false)` paints a
        // uniform field within 2 of `PLATE`, so it stands for the plate; the
        // reference handed in is the band. The incident ran the other way
        // round — plate as reference, band as pixels — and the property is
        // symmetric in exactly the way that makes the direction not matter.
        const PLATE: Rgb = Rgb::new(0xE8, 0xE8, 0xEA);
        const BAND: Rgb = Rgb::new(0xF2, 0xF2, 0xF3);
        let distance = |a: Rgb, b: Rgb| {
            let d = |x: u8, y: u8| i32::from(x).abs_diff(i32::from(y));
            d(a.r, b.r) + d(a.g, b.g) + d(a.b, b.b)
        };
        assert_eq!(
            distance(PLATE, BAND),
            29,
            "the measured distance between a collapsed group's plate and the band. If a \
             theme change brings this under `far`'s threshold of 24 the incident stops \
             being reachable — and this line is where that is noticed, rather than in a \
             check that has quietly gone vacuous"
        );

        let rect = PixRect::new(10, 8, 19, 13);
        assert_eq!(
            is_frameless(&board(false), rect, GROUND, 2),
            Some(true),
            "the control is frameless when the reference IS the surface it sits on"
        );
        assert_eq!(
            is_frameless(&board(false), rect, BAND, 2),
            None,
            "with a reference {distance} points from the pixels — a plate against a band, \
             or a band against a plate — every probe pair must be discarded and the \
             function must REFUSE. Returning `Some(true)` here would certify the whole \
             band as frameless while measuring no pixels at all",
            distance = distance(PLATE, BAND)
        );
    }
}
