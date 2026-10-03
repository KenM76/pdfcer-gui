//! `enter_breaks_a_paragraph_on_the_page` — Enter at the end of a line of a
//! paragraph already on the page opens the paragraph and breaks it there;
//! `Ctrl+S` writes two paragraphs.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/enter_paragraph.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "paragraph.pdf";
/// Edit mode with the Edit Text tool armed, so one click opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
/// On the third line, in PDF points.
const LINE: (f64, f64) = (120.0, 668.0);
/// The fixture's six lines, top to bottom; the caret's is the third.
const LINES: [&str; 6] = [
    "The drawing office keeps every revision of a sheet", // ui-text-exempt: fixture text
    "in one file, and the notes beside",                  // ui-text-exempt: fixture text
    "the title block are edited far more often",          // ui-text-exempt: fixture text
    "than the geometry is. A note that has been",         // ui-text-exempt: fixture text
    "retyped twice no longer fills",                      // ui-text-exempt: fixture text
    "its box.",                                           // ui-text-exempt: fixture text
];
const CARET_LINE: usize = 2;
const TYPING: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed
const PROMOTED: &str = "text-edit-promoted"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-edit-enter-declined"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct EnterBreaksAParagraphOnThePage;

impl Check for EnterBreaksAParagraphOnThePage {
    fn name(&self) -> &'static str {
        "enter_breaks_a_paragraph_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "Enter in a line of text already on the page only says the line cannot be split, so a \
         paragraph cannot be broken in two without retyping it in a new box"
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
    let source = repo_fixture(FIXTURE, "Run python tools/gen-reflow-fixture.py.")?;
    let doc = ctx.out("enter-paragraph-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("enter-paragraph.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("enter-paragraph.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// The value of `key` on the last `event` line, as a number.
fn last_num(session: &Session, event: &str, key: &str) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events(event)
        .last()
        .and_then(|l| l.get(key).and_then(|v| v.parse().ok())))
}

fn last_raw(session: &Session, event: &str) -> Result<Option<String>> {
    Ok(session.trace()?.events(event).last().map(|l| l.raw.clone()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    let path = session.trace_path().display().to_string();
    session.settle(45);
    let source_len = std::fs::metadata(&doc).map_or(0, |m| m.len());
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, LINE.0, LINE.1))?,
    )?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    session.settle(15);
    pointer.key(&session, None, "Enter", None)?;
    session.settle(20);

    // --- 1: the line opened into its paragraph, caret where it was ---------
    let joined = LINES.join(" ");
    let want_len = joined.chars().count();
    let want_caret = LINES[..=CARET_LINE]
        .iter()
        .map(|l| l.chars().count() + 1)
        .sum::<usize>()
        - 1;
    let promoted = last_raw(&session, PROMOTED)?;
    let len = last_num(&session, PROMOTED, "len")?;
    let caret = last_num(&session, PROMOTED, "caret")?;
    if len != Some(want_len) || caret != Some(want_caret) {
        return Ok(Some(format!(
            "Enter at the end of line 3 did not open the paragraph: promoted {promoted:?} \
             (want len={want_len} caret={want_caret}); declined {:?}. Trace: {path}.",
            last_raw(&session, DECLINED)?
        )));
    }
    let typed = last_num(&session, TYPING, "len")?;
    if typed != Some(want_len + 1) {
        return Ok(Some(format!(
            "the paragraph opened but Enter added no line break: draft length {typed:?}, want \
             {}. Trace: {path}.",
            want_len + 1
        )));
    }
    report.note(format!(
        "Enter opened the paragraph ({want_len} characters, caret {want_caret}) and broke it"
    ));
    let shot = ctx.out("enter-paragraph-open.png");
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);

    // --- 2: Ctrl+S commits two paragraphs and saves ------------------------
    pointer.key(&session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let applied = last_raw(&session, APPLIED)?;
    let saved = last_raw(&session, SAVED)?;
    if !applied
        .as_deref()
        .is_some_and(|l| l.contains("paragraphs=2"))
        || !saved.as_deref().is_some_and(|l| l.contains("outcome=ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S did not commit the paragraph and save: applied {applied:?}; saved \
             {saved:?}. Trace: {path}."
        )));
    }

    // --- 3: the saved revision ends a line on "often" and starts one on "than"
    let bytes =
        std::fs::read(&doc).map_err(|e| Error::new(format!("reading the saved copy: {e}")))?;
    let start = usize::try_from(source_len).unwrap_or(0).min(bytes.len());
    let tail = &bytes[start..];
    let has = |needle: &str| tail.windows(needle.len()).any(|w| w == needle.as_bytes());
    if !(has("often)") && has("(than") && !has("often than")) {
        return Ok(Some(format!(
            "the saved revision ({} bytes appended) does not break after \"often\": a string \
             ending `often)` {}, one starting `(than` {}, `often than` on one line {}. \
             Trace: {path}.",
            tail.len(),
            has("often)"),
            has("(than"),
            has("often than")
        )));
    }
    report.note(format!(
        "Ctrl+S committed {} and saved; the revision ends a line on \"often\" and starts the \
         next paragraph on \"than\"",
        applied.unwrap_or_default()
    ));
    pointer.gone(&session)?;
    Ok(None)
}
