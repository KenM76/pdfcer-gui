//! A web link asks before it opens, as Acrobat's Security Warning does:
//! Allow opens it, Block with "Remember" refuses it from then on without
//! asking, and the refusal names where to change it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/weblink_prompt.md`.

use super::dimdrive::{Fixture, click_on, press, run_on_with, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "view.reset_layout";
const DECISION: &str = "weblink-decision"; // ui-text-exempt: a trace event name, never displayed
const ANSWER: &str = "weblink-answer"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "external-open"; // ui-text-exempt: a trace event name, never displayed
const BODY: &str = "weblink.body";
const REMEMBER: &str = "weblink.remember";
const ALLOW: &str = "dialog.buttons.accept";
const BLOCK: &str = "dialog.buttons.cancel";
/// The `/URI` link's centre: rect 36,700–200,730 on page 1.
const URI_LINK: (f64, f64) = (118.0, 715.0);
const TARGET: &str = "https://example.invalid/spec";
const SITE: &str = "example.invalid";
const LINKS: Fixture = Fixture {
    file: "non-navigation-links.pdf",
    method: "Checked in; its four links are listed by `pdfcer list-links`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

/// See the module documentation.
pub struct AWebLinkAsksBeforeItOpens;

impl Check for AWebLinkAsksBeforeItOpens {
    fn name(&self) -> &'static str {
        "a_web_link_asks_before_it_opens"
    }

    fn defect(&self) -> &'static str {
        "a web link in a PDF is refused outright, opens without asking, or a remembered Block \
         is not honoured on the next click"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let env = [("PDFCER_DIAG_NO_LAUNCH", "1".to_owned())];
        run_on_with(self, ctx, &LINKS, INVOKE, "weblink", &env, drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    // 1 — the first click asks; Allow opens exactly the address in the file.
    if let Some(failure) = click_expect(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ALLOW)?;
    if let Some(failure) = answered(session, report, mark, "true", "false")? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    let opened = trace.last_after(OPENED, mark);
    if opened.and_then(|l| l.get("target")) != Some(TARGET) {
        return Ok(Some(format!(
            "Allow did not hand `{TARGET}` to Windows; `{OPENED}` after the answer: {}. \
             Trace: {}.",
            opened.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "Allow opened it: `{}`",
        opened.map_or("", |l| l.raw.as_str())
    ));

    // 2 — not remembered, so it asks again; Block with Remember opens nothing.
    if let Some(failure) = click_expect(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, REMEMBER)?;
    press(ctx, session, pointer, BLOCK)?;
    if let Some(failure) = answered(session, report, mark, "false", "true")? {
        return Ok(Some(failure));
    }
    if let Some(line) = session.trace()?.last_after(OPENED, mark) {
        return Ok(Some(format!(
            "Block opened the address anyway: `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }

    // 3 — the remembered Block refuses without a prompt.
    click_expect(ctx, report, session, pointer, "refuse")
}

/// Click the link and require `weblink-decision decision=<want> site=<SITE>`;
/// for `ask`, the prompt must have been drawn after the click.
fn click_expect(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    want: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &LINKS, 0, URI_LINK)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(DECISION, mark) else {
        return Ok(Some(format!(
            "the click on the web link traced no `{DECISION}` line, so nothing decided whether \
             to open it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("click: `{}`", line.raw));
    if line.get("decision") != Some(want) || line.get("site") != Some(SITE) {
        return Ok(Some(format!(
            "expected decision={want} site={SITE}, the trace says `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    // The ui-rect channel retires a region that stops drawing, so this is
    // whether the prompt is up now.
    let drawn = declared_in(&trace, ui_rect_event(ctx)?, BODY).is_some();
    if (want == "ask") != drawn {
        return Ok(Some(format!(
            "decision={want}, but the Security Warning was {} after the click. Trace: {}.",
            if drawn { "drawn" } else { "not drawn" },
            session.trace_path().display()
        )));
    }
    Ok(None)
}

/// The prompt's answer after `mark` carries `allow` and `remember`.
fn answered(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    allow: &str,
    remember: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(ANSWER, mark);
    if let Some(l) = line {
        report.note(format!("answer: `{}`", l.raw));
        if l.get("allow") == Some(allow) && l.get("remember") == Some(remember) {
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "expected `{ANSWER} allow={allow} remember={remember}`, the trace says {}. Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}
