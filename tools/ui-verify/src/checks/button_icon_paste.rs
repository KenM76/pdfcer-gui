//! `checks::button_icon_paste` — **a picture copied in another program and
//! pasted while a push button is selected becomes the button's picture**
//!
//! Drives `all-field-kinds.pdf`'s push button `PushOne`, selected in Edit,
//! with a 64×32 bitmap put on the OS clipboard by [`ClipGuard`], which hands
//! the operator's clipboard back afterwards.
//!
//! Oracles: Ctrl+V traces `clip-pasted … as=button-icon field=PushOne` and no
//! `as=content`, applies one `edit-widget-applied redrawn=yes`, and the panel
//! re-reads `button-icon-shown icon=present`; Ctrl+Z traces `undo-applied` and
//! the panel re-reads `icon=absent`.

use super::button_icon::{self as bi, APPLIED, FIELD};
use super::os_image_paste::{self as osp, ClipGuard};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys;

const STEM: &str = "button-icon-paste";
const UNDONE: &str = "undo-applied";

/// See the module documentation.
pub struct APicturePastesOntoASelectedPushButton;

impl Check for APicturePastesOntoASelectedPushButton {
    fn name(&self) -> &'static str {
        "a_picture_pastes_onto_a_selected_push_button"
    }

    fn defect(&self) -> &'static str {
        "a picture pasted while a push button is selected lands on the page as a separate \
         image instead of becoming the button's picture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = drive(ctx, &mut report, &mut guard);
        report.note(guard.release());
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    let png = crate::png::encode_rgb(4, 4, &[90u8; 4 * 4 * 3])
        .ok_or_else(|| Error::new("the harness's own PNG encoder refused its fixture"))?;
    let path = ctx.out("button-icon-paste.png");
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    let (session, pointer) = bi::launch(ctx, report, STEM, &path)?;
    let outcome = steps(ctx, report, &session, &pointer, guard);
    bi::finish(&session, &pointer, outcome)
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    if bi::shown(session)?.is_none_or(|(icon, _)| icon != "absent") {
        return Ok(Some(format!(
            "before the paste the panel traced no `button-icon-shown field={FIELD} icon=absent` \
             line."
        )));
    }
    guard.set(&[(sys::CF_DIB, osp::dib(64, 32))])?;
    let applied = bi::count(session, APPLIED)?;
    pointer.hover(session, osp::at(ctx, session, osp::FIRST)?)?;
    pointer.paste(session, None, "x")?;
    session.settle(20);
    if let Some(failure) = pasted(report, session, applied)? {
        return Ok(Some(failure));
    }
    let undos = bi::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    let after = bi::shown(session)?;
    report.note(format!("after Ctrl+Z: {after:?}"));
    if bi::count(session, UNDONE)? == undos {
        return Ok(Some("Ctrl+Z traced no `undo-applied`.".to_owned()));
    }
    Ok(after
        .as_ref()
        .is_none_or(|(icon, _)| icon != "absent")
        .then(|| format!("★★★ Ctrl+Z left the pasted picture on the button: {after:?}.")))
}

/// `None` when the paste became the button's picture and nothing else.
fn pasted(report: &mut CheckReport, session: &Session, applied: usize) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.events(osp::PASTED).last();
    report.note(format!("paste: {:?}", line.map(|l| l.raw.clone())));
    let Some(line) = line else {
        return Ok(Some(format!("Ctrl+V traced no `{}` line.", osp::PASTED)));
    };
    if line.get("as") != Some("button-icon") || line.get("field") != Some(FIELD) {
        return Ok(Some(format!(
            "★★★ the picture was not pasted onto the selected button: `{}`.",
            line.raw
        )));
    }
    let applies: Vec<_> = trace.events(APPLIED).skip(applied).collect();
    if applies.len() != 1 || applies[0].get("redrawn") != Some("yes") {
        return Ok(Some(format!(
            "★★★ the paste applied {} widget edits, wanted one with `redrawn=yes`.",
            applies.len()
        )));
    }
    let after = bi::shown(session)?;
    report.note(format!("after Ctrl+V: {after:?}"));
    Ok(after
        .as_ref()
        .is_none_or(|(icon, _)| icon != "present")
        .then(|| format!("★★★ the panel does not show the pasted picture: {after:?}.")))
}
