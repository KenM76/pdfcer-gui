//! `the_prepress_commands_change_the_page_and_say_what_they_did` — Fix
//! hairlines, Flatten transparency and Convert colours each change the
//! fixture's one case, report it, and find nothing on a second run.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/prepress.md`.

use super::dimdrive::{Fixture, launch_with, press};
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "prepress-mix.pdf",
    method: "Rebuild it with `python fixtures/prepress-mix.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const ASKED: &str = "prepress-asked"; // ui-text-exempt: a trace event name
const APPLIED: &str = "prepress-applied"; // ui-text-exempt: a trace event name
const SHOWN: &str = "prepress-shown"; // ui-text-exempt: a trace event name
const APPLY: &str = "prepress.apply."; // ui-text-exempt: a trace region name prefix

/// One command: what opens it, its window's suffix, its verb's trace token,
/// the count that names the fixture's case, and that count's first value.
struct Case {
    command: &'static str,
    kind: &'static str,
    verb: &'static str,
    count: &'static str,
    first: usize,
}

const CASES: [Case; 3] = [
    Case {
        command: "file.fix_hairlines",
        kind: "hairlines",
        verb: "hairlines",
        count: "widened",
        first: 1,
    },
    Case {
        command: "file.flatten_transparency",
        kind: "flatten",
        verb: "flatten",
        count: "flattened",
        first: 1,
    },
    Case {
        command: "file.convert_colours",
        kind: "colours",
        verb: "convert-colours",
        count: "converted",
        first: 1,
    },
];

/// `the_prepress_commands_change_the_page_and_say_what_they_did`.
pub struct ThePrepressCommandsChangeThePageAndSayWhatTheyDid;

impl Check for ThePrepressCommandsChangeThePageAndSayWhatTheyDid {
    fn name(&self) -> &'static str {
        "the_prepress_commands_change_the_page_and_say_what_they_did"
    }

    fn defect(&self) -> &'static str {
        "Fix hairlines, Flatten transparency or Convert colours finds nothing on a page with \
         its case, changes something other than its count, leaves its case for a second run \
         to find, or shows no report in its window"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        for case in &CASES {
            match one(ctx, &mut report, case) {
                Ok(None) => {}
                Ok(Some(failure)) => return report.fail(failure),
                Err(why) => return report.from_error(&why),
            }
        }
        report.pass()
    }
}

/// Launch on a fresh copy with `case`'s window open, apply twice.
fn one(ctx: &CheckContext, report: &mut CheckReport, case: &Case) -> Result<Option<String>> {
    let invoke = format!("mode.edit,{}", case.command);
    let stem = format!("prepress-{}", case.kind);
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, &invoke, &stem, &[])?;
    let outcome = twice(ctx, report, &session, &pointer, case);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

fn twice(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    case: &Case,
) -> Result<Option<String>> {
    let first = step!(apply(ctx, report, session, pointer, case));
    if let Some(failure) = expect(&first, case, case.first, true) {
        return Ok(Some(failure));
    }
    let second = step!(apply(ctx, report, session, pointer, case));
    Ok(expect(&second, case, 0, false).map(|f| format!("{f} The first run left its case.")))
}

/// `line` names `case`'s verb, `want` for its count, and `changed` to match.
fn expect(line: &TraceLine, case: &Case, want: usize, changed: bool) -> Option<String> {
    let got = line.get_usize(case.count);
    let ok = line.get("verb") == Some(case.verb)
        && got == Some(want)
        && line.get("changed") == Some(if changed { "1" } else { "0" });
    (!ok).then(|| {
        format!(
            "`{}`; expected verb={} {}={want} changed={}.",
            line.raw,
            case.verb,
            case.count,
            u8::from(changed)
        )
    })
}

