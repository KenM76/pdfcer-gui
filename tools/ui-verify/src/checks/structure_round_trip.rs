//! `hand_edits_compile_back_as_an_appended_update` — File ▸ Export ▸ Export for
//! hand editing… writes a copy with every stream decoded and its source's
//! fingerprint; a same-length text change to it, compiled back with Compile
//! hand edits…, changes one object of the open document, and Save appends it
//! to the untouched original; after a later edit, compiling the same copy is
//! refused; a copy with its fingerprint line broken is applied and says so.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/structure_round_trip.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const EXPORTED: &str = "export-structure"; // ui-text-exempt: a trace event name, never displayed
const COMPILED: &str = "import-structure"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "import-structure-refused"; // ui-text-exempt: a trace event name, never displayed
const ROTATED: &str = "rotate-pages"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// The funnel's line for the compile, which carries the receipt.
const RECEIPT: &str = "compile-hand-edits"; // ui-text-exempt: a trace event name, never displayed
/// A fragment of `text::structure::unrecorded_base`, searched for in the receipt.
const UNCHECKED: &str = "could not check it"; // ui-text-exempt: a needle matched against the trace
const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const PAGES_TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const EXPORT_ITEM: &str = "ribbon.item.file.export_structure"; // ui-text-exempt: a trace region name, never displayed
const COMPILE_ITEM: &str = "ribbon.item.file.import_structure"; // ui-text-exempt: a trace region name, never displayed
const ROTATE_ITEM: &str = "ribbon.item.pages.rotate_right"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.export.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name
/// Review, so the Pages tab's rotate is offered for the stale-base step.
const INVOKE: &str = "mode.review";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// A text-showing string in the fixture's page content, and its same-length
/// replacement, so the hand edit leaves `/Length` true.
const BEFORE: &[u8] = b"(Construction drawing)";
const AFTER: &[u8] = b"(Hand-edited drawings)";
/// Present in the fixture (its fonts are Flate-compressed) and absent from an
/// export that decoded every stream.
const FLATE: &[u8] = b"/FlateDecode";
/// The export's fingerprint line, and the same-length break that makes it
/// unrecognisable.
const MARKER: &[u8] = b"%PdfcerExportBase sha256:";
const BROKEN: &[u8] = b"%PdfcerExportBasX sha256:";

pub struct HandEditsCompileBackAsAnAppendedUpdate;

impl Check for HandEditsCompileBackAsAnAppendedUpdate {
    fn name(&self) -> &'static str {
        "hand_edits_compile_back_as_an_appended_update"
    }

    fn defect(&self) -> &'static str {
        "Export for hand editing… writes compressed streams or no fingerprint, or Compile hand \
         edits… changes more than the edited object, Save loses the original bytes, a copy \
         exported before a later edit is compiled and silently undoes it, or a copy that \
         cannot be checked is applied without saying so"
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

/// The files one run reads and writes, all under the check's output folder.
struct Files {
    source: std::path::PathBuf,
    export: std::path::PathBuf,
    original: Vec<u8>,
    unrecorded: std::path::PathBuf,
}

fn files(ctx: &CheckContext) -> Result<Files> {
    let fixture = crate::checks::driving::repo_fixture("a1-titleblock.pdf", "It is committed.")?;
    let files = Files {
        source: ctx.out("structure-source.pdf"),
        export: ctx.out("structure-export.pdf"),
        original: read(&fixture)?,
        unrecorded: ctx.out("structure-unrecorded.pdf"),
    };
    std::fs::copy(&fixture, &files.source)
        .map_err(|e| Error::new(format!("copying the fixture: {e}")))?;
    for path in [&files.export, &files.unrecorded] {
        let _ = std::fs::remove_file(path);
        if path.exists() {
            return Err(Error::new(format!("cannot clear {}.", path.display())));
        }
    }
    Ok(files)
}

