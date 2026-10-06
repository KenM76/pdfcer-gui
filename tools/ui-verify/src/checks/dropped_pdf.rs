//! `checks::dropped_pdf` — **a PDF dropped on an open document asks whether to
//! open it, insert its pages, or place its first page**
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step, in Edit mode on a copy of the engine corpus's `four-pages.pdf` open on
//! its first page, dropping a two-page 144×72 pt PDF this check writes.
//!
//! Oracles: each drop traces `drop-pdf-asked insert=true place=true`. Insert
//! traces `insert-pages page=1 n=2` and undoes; Place traces
//! `drop-pdf-chosen choice=place` with a 144×72 pt rectangle centred on the
//! drop point, then `page-content-placed` and no `custom-stamp-placed` (Edit
//! draws the page into the content rather than stamping it), and undoes; Enter in the window
//! traces `drop-pdf-chosen choice=open` and an `open ok` naming the file.

use super::os_image_paste as osp;
use crate::checks::driving::{declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

const STEM: &str = "dropped-pdf";
const FILE: &str = "dropped-pdf-logo.pdf";
const ASKED: &str = "drop-pdf-asked";
const CHOSEN: &str = "drop-pdf-chosen";
const INSERTED: &str = "insert-pages";
const PLACED: &str = "page-content-placed";
const STAMPED: &str = "custom-stamp-placed";
const OPENED: &str = "open";
const UNDONE: &str = "undo-applied";
const UI_RECT: &str = "ui-rect";
const BODY: &str = "drop-pdf.body";
const INSERT: &str = "drop-pdf.insert";
const PLACE: &str = "drop-pdf.place";
/// The fixture's page size, in points.
const SIZE: (f64, f64) = (144.0, 72.0);

/// See the module documentation.
pub struct ADroppedPdfAsksOpenInsertOrPlace;

impl Check for ADroppedPdfAsksOpenInsertOrPlace {
    fn name(&self) -> &'static str {
        "a_dropped_pdf_asks_open_insert_or_place"
    }

    fn defect(&self) -> &'static str {
        "a PDF dragged onto an open document can only be opened on its own; its pages cannot be \
         inserted after the page on screen or placed on it where it was dropped"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = fixture(ctx).and_then(|file| {
            osp::launch(ctx, &mut report, STEM)
                .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &file))
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A two-page PDF of 144×72 pt pages, each a filled blue rectangle.
fn pdf_bytes() -> Vec<u8> {
    let content = "0 0 1 rg 0 0 144 72 re f";
    let page =
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 144 72] /Resources << >> /Contents 4 0 R >>";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>".to_owned(),
        page.to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
        page.to_owned(),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        table.push_str(&format!("{offset:010} 00000 n \n"));
    }
    table.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    out.extend_from_slice(table.as_bytes());
    out
}

fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let path = ctx.out(FILE);
    std::fs::write(&path, pdf_bytes())
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    let mut failure = inserts(ctx, report, session, pointer, file)?;
    if failure.is_none() {
        failure = places(ctx, report, session, pointer, file)?;
    }
    if failure.is_none() {
        failure = opens(ctx, session, pointer, file)?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Drop the file at the first point and wait for the question.
fn ask(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    let asked = osp::count(session, ASKED)?;
    pointer.drop_files(session, osp::at(ctx, session, osp::FIRST)?, None, &[file])?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.events(ASKED).nth(asked) else {
        return Ok(Some(format!(
            "the dropped PDF traced no `{ASKED}` line; it was not asked about."
        )));
    };
    if line.get("insert") != Some("true") || line.get("place") != Some("true") {
        return Ok(Some(format!(
            "in Edit the question offered insert={} place={}, not both.",
            line.get("insert").unwrap_or("-"),
            line.get("place").unwrap_or("-")
        )));
    }
    Ok(None)
}

/// Click the window's `region`.
fn press(session: &Session, pointer: &ScriptedPointer, region: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Declared under `drop-pdf`: {}.",
            list(&declared_names(&trace, UI_RECT, "drop-pdf"))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Ctrl+Z undoes the last edit, or names `what` as not undone.
fn undoes(session: &Session, pointer: &ScriptedPointer, what: &str) -> Result<Option<String>> {
    let undos = osp::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok((osp::count(session, UNDONE)? == undos).then(|| format!("Ctrl+Z did not undo {what}.")))
}

/// Insert puts both pages directly after page 1.
fn inserts(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    if let Some(failure) = ask(ctx, session, pointer, file)? {
        return Ok(Some(failure));
    }
    let before = osp::count(session, INSERTED)?;
    press(session, pointer, INSERT)?;
    let trace = session.trace()?;
    let Some(line) = trace.events(INSERTED).nth(before) else {
        return Ok(Some(format!("Insert made no `{INSERTED}` line.")));
    };
    let (page, n) = (line.get("page"), line.get("n"));
    report.note(format!(
        "insert page={} n={}",
        page.unwrap_or("-"),
        n.unwrap_or("-")
    ));
    if page != Some("1") || n != Some("2") {
        return Ok(Some(format!(
            "Insert landed page={} n={}, not both pages at index 1.",
            page.unwrap_or("-"),
            n.unwrap_or("-")
        )));
    }
    undoes(session, pointer, "the inserted pages")
}

/// Place puts the first page at its natural size, centred on the drop.
fn places(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    if let Some(failure) = ask(ctx, session, pointer, file)? {
        return Ok(Some(failure));
    }
    let (chosen, placed) = (osp::count(session, CHOSEN)?, osp::count(session, PLACED)?);
    let stamped = osp::count(session, STAMPED)?;
    press(session, pointer, PLACE)?;
    let trace = session.trace()?;
    let Some(r) = trace
        .events(CHOSEN)
        .nth(chosen)
        .filter(|l| l.get("choice") == Some("place"))
        .and_then(osp::rect)
    else {
        return Ok(Some(format!(
            "Place traced no `{CHOSEN} choice=place` with a rectangle."
        )));
    };
    report.note(format!("placed {}", osp::show(r)));
    if let Some(failure) = osp::lands(r, osp::FIRST, SIZE) {
        return Ok(Some(failure));
    }
    if trace.events(PLACED).count() == placed {
        return Ok(Some(format!(
            "Place chose a rectangle but no `{PLACED}` line followed."
        )));
    }
    if trace.events(STAMPED).count() != stamped {
        return Ok(Some(format!(
            "Place in Edit traced `{STAMPED}`: the page went in as a stamp, not as content."
        )));
    }
    undoes(session, pointer, "the placed page")
}

/// Enter in the window takes the default, Open.
fn opens(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    if let Some(failure) = ask(ctx, session, pointer, file)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    let viewport = declared_in(&trace, UI_RECT, BODY).and_then(|(_, vp)| vp);
    let opened = osp::count(session, OPENED)?;
    pointer.key(session, viewport.as_deref(), "Enter", None)?;
    session.settle(45);
    let trace = session.trace()?;
    let Some(line) = trace.events(OPENED).nth(opened) else {
        return Ok(Some("Enter in the question opened nothing.".to_owned()));
    };
    let named = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok((!line.raw.contains(&named)).then(|| format!("Enter opened something other than {named}.")))
}
