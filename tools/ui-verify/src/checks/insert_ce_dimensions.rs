//! `inserted_pages_say_what_came_of_their_ce_dimensions` — pages carrying a ce
//! dimension inserted into a document with none: the receipt says the ce
//! dimension came across still editable, and says nothing of plain lines.
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step, in Edit mode on a copy of the engine corpus's `four-pages.pdf`,
//! dropping a copy of `fixtures/dimension-scaled.pdf` (one page, one ce
//! dimension in one calibrated group) and choosing Insert.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/insert_ce_dimensions.md`.

use super::dropped_pdf::{INSERT, ask, press};
use super::os_image_paste as osp;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

const STEM: &str = "insert-ce-dimensions";
const FIXTURE: &str = "fixtures/dimension-scaled.pdf";
const INSERTED: &str = "insert-pages"; // ui-text-exempt: a trace event name, never displayed
/// Said by `text::cedimcopy::copied` for one editable ce dimension.
const CARRIED: &str = "1 ce dimension came across still editable";
/// Said by the same sentence when any arrived as plain lines.
const AS_LINES: &str = "came across as plain lines";

/// See the module documentation.
pub struct InsertedPagesSayWhatCameOfTheirCeDimensions;

impl Check for InsertedPagesSayWhatCameOfTheirCeDimensions {
    fn name(&self) -> &'static str {
        "inserted_pages_say_what_came_of_their_ce_dimensions"
    }

    fn defect(&self) -> &'static str {
        "pages inserted with a ce dimension on them arrive with nothing telling the operator \
         whether it is still editable or what became of its group"
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

/// A copy of the fixture, so the drop never names a file in the repository.
fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE);
    let path = ctx.out("insert-ce-dimensions-dropped.pdf");
    std::fs::copy(&source, &path).map_err(|e| {
        Error::new(format!(
            "cannot copy {} to {}: {e}",
            source.display(),
            path.display()
        ))
    })?;
    Ok(path)
}

fn drive(
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
    let path = session.trace_path().display().to_string();
    let Some(line) = trace.events(INSERTED).nth(before) else {
        return Ok(Some(format!(
            "Insert made no `{INSERTED}` line. Trace: {path}."
        )));
    };
    report.note(format!("receipt: {}", line.raw));
    let failure = if !line.raw.contains(CARRIED) {
        Some(format!(
            "the insert's receipt does not say `{CARRIED}`. Line: `{}`.",
            line.raw
        ))
    } else if line.raw.contains(AS_LINES) {
        Some(format!(
            "the ce dimension was carried editable and the receipt says some came as plain \
             lines. Line: `{}`.",
            line.raw
        ))
    } else {
        None
    };
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}