/// Press the window's Apply; wait for the applied line and the window's
/// report of the same run.
fn apply(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    case: &Case,
) -> Result<std::result::Result<TraceLine, String>> {
    let mark = session.trace()?.mark();
    let region = format!("{APPLY}{}", case.kind);
    press(ctx, session, pointer, &region)?;
    let asked = match after(session, ASKED, mark, &format!("`{region}` was pressed"))? {
        Ok(line) => line,
        Err(why) => return Ok(Err(why)),
    };
    report.note(format!("`{}`", asked.raw));
    let run = asked.get("run").unwrap_or_default().to_owned();
    let mut applied = None;
    for _ in 0..200 {
        let trace = session.trace()?;
        applied = applied.or_else(|| trace.last_after(APPLIED, mark).cloned());
        let shown = trace
            .last_after(SHOWN, mark)
            .filter(|s| s.get("run") == Some(run.as_str()))
            .cloned();
        if let (Some(a), Some(s)) = (&applied, shown) {
            report.note(format!("`{}`", a.raw));
            report.note(format!("`{}`", s.raw));
            if s.get("refused") != Some("0") || s.get_usize("lines").unwrap_or(0) == 0 {
                return Ok(Err(format!("`{}`: the window showed no report.", s.raw)));
            }
            return Ok(Ok(a.clone()));
        }
        session.settle(10);
    }
    let what = format!("run {run} was asked");
    match applied {
        Some(a) => Ok(Err(format!(
            "`{}`, and no `{SHOWN} run={run}` followed: the window never showed it.",
            a.raw
        ))),
        None => after(session, APPLIED, mark, &what),
    }
}

const PROFILE: &str = "preflight.profile"; // ui-text-exempt: a trace region name
const RUN: &str = "preflight.run"; // ui-text-exempt: a trace region name
const CONVERT: &str = "preflight.convert_pdfx"; // ui-text-exempt: a trace region name
const VERDICT: &str = "preflight-verdict"; // ui-text-exempt: a trace event name

/// `the_preflight_panel_gives_a_standards_verdict`.
pub struct ThePreflightPanelGivesAStandardsVerdict;

impl Check for ThePreflightPanelGivesAStandardsVerdict {
    fn name(&self) -> &'static str {
        "the_preflight_panel_gives_a_standards_verdict"
    }

    fn defect(&self) -> &'static str {
        "The Preflight panel gives no verdict for a PDF/X profile, calls a file with an RGB \
         paint and no trim box anything but non-conforming, invents a PDF/UA verdict the \
         engine does not give, or its Convert to PDF/X opens no window"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit,view.preflight",
            "preflight-verdict",
            &[],
        )
        .and_then(|(session, pointer)| {
            let outcome = verdicts(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn verdicts(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let x = step!(verdict_of(ctx, report, session, pointer, "pdfx-4"));
    if x.get("verdict") != Some("fails") || x.get_usize("failed").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "`{}` on a file with an RGB paint and no trim box; expected verdict=fails failed>0.",
            x.raw
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, CONVERT)?;
    session.settle(30);
    let window = format!("{APPLY}pdfx");
    let opened = super::driving::declared_names(
        &session.trace()?,
        super::dimdrive::ui_rect_event(ctx)?,
        &window,
    );
    report.note(format!("after `{CONVERT}` (mark {mark}): {opened:?}"));
    if opened.is_empty() {
        return Ok(Some(format!(
            "`{CONVERT}` was pressed and no `{window}` was drawn."
        )));
    }
    press(ctx, session, pointer, "prepress.close.pdfx")?; // ui-text-exempt: a trace region name
    let ua = step!(verdict_of(ctx, report, session, pointer, "pdfua-1"));
    if ua.get("verdict") != Some("none") || ua.get("unchecked").is_none_or(|n| n == "-") {
        return Ok(Some(format!(
            "`{}`; expected verdict=none (the engine gives PDF/UA none) with a not-checked count.",
            ua.raw
        )));
    }
    Ok(None)
}

/// Pick `profile`, press Run, and wait for its verdict line.
fn verdict_of(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    profile: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    press(ctx, session, pointer, PROFILE)?;
    press(ctx, session, pointer, &format!("{PROFILE}:{profile}"))?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, RUN)?;
    for _ in 0..200 {
        if let Some(line) = session.trace()?.last_after(VERDICT, mark).cloned() {
            report.note(format!("{profile}: `{}`", line.raw));
            return Ok(Ok(line));
        }
        session.settle(10);
    }
    after(session, VERDICT, mark, &format!("{profile} was run"))
}
