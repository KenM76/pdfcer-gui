//! `a_compacted_copy_tidies_drawing_instructions` — **the compact window's
//! drawing-instruction switch reaches the engine, says what it tidied, and is
//! what Save writes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_content.md`.

use super::compact_fonts::recompressed_after;
use super::dimdrive::{Fixture, press, run_on_with, ui_rect_event};
use super::stroke_style::into_view_of;
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// One page whose two content streams carry fifteen operators that change nothing.
const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact::{REGION_CONTENT, REGION_CONTENT_SAID, REGION_SAVE, REGION_BODY}`.
const SWITCH: &str = "compact.content";
const SAID: &str = "compact.content.said";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TARGET: &str = "content-copy.pdf";

/// See the module documentation.
pub struct ACompactedCopyTidiesDrawingInstructions;

impl Check for ACompactedCopyTidiesDrawingInstructions {
    fn name(&self) -> &'static str {
        "a_compacted_copy_tidies_drawing_instructions"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy does not tidy page drawing instructions: the switch is missing or \
         not on by default, switching it does not reach the engine, what was tidied goes unsaid, \
         or Save writes a different copy from the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(TARGET);
        // A file left by a previous run would satisfy the length oracle on a
        // build that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-content", &env, body)
    }
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let Some(first) = recompressed_after(session, 0)? else {
        return Ok(Some(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    };
    report.note(format!("default: `{}`", first.raw));
    let tidied = first.get_usize("content").unwrap_or(0);
    let with = first.get_usize("bytes").unwrap_or(0);
    if tidied == 0 || with == 0 {
        return Ok(Some(format!(
            "the window opened on `{}`; tidying is on by default, so the fixture's two content \
             streams must be tidied (`content>=1`) into a smaller copy. Trace: {path}.",
            first.raw
        )));
    }
    let mark = session.trace()?.mark();
    into_view_of(ctx, session, pointer, SWITCH, BODY)?;
    press(ctx, session, pointer, SWITCH)?;
    let Some(off) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "the tidy switch was pressed and the copy was not rebuilt. Trace: {path}."
        )));
    };
    report.note(format!("switched off: `{}`", off.raw));
    let without = off.get_usize("bytes").unwrap_or(usize::MAX);
    if off.get_usize("content") != Some(0) || without <= with {
        return Ok(Some(format!(
            "with the switch off the copy reads `{}`; it must tidy nothing (`content=0`) and \
             come out larger than the {with} bytes the tidied copy measured. Trace: {path}.",
            off.raw
        )));
    }
    let mark = session.trace()?.mark();
    into_view_of(ctx, session, pointer, SWITCH, BODY)?;
    press(ctx, session, pointer, SWITCH)?;
    let Some(on) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "the tidy switch was pressed back on and the copy was not rebuilt. Trace: {path}."
        )));
    };
    report.note(format!("switched back on: `{}`", on.raw));
    let bytes = on.get_usize("bytes").unwrap_or(0);
    if on.get_usize("content") != Some(tidied) || bytes != with {
        return Ok(Some(format!(
            "switching tidying back on reads `{}`; it must restore the default's {tidied} \
             tidied part(s) and {with} bytes. Trace: {path}.",
            on.raw
        )));
    }
    session.settle(20);
    into_view_of(ctx, session, pointer, SAID, BODY)?;
    if declared(&session.trace()?, ui_rect_event(ctx)?, SAID).is_none() {
        return Ok(Some(format!(
            "the drawing instructions were tidied and no `{SAID}` sentence saying so could be \
             scrolled into view. Trace: {path}."
        )));
    }
    saved(ctx, report, session, pointer, bytes)
}

/// Press Save and require the file on disk to be the copy measured.
fn saved(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    bytes: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    press(ctx, session, pointer, BUTTON)?;
    session.settle(60);
    if session.trace()?.last(WRITTEN).is_none() {
        return Ok(Some(format!(
            "Save was pressed and no `{WRITTEN}` line followed. Trace: {path}."
        )));
    }
    let target = ctx.out(TARGET);
    let written = std::fs::read(&target).map_or(0, |b| b.len());
    report.note(format!(
        "saved {written} bytes; the window measured {bytes}"
    ));
    Ok((written != bytes).then(|| {
        format!(
            "Save wrote {written} bytes to {} against the {bytes} the window measured with the \
             drawing instructions tidied. Trace: {path}.",
            target.display()
        )
    }))
}
