//! `added_invisible_text_is_saved_invisible` — Edit ▸ Add text with the text
//! pen's Invisible switch ticked in Properties, a click on blank paper, typed
//! letters, a click away and Ctrl+S write a run in rendering mode 3 into the
//! saved file. Run with the scripted pointer in a window placed off the
//! desktop, on a copy of `fixtures/layer-assign.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/invisible_text_scripted.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const INVOKE: &str = "mode.edit,edit.add_text"; // ui-text-exempt: command ids, never displayed
const SWITCH: &str = "properties.tool.text_pen_invisible"; // ui-text-exempt: a trace region name
const PROPERTIES_BODY: &str = "dock.body.file.properties"; // ui-text-exempt: a trace region name
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: a trace region name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Letters no other content of the fixture spells, shown by one literal `Tj`.
const WORD: &str = "QZXW";
/// Blank paper on the fixture's 800 x 600 page, clear of its boxes and annotation.
const SPOT: (f64, f64) = (150.0, 450.0);
/// Blank paper above everything, where the committing click lands.
const AWAY: (f64, f64) = (350.0, 570.0);

/// See the module documentation.
pub struct AddedInvisibleTextIsSavedInvisible;

impl Check for AddedInvisibleTextIsSavedInvisible {
    fn name(&self) -> &'static str {
        "added_invisible_text_is_saved_invisible"
    }

    fn defect(&self) -> &'static str {
        "the text pen's Invisible switch is not on screen, does not reach the pen, or the added \
         run is written in a visible rendering mode — a word added to a scan's recognised text \
         is then drawn on the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let copy = ctx.out("invisible-text.pdf");
        let driven = repo_fixture(FIXTURE, METHOD)
            .and_then(|fixture| {
                std::fs::copy(&fixture, &copy)
                    .map_err(|e| Error::new(format!("copying the fixture: {e}")))
            })
            .and_then(|_| launch(ctx, &mut report, &copy))
            .and_then(|(session, pointer)| {
                let outcome = drive(ctx, &mut report, &session, &pointer, &copy);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            })
            .and_then(|failure| match failure {
                Some(f) => Ok(Some(f)),
                None => judge(&copy, &mut report),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &std::path::Path,
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("invisible_text.trace.txt"));
    spec.pdf = Some(pdf.to_path_buf());
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("invisible_text.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

/// Click a declared region at its centre, in its own viewport.
fn press(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, region: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
        let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
        Error::new(format!(
            "no `{region}` region. Declared under `{prefix}`: {}.",
            list(&declared_names(&trace, ui_rect, prefix))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(())
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    pdf: &std::path::Path,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, PROPERTIES_BODY).is_none() {
        press(session, pointer, ui_rect, PROPERTIES_TAB)?;
    }
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SWITCH).is_none() {
        return Ok(Some(format!(
            "★★★ with Add text armed, Properties publishes no `{SWITCH}`: the switch is not on \
             screen. Declared under `properties.tool`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.tool")),
            session.trace_path().display()
        )));
    }
    press(session, pointer, ui_rect, SWITCH)?;
    let trace = session.trace()?;
    match trace.last("text-pen") {
        Some(line) if line.get("invisible") == Some("1") => {
            report.note(line.raw.clone());
        }
        other => {
            return Ok(Some(format!(
                "★★★ clicking the switch did not put `invisible=1` on the pen: last `text-pen` \
                 line {:?}.",
                other.map(|l| l.raw.clone())
            )));
        }
    }
    let page = crate::fixture::page_geometry(pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, SPOT.0, SPOT.1))?,
    )?;
    session.settle(20);
    if !session
        .trace()?
        .events("text-edit-caret")
        .any(|l| l.get("kind") == Some("Add"))
    {
        return Ok(Some(format!(
            "the click on blank paper traced no `text-edit-caret kind=Add`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    pointer.type_text(session, None, WORD)?;
    session.settle(20);
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, AWAY.0, AWAY.1))?,
    )?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(added) = trace.events("add-text").next() else {
        return Ok(Some(format!(
            "the click away traced no `add-text`: the draft never reached the engine. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(added.raw.clone());
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let trace = session.trace()?;
    if !trace
        .events("save-in-place")
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `save-in-place outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(None)
}

/// The saved file must hold a content stream that shows `WORD` in mode 3.
fn judge(pdf: &std::path::Path, report: &mut CheckReport) -> Result<Option<String>> {
    let shown = format!("({WORD}) Tj");
    let modes = modes_showing(pdf, WORD)?;
    report.note(format!("`{shown}` found under rendering modes {modes:?}"));
    if modes.is_empty() {
        return Ok(Some(format!(
            "★★★ the saved file shows `{shown}` in no content stream: the typed word was not \
             written."
        )));
    }
    Ok((!modes.iter().all(|m| m == "3")).then(|| {
        format!(
            "★★★ the added run is written in rendering mode {modes:?}, not 3: the Invisible \
             switch did not reach the engine, and the word is drawn on the page."
        )
    }))
}

/// The rendering mode in force where each stream of `pdf` shows the literal
/// string `(word)`, by `Tj` or inside a `TJ` array: the last `<n> Tr` between
/// the show's `BT` and the string, or `none`.
pub(crate) fn modes_showing(pdf: &std::path::Path, word: &str) -> Result<Vec<String>> {
    let bytes = std::fs::read(pdf).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    let shown = format!("({word})");
    let mut modes = Vec::new();
    for body in streams(&bytes) {
        let text = String::from_utf8_lossy(&body);
        let Some(at) = text.find(&shown) else {
            continue;
        };
        // The engine writes `<mode> Tr` inside the run's own BT, before its show.
        let head = &text[..at];
        let bt = head.rfind("BT").unwrap_or(0);
        let mode = head[bt..]
            .rfind(" Tr")
            .and_then(|end| head[bt..bt + end].split_whitespace().last())
            .unwrap_or("none")
            .to_owned();
        modes.push(mode);
    }
    Ok(modes)
}

/// Every stream body in the file, inflated when it inflates, raw otherwise.
fn streams(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(start) = find(&bytes[from..], b"stream").map(|i| from + i) {
        let mut body = start + b"stream".len();
        if bytes.get(body) == Some(&b'\r') {
            body += 1;
        }
        if bytes.get(body) == Some(&b'\n') {
            body += 1;
        }
        let Some(end) = find(&bytes[body..], b"endstream").map(|i| body + i) else {
            break;
        };
        let raw = &bytes[body..end];
        out.push(
            miniz_oxide::inflate::decompress_to_vec_zlib(raw).unwrap_or_else(|_| raw.to_vec()),
        );
        from = end + b"endstream".len();
    }
    out
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}
