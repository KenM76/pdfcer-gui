//! `underlining_part_of_a_tagged_paragraph_says_it_is_not_recorded` — Ctrl+U
//! on one word of a tagged paragraph underlines the word, and the status bar
//! says the structure tree does not record it, because pdfcer does not split
//! structure elements.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tagged_underline.md`.

use crate::checks::word_styles::launch_on;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "tagged-paragraph.pdf";
/// Inside "rose", the second word of the `<P>` element's one line
/// ("Sales rose in every region.", Helvetica 11 pt from (72, 690)).
const IN_ROSE: (f64, f64) = (113.0, 693.0);
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const DECORATED: &str = "text-decorate-applied"; // ui-text-exempt: a trace event name, never displayed
const DISCLOSED: &str = "text-style-disclosed"; // ui-text-exempt: a trace event name, never displayed
/// `app::status::REGION_EDIT_DISCLOSURE`.
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name, never displayed
/// The engine's sentence (`text_edit::decoration::tagged`) for a decoration
/// covering part of an element; matched on its fixed tail.
const NOT_SPLIT: &str = "pdfcer does not split structure elements"; // ui-text-exempt: the engine's sentence, matched in a trace
/// The element the sentence must name.
const ELEMENT: &str = "<P>"; // ui-text-exempt: a structure type, matched in a trace

/// See the module documentation.
pub struct UnderliningPartOfATaggedParagraphSaysItIsNotRecorded;

impl Check for UnderliningPartOfATaggedParagraphSaysItIsNotRecorded {
    fn name(&self) -> &'static str {
        "underlining_part_of_a_tagged_paragraph_says_it_is_not_recorded"
    }

    fn defect(&self) -> &'static str {
        "underlining one word of a tagged paragraph leaves the structure tree without the \
         underline and the operator is not told"
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
    let (d, _doc) = launch_on(ctx, report, "tagged-underline", FIXTURE)?;
    if let Err(why) = d.caret_at(IN_ROSE)? {
        return Ok(Some(why));
    }
    d.click(FORMAT_TAB)?;
    d.session.settle(20);
    let mark = d.session.trace()?.mark();
    d.pointer.key(&d.session, None, "U", Some("ctrl"))?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    let applied = trace
        .last_after(DECORATED, mark)
        .map(|l| (l.get("applied").map(str::to_owned), l.raw.clone()));
    let disclosed = trace.last_after(DISCLOSED, mark).map(|l| l.raw.clone());
    let shown = d.declares(DISCLOSURE)?;
    d.pointer.gone(&d.session)?;
    report.note(format!(
        "decorated: {applied:?}; disclosed: {disclosed:?}; `{DISCLOSURE}` drawn {shown}"
    ));
    if applied.as_ref().and_then(|(a, _)| a.as_deref()) != Some("1") {
        return Ok(Some(format!(
            "Ctrl+U on \"rose\" applied no underline ({applied:?}). Trace: {}.",
            d.path()
        )));
    }
    let names_it = disclosed
        .as_deref()
        .is_some_and(|l| l.contains(NOT_SPLIT) && l.contains(ELEMENT));
    if !names_it {
        return Ok(Some(format!(
            "the underline covers part of a {ELEMENT} element and the edit's disclosures do not \
             carry the engine's \"{NOT_SPLIT}\" sentence naming it ({disclosed:?}). Trace: {}.",
            d.path()
        )));
    }
    if !shown {
        return Ok(Some(format!(
            "the sentence was recorded and the status bar drew no `{DISCLOSURE}` group. Trace: \
             {}.",
            d.path()
        )));
    }
    Ok(None)
}
