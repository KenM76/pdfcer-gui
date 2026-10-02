//! `a_glyph_the_subset_outlines_but_never_showed_types` — typing a character
//! that an embedded subset's program outlines, but the page never showed,
//! commits instead of being refused at the keystroke.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/subset_glyph.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Edit Text tool armed, so a click on words opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
const FIXTURE: &str = "word-shaped-subset.pdf";
/// Inside the `A` of page 1's `ABC`, set at 24 pt from (72, 600).
const AT: (f64, f64) = (80.0, 608.0);
/// Outlined by the subset's program, absent from `/Widths` and the page.
const TYPED: &str = "D";
const REFUSED: &str = "refused-char"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AGlyphTheSubsetOutlinesTypes;

impl Check for AGlyphTheSubsetOutlinesTypes {
    fn name(&self) -> &'static str {
        "a_glyph_the_subset_outlines_but_never_showed_types"
    }

    fn defect(&self) -> &'static str {
        "a character the embedded font's program draws, but the page never used, is refused \
         at the keystroke, so a Word line cannot take a letter its own font could print"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let report = CheckReport::new(self.name(), self.defect());
        match drive(ctx) {
            Ok((artifacts, outcome)) => {
                let mut report = report;
                for a in artifacts {
                    report.artifact(a);
                }
                match outcome {
                    Ok(note) => {
                        report.note(&note);
                        report.pass()
                    }
                    Err(failure) => report.fail(failure),
                }
            }
            Err(why) => report.from_error(&why),
        }
    }
}

type Artifacts = Vec<std::path::PathBuf>;

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
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
    let doc = ctx.out("subset-glyph.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("subset-glyph.trace.txt"));
    spec.pdf = Some(doc.clone());
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("subset-glyph.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// Click into `ABC`, go to its end, type the outlined letter and commit.
fn drive(ctx: &CheckContext) -> Result<(Artifacts, std::result::Result<String, String>)> {
    let (session, pointer, doc) = launch(ctx)?;
    let artifacts = vec![
        session.trace_path().to_path_buf(),
        pointer.path().to_path_buf(),
    ];
    session.settle(45);
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
    Ok((artifacts, outcome))
}

/// Pass when no keystroke was refused and the edit committed.
fn judge(session: &Session) -> Result<std::result::Result<String, String>> {
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let quoted = format!("'{TYPED}'");
    if let Some(line) = trace
        .events(REFUSED)
        .find(|l| l.get("character") == Some(quoted.as_str()))
    {
        return Ok(Err(format!(
            "★★★★ `{TYPED}` was refused at the keystroke (`{}`), though the subset's program \
             outlines it. The keystroke query (`canvas::textedit::repertoire`) and the \
             commit (`editmodel::disposition::options`) must both carry the embedded \
             program reader from `editmodel::disposition::typing`. Trace: {path}.",
            line.raw
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
        "`{TYPED}`, outlined by the subset but never shown, typed and committed"
    )))
}
