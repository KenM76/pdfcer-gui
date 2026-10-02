//! `checks::dropped_file` — **a picture dropped on a page lands where it was
//! dropped**
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step, in Edit mode on a copy of the engine corpus's `four-pages.pdf`, with
//! 48×24-pixel PNGs that declare no resolution (so 48×24 pt).
//!
//! Oracles are the app's `image-dropped` lines: one picture is centred within
//! the tolerance of the drop point and Ctrl+Z traces `undo-applied`; two
//! pictures dropped together land at the drop point and one cascade step down
//! and right of it; a GIF traces `drop-refused ext=gif` and places nothing;
//! with Alt held nothing is placed and the placement window's
//! `dialog:insert-image` region is declared.

use super::os_image_paste as osp;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

const STEM: &str = "dropped-file";
const PLACED: &str = "image-dropped";
const REFUSED: &str = "drop-refused";
const UNDONE: &str = "undo-applied";
const PLACEMENT_REGION: &str = "dialog:insert-image";
/// The app's cascade step, `app::dropped::CASCADE_PT`.
const CASCADE_PT: f64 = 18.0;
const SIZE: (f64, f64) = (48.0, 24.0);
const SECOND: (f64, f64) = (300.0, 300.0);

/// See the module documentation.
pub struct ADroppedPictureLandsWhereItWasDropped;

impl Check for ADroppedPictureLandsWhereItWasDropped {
    fn name(&self) -> &'static str {
        "a_dropped_picture_lands_where_it_was_dropped"
    }

    fn defect(&self) -> &'static str {
        "a picture dragged onto a page opens a dialog and lands at the page's centre instead of \
         where it was dropped, and only the first of several is used"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = fixtures(ctx).and_then(|files| {
            osp::launch(ctx, &mut report, STEM)
                .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &files))
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Two PNGs and a GIF, written beside the trace.
fn fixtures(ctx: &CheckContext) -> Result<[PathBuf; 3]> {
    let png = crate::png::encode_rgb(48, 24, &[90u8; 48 * 24 * 3])
        .ok_or_else(|| Error::new("the harness's own PNG encoder refused its fixture"))?;
    let files = [
        ctx.out("dropped-a.png"),
        ctx.out("dropped-b.png"),
        ctx.out("dropped-c.gif"),
    ];
    for (path, bytes) in files.iter().zip([&png[..], &png[..], b"GIF89a"]) {
        std::fs::write(path, bytes)
            .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    }
    Ok(files)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    files: &[PathBuf; 3],
) -> Result<Option<String>> {
    let [a, b, gif] = files;
    let mut failure = one(ctx, report, session, pointer, a)?;
    if failure.is_none() {
        failure = two(ctx, report, session, pointer, a, b)?;
    }
    if failure.is_none() {
        failure = refused(ctx, session, pointer, gif)?;
    }
    if failure.is_none() {
        failure = alt_opens_the_window(ctx, session, pointer, a)?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Drop `paths` at `p` with `mods`; the rectangles of the `image-dropped`
/// lines it added.
fn drop_at(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    p: (f64, f64),
    mods: Option<&str>,
    paths: &[&Path],
) -> Result<Vec<[f64; 4]>> {
    let before = osp::count(session, PLACED)?;
    pointer.drop_files(session, osp::at(ctx, session, p)?, mods, paths)?;
    session.settle(20);
    let trace = session.trace()?;
    Ok(trace
        .events(PLACED)
        .skip(before)
        .filter_map(osp::rect)
        .collect())
}

/// One picture lands centred on the drop point, then undoes.
fn one(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    a: &Path,
) -> Result<Option<String>> {
    let placed = drop_at(ctx, session, pointer, osp::FIRST, None, &[a])?;
    let [r] = placed[..] else {
        return Ok(Some(format!(
            "one dropped picture traced {} `{PLACED}` line(s), not 1.",
            placed.len()
        )));
    };
    report.note(format!("one picture {}", osp::show(r)));
    if let Some(failure) = osp::lands(r, osp::FIRST, SIZE) {
        return Ok(Some(failure));
    }
    let undos = osp::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok((osp::count(session, UNDONE)? == undos).then(|| "Ctrl+Z did not undo the drop.".to_owned()))
}

/// Two pictures land at the drop point and one cascade step from it.
fn two(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    a: &Path,
    b: &Path,
) -> Result<Option<String>> {
    let placed = drop_at(ctx, session, pointer, SECOND, None, &[a, b])?;
    let [first, second] = placed[..] else {
        return Ok(Some(format!(
            "two dropped pictures traced {} `{PLACED}` line(s), not 2.",
            placed.len()
        )));
    };
    report.note(format!(
        "two pictures {} and {}",
        osp::show(first),
        osp::show(second)
    ));
    let next = (SECOND.0 + CASCADE_PT, SECOND.1 - CASCADE_PT);
    Ok(osp::lands(first, SECOND, SIZE)
        .or_else(|| osp::lands(second, next, SIZE).map(|f| format!("the second picture: {f}"))))
}

/// A GIF is named back and nothing is placed.
fn refused(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    gif: &Path,
) -> Result<Option<String>> {
    let refusals = osp::count(session, REFUSED)?;
    let placed = drop_at(ctx, session, pointer, osp::FIRST, None, &[gif])?;
    if !placed.is_empty() {
        return Ok(Some("a dropped GIF was placed on the page.".to_owned()));
    }
    let trace = session.trace()?;
    let named = trace
        .events(REFUSED)
        .nth(refusals)
        .is_some_and(|l| l.get("ext") == Some("gif"));
    Ok((!named).then(|| format!("a dropped GIF traced no `{REFUSED} ext=gif` line.")))
}

/// With Alt held the placement window opens and nothing is placed.
fn alt_opens_the_window(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    a: &Path,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    if driving::declared(&session.trace()?, ui_rect, PLACEMENT_REGION).is_some() {
        return Err(Error::new(format!(
            "`{PLACEMENT_REGION}` was declared before the Alt drop, so it cannot witness it."
        )));
    }
    let placed = drop_at(ctx, session, pointer, osp::FIRST, Some("alt"), &[a])?;
    if !placed.is_empty() {
        return Ok(Some(
            "an Alt drop placed the picture instead of asking.".to_owned(),
        ));
    }
    session.settle(20);
    Ok(
        driving::declared(&session.trace()?, ui_rect, PLACEMENT_REGION)
            .is_none()
            .then(|| format!("an Alt drop opened no placement window (`{PLACEMENT_REGION}`).")),
    )
}
