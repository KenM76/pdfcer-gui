//! `a_compacted_copy_can_be_made_to_fit_a_size` — **the compact window's
//! size target reaches the engine's size search, the copy it keeps fits, the
//! window says so, a starting point can be chosen, and Save writes that copy.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_target.md`.

use super::dimdrive::{Fixture, press, run_on_with, ui_rect_event};
use super::stroke_style::into_view_of;
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// One 400×400 noise picture at 300 ppi: noise does not compress, so the
/// plain copy stays above the opening target and only a lossy step fits it.
const FIXTURE: Fixture = Fixture {
    file: "noisy-photo.pdf",
    method: "`python fixtures/noisy-photo.PROVENANCE.py`",
    page: PageGeometry {
        width_pt: 200.0,
        height_pt: 200.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_target::{REGION_TARGET, REGION_TARGET_SAID,
/// REGION_PRESET}` and `dialogs::compact::{REGION_SAVE, REGION_BODY}`.
const TARGET: &str = "compact.target";
const SAID: &str = "compact.target.said";
const PRESET: &str = "compact.preset";
const SCREEN: &str = "compact.preset.screen";
const BUTTON: &str = "compact.commit";
const BODY: &str = "compact.body";
const RECOMPRESSED: &str = "compact-recompressed"; // ui-text-exempt: a trace event name
const WRITTEN: &str = "compact-written"; // ui-text-exempt: a trace event name
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const COPY: &str = "target-copy.pdf";
/// A size search writes the file up to a dozen times.
pub(super) const WAIT_S: u64 = 90;

/// See the module documentation.
pub struct ACompactedCopyCanBeMadeToFitASize;

impl Check for ACompactedCopyCanBeMadeToFitASize {
    fn name(&self) -> &'static str {
        "a_compacted_copy_can_be_made_to_fit_a_size"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot be made to fit a size: the switch is missing, switching it \
         on does not run the engine's size search, the kept copy is larger than the target, \
         the result goes unsaid, the starting-point list does not reach the engine, or Save \
         writes a different copy"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let target = ctx.out(COPY);
        // A file left by a previous run would satisfy the length oracle on a
        // build that wrote nothing.
        let _ = std::fs::remove_file(&target);
        let env = [(SAVE_PATH_ENV, target.to_string_lossy().into_owned())];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-target", &env, body)
    }
}

/// Polls until a `compact-recompressed` line after `mark` appears.
pub(super) fn recompressed_after(session: &Session, mark: usize) -> Result<Option<TraceLine>> {
    let started = std::time::Instant::now();
    loop {
        session.settle(8);
        if let Some(line) = session.trace()?.last_after(RECOMPRESSED, mark) {
            return Ok(Some(line.clone()));
        }
        if started.elapsed().as_secs() > WAIT_S {
            return Ok(None);
        }
    }
}

/// Whether `region` is declared now.
pub(super) fn drawn(ctx: &CheckContext, session: &Session, region: &str) -> Result<bool> {
    Ok(declared(&session.trace()?, ui_rect_event(ctx)?, region).is_some())
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let fitted = match fit(ctx, report, session, pointer)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    let screen = match start_from_screen(ctx, report, session, pointer, &fitted)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    saved(ctx, report, session, pointer, &screen)
}

/// Wait for the unasked baseline, switch the target on, and require a search
/// that met it with a lossy step, a kept copy within it, and the result line.
fn fit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<TraceLine, String>> {
    let path = session.trace_path().display().to_string();
    let Some(plain) = recompressed_after(session, 0)? else {
        return Ok(Err(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    };
    report.note(format!("baseline: `{}`", plain.raw));
    if plain.get("target") != Some("none") {
        return Ok(Err(format!(
            "★★ A SIZE SEARCH RAN UNASKED, or the trace does not name it: `{}`. The target is \
             off until switched on, because its search can lower picture quality. \
             Trace: {path}.",
            plain.raw
        )));
    }
    if !drawn(ctx, session, TARGET)? {
        return Ok(Err(format!(
            "★ THE SIZE TARGET IS NOT IN THE WINDOW: no `{TARGET}` region. Regions beginning \
             `compact`: {}. Trace: {path}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect_event(ctx)?,
                "compact"
            ))
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, TARGET)?;
    let Some(asked) = recompressed_after(session, mark)? else {
        return Ok(Err(format!(
            "★★ THE TARGET WAS SWITCHED ON AND NO SEARCH FINISHED: no `{RECOMPRESSED}` line \
             after it in {WAIT_S} s. Trace: {path}."
        )));
    };
    report.note(format!("with the target on: `{}`", asked.raw));
    let max = asked.get_usize("max").unwrap_or(0);
    let step = asked.get_usize("step").unwrap_or(0);
    let bytes = asked.get_usize("bytes").unwrap_or(0);
    let before = plain.get_usize("bytes").unwrap_or(0);
    if asked.get("target") != Some("met") || max == 0 {
        return Ok(Err(format!(
            "★★★ THE SEARCH DID NOT MEET THE OPENING TARGET: `{}`. The fixture's noise picture \
             fits half its plain size at a lowered JPEG quality or resolution. Trace: {path}.",
            asked.raw
        )));
    }
    if bytes == 0 || bytes > max {
        return Ok(Err(format!(
            "★★★ THE KEPT COPY DOES NOT FIT: {bytes} bytes against a {max}-byte target. \
             Trace: {path}."
        )));
    }
    if step == 0 || before <= max {
        return Ok(Err(format!(
            "the plain copy ({before} bytes) already fits {max} bytes (step {step}), so the \
             search was never exercised. The opening target is meant to be half the plain \
             copy. Trace: {path}."
        )));
    }
    session.settle(20);
    into_view_of(ctx, session, pointer, SAID, BODY)?;
    if !drawn(ctx, session, SAID)? {
        return Ok(Err(format!(
            "★★ THE RESULT GOES UNSAID: the search met its target and no `{SAID}` line could \
             be scrolled into view. The operator must be told which step was taken, because it \
             lowered picture quality. Trace: {path}."
        )));
    }
    Ok(Ok(asked))
}

/// Scroll the window's body up until `region` is declared; the result line
/// sits below the starting-point list, so reaching one scrolls the other off.
fn back_up_to(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    for _ in 0..12 {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, region).is_some() {
            return Ok(());
        }
        let Some((under, vp)) = declared_in(&trace, ui_rect, BODY) else {
            break;
        };
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(under), 3.0)?;
        session.settle(15);
    }
    Ok(())
}

/// Pick Screen from the starting-point list and require the engine to scale
/// the picture under it, with the target still met.
fn start_from_screen(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    fitted: &TraceLine,
) -> Result<std::result::Result<TraceLine, String>> {
    let path = session.trace_path().display().to_string();
    back_up_to(ctx, session, pointer, PRESET)?;
    if !drawn(ctx, session, PRESET)? {
        return Ok(Err(format!(
            "★ THE STARTING-POINT LIST IS NOT IN THE WINDOW: no `{PRESET}` region. \
             Trace: {path}."
        )));
    }
    press(ctx, session, pointer, PRESET)?;
    session.settle(10);
    if !drawn(ctx, session, SCREEN)? {
        return Ok(Err(format!(
            "the starting-point list was pressed and its Screen entry never drew: no \
             `{SCREEN}` region. Regions beginning `{PRESET}`: {}. Trace: {path}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect_event(ctx)?,
                PRESET
            ))
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SCREEN)?;
    let Some(screen) = recompressed_after(session, mark)? else {
        return Ok(Err(format!(
            "★★ SCREEN WAS PICKED AND THE COPY WAS NOT REBUILT: no `{RECOMPRESSED}` line after \
             it in {WAIT_S} s. Trace: {path}."
        )));
    };
    report.note(format!("starting from Screen: `{}`", screen.raw));
    if screen.get_usize("resized").unwrap_or(0) == 0 || screen.get("target") != Some("met") {
        return Ok(Err(format!(
            "★★★ THE SCREEN STARTING POINT DID NOT REACH THE ENGINE: `{}`. Screen lowers \
             pictures above 144 ppi to 96 ppi, and the fixture's is 300 ppi; the target from \
             before must still be met. Trace: {path}.",
            screen.raw
        )));
    }
    report.note(format!(
        "the search kept step {} from Lossless and step {} from Screen",
        fitted.get_usize("step").unwrap_or(0),
        screen.get_usize("step").unwrap_or(0)
    ));
    Ok(Ok(screen))
}

/// Press Save and require the file on disk to be the copy measured.
fn saved(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    measured: &TraceLine,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let bytes = measured.get_usize("bytes").unwrap_or(0);
    let max = measured.get_usize("max").unwrap_or(0);
    into_view_of(ctx, session, pointer, BUTTON, BODY)?;
    press(ctx, session, pointer, BUTTON)?;
    session.settle(60);
    if session.trace()?.last(WRITTEN).is_none() {
        return Ok(Some(format!(
            "Save was pressed and no `{WRITTEN}` line followed. Trace: {path}."
        )));
    }
    let target = ctx.out(COPY);
    let written = std::fs::read(&target).map_or(0, |b| b.len());
    report.note(format!(
        "saved {written} bytes; the window measured {bytes} against a {max}-byte target"
    ));
    Ok((written != bytes || written > max).then(|| {
        format!(
            "★★★ SAVE WROTE A DIFFERENT COPY: {written} bytes to {} against the {bytes} the \
             window measured and a {max}-byte target. Trace: {path}.",
            target.display()
        )
    }))
}
