//! `a_key_the_subset_lacks_comes_from_its_installed_face` — a letter an
//! embedded subset has no outline for is added to the subset from the same
//! face in the font folders, so the line keeps its own font. With `--pdf` and
//! `--doc-point` it types `z` into the operator's drawing, whose subset is
//! extended from this computer's font folder.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/augment_subset.md`.

use std::path::PathBuf;

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Startup commands per route, and the route's name in artifact files: the
/// default opens the clicked paragraph; `edit.paragraphs` turns that off, so
/// the click opens the line alone, whose keystroke query is the run's own.
const PARAGRAPH: (&str, &str) = ("mode.edit,edit.text", "paragraph");
const LINE: (&str, &str) = ("mode.edit,edit.paragraphs,edit.text", "line");
const FIXTURE: &str = "augment-subset.pdf";
/// The folder holding the face the fixture's subset was cut from.
const FONTS: &str = "augment-fonts";
/// Inside the `A` of `ABC`, set at 24 pt from (10, 40).
const AT: (f64, f64) = (18.0, 48.0);
/// Outlined by the installed face, not by the subset.
const TYPED: &str = "D";
/// Typed into the operator's drawing: the letter his CenturyGothic subset lacks.
const TYPED_IN_HIS: &str = "z";
const FACE_FILE: &str = "face.ttf";
/// How many ten-frame waits the folder index is given.
const INDEX_WAITS: usize = 60;
const INDEXED: &str = "installed-faces-indexed"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "refused-char"; // ui-text-exempt: a trace event name, never displayed
/// The caret's notice for a refused key; `characters=` names it by code point.
const NOTICE: &str = "text-edit-refused-keys"; // ui-text-exempt: a trace event name, never displayed
const REFACED: &str = "text-edit-reface-planned"; // ui-text-exempt: a trace event name, never displayed
/// The commit's disclosures: a line's, then a paragraph's (a click that opens
/// a whole paragraph commits through the block verb).
const EDITS: [&str; 2] = ["edit-text", "edit-block-text"]; // ui-text-exempt: trace event names, never displayed
/// A paragraph commit's summary; `glyphs_added=` counts letters added to subsets.
const BLOCK_APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
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

/// What is driven: the file, a point on the run, the letter, and the folder
/// holding the face (none for the operator's drawing, which is extended from
/// this computer's own font folder).
struct Subject {
    doc: PathBuf,
    at: DocPoint,
    page: PageGeometry,
    typed: &'static str,
    fonts: Option<String>,
    face_file: Option<&'static str>,
    routes: &'static [(&'static str, &'static str)],
}

/// The committed fixture and the face it was cut from.
fn fixture_subject(doc: PathBuf) -> Result<Subject> {
    let source = repo_fixture(FIXTURE, "It is committed under fixtures/.")?;
    let face = repo_fixture(
        &format!("{FONTS}/{FACE_FILE}"),
        "It is committed under fixtures/.",
    )?;
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("could not read a page size from {FIXTURE}.")))?;
    Ok(Subject {
        doc,
        at: DocPoint::new(0, AT.0, AT.1),
        page,
        typed: TYPED,
        fonts: Some(face.parent().unwrap_or(&face).display().to_string()),
        face_file: Some(FACE_FILE),
        routes: &[PARAGRAPH],
    })
}

/// The operator's drawing when `--pdf` is passed (copied, since the check
/// edits it), else the committed fixture.
fn subject(ctx: &CheckContext) -> Result<Subject> {
    let doc = ctx.out("augment-subset.pdf");
    let Some(source) = ctx.pdf.clone() else {
        return fixture_subject(doc);
    };
    let at = ctx.target.ok_or_else(|| {
        Error::new("--pdf without --doc-point. Pass PAGE,X,Y in PDF user space on a text run.")
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&source).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                source.display()
            ))
        })?,
    };
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    Ok(Subject {
        doc,
        at,
        page,
        typed: TYPED_IN_HIS,
        fonts: None,
        face_file: None,
        routes: &[PARAGRAPH, LINE],
    })
}

