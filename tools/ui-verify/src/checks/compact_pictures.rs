//! `a_compacted_copy_can_make_pictures_smaller` — **the compact window's
//! picture option reaches the engine, says what it lost, and is what Save
//! writes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_pictures.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, repo_fixture, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// One 400×400 noise picture drawn 96 pt square: 300 ppi, above the fold's
/// 150 ppi default; noise does not compress, so scaling it is what shrinks it.
const FIXTURE: &str = "noisy-photo.pdf";
const METHOD: &str = "Regenerate it with `python fixtures/noisy-photo.PROVENANCE.py`.";
/// The command under test.
const INVOKE: &str = "mode.edit,file.save_compacted";
/// The fold's header, its switch, and the lossy disclosure.
const FOLD: &str = "compact.pictures";
const SWITCH: &str = "compact.pictures.on";
const LOSSY: &str = "compact.pictures.lossy";
const BUTTON: &str = "compact.commit";
/// One line per finished recompression of the copy.
const RECOMPRESSED: &str = "compact-recompressed";
const WRITTEN: &str = "compact-written";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// See the module documentation.
pub struct ACompactedCopyCanMakePicturesSmaller;

impl Check for ACompactedCopyCanMakePicturesSmaller {
    fn name(&self) -> &'static str {
        "a_compacted_copy_can_make_pictures_smaller"
    }

    fn defect(&self) -> &'static str {
        "the compacted copy cannot scale pictures down: the option is missing or unreachable, \
         switching it on does not reach the engine, the loss of detail goes unsaid, or Save \
         writes a different copy from the one the window measured"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Polls until a `compact-recompressed` line after `mark` appears, or 30 s.
fn recompressed_after(session: &Session, mark: usize) -> Result<Option<TraceLine>> {
    let started = std::time::Instant::now();
    loop {
        session.settle(8);
        let trace = session.trace()?;
        if let Some(line) = trace.last_after(RECOMPRESSED, mark) {
            return Ok(Some(line.clone()));
        }
        if started.elapsed().as_secs() > 30 {
            return Ok(None);
        }
    }
}

/// Clicks the centre of `region` once it has stopped moving.
fn click(session: &Session, driver: &Driver, ui_rect: &str, region: &str) -> Result<bool> {
    let Some(rect) = stable_rect(session, ui_rect, region, 8)? else {
        return Ok(false);
    };
    let trace = session.trace()?;
    let frame = frame_of(session, &trace, ui_rect, region)?;
    driver.click_at(frame.declared_center(rect))?;
    session.settle(20);
    Ok(true)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let target = ctx.out("pictures-copy.pdf");
    let (session, ui_rect) = launch(ctx, report, &target)?;
    let driver = Driver::new(session.window());
    let (bytes, before) = match pictures_on(&session, &driver, report, ui_rect)? {
        Ok(sizes) => sizes,
        Err(failure) => return Ok(Some(failure)),
    };
    save(&session, &driver, report, ui_rect, &target, bytes, before)
}

