//! `bold_drawn_by_a_stroke_reads_as_bold` — **text made bold by a stroke
//! (render mode 2 and a line width) shows Bold pressed, and Ctrl+B takes the
//! bold off**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/synthetic_bold.md`.

use crate::checks::driving::shell_trace;
use crate::checks::word_styles::launch_on;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "synthetic-bold.pdf";
/// Inside `heavy`, the line's first word, baseline 700.
const IN_WORD: (f64, f64) = (84.0, 703.0);
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const LADDER: &str = "text-style-ladder"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "ribbon-item-selected"; // ui-text-exempt: a trace event name, never displayed
const BOLD: &str = "format.bold";

/// See the module documentation.
pub struct BoldDrawnByAStrokeReadsAsBold;

impl Check for BoldDrawnByAStrokeReadsAsBold {
    fn name(&self) -> &'static str {
        "bold_drawn_by_a_stroke_reads_as_bold"
    }

    fn defect(&self) -> &'static str {
        "text made bold by a stroke rather than by its face showed Bold released, so Ctrl+B \
         asked for bold again instead of taking it off"
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

/// The ribbon's Bold as last drawn after shell-trace line `after`.
fn bold_pressed(session: &crate::launch::Session, after: usize) -> Result<Option<bool>> {
    Ok(shell_trace(session)?
        .events(SELECTED)
        .filter(|l| l.lineno > after && l.get("id") == Some(BOLD))
        .last()
        .map(|l| l.get("selected") == Some("1")))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _) = launch_on(ctx, report, "synthetic-bold", FIXTURE)?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Some(why));
    }
    d.click(FORMAT_TAB)?;
    d.session.settle(20);
    let before = bold_pressed(&d.session, 0)?;
    let mark = d.session.trace()?.mark();
    d.pointer.key(&d.session, None, "B", Some("ctrl"))?;
    d.session.settle(40);
    let ladder = d
        .session
        .trace()?
        .last_after(LADDER, mark)
        .map(|l| (l.raw.clone(), l.get("removed") == Some("bold")));
    d.pointer.gone(&d.session)?;
    report.note(format!(
        "Bold drawn pressed: {before:?}; Ctrl+B: {ladder:?}"
    ));
    let mut findings = Vec::new();
    if before != Some(true) {
        findings.push(format!(
            "with the caret in stroke-bold text the ribbon's Bold was drawn {before:?}; pressed \
             was owed."
        ));
    }
    if !ladder.as_ref().is_some_and(|(_, removed)| *removed) {
        findings.push(format!(
            "Ctrl+B did not ask bold off: last `{LADDER}` after it was {:?}.",
            ladder.map(|(raw, _)| raw)
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
