//! `an_ime_composition_types_once_committed`: a caret in a paragraph asks the
//! platform for an input method; a composition in progress is shown under the
//! line and never enters the text; its commit is typed at the caret
//! (`OPERATOR_REQUESTS.md` WORDLIKE step 9), driven off the desktop through
//! the scripted pointer.
//!
//! On `fixtures/paragraph.pdf`: a composition of `zq` is shown, then `é` is
//! committed and Escape commits the edit. A search then finds one `é` and no
//! `zq`.

use crate::checks::word_styles::{Driven, IN_WORD, launch};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIND: &str = "find"; // ui-text-exempt: a trace event name, never displayed
const AREA: &str = "text-ime-area"; // ui-text-exempt: a trace event name, never displayed
const PREEDIT: &str = "text-ime-preedit"; // ui-text-exempt: a trace event and region name, never displayed
const COMPOSING: &str = "zq"; // ui-text-exempt: composed input, not prose
const COMMITTED: &str = "é"; // ui-text-exempt: composed input, not prose

/// See the module documentation.
pub struct AnImeCompositionTypesOnceCommitted;

impl Check for AnImeCompositionTypesOnceCommitted {
    fn name(&self) -> &'static str {
        "an_ime_composition_types_once_committed"
    }

    fn defect(&self) -> &'static str {
        "an input method (Chinese, Japanese, Korean, dead keys, the emoji panel) could not type \
         into a text edit, or its composition leaked into the text"
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

type Outcome = std::result::Result<String, String>;

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Outcome> {
    let d = launch(ctx, report, "ime")?;
    if let Err(why) = d.caret_at(IN_WORD)? {
        return Ok(Err(why));
    }
    if d.session.trace()?.events(AREA).next().is_none() {
        return Ok(Err(format!(
            "★★★★ a caret in the paragraph asked for no input method: no `{AREA}` line, so \
             Windows never starts a composition. Trace: {}.",
            d.path()
        )));
    }
    d.pointer.ime_preedit(&d.session, COMPOSING)?;
    d.session.settle(15);
    let png = ctx.out("ime-preedit.png");
    d.pointer.screenshot(&d.session, &png)?;
    report.artifact(png);
    let shown = d
        .session
        .trace()?
        .events(PREEDIT)
        .any(|l| l.get("chars") == Some("2"));
    if !shown {
        return Ok(Err(format!(
            "★★★ the composition reached no draft: no `{PREEDIT} chars=2` line. Trace: {}.",
            d.path()
        )));
    }
    d.pointer.ime_commit(&d.session, COMMITTED)?;
    d.session.settle(20);
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(30);
    let committed = search(&d, COMMITTED, "the search for the committed text", "1")?;
    let composing = search(&d, COMPOSING, "the search for the composition", "0")?;
    d.pointer.gone(&d.session)?;
    if let Err(why) = committed {
        return Ok(Err(format!(
            "★★★★ the composition's commit was not typed: {why}"
        )));
    }
    if let Err(why) = composing {
        return Ok(Err(format!(
            "★★★★ the composition in progress entered the text: {why}"
        )));
    }
    Ok(Ok(
        "a caret asked for an input method, the composition stayed out of the text, and its \
         commit was typed once"
            .to_owned(),
    ))
}

/// Search for `query` and require `want` hits.
fn search(
    d: &Driven,
    query: &str,
    what: &str,
    want: &str,
) -> Result<std::result::Result<(), String>> {
    d.pointer.key(&d.session, None, "F", Some("ctrl"))?;
    d.session.settle(15);
    d.pointer.key(&d.session, None, "A", Some("ctrl"))?;
    d.pointer.type_text(&d.session, None, query)?;
    d.pointer.key(&d.session, None, "Enter", None)?;
    d.session.settle(30);
    let trace = d.session.trace()?;
    let got = trace
        .events(FIND)
        .last()
        .and_then(|l| l.get("hits"))
        .map(str::to_owned);
    if got.as_deref() == Some(want) {
        return Ok(Ok(()));
    }
    Ok(Err(format!(
        "{what} reported hits={} where {want} were expected. Trace: {}.",
        got.as_deref().unwrap_or("(no search)"),
        d.path()
    )))
}
