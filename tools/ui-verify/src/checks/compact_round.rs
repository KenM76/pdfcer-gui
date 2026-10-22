//! `a_compacted_copy_rounds_line_positions` — **the compact window's rounding
//! list reaches the engine, says how far positions moved, and is what Save
//! writes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_round.md`.

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

/// One page drawn with four-decimal path coordinates.
const FIXTURE: Fixture = Fixture {
    file: "hole-in-a-big-object.pdf",
    method: "`python tools/gen-hole-in-a-big-object-fixture.py`",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact::{REGION_ROUND, REGION_CONTENT_SAID, REGION_SAVE, REGION_BODY}`;
/// entry 2 of the list is 0.01 pt.
const LIST: &str = "compact.content.round";
const HUNDREDTH: &str = "compact.content.round.2";
const SAID: &str = "compact.content.said";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TARGET: &str = "round-copy.pdf";

/// See the module documentation.
pub struct ACompactedCopyRoundsLinePositions;

impl Check for ACompactedCopyRoundsLinePositions {
    fn name(&self) -> &'static str {
        "a_compacted_copy_rounds_line_positions"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot round line positions: rounding is on by default, the list is \
         missing, choosing a distance does not reach the engine, how far positions moved goes \
         unsaid, or Save writes a different copy from the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(TARGET);
        // A file left by a previous run would satisfy the length oracle on a
        // build that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-round", &env, body)
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
    let unrounded = first.get_usize("bytes").unwrap_or(0);
    if first.get_usize("rounded") != Some(0) || unrounded == 0 {
        return Ok(Some(format!(
            "the window opened on `{}`; rounding is lossy, so it must be off by default \
             (`rounded=0`). Trace: {path}.",
            first.raw
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, LIST)?;
    session.settle(10);
    press(ctx, session, pointer, HUNDREDTH)?;
    let Some(on) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "0.01 pt was chosen from the rounding list and the copy was not rebuilt. \
             Trace: {path}."
        )));
    };
    report.note(format!("0.01 pt: `{}`", on.raw));
    let bytes = on.get_usize("bytes").unwrap_or(usize::MAX);
    if on.get_usize("rounded").unwrap_or(0) == 0 || bytes >= unrounded {
        return Ok(Some(format!(
            "with 0.01 pt chosen the copy reads `{}`; the fixture's four-decimal coordinates \
             must round (`rounded>=1`) into a copy smaller than the {unrounded} bytes \
             unrounded. Trace: {path}.",
            on.raw
        )));
    }
    session.settle(20);
    into_view_of(ctx, session, pointer, SAID, BODY)?;
    if declared(&session.trace()?, ui_rect_event(ctx)?, SAID).is_none() {
        return Ok(Some(format!(
            "line positions were rounded and no `{SAID}` sentence saying so could be scrolled \
             into view. Trace: {path}."
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
            "Save wrote {written} bytes to {} against the {bytes} the window measured with line \
             positions rounded. Trace: {path}.",
            target.display()
        )
    }))
}
