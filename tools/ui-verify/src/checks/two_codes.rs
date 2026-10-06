//! `a_letter_drawn_two_ways_is_named` — **an edit to text holding a letter its
//! font draws two ways is refused for that letter, said as such, and text in
//! the same font without it still edits**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/two_codes.md`.

use crate::checks::word_styles::{Driven, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "two-codes-one-letter.pdf";
/// Inside the `B` the page shows, 48 pt from (72, 540).
const IN_B: (f64, f64) = (80.0, 555.0);
/// Inside the `A` the page shows, 48 pt from (72, 600).
const IN_A: (f64, f64) = (80.0, 615.0);
const KEY_DECLINED: &str = "text-edit-key-declined"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-text"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "edit-text-refused"; // ui-text-exempt: a trace event name, never displayed
const CLASSIFIED: &str = "edit-text-classified"; // ui-text-exempt: a trace event name, never displayed
const OFFER: &str = "refused-char"; // ui-text-exempt: a trace event name, never displayed
/// The sentence `text::reface::set_in` owes the `A` set in another face.
const CANNOT_WRITE: &str = "pdfcer cannot write \u{2018}A\u{2019}"; // ui-text-exempt: a needle for the app's sentence
/// The sentence it must not say: the font has the letter, twice.
const HAS_NO: &str = "has no \u{2018}A\u{2019}"; // ui-text-exempt: a needle for the app's sentence

/// See the module documentation.
pub struct ALetterDrawnTwoWaysIsNamed;

impl Check for ALetterDrawnTwoWaysIsNamed {
    fn name(&self) -> &'static str {
        "a_letter_drawn_two_ways_is_named"
    }

    fn defect(&self) -> &'static str {
        "an edit to text holding a letter its font draws two ways was said as the operator \
         having typed that letter, or as the font lacking it, or the refusal stopped text in \
         the same font from editing"
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

/// Every trace line of the outcome events after `mark`, raw.
fn outcomes(d: &Driven, mark: usize) -> Result<Vec<String>> {
    Ok(d.session
        .trace()?
        .lines
        .iter()
        .filter(|l| l.lineno > mark)
        .filter(|l| [KEY_DECLINED, APPLIED, REFUSED, CLASSIFIED].contains(&l.event.as_str()))
        .map(|l| l.raw.clone())
        .collect())
}

/// Put the caret at the end of the text at `at`, type `typed`, commit.
fn type_into(
    d: &Driven,
    at: (f64, f64),
    typed: &str,
) -> Result<std::result::Result<Vec<String>, String>> {
    if let Err(why) = d.caret_at(at)? {
        return Ok(Err(why));
    }
    let mark = d.session.trace()?.mark();
    d.pointer.key(&d.session, None, "End", None)?;
    d.pointer.type_text(&d.session, None, typed)?;
    d.session.settle(20);
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(30);
    Ok(Ok(outcomes(d, mark)?))
}

/// The last offer's `character` and `two_ways` after trace line `mark`.
fn offer_after(d: &Driven, mark: usize) -> Result<Option<(String, String)>> {
    Ok(d.session.trace()?.last_after(OFFER, mark).map(|l| {
        let field = |k: &str| l.get(k).unwrap_or_default().to_owned();
        (field("character"), field("two_ways"))
    }))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _doc) = launch_on(ctx, report, "two-codes", FIXTURE)?;
    let mark = d.session.trace()?.mark();
    let into_a = match type_into(&d, IN_A, "B")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    let offer = offer_after(&d, mark)?;
    let into_b = match type_into(&d, IN_B, "AB")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    d.pointer.gone(&d.session)?;
    report.note(format!(
        "typing `B` after `A`: {into_a:?}; offer {offer:?}. Typing `AB` after `B`: {into_b:?}"
    ));
    let said = |lines: &[String], needle: &str| lines.iter().any(|l| l.contains(needle));
    let mut findings = Vec::new();
    if !said(&into_a, "said=TextHoldsTwoGlyphsFor") || said(&into_a, "edit-text page=") {
        findings.push(
            "typing `B` after the `A` was not refused as text holding a letter drawn two ways \
             (said=TextHoldsTwoGlyphsFor)."
                .to_owned(),
        );
    }
    if offer != Some(("'A'".to_owned(), "1".to_owned())) {
        findings.push(format!(
            "the Properties offer read {offer:?}; character='A' with two_ways=1 was owed."
        ));
    }
    if !said(&into_b, "edit-text page=") || !said(&into_b, CANNOT_WRITE) || said(&into_b, HAS_NO) {
        findings.push(format!(
            "typing `AB` after the `B` did not commit saying `{CANNOT_WRITE}` without `{HAS_NO}`."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