/// Launch on the fixture with the compacted-copy window invoked and Save aimed
/// at `target`, which is removed first. Returns the session and the profile's
/// ui-rect event.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    target: &std::path::Path,
) -> Result<(Session, &'static str)> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the Pictures fold, its switch, \
             and Save.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // A file left by a previous run would satisfy the length oracle on a
    // build that wrote nothing.
    let _ = std::fs::remove_file(target);
    if target.exists() {
        return Err(Error::new(format!(
            "{} could not be removed before the run. SKIPPED rather than failed.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("compact-pictures.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env.push((
        SAVE_PATH_ENV.to_owned(),
        target.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    Ok((session, ui_rect))
}

/// Wait for the unasked baseline, switch pictures on, and require a smaller
/// scaled copy with the loss disclosure matching the plan. `Ok` carries the
/// scaled and baseline byte counts; the inner `Err` is the failure.
fn pictures_on(
    session: &Session,
    driver: &Driver,
    report: &mut CheckReport,
    ui_rect: &str,
) -> Result<std::result::Result<(usize, usize), String>> {
    let path = session.trace_path().display().to_string();

    // The window opens with recompression on, so one line lands unasked; it
    // is the baseline the pictures run must beat.
    let Some(plain) = recompressed_after(session, 0)? else {
        return Ok(Err(format!(
            "the compacted-copy window never finished its first recompression: no \
             `{RECOMPRESSED}` line in 30 s. Regions beginning `compact`: {}. Trace: {path}.",
            list(&declared_names(&session.trace()?, ui_rect, "compact"))
        )));
    };
    report.note(format!("baseline: `{}`", plain.raw));
    if plain.get_usize("resized").unwrap_or(0) != 0 {
        return Ok(Err(format!(
            "★★ PICTURES WERE SCALED WITH THE OPTION OFF: `{}`. It is the one choice that \
             changes how the page looks and must never be assumed. Trace: {path}.",
            plain.raw
        )));
    }

    if !click(session, driver, ui_rect, FOLD)? {
        return Ok(Err(format!(
            "★ THE PICTURES FOLD IS NOT IN THE WINDOW: no visible `{FOLD}` region. Regions \
             beginning `compact`: {}. Trace: {path}.",
            list(&declared_names(&session.trace()?, ui_rect, "compact"))
        )));
    }
    let mark = session.trace()?.mark();
    if !click(session, driver, ui_rect, SWITCH)? {
        return Ok(Err(format!(
            "the Pictures fold was clicked and its switch never drew: no visible `{SWITCH}` \
             region. The fold did not open, or the switch is clipped below the window's \
             edge. Trace: {path}."
        )));
    }

    let Some(asked) = recompressed_after(session, mark)? else {
        return Ok(Err(format!(
            "★★ THE SWITCH WAS CLICKED AND THE COPY WAS NOT REBUILT: no `{RECOMPRESSED}` line \
             after it in 30 s. A changed choice must start the work again. Trace: {path}."
        )));
    };
    report.note(format!("with pictures on: `{}`", asked.raw));
    let (resized, lossy, bytes) = (
        asked.get_usize("resized").unwrap_or(0),
        asked.get("lossy") == Some("true"),
        asked.get_usize("bytes").unwrap_or(0),
    );
    let before = plain.get_usize("bytes").unwrap_or(0);
    if resized == 0 {
        return Ok(Err(format!(
            "★★★ PICTURES ON SCALED NOTHING: `{}`. The fixture's picture is 300 ppi against the \
             150 ppi default, so the image rule never reached the plan. Trace: {path}.",
            asked.raw
        )));
    }
    if bytes == 0 || (before != 0 && bytes >= before) {
        return Ok(Err(format!(
            "★★ THE SCALED COPY IS NOT SMALLER: {bytes} bytes against {before} without the \
             option. Trace: {path}."
        )));
    }
    if let Some(failure) = disclosed(session, report, ui_rect, lossy)? {
        return Ok(Err(failure));
    }
    Ok(Ok((bytes, before)))
}

/// The loss disclosure is drawn exactly when the engine calls the plan lossy.
fn disclosed(
    session: &Session,
    report: &mut CheckReport,
    ui_rect: &str,
    lossy: bool,
) -> Result<Option<String>> {
    session.settle(20);
    let shown = declared(&session.trace()?, ui_rect, LOSSY).is_some();
    if lossy != shown {
        return Ok(Some(format!(
            "★★★ THE LOSS DISCLOSURE DISAGREES WITH THE ENGINE: the plan says lossy={lossy} \
             and the `{LOSSY}` line is {}. Trace: {}.",
            if shown { "drawn" } else { "absent" },
            session.trace_path().display()
        )));
    }
    report.note(format!("lossy={lossy}; the full-size line drawn={shown}"));
    Ok(None)
}

/// Click Save and require the file on disk to be the scaled copy measured.
fn save(
    session: &Session,
    driver: &Driver,
    report: &mut CheckReport,
    ui_rect: &str,
    target: &std::path::Path,
    bytes: usize,
    before: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if !click(session, driver, ui_rect, BUTTON)? {
        return Ok(Some(format!(
            "the window declared no visible `{BUTTON}` region. Trace: {path}."
        )));
    }
    session.settle(60);
    if session.trace()?.last(WRITTEN).is_none() {
        return Ok(Some(format!(
            "Save was clicked and no `{WRITTEN}` line followed. Trace: {path}."
        )));
    }
    let Ok(written) = std::fs::read(target) else {
        return Ok(Some(format!(
            "★★★ THE SAVE TRACED SUCCESS AND {} DOES NOT EXIST. Trace: {path}.",
            target.display()
        )));
    };
    if written.len() != bytes {
        return Ok(Some(format!(
            "★★★ SAVE WROTE A DIFFERENT COPY: {} bytes on disk against the {bytes} the window \
             measured with pictures on. Trace: {path}.",
            written.len()
        )));
    }
    report.note(format!(
        "★★★ the saved copy is the scaled one: {bytes} bytes, against {before} with pictures \
         off"
    ));
    Ok(None)
}
