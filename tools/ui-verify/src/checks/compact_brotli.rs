//! Two checks on the compact window's Brotli switch and its *Make compatible
//! with* target:
//!
//! - `a_compacted_copy_is_packed_with_brotli` — **the switch reaches the
//!   engine and Save writes a PDF 2.0 copy with `/BrotliDecode` streams.**
//! - `an_older_target_leaves_brotli_out_and_lists_what_the_copy_uses` —
//!   **with PDF 1.4 chosen the same request is skipped with the target as
//!   its reason, the features the copy uses are listed, and Save writes a
//!   1.4 copy with no Brotli in it.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_brotli.md`.

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

/// A 1.7 file whose content needs nothing past 1.4, with Flate streams.
const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_streams::REGION_BROTLI`,
/// `dialogs::compact_version::{REGION_VERSION, REGION_FEATURES}` and
/// `dialogs::compact::{REGION_SAVE, REGION_BODY}`.
const SWITCH: &str = "compact.brotli";
const LIST: &str = "compact.version";
/// The list's entry for PDF 1.4: `0` is *Keep*, then 1.4 upward.
const TO_14: &str = "compact.version.1";
const FEATURES: &str = "compact.version.features";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const LOWERED: &str = "compact-lowered"; // ui-text-exempt: a trace event name, never displayed
const USED: &str = "compact-features"; // ui-text-exempt: a trace event name, never displayed
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const BROTLI_COPY: &str = "brotli-copy.pdf";
const OLDER_COPY: &str = "brotli-older-copy.pdf";
const BROTLI: &[u8] = b"/BrotliDecode";
/// Seconds allowed for the engine to lower the copy.
const WAIT_S: u64 = 30;

/// See the module documentation.
pub struct ACompactedCopyIsPackedWithBrotli;

impl Check for ACompactedCopyIsPackedWithBrotli {
    fn name(&self) -> &'static str {
        "a_compacted_copy_is_packed_with_brotli"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot be packed with Brotli: the switch is missing, it does not \
         reach the engine, or Save writes a copy with no Brotli streams or without the PDF 2.0 \
         header Brotli needs"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        launch(self, ctx, BROTLI_COPY, brotli_body)
    }
}

/// See the module documentation.
pub struct AnOlderTargetLeavesBrotliOutAndListsWhatTheCopyUses;

impl Check for AnOlderTargetLeavesBrotliOutAndListsWhatTheCopyUses {
    fn name(&self) -> &'static str {
        "an_older_target_leaves_brotli_out_and_lists_what_the_copy_uses"
    }

    fn defect(&self) -> &'static str {
        "an older *Make compatible with* target does not hold the plan to it: Brotli is still \
         written, nothing says it was left out for the target, the features the copy uses are \
         not listed, or the saved copy does not carry the older header"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        launch(self, ctx, OLDER_COPY, older_body)
    }
}

type Body =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

fn launch(check: &dyn Check, ctx: &CheckContext, copy: &str, body: Body) -> CheckReport {
    let target = ctx.out(copy);
    // A file left by a previous run would satisfy the oracle on a build that
    // wrote nothing.
    let _ = std::fs::remove_file(&target);
    let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
    run_on_with(check, ctx, &FIXTURE, INVOKE, "compact-brotli", &env, body)
}

/// Waits for the first recompression, presses the switch and returns the
/// recompression it caused, or the failure.
fn switched_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<TraceLine, String>> {
    let path = session.trace_path().display().to_string();
    let Some(first) = recompressed_after(session, 0)? else {
        return Ok(Err(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    };
    if first.get("brotli") != Some("false") {
        return Ok(Err(format!(
            "the first recompression reads `{}`; owed `brotli=false` before the switch is \
             pressed. Trace: {path}.",
            first.raw
        )));
    }
    into_view_of(ctx, session, pointer, SWITCH, BODY)?;
    if !drawn(ctx, session, SWITCH)? {
        return Ok(Err(format!(
            "no `{SWITCH}` switch could be scrolled into view in the compacted-copy window. \
             Trace: {path}."
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SWITCH)?;
    let Some(line) = recompressed_after(session, mark)? else {
        return Ok(Err(format!(
            "the Brotli switch was pressed and no recompression followed. Trace: {path}."
        )));
    };
    report.note(format!("with Brotli: `{}`", line.raw));
    if line.get("brotli") != Some("true") || line.get("ok") != Some("true") {
        return Ok(Err(format!(
            "with the Brotli switch on the trace reads `{}`; owed `brotli=true ok=true`. \
             Trace: {path}.",
            line.raw
        )));
    }
    Ok(Ok(line))
}

fn brotli_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let line = match switched_on(ctx, report, session, pointer)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    if line.get_usize("above") != Some(0) || line.get("compat") != Some("keep") {
        return Ok(Some(format!(
            "with no target chosen the trace reads `{}`; owed `compat=keep above=0`. \
             Trace: {path}.",
            line.raw
        )));
    }
    let written = match saved(ctx, session, pointer, BROTLI_COPY)? {
        Ok(bytes) => bytes,
        Err(failure) => return Ok(Some(failure)),
    };
    let brotli = written.windows(BROTLI.len()).any(|w| w == BROTLI);
    let header = written.starts_with(b"%PDF-2.0");
    report.note(format!(
        "saved {} bytes; /BrotliDecode={brotli}; 2.0 header={header}",
        written.len()
    ));
    Ok((!brotli || !header).then(|| {
        format!(
            "Save wrote {} bytes with `/BrotliDecode` present={brotli} and a `%PDF-2.0` header \
             ={header}; both are owed with the switch on. Trace: {path}.",
            written.len()
        )
    }))
}

fn older_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if let Err(failure) = switched_on(ctx, report, session, pointer)? {
        return Ok(Some(failure));
    }
    into_view_of(ctx, session, pointer, LIST, BODY)?;
    press(ctx, session, pointer, LIST)?;
    if !drawn(ctx, session, TO_14)? {
        return Ok(Some(format!(
            "the `{LIST}` list was opened and offers no `{TO_14}` (PDF 1.4). Trace: {path}."
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, TO_14)?;
    let Some(line) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "PDF 1.4 was chosen and the copy was not recompressed again. Trace: {path}."
        )));
    };
    report.note(format!("with 1.4: `{}`", line.raw));
    let above = line.get_usize("above").unwrap_or(0);
    if line.get("brotli") != Some("true") || line.get("compat") != Some("1.4") || above == 0 {
        return Ok(Some(format!(
            "with Brotli on and PDF 1.4 chosen the trace reads `{}`; owed `brotli=true \
             compat=1.4 above>0`, the target refusing the filter. Trace: {path}.",
            line.raw
        )));
    }
    if let Some(failure) = listed(ctx, report, session, pointer, mark)? {
        return Ok(Some(failure));
    }
    let written = match saved(ctx, session, pointer, OLDER_COPY)? {
        Ok(bytes) => bytes,
        Err(failure) => return Ok(Some(failure)),
    };
    let brotli = written.windows(BROTLI.len()).any(|w| w == BROTLI);
    let header = written.starts_with(b"%PDF-1.4");
    report.note(format!(
        "saved {} bytes; /BrotliDecode={brotli}; 1.4 header={header}",
        written.len()
    ));
    Ok((brotli || !header).then(|| {
        format!(
            "Save wrote {} bytes with `/BrotliDecode` present={brotli} (owed absent) and a \
             `%PDF-1.4` header={header} (owed). Trace: {path}.",
            written.len()
        )
    }))
}

/// Requires the lowering to finish with its features listed and the fold
/// drawn.
fn listed(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let started = std::time::Instant::now();
    while session.trace()?.last_after(LOWERED, mark).is_none() {
        if started.elapsed().as_secs() > WAIT_S {
            return Ok(Some(format!(
                "PDF 1.4 was chosen and no `{LOWERED}` line followed within {WAIT_S} s. \
                 Trace: {path}."
            )));
        }
        session.settle(8);
    }
    let Some(used) = session.trace()?.last_after(USED, mark).cloned() else {
        return Ok(Some(format!(
            "the copy was lowered and no `{USED}` line listed what it uses. Trace: {path}."
        )));
    };
    report.note(format!("features: `{}`", used.raw));
    if used.get("to") != Some("1.4") || used.get("list").is_none() {
        return Ok(Some(format!(
            "the features line reads `{}`; owed `to=1.4` and a `list=`. Trace: {path}.",
            used.raw
        )));
    }
    let any = used.get_usize("n").unwrap_or(0) > 0;
    into_view_of(ctx, session, pointer, FEATURES, BODY)?;
    let fold = drawn(ctx, session, FEATURES)?;
    Ok((any != fold).then(|| {
        format!(
            "the trace lists {} feature(s) and the `{FEATURES}` fold drawn={fold}; a fold is \
             owed exactly when there is something to list. Trace: {path}.",
            used.get_usize("n").unwrap_or(0)
        )
    }))
}

/// Presses Save and returns the bytes written, or the failure.
fn saved(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    copy: &str,
) -> Result<std::result::Result<Vec<u8>, String>> {
    let path = session.trace_path().display().to_string();
    into_view_of(ctx, session, pointer, BUTTON, BODY)?;
    press(ctx, session, pointer, BUTTON)?;
    session.settle(60);
    if session.trace()?.last(WRITTEN).is_none() {
        return Ok(Err(format!(
            "Save was pressed and no `{WRITTEN}` line followed. Trace: {path}."
        )));
    }
    Ok(Ok(std::fs::read(ctx.out(copy)).unwrap_or_default()))
}
