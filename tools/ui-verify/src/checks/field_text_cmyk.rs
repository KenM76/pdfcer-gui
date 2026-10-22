//! `a_fields_text_colour_is_stored_in_cmyk` — a form field's text colour, set
//! in Properties on the picker's CMYK tab, is written into the field's `/DA`
//! as CMYK (`k`). Scripted pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_text_cmyk.md`.

use crate::checks::driving::declared_in;
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "three-text-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
const LABEL: &str = "field-text-cmyk";
const SWATCH: &str = "properties.field_edit.text_colour"; // ui-text-exempt: trace region name
const CMYK_TAB: &str = "properties.field_edit.text_colour.space.cmyk"; // ui-text-exempt: trace region name
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct AFieldsTextColourIsStoredInCmyk;

impl Check for AFieldsTextColourIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_fields_text_colour_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "a form field's text colour offers no CMYK, or a colour chosen on its CMYK tab is not \
         written to the field as CMYK"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven =
            properties_pane::launch_on_field(ctx, &mut report, (FIXTURE, METHOD), FIELD, LABEL)
                .and_then(|(session, pointer)| {
                    let outcome = steps(ctx, &mut report, &session, &pointer);
                    let parked = pointer.gone(&session);
                    match outcome? {
                        Some(failure) => Ok(Some(failure)),
                        None => parked.map(|_| None),
                    }
                });
        match driven {
            Ok(None) => report.pass(),
            Ok(Some(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The `/DA` strings of a file, as written.
/// Every `/DA` value, literal `( … )` or hex `< … >` (the engine writes a
/// rewritten `/DA` in hex), with its whitespace collapsed to single spaces.
fn appearance_strings(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    text.match_indices("/DA")
        .filter_map(|(i, _)| {
            let rest = text[i + 3..].trim_start();
            let raw = if let Some(lit) = rest.strip_prefix('(') {
                lit.split(')').next()?.to_owned()
            } else {
                let hex: String = rest
                    .strip_prefix('<')?
                    .split('>')
                    .next()?
                    .split_whitespace()
                    .collect();
                let decoded: Option<Vec<u8>> = (0..hex.len() / 2)
                    .map(|k| u8::from_str_radix(hex.get(2 * k..2 * k + 2)?, 16).ok())
                    .collect();
                String::from_utf8_lossy(&decoded?).into_owned()
            };
            Some(raw.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .collect()
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let copy = ctx.out(&format!("{LABEL}.pdf"));
    let read = |p: &std::path::Path| {
        std::fs::read(p).map_err(|e| Error::new(format!("reading {}: {e}", p.display())))
    };
    // The fixture's fields are all grey, so a CMYK `/DA` can only be the edit's.
    if appearance_strings(&read(&copy)?)
        .iter()
        .any(|s| s.ends_with(" k"))
    {
        return Ok(Some(format!(
            "the fixture already holds a CMYK `/DA`, so this check cannot tell the edit's. {METHOD}"
        )));
    }
    properties_pane::press(ctx, session, pointer, SWATCH, 20)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect, CMYK_TAB) else {
        return Ok(Some(format!(
            "★★★ the field's text colour picker opened with no CMYK tab (`{CMYK_TAB}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    properties_pane::press(ctx, session, pointer, CMYK_TAB, 20)?;
    // The close is the commit.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let strings = appearance_strings(&read(&copy)?);
    report.note(format!("/DA strings in the saved file: {strings:?}"));
    Ok((!strings.iter().any(|s| s.ends_with(" k"))).then(|| {
        format!(
            "★★★ after the CMYK tab was chosen and the picker closed, the saved file holds no \
             `/DA` ending in a CMYK `k` (found {strings:?}). Trace: {}.",
            session.trace_path().display()
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::appearance_strings;

    #[test]
    fn appearance_strings_reads_each_da_up_to_its_close() {
        let got = appearance_strings(b"<< /DA (/Helv 10 Tf 0 g) >> << /DA(/F1 9 Tf 0 0 0 1 k) >>");
        assert_eq!(got, vec!["/Helv 10 Tf 0 g", "/F1 9 Tf 0 0 0 1 k"]);
    }

    /// `/Helv 10 Tf\n0.5 0 0 1 k\n` as the engine writes it, in hex.
    #[test]
    fn appearance_strings_decodes_a_hex_da() {
        let got =
            appearance_strings(b"<< /DA <2F48656C762031302054660A302E35203020302031206B0A> >>");
        assert_eq!(got, vec!["/Helv 10 Tf 0.5 0 0 1 k"]);
    }
}
