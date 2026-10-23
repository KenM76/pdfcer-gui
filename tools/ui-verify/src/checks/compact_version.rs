//! `a_compacted_copy_is_made_compatible_with_an_older_version` — **the compact
//! window's *Make compatible with* list reaches the engine, and Save writes
//! the lowered copy it measured, with the older header.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_version.md`.

use super::compact_target::{drawn, recompressed_after};
use super::dimdrive::{Fixture, press, run_on_with};
use super::stroke_style::into_view_of;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// A 1.7 file whose content needs nothing past 1.4.
const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_version::REGION_VERSION` and
/// `dialogs::compact::{REGION_SAVE, REGION_BODY}`.
const LIST: &str = "compact.version";
/// The list's entry for PDF 1.4: `0` is *Keep*, then 1.4 upward.
const TO_14: &str = "compact.version.1";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const LOWERED: &str = "compact-lowered"; // ui-text-exempt: a trace event name, never displayed
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TARGET: &str = "version-copy.pdf";
const HEADER_14: &[u8] = b"%PDF-1.4";
/// Seconds allowed for the engine to lower the copy.
const WAIT_S: u64 = 30;

/// See the module documentation.
pub struct ACompactedCopyIsMadeCompatibleWithAnOlderVersion;

impl Check for ACompactedCopyIsMadeCompatibleWithAnOlderVersion {
    fn name(&self) -> &'static str {
        "a_compacted_copy_is_made_compatible_with_an_older_version"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot be made compatible with an older PDF version: the list is \
         missing, choosing a version does not reach the engine, or Save writes a copy that does \
         not carry the older header or is not the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(TARGET);
        // A file left by a previous run would satisfy the oracle on a build
        // that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-version", &env, body)
    }
}

fn lowered_after(session: &Session, mark: usize) -> Result<Option<TraceLine>> {
    let started = std::time::Instant::now();
    loop {
        session.settle(8);
        if let Some(line) = session.trace()?.last_after(LOWERED, mark) {
            return Ok(Some(line.clone()));
        }
        if started.elapsed().as_secs() > WAIT_S {
            return Ok(None);
        }
    }
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if recompressed_after(session, 0)?.is_none() {
        return Ok(Some(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    }
    if session.trace()?.last(LOWERED).is_some() {
        return Ok(Some(format!(
            "the copy was lowered before any version was chosen; the list must start at Keep. \
             Trace: {path}."
        )));
    }
    into_view_of(ctx, session, pointer, LIST, BODY)?;
    if !drawn(ctx, session, LIST)? {
        return Ok(Some(format!(
            "no `{LIST}` list could be scrolled into view in the compacted-copy window. \
             Trace: {path}."
        )));
    }
    press(ctx, session, pointer, LIST)?;
    if !drawn(ctx, session, TO_14)? {
        return Ok(Some(format!(
            "the `{LIST}` list was opened and offers no `{TO_14}` (PDF 1.4). Trace: {path}."
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, TO_14)?;
    let Some(line) = lowered_after(session, mark)? else {
        return Ok(Some(format!(
            "PDF 1.4 was chosen and no `{LOWERED}` line followed within {WAIT_S} s. Trace: {path}."
        )));
    };
    report.note(format!("lowered: `{}`", line.raw));
    let bytes = line.get_usize("bytes").unwrap_or(0);
    if line.get("to") != Some("1.4") || line.get("outcome") != Some("lowered") || bytes == 0 {
        return Ok(Some(format!(
            "with PDF 1.4 chosen the trace reads `{}`; owed `to=1.4 outcome=lowered` and \
             `bytes>0`. Trace: {path}.",
            line.raw
        )));
    }
    saved(ctx, report, session, pointer, bytes)
}

/// Press Save and require the file on disk to be the lowered copy measured.
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
    let header = written.starts_with(HEADER_14);
    report.note(format!(
        "saved {} bytes, 1.4 header={header}; the window measured {bytes}",
        written.len()
    ));
    Ok((written.len() != bytes || !header).then(|| {
        format!(
            "Save wrote {} bytes to {} (starts with `%PDF-1.4`: {header}) against the \
             {bytes}-byte lowered copy the window measured. Trace: {path}.",
            written.len(),
            target.display()
        )
    }))
}
