//! `checks::signing::placement` — in the Sign here window's preview, a corner
//! drag resizes the signature with its proportions kept, a body drag moves
//! it, Reset to fit puts it back, and Place writes it where it was put.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/placement.md`.

use super::hand_sign::{ink_pixels, shoot, signing_area};
use super::picture_sign::{REGION_PLACE, open, undo_and_save};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::report::CheckReport;

const REGION_INK: &str = "handsign.ink";
const REGION_AREA: &str = "handsign.placement";
const REGION_RESET: &str = "handsign.reset-fit";

/// The fraction of the box's width, from its left edge, that must hold no
/// ink once the signature is halved and moved to the right end.
const LEFT_CLEAR: f32 = 0.4;

fn point(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

/// See the module documentation.
pub struct ASignatureLandsWhereItWasPut;

impl Check for ASignatureLandsWhereItWasPut {
    fn name(&self) -> &'static str {
        "a_signature_lands_where_it_was_put"
    }

    fn defect(&self) -> &'static str {
        "the Sign here window cannot resize or move the signature before it is placed, or Reset \
         to fit does not put it back, or Place ignores where the operator put it"
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

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let opened = match open(ctx, report, "placement")? {
        Ok(opened) => opened,
        Err(finding) => return Ok(Some(finding)),
    };
    let (session, pointer) = (&opened.session, &opened.pointer);
    let mut findings = Vec::new();

    // --- A: halve it by its lower-right corner ------------------------------
    let (fit, vp) = opened.region(REGION_INK)?;
    let (area, _) = opened.region(REGION_AREA)?;
    let half = point(
        fit.min.x + fit.width() / 2.0,
        fit.min.y + fit.height() / 2.0,
    );
    pointer.drag_in(
        session,
        vp.as_deref(),
        point(fit.max.x, fit.max.y),
        half,
        8,
        "l",
    )?;
    session.settle(15);
    let (halved, _) = opened.region(REGION_INK)?;
    let ratio = |r: LRect| r.width() / r.height().max(f32::EPSILON);
    if (halved.width() - fit.width() / 2.0).abs() > 3.0
        || (ratio(halved) - ratio(fit)).abs() > 0.1 * ratio(fit)
    {
        findings.push(format!(
            "a corner drag to the middle took the signature from {fit:?} to {halved:?}; half its \
             size with its proportions kept was expected."
        ));
    }

    // --- B: Reset to fit puts it back, then halve and move it right --------
    opened.press(REGION_RESET, 15)?;
    let (reset, _) = opened.region(REGION_INK)?;
    let reset_traced = session
        .trace()?
        .events("hand-sign-placement")
        .any(|l| l.get_usize("reset") == Some(1));
    if (reset.min.x - fit.min.x).abs() > 1.0 || (reset.width() - fit.width()).abs() > 1.0 {
        findings.push(format!(
            "after Reset to fit the signature is at {reset:?}; the fit at {fit:?} was expected."
        ));
    }
    pointer.drag_in(
        session,
        vp.as_deref(),
        point(fit.max.x, fit.max.y),
        half,
        8,
        "l",
    )?;
    session.settle(15);
    let (small, _) = opened.region(REGION_INK)?;
    let centre = point(
        small.min.x + small.width() / 2.0,
        small.min.y + small.height() / 2.0,
    );
    let shift = area.max.x - small.max.x;
    let to = point(
        small.min.x + small.width() / 2.0 + shift,
        small.min.y + small.height() / 2.0,
    );
    pointer.drag_in(session, vp.as_deref(), centre, to, 8, "l")?;
    session.settle(15);
    let (moved, _) = opened.region(REGION_INK)?;
    let drags = session
        .trace()?
        .events("hand-sign-placement")
        .filter(|l| l.get("grip").is_some())
        .count();

    // --- C: Place, and the ink is where it was put -----------------------
    opened.press(REGION_PLACE, 40)?;
    let requested = session
        .trace()?
        .events("hand-sign-requested")
        .last()
        .and_then(|l| l.get("placement").map(str::to_owned));
    let first = opened.first;
    let area_box = signing_area(first);
    let left = LRect::new(
        area_box.min,
        Pt::new(first.min.x + LEFT_CLEAR * first.width(), area_box.max.y),
    );
    let after = shoot(ctx, pointer, session, report, "placement-after.png")?;
    let ink_left = ink_pixels(&after, session, left)?;
    let ink_all = ink_pixels(&after, session, area_box)?;
    let (ink_undone, _) = undo_and_save(ctx, report, &opened, "placement")?;
    drop(opened);

    report.note(format!(
        "preview area {area:?}; fit {fit:?}; halved {halved:?}; reset {reset:?} (traced \
         {reset_traced}); moved {moved:?}; grip drags traced {drags}; requested \
         placement={requested:?}; box {first:?}; ink in the box's left {LEFT_CLEAR} \
         {ink_left} of {ink_all}; after undo {ink_undone}"
    ));
    if !reset_traced {
        findings.push("Reset to fit traced no `hand-sign-placement reset`.".to_owned());
    }
    if moved.min.x - small.min.x < small.width() / 2.0 {
        findings.push(format!(
            "a body drag toward the right end took the signature from {small:?} to {moved:?}; \
             it should have moved right by more than half its width."
        ));
    }
    if drags < 3 {
        findings.push(format!(
            "{drags} grip drags traced; three (two corners, one body) were made."
        ));
    }
    if requested.as_deref() != Some("chosen") {
        findings.push(format!(
            "Place requested placement={requested:?}; the chosen placement was expected."
        ));
    }
    if ink_all < 10 {
        findings.push(format!(
            "only {ink_all} ink pixels in or above the box after Place: nothing landed."
        ));
    }
    if ink_left != 0 {
        findings.push(format!(
            "{ink_left} ink pixels in the left {LEFT_CLEAR} of the box: the signature landed by \
             the fit rule, not where it was put."
        ));
    }
    if ink_undone != 0 {
        findings.push(format!(
            "{ink_undone} ink pixels remain after Ctrl+Z: one undo did not remove it."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
