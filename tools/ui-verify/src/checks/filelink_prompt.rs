//! A `/Launch` link asks before it opens a file, as Acrobat's Launch File
//! prompt does: Do Not Open opens nothing, Open with "Do not show this message
//! again" opens it and stops asking; and a `/GoToR` opens the PDF it names here
//! at the place it names, without asking.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/filelink_prompt.md`.

use super::dimdrive::{Fixture, click_on, press, run_on_with, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "view.reset_layout";
const DECISION: &str = "launch-decision"; // ui-text-exempt: a trace event name, never displayed
const ANSWER: &str = "launch-answer"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "external-open"; // ui-text-exempt: a trace event name, never displayed
const ARRIVED: &str = "filelink-open"; // ui-text-exempt: a trace event name, never displayed
const BODY: &str = "launch.body";
const DONT_ASK: &str = "launch.dont_ask";
const OPEN: &str = "dialog.buttons.accept";
const REFUSE: &str = "dialog.buttons.cancel";
/// The `/Launch` link's centre: rect 36,620–200,650 on page 1, `/F (payload.exe)`.
const LAUNCH_LINK: (f64, f64) = (118.0, 635.0);
/// The `/GoToR` link's centre: rect 36,580–200,610, `/F (other.pdf) /D (tree-target)`.
const REMOTE_LINK: (f64, f64) = (118.0, 595.0);
const PAYLOAD: &str = "payload.exe";
const OTHER: &str = "other.pdf";
/// `tree-target` names the fixture's third page, index 2.
const OTHER_PAGE: &str = "2";
const LINKS: Fixture = Fixture {
    file: "non-navigation-links.pdf",
    method: "Checked in; its four links are listed by `pdfcer list-links`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

/// See the module documentation.
pub struct AFileLinkAsksOrOpensHere;

impl Check for AFileLinkAsksOrOpensHere {
    fn name(&self) -> &'static str {
        "a_file_link_asks_or_opens_here"
    }

    fn defect(&self) -> &'static str {
        "a link to a file or another PDF only says where it points, opens a file without \
         asking, or opens another PDF at the wrong place"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        // The two files the links name, beside the opened copy. The payload is
        // never run: `PDFCER_DIAG_NO_LAUNCH` traces the hand-off instead.
        if let Err(why) = place_targets(ctx) {
            return CheckReport::new(self.name(), self.defect()).from_error(&why);
        }
        let env = [("PDFCER_DIAG_NO_LAUNCH", "1".to_owned())];
        run_on_with(self, ctx, &LINKS, INVOKE, "filelink", &env, drive)
    }
}

fn place_targets(ctx: &CheckContext) -> Result<()> {
    std::fs::write(ctx.out(PAYLOAD), b"not a program")
        .map_err(|e| Error::new(format!("writing {PAYLOAD}: {e}")))?;
    let fixture = crate::checks::driving::repo_fixture(LINKS.file, LINKS.method)?;
    std::fs::copy(fixture, ctx.out(OTHER))
        .map(drop)
        .map_err(|e| Error::new(format!("copying {OTHER}: {e}")))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    // 1 — the first click asks; Do Not Open opens nothing.
    if let Some(failure) = click_launch(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, REFUSE)?;
    if let Some(failure) = answered(session, report, mark, "false", "false")? {
        return Ok(Some(failure));
    }
    if let Some(line) = session.trace()?.last_after(OPENED, mark) {
        return Ok(Some(format!(
            "Do Not Open opened the file anyway: `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }

    // 2 — it asks again; Open with "Do not show this message again" opens it.
    if let Some(failure) = click_launch(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, DONT_ASK)?;
    press(ctx, session, pointer, OPEN)?;
    if let Some(failure) = answered(session, report, mark, "true", "true")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = handed_over(session, report, mark)? {
        return Ok(Some(failure));
    }

    // 3 — asking is off now: the click opens it without a prompt.
    let mark = session.trace()?.mark();
    if let Some(failure) = click_launch(ctx, report, session, pointer, "open")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = handed_over(session, report, mark)? {
        return Ok(Some(failure));
    }

    // 4 — a `/GoToR` opens the other PDF here, at the page its name names there.
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &LINKS, 0, REMOTE_LINK)?;
    session.settle(60);
    let trace = session.trace()?;
    let line = trace.last_after(ARRIVED, mark);
    let path_ok = line
        .and_then(|l| l.get("path"))
        .is_some_and(|p| p.ends_with(OTHER));
    if !path_ok || line.and_then(|l| l.get("page")) != Some(OTHER_PAGE) {
        return Ok(Some(format!(
            "the `/GoToR` click should open {OTHER} here at page index {OTHER_PAGE}; \
             `{ARRIVED}` after it: {}. Trace: {}.",
            line.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    }
    report.note(format!("GoToR: `{}`", line.map_or("", |l| l.raw.as_str())));
    Ok(None)
}

/// Click the `/Launch` link and require `launch-decision decision=<want>` naming
/// the payload; the prompt must be drawn exactly when the decision is `ask`.
fn click_launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    want: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &LINKS, 0, LAUNCH_LINK)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(DECISION, mark) else {
        return Ok(Some(format!(
            "the click on the `/Launch` link traced no `{DECISION}` line, so nothing decided \
             whether to open its file. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("click: `{}`", line.raw));
    let names_payload = line.get("file").is_some_and(|f| f.ends_with(PAYLOAD));
    if line.get("decision") != Some(want) || !names_payload {
        return Ok(Some(format!(
            "expected decision={want} naming {PAYLOAD}, the trace says `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    let drawn = declared_in(&trace, ui_rect_event(ctx)?, BODY).is_some();
    if (want == "ask") != drawn {
        return Ok(Some(format!(
            "decision={want}, but the Launch File prompt was {} after the click. Trace: {}.",
            if drawn { "drawn" } else { "not drawn" },
            session.trace_path().display()
        )));
    }
    Ok(None)
}

/// The prompt's answer after `mark` carries `open` and `dont_ask`.
fn answered(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    open: &str,
    dont_ask: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(ANSWER, mark);
    if let Some(l) = line {
        report.note(format!("answer: `{}`", l.raw));
        if l.get("open") == Some(open) && l.get("dont_ask") == Some(dont_ask) {
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "expected `{ANSWER} open={open} dont_ask={dont_ask}`, the trace says {}. Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}

/// The payload was handed to Windows as a file after `mark`.
fn handed_over(session: &Session, report: &mut CheckReport, mark: usize) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(OPENED, mark);
    let ok = line.is_some_and(|l| {
        l.get("kind") == Some("file") && l.get("target").is_some_and(|t| t.ends_with(PAYLOAD))
    });
    if ok {
        report.note(format!("opened: `{}`", line.map_or("", |l| l.raw.as_str())));
        return Ok(None);
    }
    Ok(Some(format!(
        "Open did not hand {PAYLOAD} to Windows as a file; `{OPENED}` after it: {}. Trace: {}.",
        line.map_or("none", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}
