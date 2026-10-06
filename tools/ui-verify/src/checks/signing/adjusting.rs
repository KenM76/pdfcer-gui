//! `checks::signing::adjusting` — a hand signature already in its box is
//! selected by a click on the page, resized by a corner grip and moved by its
//! body, kept inside the room its box allows, each change one undo, and still
//! tagged as the box's signature when saved.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/adjusting.md`.

use super::hand_sign::{count, ink_pixels, shoot, signing_area};
use super::picture_sign::{REGION_PLACE, open, undo_and_save};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::report::CheckReport;

const REGION_MARK: &str = "form.hand-sig";
const REGION_SELECTED: &str = "form.hand-sig.selected";

/// The fraction of the box's width, from its left edge, that must hold no
/// ink once the signature is pushed past the box's right end.
const LEFT_CLEAR: f32 = 0.4;

fn point(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

/// See the module documentation.
pub struct APlacedSignatureMovesAndResizes;

impl Check for APlacedSignatureMovesAndResizes {
    fn name(&self) -> &'static str {
        "a_placed_signature_moves_and_resizes"
    }

    fn defect(&self) -> &'static str {
        "a hand signature already placed cannot be selected on the page, or its grips do not \
         resize it, or dragging it does not move it, or it leaves the room its box allows, or a \
         change is not one undo, or it stops counting as the box's signature"
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

/// The last `hand-sign-adjusted` line: (tagged, undo depth).
fn adjusted(
    opened: &super::picture_sign::Opened,
) -> Result<Option<(Option<usize>, Option<usize>)>> {
    Ok(opened
        .session
        .trace()?
        .events("hand-sign-adjusted")
        .last()
        .map(|l| (l.get_usize("tagged"), l.get_usize("undo_depth"))))
}

/// The first hand signature's rectangle on the page, `None` when there is
/// none: an absence is a finding, never a reason to skip.
fn mark_now(opened: &super::picture_sign::Opened) -> Option<LRect> {
    opened.region(REGION_MARK).ok().map(|(r, _)| r)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let opened = match open(ctx, report, "adjusting")? {
        Ok(opened) => opened,
        Err(finding) => return Ok(Some(finding)),
    };
    opened.press(REGION_PLACE, 40)?;
    let (session, pointer) = (&opened.session, &opened.pointer);
    let mut findings = Vec::new();
    let placed_depth = session
        .trace()?
        .events("hand-sign-placed")
        .last()
        .and_then(|l| l.get_usize("undo_depth"));
    let Ok((mark, vp)) = opened.region(REGION_MARK) else {
        pointer.gone(session)?;
        return Ok(Some(
            "after Place no hand signature is offered on the page (no `form.hand-sig`).".to_owned(),
        ));
    };

    // --- A: a click selects it ---------------------------------------------
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(mark))?;
    session.settle(20);
    let selected = opened.region(REGION_SELECTED).ok().map(|(r, _)| r);

    // --- B: the lower-right grip, dragged to the middle, halves it ---------
    let centre = point(
        mark.min.x + mark.width() / 2.0,
        mark.min.y + mark.height() / 2.0,
    );
    pointer.drag_in(
        session,
        vp.as_deref(),
        point(mark.max.x, mark.max.y),
        centre,
        8,
        "l",
    )?;
    session.settle(40);
    let resize = adjusted(&opened)?;
    let Some(halved) = mark_now(&opened) else {
        pointer.gone(session)?;
        return Ok(Some(format!(
            "after the corner drag no hand signature is on the page; the corner drag traced              (tagged, undo depth)={resize:?}."
        )));
    };

    // --- C: the body, dragged far past the box's right end, stops at it ----
    let from = point(
        halved.min.x + halved.width() / 2.0,
        halved.min.y + halved.height() / 2.0,
    );
    let to = point(
        halved.min.x + halved.width() / 2.0 + 3.0 * opened.first.width(),
        halved.min.y + halved.height() / 2.0,
    );
    pointer.drag_in(session, vp.as_deref(), from, to, 10, "l")?;
    session.settle(40);
    let moved_line = adjusted(&opened)?;
    let Some(moved) = mark_now(&opened) else {
        pointer.gone(session)?;
        return Ok(Some(format!(
            "after the body drag no hand signature is on the page; it traced (tagged, undo depth)={moved_line:?}."
        )));
    };
    let first = opened.first;
    let area = signing_area(first);
    let left = LRect::new(
        area.min,
        Pt::new(first.min.x + LEFT_CLEAR * first.width(), area.max.y),
    );
    let after = shoot(ctx, pointer, session, report, "adjusting-after.png")?;
    let ink_left = ink_pixels(&after, session, left)?;
    let ink_all = ink_pixels(&after, session, area)?;

    // --- D: Ctrl+Z takes back the move alone --------------------------------
    pointer.gone(session)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let back = mark_now(&opened);
    pointer.key(session, None, "Z", Some("ctrl+shift"))?;
    session.settle(40);
    let (_, bytes) = undo_and_save(ctx, report, &opened, "adjusting")?;
    drop(opened);
    let tags = count(&bytes, b"/pdfc_HandSig");

    report.note(format!(
        "box {first:?}; mark {mark:?}; selected {selected:?}; placed at undo depth \
         {placed_depth:?}; resize (tagged, depth)={resize:?} -> {halved:?}; move (tagged, \
         depth)={moved_line:?} -> {moved:?}; ink in the box's left {LEFT_CLEAR} {ink_left} of \
         {ink_all}; after Ctrl+Z {back:?}; saved /pdfc_HandSig x{tags}"
    ));
    if selected.is_none() {
        findings.push(
            "a click on the placed signature selected nothing (no `form.hand-sig.selected`)."
                .to_owned(),
        );
    }
    let next = |d: Option<usize>, by: usize| placed_depth.map(|p| p + by) == d;
    match resize {
        Some((Some(1), depth)) if next(depth, 1) => {}
        other => findings.push(format!(
            "the corner drag traced (tagged, undo depth)={other:?}; tagged at depth {:?} was \
             expected.",
            placed_depth.map(|p| p + 1)
        )),
    }
    let ratio = |r: LRect| r.width() / r.height().max(f32::EPSILON);
    if (halved.width() - mark.width() / 2.0).abs() > 0.15 * mark.width()
        || (ratio(halved) - ratio(mark)).abs() > 0.15 * ratio(mark)
    {
        findings.push(format!(
            "the corner drag to the middle took the signature from {mark:?} to {halved:?}; half \
             its size with its proportions kept was expected."
        ));
    }
    match moved_line {
        Some((Some(1), depth)) if next(depth, 2) => {}
        other => findings.push(format!(
            "the body drag traced (tagged, undo depth)={other:?}; tagged at depth {:?} was \
             expected.",
            placed_depth.map(|p| p + 2)
        )),
    }
    if moved.max.x > first.max.x + 1.5 || moved.max.x < first.max.x - 3.0 {
        findings.push(format!(
            "dragged far past the box's right end the signature is at {moved:?}; its right edge \
             at the box's, {:.1}, was expected.",
            first.max.x
        ));
    }
    if ink_all < 10 || ink_left != 0 {
        findings.push(format!(
            "after the move the page shows {ink_left} ink pixels in the box's left \
             {LEFT_CLEAR} and {ink_all} in all: the content did not move where the outline did."
        ));
    }
    if back.is_none_or(|b| {
        (b.min.x - halved.min.x).abs() > 1.5 || (b.width() - halved.width()).abs() > 1.5
    }) {
        findings.push(format!(
            "after Ctrl+Z the signature is at {back:?}; back at {halved:?} (the move undone, the \
             resize kept) was expected."
        ));
    }
    if tags == 0 {
        findings.push(
            "the saved copy holds no `/pdfc_HandSig` tag: adjusting untagged the signature."
                .to_owned(),
        );
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
