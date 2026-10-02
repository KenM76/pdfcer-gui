//! `a_key_the_subset_lacks_comes_from_its_installed_face` — a letter an
//! embedded subset has no outline for is added to the subset from the same
//! face in the font folders, so the line keeps its own font.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/augment_subset.md`.

use std::path::PathBuf;

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const INVOKE: &str = "mode.edit,edit.text";
const FIXTURE: &str = "augment-subset.pdf";
/// The folder holding the face the fixture's subset was cut from.
const FONTS: &str = "augment-fonts";
/// Inside the `A` of `ABC`, set at 24 pt from (10, 40).
const AT: (f64, f64) = (18.0, 48.0);
/// Outlined by the installed face, not by the subset.
const TYPED: &str = "D";
const FACE_FILE: &str = "face.ttf";
/// How many ten-frame waits the folder index is given.
const INDEX_WAITS: usize = 60;
const INDEXED: &str = "installed-faces-indexed"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "refused-char"; // ui-text-exempt: a trace event name, never displayed
const REFACED: &str = "text-edit-reface-planned"; // ui-text-exempt: a trace event name, never displayed
const EDIT: &str = "edit-text"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AKeyTheSubsetLacksComesFromItsInstalledFace;

impl Check for AKeyTheSubsetLacksComesFromItsInstalledFace {
    fn name(&self) -> &'static str {
        "a_key_the_subset_lacks_comes_from_its_installed_face"
    }

    fn defect(&self) -> &'static str {
        "a letter the embedded subset never carried is set in another face, though the \
         line's own font is installed on this computer"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Ok(note)) => {
                report.note(&note);
                report.pass()
            }
            Ok(Err(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "It is committed under fixtures/.")?;
    let face = repo_fixture(
        &format!("{FONTS}/{FACE_FILE}"),
        "It is committed under fixtures/.",
    )?;
    let fonts = face.parent().unwrap_or(&face).to_path_buf();
    let doc = ctx.out("augment-subset.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("augment-subset.trace.txt"));
    spec.pdf = Some(doc.clone());
    let fonts = fonts.display().to_string();
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
        ("PDFCER_DIAG_FONT_DIR", fonts.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("augment-subset.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// Wait for the font folders to be indexed, click into `ABC`, go to its end,
/// type the letter and commit.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mut indexed = None;
    for _ in 0..INDEX_WAITS {
        indexed = session
            .trace()?
            .events(INDEXED)
            .last()
            .map(|l| l.raw.clone());
        if indexed.is_some() {
            break;
        }
        session.settle(10);
    }
    let Some(indexed) = indexed else {
        return Err(Error::new(format!(
            "no `{INDEXED}` line: the font folders were never indexed, so there is no face to \
             add the letter from."
        )));
    };
    report.note(&indexed);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("could not read a page size from {FIXTURE}.")))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, AT.0, AT.1))?,
    )?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, TYPED)?;
    session.settle(20);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(30);
    let outcome = judge(&session)?;
    pointer.gone(&session)?;
    Ok(outcome)
}

/// Pass when the letter was neither refused nor re-faced, and the commit says
/// it added a glyph from the installed face.
fn judge(session: &Session) -> Result<std::result::Result<String, String>> {
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let quoted = format!("'{TYPED}'");
    if let Some(line) = trace
        .events(REFUSED)
        .find(|l| l.get("character") == Some(quoted.as_str()))
    {
        return Ok(Err(format!(
            "★★★★ `{TYPED}` was refused at the keystroke (`{}`): the keystroke query \
             (`canvas::textedit::repertoire`) was not asked with the installed faces. Trace: {path}.",
            line.raw
        )));
    }
    if let Some(line) = trace.events(REFACED).last() {
        return Ok(Err(format!(
            "★★★★ `{TYPED}` was planned into another face (`{}`) although the subset's own \
             face is in the font folders. Trace: {path}.",
            line.raw
        )));
    }
    let edit = trace.events(EDIT).last().map(|l| l.raw.clone());
    let added = edit
        .as_deref()
        .is_some_and(|l| l.contains("added 1 glyph(s)") && l.contains(FACE_FILE));
    if !added {
        return Ok(Err(format!(
            "★★★★ the commit does not say it added `{TYPED}` from `{FACE_FILE}` (`{edit:?}`). \
             Trace: {path}."
        )));
    }
    let committed = trace
        .events(LEFT_EDGE)
        .any(|l| l.get("committed") == Some("yes"));
    if !committed {
        return Ok(Err(format!(
            "★★★★ typing `{TYPED}` into `ABC` committed nothing: no `{LEFT_EDGE} \
             committed=yes`. Trace: {path}."
        )));
    }
    Ok(Ok(format!(
        "`{TYPED}` was added to the subset from `{FACE_FILE}` and committed in the line's own font"
    )))
}
