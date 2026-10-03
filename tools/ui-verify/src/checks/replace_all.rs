//! `replace_all_is_one_undo`: the Find bar's Replace and Replace all rewrite
//! the hits in place, and one Undo puts back everything one Replace all did
//! (`OPERATOR_REQUESTS.md` WORDLIKE step 8), driven off the desktop through
//! the scripted pointer.
//!
//! On `fixtures/paragraph.pdf` a case-insensitive `the` has four hits, each
//! inside one show operator: Replace takes one, Replace all the other three,
//! and a search after one Undo finds those three again.

use crate::checks::word_styles::{Driven, launch};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIND: &str = "find"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "text-replace-applied"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed
const FIELD: &str = "find-field"; // ui-text-exempt: a trace region name, never displayed
const TOGGLE: &str = "find-replace-toggle"; // ui-text-exempt: a trace region name, never displayed
const WITH: &str = "find-replace-field"; // ui-text-exempt: a trace region name, never displayed
const ONE: &str = "find-replace-one"; // ui-text-exempt: a trace region name, never displayed
const ALL: &str = "find-replace-all"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct ReplaceAllIsOneUndo;

impl Check for ReplaceAllIsOneUndo {
    fn name(&self) -> &'static str {
        "replace_all_is_one_undo"
    }

    fn defect(&self) -> &'static str {
        "the Find bar could find text but not replace it, or Replace all took one Undo per hit"
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
    let d = launch(ctx, report, "replace-all")?;
    d.pointer.key(&d.session, None, "F", Some("ctrl"))?;
    d.session.settle(15);
    d.pointer.type_text(&d.session, None, "the")?;
    d.pointer.key(&d.session, None, "Enter", None)?;
    d.session.settle(30);
    if let Err(why) = hits_now(&d, "the first search", "4")? {
        return Ok(Err(why));
    }
    d.click(TOGGLE)?;
    d.click(WITH)?;
    d.pointer.type_text(&d.session, None, "our")?;
    d.session.settle(10);
    let png = ctx.out("replace-all.png");
    d.pointer.screenshot(&d.session, &png)?;
    report.artifact(png);
    d.click(ONE)?;
    d.session.settle(40);
    if let Err(why) = replaced(&d, report, "current", "1", "1")? {
        return Ok(Err(why));
    }
    if let Err(why) = hits_now(&d, "the search after Replace", "3")? {
        return Ok(Err(why));
    }
    d.click(ALL)?;
    d.session.settle(40);
    if let Err(why) = replaced(&d, report, "all", "3", "3")? {
        return Ok(Err(why));
    }
    if let Err(why) = hits_now(&d, "the search after Replace all", "0")? {
        return Ok(Err(why));
    }
    // Leave the field so Ctrl+Z is the document's Undo, not the field's.
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(10);
    let undos = d.session.trace()?.events(UNDONE).count();
    d.pointer.key(&d.session, None, "Z", Some("ctrl"))?;
    d.session.settle(40);
    if d.session.trace()?.events(UNDONE).count() != undos + 1 {
        return Ok(Err(format!(
            "★★★ Ctrl+Z after Replace all undid nothing: no new `{UNDONE}` line. Trace: {}.",
            d.path()
        )));
    }
    d.click(FIELD)?;
    d.pointer.key(&d.session, None, "Enter", None)?;
    d.session.settle(30);
    let after = hits_now(&d, "the search after one Undo", "3")?;
    d.pointer.gone(&d.session)?;
    if let Err(why) = after {
        return Ok(Err(format!(
            "★★★★ one Undo did not restore every hit Replace all rewrote: {why}"
        )));
    }
    Ok(Ok(
        "Replace rewrote one hit, Replace all the other three, and one Undo restored all three"
            .to_owned(),
    ))
}

/// Require the latest search to report `want` hits.
fn hits_now(d: &Driven, what: &str, want: &str) -> Result<std::result::Result<(), String>> {
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

/// Require the latest replace to be `which` and to have replaced `want` of `found`.
fn replaced(
    d: &Driven,
    report: &mut CheckReport,
    which: &str,
    found: &str,
    want: &str,
) -> Result<std::result::Result<(), String>> {
    let trace = d.session.trace()?;
    let Some(line) = trace.events(APPLIED).last() else {
        return Ok(Err(format!(
            "★★★★ Replace ({which}) rewrote nothing: no `{APPLIED}` line. Trace: {}.",
            d.path()
        )));
    };
    report.note(&line.raw);
    let ok = line.get("which") == Some(which)
        && line.get("found") == Some(found)
        && line.get("replaced") == Some(want);
    if ok {
        return Ok(Ok(()));
    }
    Ok(Err(format!(
        "★★★ `{}` is not a Replace ({which}) of {want} of {found} hits. Trace: {}.",
        line.raw,
        d.path()
    )))
}
