//! `added_text_keeps_its_cmyk_and_opacity` — the text pen's swatch in
//! Properties, set on its CMYK tab with an opacity below full, writes the
//! next added run in `k` under a `gs` carrying that opacity. Scripted pointer,
//! window off the desktop, on a copy of `fixtures/layer-assign.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/add_text_ink_scripted.md`.

use crate::checks::driving::{declared, declared_in, repo_fixture};
use crate::checks::invisible_text_scripted::{launch, press, streams};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const PROPERTIES_BODY: &str = "dock.body.file.properties"; // ui-text-exempt: a trace region name
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: a trace region name
const SWATCH: &str = "properties.tool.text_pen_colour"; // ui-text-exempt: a trace region name
const CMYK_TAB: &str = "properties.tool.text_pen_colour.space.cmyk"; // ui-text-exempt: a trace region name
const OPACITY: &str = "properties.tool.text_pen_colour.opacity"; // ui-text-exempt: a trace region name
/// Letters no other content of the fixture spells, shown by one literal `Tj`.
const WORD: &str = "QKMYW";
/// Blank paper on the fixture's 800 x 600 page, clear of its boxes and annotation.
const SPOT: (f64, f64) = (150.0, 450.0);
/// Blank paper above everything, where the committing click lands.
const AWAY: (f64, f64) = (350.0, 570.0);

/// See the module documentation.
pub struct AddedTextKeepsItsCmykAndOpacity;

impl Check for AddedTextKeepsItsCmykAndOpacity {
    fn name(&self) -> &'static str {
        "added_text_keeps_its_cmyk_and_opacity"
    }

    fn defect(&self) -> &'static str {
        "the text pen offers no CMYK or no opacity, or the added run is written in another \
         colour space or fully opaque"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let copy = ctx.out("add-text-ink.pdf");
        let driven = repo_fixture(FIXTURE, METHOD)
            .and_then(|fixture| {
                std::fs::copy(&fixture, &copy)
                    .map_err(|e| Error::new(format!("copying the fixture: {e}")))
            })
            .and_then(|_| launch(ctx, &mut report, &copy, "add_text_ink"))
            .and_then(|(session, pointer)| {
                let outcome = drive(ctx, &mut report, &session, &pointer, &copy);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            })
            .and_then(|failure| match failure {
                Some(f) => Ok(Some(f)),
                None => judge(&copy, &mut report),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Set the pen to CMYK below full opacity. `Some(failure)` when the picker
/// lacks either, or the pen does not take them.
fn set_pen(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    if declared(&session.trace()?, ui_rect, PROPERTIES_BODY).is_none() {
        press(session, pointer, ui_rect, PROPERTIES_TAB)?;
    }
    press(session, pointer, ui_rect, SWATCH)?;
    let trace = session.trace()?;
    let (Some((_, vp)), Some(_)) = (
        declared_in(&trace, ui_rect, CMYK_TAB),
        declared_in(&trace, ui_rect, OPACITY),
    ) else {
        return Ok(Some(format!(
            "★★★ the text pen's picker publishes no `{CMYK_TAB}` or no `{OPACITY}`: it offers \
             no CMYK or no opacity. Trace: {}.",
            session.trace_path().display()
        )));
    };
    press(session, pointer, ui_rect, CMYK_TAB)?;
    press(session, pointer, ui_rect, OPACITY)?;
    // Closing the picker is the commit.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(30);
    let trace = session.trace()?;
    let line = trace.last("text-pen");
    let opacity = line
        .and_then(|l| l.get("opacity"))
        .and_then(|v| v.parse::<f32>().ok());
    let ink = line.and_then(|l| l.get("ink")).unwrap_or_default();
    report.note(format!("pen: {:?}", line.map(|l| l.raw.clone())));
    Ok(
        (!ink.starts_with("cmyk:") || !opacity.is_some_and(|o| o < 1.0)).then(|| {
            format!(
                "★★★ after the CMYK tab, a press on the opacity slider and Escape, the pen must \
                 read `ink=cmyk:..` and an opacity below 1; the last `text-pen` line is {:?}.",
                line.map(|l| l.raw.clone())
            )
        }),
    )
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    pdf: &std::path::Path,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    if let Some(failure) = set_pen(report, session, pointer, ui_rect)? {
        return Ok(Some(failure));
    }
    let page = crate::fixture::page_geometry(pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, SPOT.0, SPOT.1))?,
    )?;
    session.settle(20);
    if !session
        .trace()?
        .events("text-edit-caret")
        .any(|l| l.get("kind") == Some("Add"))
    {
        return Ok(Some(format!(
            "the click on blank paper traced no `text-edit-caret kind=Add`: Escape on the \
             picker may have disarmed Add text. Trace: {}.",
            session.trace_path().display()
        )));
    }
    pointer.type_text(session, None, WORD)?;
    session.settle(20);
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, AWAY.0, AWAY.1))?,
    )?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let saved = session
        .trace()?
        .events("save-in-place")
        .any(|l| l.get("outcome") == Some("ok"));
    Ok((!saved).then(|| {
        format!(
            "typing, clicking away and Ctrl+S traced no `save-in-place outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// The saved stream showing `WORD` must fill it in CMYK (`k` after its `BT`)
/// under a graphics state (`gs` before the show; the engine wraps the run in
/// an outer `q /name gs`), and the file must hold a `/ca` below 1.
fn judge(pdf: &std::path::Path, report: &mut CheckReport) -> Result<Option<String>> {
    let bytes = std::fs::read(pdf).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    let shown = format!("({WORD})");
    let Some(head) = streams(&bytes).into_iter().find_map(|body| {
        let text = String::from_utf8_lossy(&body).into_owned();
        let at = text.find(&shown)?;
        Some(text[..at].to_owned())
    }) else {
        return Ok(Some(format!(
            "★★★ the saved file shows `{shown}` in no content stream: the typed word was not \
             written."
        )));
    };
    let ops: Vec<&str> = head.split_whitespace().collect();
    let run: Vec<&str> = head[head.rfind("BT").unwrap_or(0)..]
        .split_whitespace()
        .collect();
    report.note(format!("before the show: {}", ops.join(" ")));
    let cmyk = run.contains(&"k");
    let faded = ops.contains(&"gs");
    let alphas = fill_alphas(&bytes);
    report.note(format!("/ca values in the file: {alphas:?}"));
    let below = alphas.iter().any(|a| *a < 1.0);
    Ok((!cmyk || !faded || !below).then(|| {
        format!(
            "★★★ the added run must be filled in CMYK (`k`: {cmyk}) under a graphics state \
             (`gs`: {faded}) whose `/ca` is below 1 (found {alphas:?}); before its show the \
             stream reads `{}`.",
            ops.join(" ")
        )
    }))
}

/// Every numeric `/ca` in the file's uncompressed bytes.
pub(crate) fn fill_alphas(bytes: &[u8]) -> Vec<f64> {
    let text = String::from_utf8_lossy(bytes);
    text.match_indices("/ca ")
        .filter_map(|(i, _)| {
            text[i + 4..]
                .split(|c: char| !(c.is_ascii_digit() || c == '.'))
                .next()
        })
        .filter_map(|n| n.parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fill_alphas;

    #[test]
    fn fill_alphas_reads_every_numeric_ca_and_skips_stroke_alpha() {
        let got = fill_alphas(b"<< /CA 0.2 /ca 0.53 >> << /ca 1 >> << /ca /Bad >>");
        assert_eq!(got, vec![0.53, 1.0]);
    }
}
