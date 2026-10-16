//! `a_select_click_on_recognised_text_takes_its_paragraph` — with the Select
//! tool, a click on any word of a recognised (invisible) text block selects
//! the whole block, not the word and not the scan beneath it. See
//! `docs/modules/ui-verify/checks/ocr_select_block.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

pub(super) const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Regenerate it: python tools/gen-ocr-layer-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// `SITE`, `SHEET` and `SCANNED`: one word on each of the layer's three lines.
const WORDS: [(&str, (f64, f64)); 3] = [
    ("SITE", (85.0, 704.0)),
    ("SHEET", (240.0, 681.0)),
    ("SCANNED", (90.0, 657.0)),
];
/// Scan only, no recognised text: the control.
const SCAN: (f64, f64) = (450.0, 400.0);

/// See the module documentation.
pub struct ASelectClickOnRecognisedTextTakesItsParagraph;

impl Check for ASelectClickOnRecognisedTextTakesItsParagraph {
    fn name(&self) -> &'static str {
        "a_select_click_on_recognised_text_takes_its_paragraph"
    }

    fn defect(&self) -> &'static str {
        "A Select click on a scan's recognised text selects one word, or the scan under it, \
         so the paragraph cannot be reached"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit", "ocr-select-block", drive)
    }
}

/// Click at `at`; the `(sel, first)` the click left, or why not. The trace
/// line is de-duplicated, so the caller alternates with [`SCAN`].
fn click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<std::result::Result<(usize, String), String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    session.settle(20);
    let trace = session.trace()?;
    Ok(trace
        .last_after(SELECTION, mark)
        .and_then(|l| Some((l.get_usize("sel")?, l.get("first")?.to_owned())))
        .ok_or_else(|| format!("no `{SELECTION}` line after the click at {at:?}")))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let scan = match click(ctx, session, pointer, SCAN)? {
        Ok(v) => v,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("scan: sel={} first={}", scan.0, scan.1));
    let mut block: Option<String> = None;
    for (word, at) in WORDS {
        if block.is_some()
            && let Err(why) = click(ctx, session, pointer, SCAN)?
        {
            return Ok(Some(why));
        }
        let got = match click(ctx, session, pointer, at)? {
            Ok(v) => v,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("{word}: sel={} first={}", got.0, got.1));
        if got.0 != 1 || got.1 == scan.1 {
            return Ok(Some(format!(
                "★ a click on `{word}` selected {} entr(ies), first {} (the scan alone is {}), \
                 not the recognised text.",
                got.0, got.1, scan.1
            )));
        }
        if let Some(b) = &block
            && *b != got.1
        {
            return Ok(Some(format!(
                "★★ a click on `{word}` selected {}, but the block's other lines selected {b}: \
                 one line, not the paragraph.",
                got.1
            )));
        }
        block = Some(got.1);
    }
    Ok(None)
}
