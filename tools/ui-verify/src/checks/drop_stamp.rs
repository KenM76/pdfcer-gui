//! `checks::drop_stamp` — **a picture or drawing dropped on a page in Review
//! becomes a stamp comment, and Read places nothing**
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step on the shared paste fixture, switching mode with `Ctrl+2` (Review)
//! and `Ctrl+1` (Read).
//!
//! Oracles: in Review a dropped PNG traces `image-dropped … as=stamp` and
//! then `picture-stamp-placed kind=image` with a non-zero `id`, and a dropped
//! SVG `picture-stamp-placed kind=svg`; in Read a dropped PNG traces
//! `drop-declined` and no `image-dropped`.

use super::os_image_paste as osp;
use crate::checks::driving::repo_fixture;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::Path;

const STEM: &str = "drop-stamp";
const DROPPED: &str = "image-dropped";
const PLACED: &str = "picture-stamp-placed";
const DECLINED: &str = "drop-declined";
const DRAWING: &str = "vector-art.svg";
const DRAWING_METHOD: &str = "See fixtures/vector-art.PROVENANCE.md.";

/// See the module documentation.
pub struct ADroppedPictureStampsInReview;

impl Check for ADroppedPictureStampsInReview {
    fn name(&self) -> &'static str {
        "a_dropped_picture_stamps_in_review"
    }

    fn defect(&self) -> &'static str {
        "a picture dropped on a page in Review is refused, although Review adds comments and a \
         picture stamp is one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = fixtures(ctx).and_then(|(png, svg)| {
            let (session, pointer) = osp::launch(ctx, &mut report, STEM)?;
            let outcome = drive(ctx, &mut report, &session, &pointer, &png, &svg);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn fixtures(ctx: &CheckContext) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let png = crate::png::encode_rgb(48, 24, &[90u8; 48 * 24 * 3])
        .ok_or_else(|| Error::new("the harness's own PNG encoder refused its fixture"))?;
    let path = ctx.out("drop-stamp.png");
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok((path, repo_fixture(DRAWING, DRAWING_METHOD)?))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    png: &Path,
    svg: &Path,
) -> Result<Option<String>> {
    switch(session, pointer, "2")?;
    let mut failure = stamps(ctx, report, session, pointer, png, "image")?;
    if failure.is_none() {
        failure = stamps(ctx, report, session, pointer, svg, "svg")?;
    }
    if failure.is_none() {
        switch(session, pointer, "1")?;
        failure = read_declines(ctx, session, pointer, png)?;
    }
    Ok(failure)
}

/// Change mode with `Ctrl+digit`.
fn switch(session: &Session, pointer: &ScriptedPointer, digit: &str) -> Result<()> {
    pointer.key(session, None, digit, Some("ctrl"))?;
    session.settle(20);
    Ok(())
}

/// Drop `path` in Review: one `image-dropped as=stamp`, then one
/// `picture-stamp-placed kind={kind}` naming the stamp it made.
fn stamps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &Path,
    kind: &str,
) -> Result<Option<String>> {
    let (dropped, placed) = (osp::count(session, DROPPED)?, osp::count(session, PLACED)?);
    pointer.drop_files(session, osp::at(ctx, session, osp::FIRST)?, None, &[path])?;
    session.settle(30);
    let trace = session.trace()?;
    let drops: Vec<_> = trace.events(DROPPED).skip(dropped).collect();
    let [drop] = drops[..] else {
        return Ok(Some(format!(
            "a dropped {kind} in Review traced {} `{DROPPED}` line(s), not 1.",
            drops.len()
        )));
    };
    if drop.get("as") != Some("stamp") {
        return Ok(Some(format!(
            "★★★ a dropped {kind} in Review went in as `{}`, not as a stamp.",
            drop.raw
        )));
    }
    let Some(line) = trace.events(PLACED).nth(placed) else {
        return Ok(Some(format!(
            "★★★ a dropped {kind} in Review traced no `{PLACED}` line, so no stamp was made."
        )));
    };
    report.note(format!("{kind}: {} | {}", drop.raw, line.raw));
    let made = line.get("kind") == Some(kind) && line.get("id").is_some_and(|id| id != "0");
    Ok((!made).then(|| {
        format!(
            "★★★ the {kind} drop's stamp line is `{}`; it must carry kind={kind} and a non-zero id.",
            line.raw
        )
    }))
}

/// In Read a drop places nothing and is declined.
fn read_declines(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    png: &Path,
) -> Result<Option<String>> {
    let (dropped, declined) = (
        osp::count(session, DROPPED)?,
        osp::count(session, DECLINED)?,
    );
    pointer.drop_files(session, osp::at(ctx, session, osp::FIRST)?, None, &[png])?;
    session.settle(20);
    if osp::count(session, DROPPED)? != dropped {
        return Ok(Some(
            "Read placed a dropped picture on the page.".to_owned(),
        ));
    }
    Ok((osp::count(session, DECLINED)? == declined)
        .then(|| format!("a picture dropped in Read traced no `{DECLINED}` line.")))
}
