//! `a_letter_drawn_two_ways_is_named` — **text holding a letter its font
//! draws two ways still edits around it, and typing that letter is said as
//! drawn two ways, not as missing**
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
const NOTICE: &str = "text-edit-refused-keys"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// The sentence `text::reface::set_in` owes the `A` set in another face.
const TWO_WAYS: &str = "draws \u{2018}A\u{2019} two different ways"; // ui-text-exempt: a needle for the app's sentence
/// The sentences it must not say: the font has the letter, twice.
const HAS_NO: &str = "has no \u{2018}A\u{2019}"; // ui-text-exempt: a needle for the app's sentence
const CANNOT_WRITE: &str = "cannot write \u{2018}A\u{2019}"; // ui-text-exempt: a needle for the app's sentence
/// The `A` keeping its code 1 beside the added `B`'s code 3, as a hex or a
/// literal string; the engine chooses the spelling.
const KEPT_CODE: [&str; 2] = ["<00010003>", r"(\000\001\000\003)"];

/// See the module documentation.
pub struct ALetterDrawnTwoWaysIsNamed;

impl Check for ALetterDrawnTwoWaysIsNamed {
    fn name(&self) -> &'static str {
        "a_letter_drawn_two_ways_is_named"
    }

    fn defect(&self) -> &'static str {
        "an edit beside a letter its font draws two ways was refused, or typing that letter \
         was said as the font lacking it"
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
        .filter(|l| {
            [KEY_DECLINED, APPLIED, REFUSED, CLASSIFIED, NOTICE].contains(&l.event.as_str())
        })
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

/// Save in place and answer whether it saved and the revision it appended.
fn save(d: &Driven, doc: &std::path::Path, source_len: usize) -> Result<(bool, Vec<u8>)> {
    d.pointer.key(&d.session, None, "S", Some("ctrl"))?;
    d.session.settle(40);
    let saved = d
        .session
        .trace()?
        .events(SAVED)
        .last()
        .is_some_and(|l| l.raw.contains("outcome=ok"));
    let bytes = std::fs::read(doc)
        .map_err(|e| crate::error::Error::new(format!("reading the saved copy: {e}")))?;
    let tail = bytes[source_len.min(bytes.len())..].to_vec();
    Ok((saved, tail))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, doc) = launch_on(ctx, report, "two-codes", FIXTURE)?;
    let source_len = std::fs::metadata(&doc).map_or(0, |m| usize::try_from(m.len()).unwrap_or(0));
    let into_a = match type_into(&d, IN_A, "B")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    let (saved, tail) = save(&d, &doc, source_len)?;
    let kept = KEPT_CODE
        .iter()
        .any(|k| tail.windows(k.len()).any(|w| w == k.as_bytes()));
    let into_b = match type_into(&d, IN_B, "AB")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    d.pointer.gone(&d.session)?;
    report.note(format!(
        "typing `B` after `A`: {into_a:?}; saved={saved} codes 1,3 {kept}. \
         Typing `AB` after `B`: {into_b:?}"
    ));
    let said = |lines: &[String], needle: &str| lines.iter().any(|l| l.contains(needle));
    let mut findings = Vec::new();
    if !said(&into_a, "edit-text page=") || said(&into_a, REFUSED) {
        findings.push(
            "typing `B` after the `A` did not commit: a letter the edit leaves alone was \
             refused because its font draws it two ways."
                .to_owned(),
        );
    }
    if !saved || !kept {
        findings.push(format!(
            "the saved revision does not show codes 1 then 3 (saved={saved}): the `A` did not \
             keep its code beside the added `B`."
        ));
    }
    if !said(&into_b, "two_ways=1") {
        findings.push(
            "typing `A` after the `B` was not noted as a letter drawn two ways (two_ways=1)."
                .to_owned(),
        );
    }
    if !said(&into_b, "edit-text page=")
        || !said(&into_b, TWO_WAYS)
        || said(&into_b, HAS_NO)
        || said(&into_b, CANNOT_WRITE)
    {
        findings.push(format!(
            "typing `AB` after the `B` did not commit saying `{TWO_WAYS}` without `{HAS_NO}` \
             or `{CANNOT_WRITE}`."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
