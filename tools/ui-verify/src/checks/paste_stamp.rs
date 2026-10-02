//! `checks::paste_stamp` — **in Review, a picture another program copied
//! pastes as a stamp, by Ctrl+V at the pointer or by Markup ▸ Paste picture as
//! stamp**
//!
//! Drives the window off the desktop through the scripted pointer on a copy of
//! the engine corpus's `four-pages.pdf`, with the clipboard snapshotted and
//! restored as in [`super::os_image_paste`]. The clipboard holds a 64×32
//! bitmap at 96 pixels per inch, which is 48×24 pt.
//!
//! Oracles: in Review a paste traces `clip-pasted kind=image as=stamp` with a
//! 48×24 pt rectangle centred on the pointer, then `custom-stamp-placed`, and
//! Ctrl+Z traces `undo-applied`; the ribbon command traces a second
//! `as=stamp` line of the same size and another `custom-stamp-placed`; in Read
//! a paste traces no `clip-pasted` line and a `command-declined id=edit.paste`.

use super::os_image_paste::{self as osp, ClipGuard};
use crate::checks::driving::{declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys;

const STEM: &str = "paste-stamp";
const PLACED: &str = "custom-stamp-placed";
const DECLINED: &str = "command-declined";
const UNDONE: &str = "undo-applied";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.paste_image_stamp";
/// The bitmap's size on the page, in points.
const SIZE: (f64, f64) = (48.0, 24.0);
/// How far a size may differ from [`SIZE`], in points.
const TOLERANCE_PT: f64 = 0.5;

/// See the module documentation.
pub struct ACopiedPicturePastesAsAStampInReview;

impl Check for ACopiedPicturePastesAsAStampInReview {
    fn name(&self) -> &'static str {
        "a_copied_picture_pastes_as_a_stamp_in_review"
    }

    fn defect(&self) -> &'static str {
        "a picture copied in another program cannot be put on the page in Review, where pasting \
         content is refused and nothing offers it as a stamp"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = osp::launch(ctx, &mut report, STEM)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &mut guard));
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
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_DIB, osp::dib(64, 32))])?;
    switch(session, pointer, "2")?;
    let mut failure = by_chord(ctx, report, session, pointer)?;
    if failure.is_none() {
        failure = by_command(report, session, pointer)?;
    }
    if failure.is_none() {
        switch(session, pointer, "1")?;
        failure = read_refuses(ctx, session, pointer)?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Change mode with `Ctrl+digit`.
fn switch(session: &Session, pointer: &ScriptedPointer, digit: &str) -> Result<()> {
    pointer.key(session, None, digit, Some("ctrl"))?;
    session.settle(20);
    Ok(())
}

/// The `n`th `clip-pasted` line's rectangle, required to be a stamp of
/// [`SIZE`], and a `custom-stamp-placed` line after it.
fn stamped(
    session: &Session,
    n: usize,
    placed: usize,
) -> Result<std::result::Result<[f64; 4], String>> {
    let trace = session.trace()?;
    let Some(line) = trace.events(osp::PASTED).nth(n) else {
        return Ok(Err(format!("the paste traced no `{}` line.", osp::PASTED)));
    };
    if line.get("kind") != Some("image") || line.get("as") != Some("stamp") {
        return Ok(Err(format!(
            "the paste was kind={} as={}, not a picture as a stamp.",
            line.get("kind").unwrap_or("-"),
            line.get("as").unwrap_or("-")
        )));
    }
    let Some(r) = osp::rect(line) else {
        return Ok(Err(format!(
            "a `{}` line without its rectangle.",
            osp::PASTED
        )));
    };
    let (w, h) = (r[2] - r[0], r[3] - r[1]);
    if (w - SIZE.0).abs() > TOLERANCE_PT || (h - SIZE.1).abs() > TOLERANCE_PT {
        return Ok(Err(format!(
            "the stamp is {w:.2}×{h:.2} pt, not the picture's 48×24."
        )));
    }
    if trace.events(PLACED).count() == placed {
        return Ok(Err(format!(
            "the stamp was asked for but no `{PLACED}` line followed."
        )));
    }
    Ok(Ok(r))
}

/// Ctrl+V at the pointer stamps the picture there, and Ctrl+Z takes it back.
fn by_chord(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let (pasted, placed) = (
        osp::count(session, osp::PASTED)?,
        osp::count(session, PLACED)?,
    );
    pointer.hover(session, osp::at(ctx, session, osp::FIRST)?)?;
    pointer.paste(session, None, "x")?;
    session.settle(30);
    let r = match stamped(session, pasted, placed)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(format!("Ctrl+V in Review: {failure}"))),
    };
    report.note(format!("Ctrl+V stamp {}", osp::show(r)));
    if let Some(failure) = osp::lands(r, osp::FIRST, SIZE) {
        return Ok(Some(failure));
    }
    let undos = osp::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok(
        (osp::count(session, UNDONE)? == undos)
            .then(|| "Ctrl+Z did not undo the stamp.".to_owned()),
    )
}

/// Click the declared `region`.
fn press(session: &Session, pointer: &ScriptedPointer, region: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Markup items declared: {}.",
            list(&declared_names(&trace, UI_RECT, "ribbon.item.markup."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Markup ▸ Paste picture as stamp stamps the picture too.
fn by_command(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(session, pointer, TAB)?;
    let (pasted, placed) = (
        osp::count(session, osp::PASTED)?,
        osp::count(session, PLACED)?,
    );
    press(session, pointer, ITEM)?;
    Ok(match stamped(session, pasted, placed)? {
        Ok(r) => {
            report.note(format!("command stamp {}", osp::show(r)));
            None
        }
        Err(failure) => Some(format!("Paste picture as stamp: {failure}")),
    })
}

/// In Read a paste places nothing and is declined.
fn read_refuses(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let (pasted, declined) = (
        osp::count(session, osp::PASTED)?,
        osp::count(session, DECLINED)?,
    );
    pointer.hover(session, osp::at(ctx, session, osp::FIRST)?)?;
    pointer.paste(session, None, "x")?;
    session.settle(20);
    if osp::count(session, osp::PASTED)? != pasted {
        return Ok(Some("Read pasted the picture onto the page.".to_owned()));
    }
    let trace = session.trace()?;
    let refused = trace
        .events(DECLINED)
        .nth(declined)
        .is_some_and(|l| l.get("id") == Some("edit.paste"));
    Ok((!refused).then(|| format!("Read traced no `{DECLINED} id=edit.paste` line.")))
}
