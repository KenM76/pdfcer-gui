//! `checks::dropped_text` — **a text file dropped on the window becomes pages
//! after the one on screen**
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step, on a copy of the engine corpus's `four-pages.pdf` open on its first page.
//!
//! Oracles: a two-line `.txt` traces `text-dropped` and then
//! `import-text-applied pages=1 first=1` (the new sheet directly after page 1);
//! Ctrl+Z traces `undo-applied`; an empty `.txt` traces
//! `import-text-refused` and no further `import-text-applied`.

use super::os_image_paste as osp;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

const STEM: &str = "dropped-text";
const DROPPED: &str = "text-dropped";
const APPLIED: &str = "import-text-applied";
const REFUSED: &str = "import-text-refused";
const UNDONE: &str = "undo-applied";
/// The page the document opens at, 0-based; the new sheet goes after it.
const CURRENT: usize = 0;

/// See the module documentation.
pub struct ADroppedTextFileBecomesPagesAfterThisOne;

impl Check for ADroppedTextFileBecomesPagesAfterThisOne {
    fn name(&self) -> &'static str {
        "a_dropped_text_file_becomes_pages_after_this_one"
    }

    fn defect(&self) -> &'static str {
        "a .txt dragged onto the window is refused as a file pdfcer does not read, although File \
         ▸ Import text as pages turns the same file into pages"
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

/// A two-line text file and an empty one, written beside the trace.
fn fixtures(ctx: &CheckContext) -> Result<[PathBuf; 2]> {
    let files = [ctx.out("dropped-notes.txt"), ctx.out("dropped-empty.txt")];
    for (path, text) in files.iter().zip(["Hello\nWorld\n", ""]) {
        std::fs::write(path, text)
            .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    }
    Ok(files)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    files: &[PathBuf; 2],
) -> Result<Option<String>> {
    let [notes, empty] = files;
    let mut failure = becomes_pages(ctx, report, session, pointer, notes)?;
    if failure.is_none() {
        failure = empty_is_refused(ctx, session, pointer, empty)?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Drop `path` on the canvas and let the frame drain it.
fn drop_one(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &Path,
) -> Result<()> {
    pointer.drop_files(session, osp::at(ctx, session, osp::FIRST)?, None, &[path])?;
    session.settle(30);
    Ok(())
}

/// The text becomes one page directly after the current one, then undoes.
fn becomes_pages(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    notes: &Path,
) -> Result<Option<String>> {
    let (dropped, applied) = (osp::count(session, DROPPED)?, osp::count(session, APPLIED)?);
    drop_one(ctx, session, pointer, notes)?;
    let trace = session.trace()?;
    if trace.events(DROPPED).count() == dropped {
        return Ok(Some(format!(
            "the dropped .txt traced no `{DROPPED}` line."
        )));
    }
    let Some(line) = trace.events(APPLIED).nth(applied) else {
        return Ok(Some(format!(
            "the dropped .txt made no pages: no new `{APPLIED}` line."
        )));
    };
    let (pages, first) = (line.get("pages"), line.get("first"));
    report.note(format!("pages={pages:?} first={first:?}"));
    let after = (CURRENT + 1).to_string();
    if pages != Some("1") || first != Some(after.as_str()) {
        return Ok(Some(format!(
            "the dropped .txt made pages={} first={}, not one page at index {after}.",
            pages.unwrap_or("-"),
            first.unwrap_or("-")
        )));
    }
    let undos = osp::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok(
        (osp::count(session, UNDONE)? == undos)
            .then(|| "Ctrl+Z did not undo the pages.".to_owned()),
    )
}

/// An empty file is refused and makes nothing.
fn empty_is_refused(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    empty: &Path,
) -> Result<Option<String>> {
    let (refused, applied) = (osp::count(session, REFUSED)?, osp::count(session, APPLIED)?);
    drop_one(ctx, session, pointer, empty)?;
    if osp::count(session, APPLIED)? != applied {
        return Ok(Some("an empty .txt made pages.".to_owned()));
    }
    Ok((osp::count(session, REFUSED)? == refused)
        .then(|| format!("an empty .txt traced no `{REFUSED}` line.")))
}
