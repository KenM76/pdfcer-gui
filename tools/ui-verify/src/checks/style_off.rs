//! `ctrl_b_again_takes_bold_off` — Bold and Italic are toggles in both
//! directions (`OPERATOR_REQUESTS.md` O285): a caret inside a word of
//! `fixtures/paragraph.pdf` (Helvetica), Ctrl+B twice, then Ctrl+I twice.
//!
//! Each second press must ask its axis off and the engine's ladder must bind
//! the run's own regular face, which the page carries (`text-style-ladder
//! removed=<axis> rung=page-face bound=Helvetica`), and the ribbon toggle must
//! be drawn released.

use crate::checks::driving::shell_trace;
use crate::checks::word_styles::{Driven, IN_WORD, launch};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const LADDER: &str = "text-style-ladder"; // ui-text-exempt: a trace event name, never displayed
const SPAN_APPLIED: &str = "text-span-style-applied"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "ribbon-item-selected"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
/// The run's own face, which the fixture's page carries as a resource.
const REGULAR: &str = "Helvetica";

/// See the module documentation.
pub struct CtrlBAgainTakesBoldOff;

impl Check for CtrlBAgainTakesBoldOff {
    fn name(&self) -> &'static str {
        "ctrl_b_again_takes_bold_off"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+B or Ctrl+I on text that was already bold or italic was declined or did \
         nothing, so a style could be put on and never taken off"
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

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let d = launch(ctx, report, "style-off")?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Err(why));
    }
    d.click(FORMAT_TAB)?;
    for (key, axis, id) in [
        ("B", "bold", "format.bold"),
        ("I", "italic", "format.italic"),
    ] {
        if let Err(why) = press(&d, report, key, axis, id, true)? {
            return Ok(Err(why));
        }
        if let Err(why) = press(&d, report, key, axis, id, false)? {
            return Ok(Err(why));
        }
    }
    d.pointer.gone(&d.session)?;
    Ok(Ok(
        "Ctrl+B and Ctrl+I each put their style on and, pressed again, took it off onto the \
         page's own Helvetica, with the toggle released"
            .to_owned(),
    ))
}

/// One Ctrl+`key` press that must turn `axis` on (`on`) or off.
fn press(
    d: &Driven,
    report: &mut CheckReport,
    key: &str,
    axis: &str,
    id: &str,
    on: bool,
) -> Result<std::result::Result<(), String>> {
    let mark = d.session.trace()?.mark();
    let shell_mark = shell_trace(&d.session)?.mark();
    d.pointer.key(&d.session, None, key, Some("ctrl"))?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    let applied = trace
        .last_after(SPAN_APPLIED, mark)
        .filter(|l| l.get("applied").is_some_and(|n| n != "0"));
    let Some(applied) = applied else {
        return Ok(Err(format!(
            "Ctrl+{key} to turn {axis} {} restyled nothing: no `{SPAN_APPLIED} applied>0` \
             after the press. Trace: {}.",
            if on { "on" } else { "off" },
            d.path()
        )));
    };
    report.note(&applied.raw);
    let Some(ladder) = trace.last_after(LADDER, mark) else {
        return Ok(Err(format!(
            "Ctrl+{key} applied but no `{LADDER}` line followed, so no style ladder ran. \
             Trace: {}.",
            d.path()
        )));
    };
    report.note(&ladder.raw);
    let (asked_on, asked_off) = (ladder.get("requested"), ladder.get("removed"));
    if on && asked_on != Some(axis) {
        return Ok(Err(format!(
            "the first Ctrl+{key} did not ask {axis} on: `{}`. Trace: {}.",
            ladder.raw,
            d.path()
        )));
    }
    if !on
        && (asked_off != Some(axis)
            || ladder.get("rung") != Some("page-face")
            || ladder.get("bound") != Some(REGULAR))
    {
        return Ok(Err(format!(
            "Ctrl+{key} on {axis} text did not take {axis} off onto the page's own {REGULAR}: \
             `{}` (wanted removed={axis} rung=page-face bound={REGULAR}). Trace: {}.",
            ladder.raw,
            d.path()
        )));
    }
    let pressed = shell_trace(&d.session)?
        .events(SELECTED)
        .filter(|l| l.lineno > shell_mark && l.get("id") == Some(id))
        .last()
        .map(|l| l.get("selected") == Some("1"));
    if !on && pressed != Some(false) {
        return Ok(Err(format!(
            "{axis} was taken off but the ribbon's `{id}` was not drawn released after the \
             press (last `{SELECTED}` for it: {pressed:?}). Trace: {}.",
            d.path()
        )));
    }
    Ok(Ok(()))
}
