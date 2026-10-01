//! `a_line_written_in_pieces_edits` — typing into a line a word processor wrote
//! as several text objects previews in the line's own face and commits: a
//! one-font line by the whole-line request, a two-font line by the narrowed
//! request that reaches only the piece that changed.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/word_line_edit.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "word-fragmented-lines.pdf";
/// Edit mode with the Edit Text tool armed, so one click opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
/// Inside `Required` on the fixture's one-font first line, in PDF points.
const ONE_FONT: (f64, f64) = (150.0, 703.0);
/// Inside `Applicant` on the fixture's two-font third line, in PDF points.
const TWO_FONTS: (f64, f64) = (100.0, 643.0);
const NARROWED: &str = "edit-text-narrowed"; // ui-text-exempt: a trace event name, never displayed
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ALineWrittenInPiecesEdits;

impl Check for ALineWrittenInPiecesEdits {
    fn name(&self) -> &'static str {
        "a_line_written_in_pieces_edits"
    }

    fn defect(&self) -> &'static str {
        "typing into a line a word processor wrote as several text objects is refused, or \
         previews in a stand-in face, because the whole line is sent to the engine as one \
         find and matches nothing"
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
    let source = repo_fixture(FIXTURE, "Run fixtures/word-fragmented-lines.PROVENANCE.py.")?;
    let doc = ctx.out("word-line-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("word-line.trace.txt"));
    spec.pdf = Some(doc.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("word-line.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let path = session.trace_path().display().to_string();
    for (at, tier) in [(ONE_FONT, "tier=Line"), (TWO_FONTS, "tier=Narrowed")] {
        let edited = edit_once(&session, &pointer, &mapping, at)?;
        if let Some(failure) = judge(&edited, tier, &path) {
            return Ok(Some(failure));
        }
        report.note(format!(
            "the line at y={} previewed and committed on {tier}",
            at.1
        ));
    }
    pointer.gone(&session)?;
    Ok(None)
}

/// What one click, `End`, `_` and `Escape` left in the trace: the last
/// preview line, the narrowed commit line counted from this edit, and the
/// last commit line.
struct Edited {
    preview: Option<String>,
    narrowed: Option<String>,
    committed: Option<String>,
}

fn edit_once(
    session: &Session,
    pointer: &ScriptedPointer,
    mapping: &CanvasMapping,
    click: (f64, f64),
) -> Result<Edited> {
    let before = session.trace()?.events(LEFT_EDGE).count();
    let narrowed_before = session.trace()?.events(NARROWED).count();
    let at = mapping.doc_to_window(DocPoint::new(0, click.0, click.1))?;
    pointer.click(session, at)?;
    session.settle(20);
    pointer.key(session, None, "End", None)?;
    pointer.type_text(session, None, "_")?;
    session.settle(25);
    let preview = session
        .trace()?
        .events(SHAPED)
        .last()
        .map(|l| l.raw.clone());
    pointer.key(session, None, "Escape", None)?;
    session.settle(25);
    let trace = session.trace()?;
    let narrowed = trace
        .events(NARROWED)
        .skip(narrowed_before)
        .filter(|l| l.get("for") == Some("commit"))
        .last()
        .map(|l| l.raw.clone());
    let committed = trace
        .events(LEFT_EDGE)
        .skip(before)
        .last()
        .map(|l| l.raw.clone());
    Ok(Edited {
        preview,
        narrowed,
        committed,
    })
}

/// The failure sentence for an edit that should have previewed shaped and
/// committed on `tier`, or `None` when it did.
fn judge(edited: &Edited, tier: &str, path: &str) -> Option<String> {
    let Some(preview) = &edited.preview else {
        return Some(format!(
            "★ a click and a keystroke produced no `{SHAPED}` line: no caret opened or no key \
             reached it. Trace: {path}."
        ));
    };
    if !(preview.contains("shaped=1") && preview.contains(tier)) {
        return Some(format!(
            "★★ the live preview was not the engine's layout on {tier}, so it shows a stand-in \
             face: `{preview}`. Trace: {path}."
        ));
    }
    let narrowed_ok = match (&edited.narrowed, tier) {
        (Some(l), "tier=Narrowed") => l.contains("landed=1"),
        (None, "tier=Line") => true,
        _ => false,
    };
    match &edited.committed {
        Some(c) if c.contains("committed=yes") && narrowed_ok => None,
        c => Some(format!(
            "★★★ Escape did not commit the edit on {tier}. Narrowed: {:?}; commit: {c:?}. \
             Trace: {path}.",
            edited.narrowed
        )),
    }
}
