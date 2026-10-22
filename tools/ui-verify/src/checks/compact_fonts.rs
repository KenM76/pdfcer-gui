//! `a_compacted_copy_can_cut_fonts_down` — **the compact window's font switch
//! reaches the engine, says what it cut, and is what Save writes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_fonts.md`.

use super::dimdrive::{Fixture, press, run_on_with, ui_rect_event};
use super::stroke_style::into_view_of;
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// One face embedded twice; cutting keeps 3 of its 8 glyphs and merges the copies.
const FIXTURE: Fixture = Fixture {
    file: "resubset-pair.pdf",
    method: "Copy it again from the engine's `fixtures/synthetic/resubset/pair.pdf`.",
    page: PageGeometry {
        width_pt: 140.0,
        height_pt: 80.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact::{REGION_FONTS, REGION_FONTS_SAID, REGION_SAVE}`.
const SWITCH: &str = "compact.fonts";
const SAID: &str = "compact.fonts.said";
const BUTTON: &str = "compact.commit";
/// Mirrors `dialogs::compact::REGION_BODY`, the window body the choices scroll in.
const BODY: &str = "compact.body";
/// One line per finished recompression of the copy.
const RECOMPRESSED: &str = "compact-recompressed"; // ui-text-exempt: a trace event name, never displayed
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TARGET: &str = "fonts-copy.pdf";

/// See the module documentation.
pub struct ACompactedCopyCanCutFontsDown;

impl Check for ACompactedCopyCanCutFontsDown {
    fn name(&self) -> &'static str {
        "a_compacted_copy_can_cut_fonts_down"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot cut embedded fonts down: the switch is missing, switching it \
         on does not reach the engine, what was cut goes unsaid, or Save writes a different copy \
         from the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(TARGET);
        // A file left by a previous run would satisfy the length oracle on a
        // build that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-fonts", &env, body)
    }
}

/// Polls until a `compact-recompressed` line after `mark` appears, or 30 s.
pub(super) fn recompressed_after(session: &Session, mark: usize) -> Result<Option<TraceLine>> {
    let started = std::time::Instant::now();
    loop {
        session.settle(8);
        if let Some(line) = session.trace()?.last_after(RECOMPRESSED, mark) {
            return Ok(Some(line.clone()));
        }
        if started.elapsed().as_secs() > 30 {
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
    let Some(plain) = recompressed_after(session, 0)? else {
        return Ok(Some(format!(
            "the compacted-copy window never finished its first recompression: no \
             `{RECOMPRESSED}` line in 30 s. Trace: {path}."
        )));
    };
    report.note(format!("baseline: `{}`", plain.raw));
    if plain.get_usize("fonts_cut") != Some(0) {
        return Ok(Some(format!(
            "with the font switch off the copy reads `{}`; it must cut no font (`fonts_cut=0`). \
             Trace: {path}.",
            plain.raw
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SWITCH)?;
    let Some(asked) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "the font switch was pressed and the copy was not rebuilt: no `{RECOMPRESSED}` line \
             after it in 30 s. Trace: {path}."
        )));
    };
    report.note(format!("with fonts cut: `{}`", asked.raw));
    let bytes = asked.get_usize("bytes").unwrap_or(0);
    let before = plain.get_usize("bytes").unwrap_or(0);
    if asked.get_usize("fonts_cut") != Some(1) || bytes == 0 || bytes >= before {
        return Ok(Some(format!(
            "the font switch reads `{}`; the fixture's one face must be cut once \
             (`fonts_cut=1`) into a copy smaller than the {before} bytes without it. Trace: \
             {path}.",
            asked.raw
        )));
    }
    session.settle(20);
    // The sentences sit under the switches, below the choices' scrolled view.
    into_view_of(ctx, session, pointer, SAID, BODY)?;
    if declared(&session.trace()?, ui_rect_event(ctx)?, SAID).is_none() {
        return Ok(Some(format!(
            "the fonts were cut and no `{SAID}` sentence saying so could be scrolled into \
             view. Trace: {path}."
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
             fonts cut. Trace: {path}.",
            target.display()
        )
    }))
}