fn launch(
    ctx: &CheckContext,
    subject: &Subject,
    (invoke, route): (&str, &str),
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("augment-subset.{route}.trace.txt")));
    spec.pdf = Some(subject.doc.clone());
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", invoke),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    if let Some(fonts) = &subject.fonts {
        spec.env
            .push(("PDFCER_DIAG_FONT_DIR".to_owned(), fonts.clone()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("augment-subset.{route}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

/// Drive every route of the subject; the first failure is the verdict.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let subject = subject(ctx)?;
    let mut notes = Vec::new();
    for &route in subject.routes {
        match drive_route(ctx, report, &subject, route)? {
            Ok(note) => notes.push(format!("{}: {note}", route.1)),
            Err(failure) => return Ok(Err(format!("{}: {failure}", route.1))),
        }
    }
    Ok(Ok(notes.join(
        "
",
    )))
}

/// Wait for the font folders to be indexed, click into the run, go to its
/// end, type the letter and commit.
fn drive_route(
    ctx: &CheckContext,
    report: &mut CheckReport,
    subject: &Subject,
    route: (&str, &str),
) -> Result<std::result::Result<String, String>> {
    if subject.routes.len() > 1 {
        std::fs::copy(ctx.pdf.as_deref().unwrap_or(&subject.doc), &subject.doc)
            .map_err(|e| Error::new(format!("refreshing the copy: {e}")))?;
    }
    let (session, pointer) = launch(ctx, subject, route)?;
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
    let mapping = CanvasMapping::from_trace(
        &session.trace()?,
        &ctx.profile.vocab,
        subject.page,
        subject.at.page,
    )?;
    pointer.click(&session, mapping.doc_to_window(subject.at)?)?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, subject.typed)?;
    session.settle(30);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(60);
    let outcome = judge(&session, subject)?;
    pointer.gone(&session)?;
    Ok(outcome)
}

/// A refusal of the letter at the keystroke, or the caret's notice that it
/// will be set in another face, or a plan into another face.
fn refusal(trace: &crate::trace::Trace, typed: &str, path: &str) -> Option<String> {
    let quoted = format!("'{typed}'");
    let code = format!("U+{:04X}", typed.chars().next().map_or(0, u32::from));
    if let Some(line) = trace
        .events(REFUSED)
        .find(|l| l.get("character") == Some(quoted.as_str()))
    {
        return Some(format!(
            "★★★★ `{typed}` was refused at the keystroke (`{}`): the keystroke query \
             (`canvas::textedit::repertoire`) was not asked with the installed faces. Trace: {path}.",
            line.raw
        ));
    }
    if let Some(line) = trace
        .events(NOTICE)
        .find(|l| l.get("characters").is_some_and(|c| c.contains(&code)))
    {
        return Some(format!(
            "★★★★ `{typed}` drew the refused-key notice (`{}`): the caret says the letter will \
             be set in another face, though the line's own font is installed. Trace: {path}.",
            line.raw
        ));
    }
    trace.events(REFACED).last().map(|line| {
        format!(
            "★★★★ `{typed}` was planned into another face (`{}`) although the subset's own \
             face is in the font folders. Trace: {path}.",
            line.raw
        )
    })
}

/// Pass when the letter was neither refused, nor noticed as set in another
/// face, nor re-faced, and the commit says it added a glyph from an installed
/// face (the fixture's own `face.ttf`, when the fixture is driven).
fn judge(session: &Session, subject: &Subject) -> Result<std::result::Result<String, String>> {
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let typed = subject.typed;
    if let Some(failure) = refusal(&trace, typed, &path) {
        return Ok(Err(failure));
    }
    let edit = trace
        .lines
        .iter()
        .rfind(|l| EDITS.contains(&l.event.as_str()))
        .map(|l| l.raw.clone());
    let added = edit.as_deref().is_some_and(|l| {
        l.contains("added 1 glyph(s)") && subject.face_file.is_none_or(|f| l.contains(f))
    });
    if !added {
        let from = subject.face_file.unwrap_or("an installed face");
        return Ok(Err(format!(
            "★★★★ the commit does not say it added `{typed}` from `{from}` (`{edit:?}`). \
             Trace: {path}."
        )));
    }
    let committed = trace
        .events(LEFT_EDGE)
        .any(|l| l.get("committed") == Some("yes"))
        || trace
            .events(BLOCK_APPLIED)
            .any(|l| l.get_usize("glyphs_added").is_some_and(|n| n > 0));
    if !committed {
        return Ok(Err(format!(
            "★★★★ typing `{typed}` committed nothing: no `{LEFT_EDGE} committed=yes` and no \
             `{BLOCK_APPLIED}` with a glyph added. Trace: {path}."
        )));
    }
    Ok(Ok(format!(
        "`{typed}` was added to the subset and committed in the line's own font: `{}`",
        edit.unwrap_or_default()
    )))
}
