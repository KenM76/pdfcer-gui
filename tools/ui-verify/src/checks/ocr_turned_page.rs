//! `ocr_on_a_turned_page_lies_along_the_words` — Recognise text on a page
//! stored sideways with `/Rotate 90` writes every word of the layer along the
//! displayed reading direction, so it reads and selects upright over the
//! words it covers.
//!
//! Driven with the scripted pointer off the desktop on a copy of
//! `fixtures/scan-rotate-90.pdf`, with `ocrs`, then saved. The oracle is the
//! saved file: every text matrix in an invisible (`3 Tr`) stream must be the
//! quarter-turn `0 b c 0` with `b > 0`, `c < 0`. An upright `a 0 0 d` matrix
//! is the defect: a word laid across the page's long axis, at 90 degrees to
//! the scan.

use crate::checks::invisible_text_scripted::streams;
use crate::checks::ocr_scripted::recognise;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const ENGINE: &str = "ocrs";
const FIXTURE: &str = "scan-rotate-90.pdf";

/// See the module documentation.
pub struct OcrOnATurnedPageLiesAlongTheWords;

impl Check for OcrOnATurnedPageLiesAlongTheWords {
    fn name(&self) -> &'static str {
        "ocr_on_a_turned_page_lies_along_the_words"
    }

    fn defect(&self) -> &'static str {
        "recognised text on a page with /Rotate is written at 90 degrees to the words it covers"
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
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !fixture.is_file() {
        return Err(Error::new(format!(
            "{} is missing. Regenerate it: python fixtures/scan-rotate-90.PROVENANCE.py.",
            fixture.display()
        )));
    }
    let copy = ctx.out("ocr-turned.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let (_trace, path) = recognise(ctx, report, &copy, ENGINE, "turned", true)?;
    let saved = std::fs::read(&copy).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    let matrices: Vec<[f64; 6]> = streams(&saved)
        .iter()
        .filter(|body| contains(body, b"3 Tr"))
        .flat_map(|body| text_matrices(body))
        .collect();
    report.note(format!(
        "{} text matrices in the layer; first {:?}",
        matrices.len(),
        matrices.first()
    ));
    Ok(judge(&matrices).map(|why| format!("{why} Trace: {path}.")))
}

/// `None` when there is at least one matrix and every one is a quarter-turn
/// counter-clockwise from the page's x axis, as `/Rotate 90` displays it.
fn judge(matrices: &[[f64; 6]]) -> Option<String> {
    if matrices.is_empty() {
        return Some("★ the saved file holds no invisible text with a text matrix.".into());
    }
    let turned = |m: &[f64; 6]| m[0].abs() < 1e-3 && m[3].abs() < 1e-3 && m[1] > 0.0 && m[2] < 0.0;
    let wrong: Vec<_> = matrices.iter().filter(|m| !turned(m)).collect();
    if wrong.is_empty() {
        return None;
    }
    Some(format!(
        "★★ {} of {} words are not laid along the displayed text; first {:?}, where /Rotate 90 \
         needs `0 b c 0` with b > 0 and c < 0.",
        wrong.len(),
        matrices.len(),
        wrong[0]
    ))
}

/// The six operands before every `Tm` in a content stream.
pub(super) fn text_matrices(body: &[u8]) -> Vec<[f64; 6]> {
    let text = String::from_utf8_lossy(body);
    let tokens: Vec<&str> = text.split_ascii_whitespace().collect();
    tokens
        .iter()
        .enumerate()
        .filter(|(i, t)| **t == "Tm" && *i >= 6)
        .filter_map(|(i, _)| {
            let mut m = [0.0; 6];
            for (k, slot) in m.iter_mut().enumerate() {
                *slot = tokens[i - 6 + k].parse().ok()?;
            }
            Some(m)
        })
        .collect()
}

pub(super) fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::{judge, text_matrices};

    #[test]
    fn only_a_quarter_turn_passes() {
        assert!(judge(&[]).is_some());
        assert!(judge(&[[0.0, 1.0, -1.0, 0.0, 5.0, 6.0]]).is_none());
        assert!(judge(&[[1.0, 0.0, 0.0, 1.0, 5.0, 6.0]]).is_some());
        assert!(judge(&[[0.0, -1.0, 1.0, 0.0, 5.0, 6.0]]).is_some());
    }

    #[test]
    fn matrices_are_read_from_their_operands() {
        let got = text_matrices(b"BT 3 Tr 0 1 -1 0 10 20 Tm (a) Tj ET");
        assert_eq!(got, vec![[0.0, 1.0, -1.0, 0.0, 10.0, 20.0]]);
    }
}
