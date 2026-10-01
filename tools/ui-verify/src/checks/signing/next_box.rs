//! `checks::signing::next_box` — the signing strip counts the signature boxes,
//! its Next button brings each box still to sign into view in document order,
//! and a box signed by drawing is counted and skipped.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/next_box.md`.

use super::hand_sign::shoot;
use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "esign-three-pages.pdf";
const REGION_STRIP: &str = "signstrip";
const REGION_NEXT: &str = "signstrip.next";
const REGION_CANVAS: &str = "canvas-viewport";
const BOX_PREFIX: &str = "form.sign-box.";
const REGION_PAD: &str = "handsign.pad";
const REGION_PLACE: &str = "handsign.place";

/// See the module documentation.
pub struct NextReachesEveryBoxToSign;

impl Check for NextReachesEveryBoxToSign {
    fn name(&self) -> &'static str {
        "next_reaches_every_box_to_sign"
    }

    fn defect(&self) -> &'static str {
        "the signing strip is missing or miscounts the signature boxes, or Next does not bring \
         the next box still to sign into view in document order, or it lands on a box already \
         signed"
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

fn at(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

/// The last `sign-strip` count, as `(signed, total)`.
fn tally(trace: &Trace) -> Option<(usize, usize)> {
    let line = trace.events("sign-strip").last()?;
    Some((line.get_usize("signed")?, line.get_usize("total")?))
}

/// Which box tags are drawn wholly inside the canvas viewport now.
fn on_screen(trace: &Trace, ui_rect: &str) -> Vec<usize> {
    let Some(canvas) = declared(trace, ui_rect, REGION_CANVAS) else {
        return Vec::new();
    };
    let mut out: Vec<usize> = declared_names(trace, ui_rect, BOX_PREFIX)
        .iter()
        .filter(|name| declared(trace, ui_rect, name).is_some_and(|r| canvas.contains_rect(r)))
        .filter_map(|name| name.strip_prefix(BOX_PREFIX)?.parse().ok())
        .collect();
    out.sort_unstable();
    out
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let source = repo_fixture(FIXTURE, "Run fixtures/esign-three-pages.PROVENANCE.py.")?;
    let doc = ctx.out("next-box-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("next-box.trace.txt"));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("next-box.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let region = |name: &str| -> Result<(LRect, Option<String>)> {
        let trace = session.trace()?;
        declared_in(&trace, ui_rect, name).ok_or_else(|| {
            let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
            Error::new(format!(
                "no `{name}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })
    };
    let mut findings = Vec::new();

    // --- A: the strip sits above the canvas and counts three --------------
    let Some((strip, _)) = declared_in(&session.trace()?, ui_rect, REGION_STRIP) else {
        pointer.gone(&session)?;
        return Ok(Some(
            "a document with three unsigned signature boxes opened with no signing strip \
             (no `signstrip` region)."
                .to_owned(),
        ));
    };
    let (canvas, _) = region(REGION_CANVAS)?;
    let start = tally(&session.trace()?);
    let visible_at_start = on_screen(&session.trace()?, ui_rect);
    if strip.max.y > canvas.min.y + 0.5 {
        findings.push(format!(
            "the strip at {strip:?} overlaps the canvas at {canvas:?}: it must sit above the page."
        ));
    }
    if start != Some((0, 3)) {
        findings.push(format!(
            "on opening the strip counted (signed, total)={start:?}; (0, 3) was expected."
        ));
    }
    if visible_at_start.contains(&2) {
        findings.push(
            "the third page's box is on screen at opening, so the fixture cannot show Next \
             scrolling to it."
                .to_owned(),
        );
    }
    let _ = shoot(ctx, &pointer, &session, report, "next-box-open.png")?;

    // --- B: Next walks 0, 1, 2, each brought on screen ---------------------
    let press_next =
        |report: &mut CheckReport, shot: &str| -> Result<(Option<usize>, Vec<usize>)> {
            let mark = session.trace()?.mark();
            // A missing button is a finding: the walk records no index.
            let Some((next, next_vp)) = declared_in(&session.trace()?, ui_rect, REGION_NEXT) else {
                return Ok((None, Vec::new()));
            };
            pointer.click_in(&session, next_vp.as_deref(), WindowPoint::centre_of(next))?;
            session.settle(40);
            let trace = session.trace()?;
            let index = trace
                .last_after("sign-next", mark)
                .and_then(|l| l.get_usize("index"));
            let shown = on_screen(&trace, ui_rect);
            let _ = shoot(ctx, &pointer, &session, report, shot)?;
            Ok((index, shown))
        };
    let mut walk = Vec::new();
    for (step, shot) in ["next-box-1.png", "next-box-2.png", "next-box-3.png"]
        .iter()
        .enumerate()
    {
        let (index, shown) = press_next(report, shot)?;
        walk.push((index, shown.clone()));
        match index {
            Some(i) if i == step && shown.contains(&i) => {}
            other => findings.push(format!(
                "Next press {} went to box {other:?} with boxes {shown:?} on screen; box {step} \
                 brought into view was expected.",
                step + 1
            )),
        }
    }

    // --- C: sign the second box by drawing --------------------------------
    // Back to box 1 (wrap from 2 to 0, then 1), then click it.
    let _ = press_next(report, "next-box-wrap.png")?;
    let (index, _) = press_next(report, "next-box-back.png")?;
    let mut counted = None;
    let mut after_sign = Vec::new();
    let trace = session.trace()?;
    let box_one = declared(&trace, ui_rect, "form.sign-box.1")
        .filter(|r| declared(&trace, ui_rect, REGION_CANVAS).is_some_and(|c| c.contains_rect(*r)));
    if index == Some(1)
        && let Some(target) = box_one
    {
        pointer.click(&session, WindowPoint::centre_of(target))?;
        session.settle(30);
        let trace = session.trace()?;
        if let (Some((pad, vp)), Some(_)) = (
            declared_in(&trace, ui_rect, REGION_PAD),
            declared_in(&trace, ui_rect, REGION_PLACE),
        ) {
            let (x0, y0, w, h) = (pad.min.x, pad.min.y, pad.width(), pad.height());
            pointer.drag_in(
                &session,
                vp.as_deref(),
                at(x0 + 0.15 * w, y0 + 0.70 * h),
                at(x0 + 0.85 * w, y0 + 0.35 * h),
                12,
                "l",
            )?;
            session.settle(10);
            let (place, place_vp) = region(REGION_PLACE)?;
            pointer.click_in(&session, place_vp.as_deref(), WindowPoint::centre_of(place))?;
            session.settle(40);
            counted = tally(&session.trace()?);
            // --- D: Next from the signed box skips it: 2, then 0, then 2 ---
            for shot in [
                "next-box-after-1.png",
                "next-box-after-2.png",
                "next-box-after-3.png",
            ] {
                after_sign.push(press_next(report, shot)?.0);
            }
        } else {
            findings.push(
                "a click on box 1's tag opened no Sign here window, so the signing step could \
                 not run."
                    .to_owned(),
            );
        }
    } else {
        findings.push(format!(
            "after wrapping, Next went to box {index:?} with box 1 at {box_one:?} inside the \
             canvas; box 1 on screen was expected, so the signing step could not run."
        ));
    }
    pointer.gone(&session)?;
    drop(session);

    report.note(format!(
        "strip {strip:?}, canvas {canvas:?}; count at open {start:?}; on screen at open \
         {visible_at_start:?}; walk (index, on screen)={walk:?}; count after signing box 1 \
         {counted:?}; Next after signing went to {after_sign:?}"
    ));
    if counted != Some((1, 3)) {
        findings.push(format!(
            "after drawing a signature into box 1 the strip counted (signed, total)={counted:?}; \
             (1, 3) was expected."
        ));
    }
    if after_sign != [Some(2), Some(0), Some(2)] {
        findings.push(format!(
            "after box 1 was signed, Next went to {after_sign:?}; [2, 0, 2] was expected: the \
             signed box must be skipped."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
