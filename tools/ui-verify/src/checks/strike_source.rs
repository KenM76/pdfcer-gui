//! `a_strikethrough_says_where_its_line_came_from` — **striking a word
//! traces where the line's height came from and says the same in the edit's
//! disclosure**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/strike_source.md`.

use crate::checks::word_styles::{IN_WORD, launch};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const SOURCE: &str = "text-strike-source"; // ui-text-exempt: a trace event name, never displayed
const DISCLOSED: &str = "text-style-disclosed"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const STRIKE: &str = "ribbon.item.format.strikethrough"; // ui-text-exempt: a trace region name, never displayed
const FONT_COLLAPSED: &str = "ribbon.group.format.font.collapsed"; // ui-text-exempt: a trace region name, never displayed
/// Unembedded Helvetica: no strikeout table and no `/XHeight`, so the AFM
/// x-height places the line.
const OWED: &str = "x_height";
/// The engine's clause for that source.
const CLAUSE: &str = "the strikethrough is inferred at half the font's x-height"; // ui-text-exempt: a needle for the engine's sentence
/// The clauses for the other two sources, which must not be said. The engine's
/// generic decoration sentence names all three sources, so these are the
/// per-source clauses alone.
const OTHERS: [&str; 2] = [
    "the strikethrough is placed by",
    "the strikethrough is guessed at",
]; // ui-text-exempt: needles for the engine's sentences

/// See the module documentation.
pub struct AStrikethroughSaysWhereItsLineCameFrom;

impl Check for AStrikethroughSaysWhereItsLineCameFrom {
    fn name(&self) -> &'static str {
        "a_strikethrough_says_where_its_line_came_from"
    }

    fn defect(&self) -> &'static str {
        "a strikethrough was drawn without saying whether its height came from the font's \
         strikeout metric, its x-height or a guess"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let d = launch(ctx, report, "strike-source")?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Some(why));
    }
    d.click(FORMAT_TAB)?;
    if !d.declares(STRIKE)? && d.declares(FONT_COLLAPSED)? {
        d.click(FONT_COLLAPSED)?;
    }
    let mark = d.session.trace()?.mark();
    d.click(STRIKE)?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    d.pointer.gone(&d.session)?;
    let source = trace
        .last_after(SOURCE, mark)
        .and_then(|l| l.get("source").map(str::to_owned));
    let notes = trace
        .last_after(DISCLOSED, mark)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("source={source:?}; disclosed: {notes}"));
    let mut findings = Vec::new();
    if source.as_deref() != Some(OWED) {
        findings.push(format!(
            "striking the word traced source={source:?}; `{OWED}` was owed for unembedded \
             Helvetica."
        ));
    }
    if !notes.contains(CLAUSE) || OTHERS.iter().any(|o| notes.contains(o)) {
        findings.push(format!(
            "the edit's disclosure does not say `{CLAUSE}` alone: {notes:?}."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
