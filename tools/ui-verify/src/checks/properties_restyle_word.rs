//! `properties_italic_restyles_the_word_at_the_caret` — with the I-beam's caret
//! inside a word, the Properties panel's Italic restyles that word, not the
//! whole line.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/properties_restyle_word.md`.

use super::word_styles::{IN_WORD, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

/// The Text tool armed, then the Properties panel shown.
const INVOKE: &str = "mode.edit,edit.text,file.properties";
const ITALIC: &str = "properties.text.italic"; // ui-text-exempt: a trace region name, never displayed
const SPAN_APPLIED: &str = "text-span-style-applied"; // ui-text-exempt: a trace event name, never displayed
const RUNS_APPLIED: &str = "text-style-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-style-declined"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct PropertiesItalicRestylesTheWordAtTheCaret;

impl Check for PropertiesItalicRestylesTheWordAtTheCaret {
    fn name(&self) -> &'static str {
        "properties_italic_restyles_the_word_at_the_caret"
    }

    fn defect(&self) -> &'static str {
        "with the I-beam's caret in a word, the Properties panel's Italic restyled the whole \
         line, or nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(None) => report.pass(),
            Ok(Some(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _) = launch_invoking(
        ctx,
        report,
        "properties-restyle-word",
        "paragraph.pdf",
        INVOKE,
    )?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Some(why));
    }
    if !d.declares(ITALIC)? {
        d.pointer.gone(&d.session)?;
        return Ok(Some(format!(
            "★★★★ with a caret in a word the Properties panel drew no Italic: no `{ITALIC}` \
             region. Trace: {}.",
            d.path()
        )));
    }
    let mark = d.session.trace()?.mark();
    d.click(ITALIC)?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    d.pointer.gone(&d.session)?;
    let after = |event: &str| trace.last_after(event, mark).cloned();
    if let Some(line) = after(DECLINED) {
        return Ok(Some(format!(
            "★★★★ Italic with a caret in `drawing` was declined: `{}`. Trace: {}.",
            line.raw,
            d.path()
        )));
    }
    if let Some(line) = after(RUNS_APPLIED) {
        return Ok(Some(format!(
            "★★★★ Italic with a caret in `drawing` restyled whole runs, not the word: `{}`. \
             Trace: {}.",
            line.raw,
            d.path()
        )));
    }
    let Some(applied) = after(SPAN_APPLIED) else {
        return Ok(Some(format!(
            "★★★★ Italic with a caret in `drawing` restyled nothing: no `{SPAN_APPLIED}` line. \
             Trace: {}.",
            d.path()
        )));
    };
    report.note(&applied.raw);
    let ok = applied.get("applied").is_some_and(|n| n != "0")
        && applied.get("reselected") == Some("true");
    Ok((!ok).then(|| {
        format!(
            "★★★ the word's restyle `{}` applied nothing or lost the word's selection. \
             Trace: {}.",
            applied.raw,
            d.path()
        )
    }))
}
