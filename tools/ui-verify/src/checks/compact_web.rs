//! `a_compacted_copy_is_laid_out_for_fast_web_view` — **the compact window's
//! Fast Web View switch reaches the engine, and Save writes the linearized
//! copy it measured.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_web.md`.

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

const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact::{REGION_LINEARIZE, REGION_SAVE, REGION_BODY}`.
const SWITCH: &str = "compact.linearize";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TARGET: &str = "web-copy.pdf";
/// The key of the linearization parameter dictionary (ISO 32000-2 Annex F.3.3).
const LINEARIZED: &[u8] = b"/Linearized";

/// See the module documentation.
pub struct ACompactedCopyIsLaidOutForFastWebView;

impl Check for ACompactedCopyIsLaidOutForFastWebView {
    fn name(&self) -> &'static str {
        "a_compacted_copy_is_laid_out_for_fast_web_view"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot be laid out for Fast Web View: the switch is missing or on by \
         default, switching it does not reach the engine, or Save writes a copy that is not \
         linearized or is not the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(TARGET);
        // A file left by a previous run would satisfy the oracle on a build
        // that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-web", &env, body)
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
    if first.get("linearized") != Some("false") {
        return Ok(Some(format!(
            "the window opened on `{}`; Fast Web View is off by default, so the line must carry \
             `linearized=false`. Trace: {path}.",
            first.raw
        )));
    }
    into_view_of(ctx, session, pointer, SWITCH, BODY)?;
    if declared(&session.trace()?, ui_rect_event(ctx)?, SWITCH).is_none() {
        return Ok(Some(format!(
            "no `{SWITCH}` switch could be scrolled into view in the compacted-copy window. \
             Trace: {path}."
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SWITCH)?;
    let Some(on) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "the Fast Web View switch was pressed and the copy was not rebuilt. Trace: {path}."
        )));
    };
    report.note(format!("switched on: `{}`", on.raw));
    let bytes = on.get_usize("bytes").unwrap_or(0);
    if on.get("linearized") != Some("true") || bytes == 0 {
        return Ok(Some(format!(
            "with Fast Web View on the copy reads `{}`; it must be linearized \
             (`linearized=true`) and written (`bytes>0`) even if it is larger. Trace: {path}.",
            on.raw
        )));
    }
    saved(ctx, report, session, pointer, bytes)
}

/// Press Save and require the file on disk to be the linearized copy
/// measured.
fn saved(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    bytes: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    into_view_of(ctx, session, pointer, BUTTON, BODY)?;
    press(ctx, session, pointer, BUTTON)?;
    session.settle(60);
    if session.trace()?.last(WRITTEN).is_none() {
        return Ok(Some(format!(
            "Save was pressed and no `{WRITTEN}` line followed. Trace: {path}."
        )));
    }
    let target = ctx.out(TARGET);
    let written = std::fs::read(&target).unwrap_or_default();
    let linearized = written.windows(LINEARIZED.len()).any(|w| w == LINEARIZED);
    report.note(format!(
        "saved {} bytes, linearized={linearized}; the window measured {bytes}",
        written.len()
    ));
    Ok((written.len() != bytes || !linearized).then(|| {
        format!(
            "Save wrote {} bytes to {} (linearization dictionary present: {linearized}) against \
             the {bytes}-byte linearized copy the window measured. Trace: {path}.",
            written.len(),
            target.display()
        )
    }))
}
