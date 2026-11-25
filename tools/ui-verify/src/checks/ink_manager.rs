//! `the_ink_manager_aliases_and_converts_spots_as_one_step` — File ▸ Print ▸
//! Ink Manager lists the fixture's three spot inks, aliases one onto another
//! and converts a third to process in one Apply, and Output Preview's plates
//! show both inks gone.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ink_manager.md`.

use super::dimdrive::{Fixture, launch_with, press};
use super::multiselect::step;
use super::print_shop::{OUTPUT_SWITCH, OUTPUT_TRACE, await_line};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "ink-manager.pdf",
    method: "Rebuild it with `python fixtures/ink-manager.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const LISTED: &str = "ink-manager-listed"; // ui-text-exempt: a trace event name
const APPLIED: &str = "prepress-applied"; // ui-text-exempt: a trace event name
const CHOICE: &str = "prepress.ink-choice"; // ui-text-exempt: a trace region name
const APPLY: &str = "prepress.apply.inks"; // ui-text-exempt: a trace region name
/// The fixture's spot inks, as the trace writes them.
const KEPT: &str = "PANTONE_185_C"; // ui-text-exempt: a fixture ink name
const ALIASED: &str = "PMS185"; // ui-text-exempt: a fixture ink name
const CONVERTED: &str = "Gold"; // ui-text-exempt: a fixture ink name

/// `the_ink_manager_aliases_and_converts_spots_as_one_step`.
pub struct TheInkManagerAliasesAndConvertsSpotsAsOneStep;

impl Check for TheInkManagerAliasesAndConvertsSpotsAsOneStep {
    fn name(&self) -> &'static str {
        "the_ink_manager_aliases_and_converts_spots_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "The Ink Manager misses a spot ink, its Apply does not alias and convert what was \
         chosen, takes more than one undo step, or leaves the inks it removed on Output \
         Preview's plates or in its own list"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit,view.output_preview,file.ink_manager",
            "ink-manager",
            &[],
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
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

/// The `|`-joined list field `key` of `line` holds `ink`.
fn lists(line: &TraceLine, key: &str, ink: &str) -> bool {
    line.get(key)
        .is_some_and(|v| v.split('|').any(|n| n == ink))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let all = [KEPT, ALIASED, CONVERTED];
    let first = session.trace()?.last(LISTED).cloned();
    report.note(format!("listed: {:?}", first.as_ref().map(|l| &l.raw)));
    let Some(first) = first.filter(|l| all.iter().all(|i| lists(l, "drawn", i))) else {
        return Ok(Some(format!(
            "the Ink Manager opened and no `{LISTED}` line named all of {all:?}."
        )));
    };
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, OUTPUT_SWITCH)?;
    let plates_before =
        |l: &TraceLine| l.get("on") == Some("1") && all.iter().all(|i| lists(l, "inks", i));
    let Some(before) = await_line(session, OUTPUT_TRACE, mark, plates_before)? else {
        let last = session.trace()?.last(OUTPUT_TRACE).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "Output Preview switched on and listed no plate for all of {all:?}; last was {last:?}."
        )));
    };
    report.note(format!("plates before: `{}`", before.raw));
    let applied = step!(apply(ctx, report, session, pointer));
    let counts_ok = applied.get("changed") == Some("1")
        && applied.get_usize("aliased") == Some(1)
        && applied.get_usize("converted").is_some_and(|n| n >= 1)
        && applied.get_usize("commands") == Some(2)
        && applied.get("folded") == Some("1");
    if !counts_ok {
        return Ok(Some(format!(
            "`{}`; expected changed=1 aliased=1 converted>=1 commands=2 folded=1.",
            applied.raw
        )));
    }
    let gone = |l: &TraceLine, key: &str| {
        lists(l, key, KEPT) && !lists(l, key, ALIASED) && !lists(l, key, CONVERTED)
    };
    let epoch = first.get_usize("epoch").unwrap_or(0);
    let relisted = await_line(session, LISTED, applied.lineno, |l| {
        l.get_usize("epoch").is_some_and(|e| e > epoch)
    })?;
    report.note(format!(
        "re-listed: {:?}",
        relisted.as_ref().map(|l| &l.raw)
    ));
    let relisted_ok = relisted.as_ref().is_some_and(|l| {
        gone(l, "drawn") && !lists(l, "inks", ALIASED) && lists(l, "inks", CONVERTED)
    });
    if !relisted_ok {
        return Ok(Some(format!(
            "after Apply the Ink Manager re-listed {:?}; expected {ALIASED} gone, {CONVERTED} \
             still defined but no longer drawn, and {KEPT} drawn.",
            relisted.map(|l| l.raw)
        )));
    }
    let plates = await_line(session, OUTPUT_TRACE, applied.lineno, |l| {
        l.get("on") == Some("1") && gone(l, "inks")
    })?;
    let Some(plates) = plates else {
        let last = session.trace()?.last(OUTPUT_TRACE).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "after Apply Output Preview still lists {ALIASED} or {CONVERTED}, or lost {KEPT}; \
             last was {last:?}."
        )));
    };
    report.note(format!("plates after: `{}`", plates.raw));
    Ok(None)
}

/// Alias PMS185 onto PANTONE 185 C, convert Gold to process, press Apply.
fn apply(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<TraceLine, String>> {
    press(ctx, session, pointer, &format!("{CHOICE}:{ALIASED}"))?;
    press(
        ctx,
        session,
        pointer,
        &format!("{CHOICE}:{ALIASED}:as:{KEPT}"),
    )?;
    press(ctx, session, pointer, &format!("{CHOICE}:{CONVERTED}"))?;
    press(
        ctx,
        session,
        pointer,
        &format!("{CHOICE}:{CONVERTED}:process"),
    )?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY)?;
    match await_line(session, APPLIED, mark, |_| true)? {
        Some(line) => {
            report.note(format!("`{}`", line.raw));
            Ok(Ok(line))
        }
        None => Ok(Err(format!(
            "`{APPLY}` was pressed and no `{APPLIED}` followed. Trace: {}.",
            session.trace_path().display()
        ))),
    }
}