fn launch(ctx: &CheckContext, files: &Files) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("structure.trace.txt"));
    spec.pdf = Some(files.source.clone());
    let saves = files.export.display().to_string();
    let edited = files.export.display().to_string();
    let unrecorded = files.unrecorded.display().to_string();
    for (key, value) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
        (viewport_env, OFFSCREEN.to_owned()),
        ("PDFCER_DIAG_INVOKE", INVOKE.to_owned()),
        (SAVE_PATH_ENV, saves),
        (OPEN_PATH_ENV, format!("{edited};{edited};{unrecorded}")),
    ] {
        spec.env.push((key.to_owned(), value));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("structure.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let files = files(ctx)?;
    let (session, pointer) = launch(ctx, &files)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let path = session.trace_path().display().to_string();

    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    let press = |item: &str| -> Result<()> {
        click(FILE_TAB)?;
        let trace = session.trace()?;
        if declared(&trace, ui_rect, item).is_none()
            && declared(&trace, ui_rect, COLLAPSED).is_some()
        {
            click(COLLAPSED)?;
        }
        click(item)?;
        session.settle(20);
        Ok(())
    };

    press(EXPORT_ITEM)?;
    if let Some(failure) = judge_export(&session.trace()?, &files, &path)? {
        return Ok(Some(failure));
    }
    hand_edit(&files.export, &files.unrecorded)?;

    press(COMPILE_ITEM)?;
    if let Some(failure) = judge_compile(&session.trace()?, "matches", &path) {
        return Ok(Some(failure));
    }
    let viewport = declared_in(&session.trace()?, ui_rect, FILE_TAB).and_then(|(_, vp)| vp);
    pointer.key(&session, viewport.as_deref(), "S", Some("ctrl"))?;
    session.settle(40);
    if let Some(failure) = judge_saved(&session.trace()?, &files, &path)? {
        return Ok(Some(failure));
    }

    click(PAGES_TAB)?;
    click(ROTATE_ITEM)?;
    session.settle(20);
    if session.trace()?.last(ROTATED).is_none() {
        return Ok(Some(format!(
            "the rotate was pressed and no `{ROTATED}` line followed, so the stale-base step \
             has no later edit to protect. Trace: {path}."
        )));
    }
    press(COMPILE_ITEM)?;
    let trace = session.trace()?;
    let refused = trace.last(REFUSED).map(|l| l.raw.clone());
    let applied = trace.events(COMPILED).count();
    if !refused
        .as_deref()
        .is_some_and(|l| l.contains("reason=stale-base"))
        || applied != 1
    {
        return Ok(Some(format!(
            "★★★ the copy was compiled again after the page was rotated. Refusal: {refused:?}; \
             `{COMPILED}` lines: {applied}. Compiling it undoes the rotation. Trace: {path}."
        )));
    }

    press(COMPILE_ITEM)?;
    pointer.gone(&session)?;
    if let Some(failure) = judge_compile(&session.trace()?, "unrecorded", &path) {
        return Ok(Some(failure));
    }
    report.artifact(files.export.clone());
    report.artifact(files.source.clone());
    Ok(None)
}

fn judge_export(trace: &crate::trace::Trace, files: &Files, path: &str) -> Result<Option<String>> {
    let Some(line) = trace.last(EXPORTED) else {
        return Ok(Some(format!(
            "Export for hand editing… was pressed and no `{EXPORTED}` line followed. Last \
             failure: {:?}. Trace: {path}.",
            trace.last("structure-failed").map(|l| l.raw.clone())
        )));
    };
    let bytes = read(&files.export)?;
    let source = read(&files.source)?;
    if !contains(&source, FLATE)
        || contains(&bytes, FLATE)
        || !contains(&bytes, BEFORE)
        || !contains(&bytes, MARKER)
    {
        return Ok(Some(format!(
            "★ the export does not read as decoded and fingerprinted: `/FlateDecode` in the \
             fixture {}, in the export {}; fingerprint line {}; `{}` occurs {} times in the \
             export. Line: `{}`.",
            contains(&source, FLATE),
            contains(&bytes, FLATE),
            contains(&bytes, MARKER),
            String::from_utf8_lossy(BEFORE),
            count(&bytes, BEFORE),
            line.raw
        )));
    }
    Ok(None)
}

/// The newest `import-structure` line: one changed object, a stream matched
/// after decoding, and the base the copy should have been judged to have.
fn judge_compile(trace: &crate::trace::Trace, base: &str, path: &str) -> Option<String> {
    let Some(line) = trace.last(COMPILED) else {
        return Some(format!(
            "Compile hand edits… was pressed and no `{COMPILED}` line followed. Refusal: {:?}; \
             failure: {:?}. Trace: {path}.",
            trace.last(REFUSED).map(|l| l.raw.clone()),
            trace.last("structure-failed").map(|l| l.raw.clone())
        ));
    };
    let one_object = line.get("modified") == Some("1")
        && line.get("added") == Some("0")
        && line.get("removed") == Some("0")
        && line.get("streams_matched").is_some_and(|n| n != "0");
    if !one_object || line.get("base") != Some(base) {
        return Some(format!(
            "★★ one object differs from the document and the copy's base is `{base}`; the \
             compile traced `{}`. A compressed stream the export decoded must compare \
             unchanged, and an unchecked copy must say so.",
            line.raw
        ));
    }
    let receipt = trace
        .last(RECEIPT)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    (receipt.contains(UNCHECKED) != (base == "unrecorded")).then(|| {
        format!(
            "★★ the copy's base is `{base}`, and the receipt {} that the copy could not be \
             checked: `{receipt}`. Trace: {path}.",
            if base == "unrecorded" {
                "does not say"
            } else {
                "says"
            }
        )
    })
}

/// Ctrl+S after the compile: saved in place, the original's bytes first, the
/// hand edit after them.
fn judge_saved(trace: &crate::trace::Trace, files: &Files, path: &str) -> Result<Option<String>> {
    let saved = trace.last(SAVED).map(|l| l.raw.clone());
    let source = read(&files.source)?;
    let appended = source.len() > files.original.len() && source.starts_with(&files.original);
    if !saved.as_deref().is_some_and(|l| l.contains("outcome=ok"))
        || !appended
        || !contains(&source, AFTER)
    {
        return Ok(Some(format!(
            "★★ the compiled edit was not saved as an update: `{SAVED}` {saved:?}; original \
             bytes kept and extended {appended}; holds the hand edit {}. Trace: {path}.",
            contains(&source, AFTER)
        )));
    }
    Ok(None)
}

/// The operator's text editor: the first occurrence replaced at the same
/// length, saved in place; and a second copy with the fingerprint line broken.
fn hand_edit(export: &std::path::Path, unrecorded: &std::path::Path) -> Result<()> {
    let mut bytes = read(export)?;
    let at = bytes
        .windows(BEFORE.len())
        .position(|w| w == BEFORE)
        .ok_or_else(|| Error::new("the export holds no text to edit."))?;
    bytes[at..at + AFTER.len()].copy_from_slice(AFTER);
    std::fs::write(export, &bytes).map_err(|e| Error::new(format!("saving the hand edit: {e}")))?;
    let mark = bytes
        .windows(MARKER.len())
        .position(|w| w == MARKER)
        .ok_or_else(|| Error::new("the export carries no fingerprint line."))?;
    bytes[mark..mark + BROKEN.len()].copy_from_slice(BROKEN);
    std::fs::write(unrecorded, bytes)
        .map_err(|e| Error::new(format!("writing the unrecorded copy: {e}")))
}

fn read(path: &std::path::Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| Error::new(format!("reading {}: {e}", path.display())))
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    count(haystack, needle) > 0
}
