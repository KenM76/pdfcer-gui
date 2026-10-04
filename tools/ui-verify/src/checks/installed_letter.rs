//! # `a_letter_no_page_font_has_is_typed_in_an_installed_face`
//!
//! Types Greek capital omega into `retype-seam.pdf`, whose fonts are
//! non-embedded WinAnsi Helvetica and Times. The key is refused; the check
//! asserts that the Properties panel offers to type it in an installed font,
//! that pressing the offer commits the word with the letter set in a face the
//! engine's replacement-face ladder picked from the font folders, and that the
//! word, drawn in two fonts, is retyped to get there.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/installed_letter.md`.

use super::workaround_offer::{Offer, launch};
use crate::checks::driving::{self, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::Click;
use crate::report::CheckReport;

/// Inside `Hello`, whose baseline is y=700 at 12 pt. Omega is in no WinAnsi
/// font, so neither page font nor a standard-14 face can take it.
const SEAM: Offer = Offer {
    fixture: "retype-seam.pdf",
    stem: "installed-letter",
    at: (80.0, 704.0),
    typed: "\u{3A9}",
    label: "retype",
    env: &[("PDFCER_DIAG_FONT_DIR", r"C:\Windows\Fonts")],
};
const PROPERTIES_PANEL: &str = "file.properties";
const OFFER: &str = "installed-letter-offer"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-text-workaround"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-edit-key-declined"; // ui-text-exempt: a trace event name, never displayed
const REGION: &str = "properties.refusedchar.installed"; // ui-text-exempt: a region name, never displayed

pub struct ALetterNoPageFontHasIsTypedInAnInstalledFace;

impl Check for ALetterNoPageFontHasIsTypedInAnInstalledFace {
    fn name(&self) -> &'static str {
        "a_letter_no_page_font_has_is_typed_in_an_installed_face"
    }

    fn defect(&self) -> &'static str {
        "a letter no font on the page can take is simply refused, though a font in the font \
         folders has it; or the offer to use one commits nothing, or no face from the folders"
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer, doc) = launch(ctx, &SEAM)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    driving::raise_dock_tab(&session, &pointer, ui_rect, PROPERTIES_PANEL)?;
    session.settle(14);
    let path = session.trace_path().display().to_string();
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, SEAM.at.0, SEAM.at.1))?;
    pointer.click(&session, at)?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, SEAM.typed)?;
    session.settle(30);
    let trace = session.trace()?;
    let declined = trace.events(DECLINED).next().is_some();
    let offered = trace.events(OFFER).last().map(|l| l.raw.clone());
    let button = declared(&trace, ui_rect, REGION);
    let (Some(_), Some(button), true) = (&offered, button, declined) else {
        return Ok(Some(format!(
            "★ typing omega into a Helvetica word did not leave the installed-font offer. Key \
             declined: {declined}; offer: {offered:?}; regions beginning `properties.`: {}. \
             Trace: {path}.",
            list(&declared_names(&trace, ui_rect, "properties."))
        )));
    };
    if let Some(early) = trace.events(APPLIED).next() {
        return Ok(Some(format!(
            "★★ an edit was committed before the offer was pressed: `{}`. Trace: {path}.",
            early.raw
        )));
    }
    let edges_before = trace.events(LEFT_EDGE).count();
    pointer.click_rect(&session, button)?;
    session.settle(90);
    let trace = session.trace()?;
    let applied = trace.events(APPLIED).last().map(|l| l.raw.clone());
    let committed = trace
        .events(LEFT_EDGE)
        .skip(edges_before)
        .last()
        .map(|l| l.raw.clone());
    pointer.gone(&session)?;
    Ok(judge_press(
        report,
        applied.as_deref(),
        committed.as_deref(),
        &path,
    ))
}

/// The press must commit, retyping the two-font word, with omega in a face the
/// ladder picked.
fn judge_press(
    report: &mut CheckReport,
    applied: Option<&str>,
    committed: Option<&str>,
    path: &str,
) -> Option<String> {
    let line = applied.unwrap_or_default();
    let retyped = line.contains(&format!("used={}", SEAM.label));
    let laddered = line.contains(" face=") && !line.contains(" face=none");
    let landed = committed.is_some_and(|c| c.contains("committed=yes"));
    if retyped && laddered && landed {
        report.note(format!("pressed: `{line}`"));
        return None;
    }
    Some(format!(
        "★★★ pressing the installed-font offer did not commit omega in a face from the font \
         folders. Retyped: {retyped}; face picked: {laddered}; committed: {landed}. Workaround \
         line: {applied:?}; commit: {committed:?}. Trace: {path}."
    ))
}
