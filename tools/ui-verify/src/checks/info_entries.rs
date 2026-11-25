//! `a_custom_info_entry_reaches_info_and_xmp` — File ▸ Document ▸ Properties
//! on `fixtures/drawing-sheets.pdf`: an entry added under *Other entries* is
//! one undo step, the engine keeps the XMP copy in step, a reserved name is
//! refused with its own sentence, and the saved copy carries the entry in both
//! `/Info` and the XMP packet — read back by a second process opening it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/info_entries.md`.

use super::dimdrive::{Fixture, launch_path, launch_with, press};
use super::multiselect::{after, save, step};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

pub(super) const SHEETS: Fixture = Fixture {
    file: "drawing-sheets.pdf",
    method: "Rebuild it with `python fixtures/drawing-sheets.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 420.0,
        height_pt: 297.0,
    },
};
const INVOKE: &str = "mode.review,file.document_properties"; // ui-text-exempt: ribbon command ids
/// Mirrors `panels::docprops::custom`'s regions.
const NEW_KEY: &str = "properties.custom.key"; // ui-text-exempt: a trace region name
const NEW_VALUE: &str = "properties.custom.value"; // ui-text-exempt: a trace region name
const ADD: &str = "properties.custom.add"; // ui-text-exempt: a trace region name
const SET: &str = "info-entry-set"; // ui-text-exempt: a trace event name
const APPLIED: &str = "set-info-entry"; // ui-text-exempt: a trace event name
const REFUSED: &str = "set-info-entry-refused"; // ui-text-exempt: a trace event name
const LISTED: &str = "info-entries-now"; // ui-text-exempt: a trace event name
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// The entry added, and what the saved bytes must hold for it.
const KEY: &str = "DrawingNo"; // ui-text-exempt: typed input
const VALUE: &str = "S-200"; // ui-text-exempt: typed input
const INFO_KEY: &str = "/DrawingNo"; // ui-text-exempt: a PDF fragment
const INFO_VALUE: &str = "(S-200)"; // ui-text-exempt: a PDF fragment
/// The XMP element's open tag, which may carry its namespace declaration.
const XMP_OPEN: &str = "<pdfx:DrawingNo"; // ui-text-exempt: an XMP fragment
/// What follows the open tag's `>`: the value and the close tag.
const XMP_BODY: &str = "S-200</pdfx:DrawingNo>"; // ui-text-exempt: an XMP fragment

/// See the module documentation.
pub struct ACustomInfoEntryReachesInfoAndXmp;

impl Check for ACustomInfoEntryReachesInfoAndXmp {
    fn name(&self) -> &'static str {
        "a_custom_info_entry_reaches_info_and_xmp"
    }

    fn defect(&self) -> &'static str {
        "Document properties cannot add an entry of the operator's own name, the entry is not \
         saved, its XMP copy is not kept in step, or a reserved name is accepted silently"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saved = ctx.out("info-entries.saved.pdf");
        let _ = std::fs::remove_file(&saved);
        let env = [(SAVE_PATH_ENV, saved.display().to_string())];
        let outcome = launch_with(ctx, &mut report, &SHEETS, INVOKE, "info-entries", &env)
            .and_then(|(session, pointer)| {
                let outcome = drive(ctx, &mut report, &session, &pointer);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            })
            .and_then(|failure| match failure {
                Some(f) => Ok(Some(f)),
                None => judge_saved(ctx, &mut report, &saved),
            });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Type `key` and `value` into the Add row and press Add.
fn add(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    key: &str,
    value: &str,
) -> Result<usize> {
    press(ctx, session, pointer, NEW_KEY)?;
    pointer.type_text(session, None, key)?;
    press(ctx, session, pointer, NEW_VALUE)?;
    pointer.type_text(session, None, value)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD)?;
    session.settle(30);
    Ok(mark)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let first = step!(after(session, LISTED, 0, "Document properties was opened"));
    report.note(format!("before: `{}`", first.raw));
    if first.get("keys").is_some_and(|k| k.contains(KEY)) {
        return Ok(Some(format!(
            "the fixture already lists `{KEY}`, so adding it proves nothing: `{}`.",
            first.raw
        )));
    }

    let mark = add(ctx, session, pointer, KEY, VALUE)?;
    let set = step!(after(session, SET, mark, "Add entry was pressed"));
    let applied = step!(after(session, APPLIED, mark, "the entry was set"));
    report.note(format!("`{}` / `{}`", set.raw, applied.raw));
    if set.get("key") != Some(KEY)
        || set.get("info_changed") != Some("1")
        || !set.get("xmp").is_some_and(|x| x.starts_with("updated:"))
    {
        return Ok(Some(format!(
            "adding `{KEY}` traced `{}`: the key, a changed /Info and an updated XMP copy were \
             wanted.",
            set.raw
        )));
    }
    let listed = step!(after(session, LISTED, mark, "the entry was set"));
    let want = format!("{KEY}:{}", VALUE.chars().count());
    if !listed
        .get("keys")
        .is_some_and(|k| k.split('|').any(|e| e == want))
    {
        return Ok(Some(format!(
            "after the add the panel lists `{}`; `{want}` was wanted among its entries.",
            listed.raw
        )));
    }

    // A reserved name is refused, and with its own sentence.
    let mark = add(ctx, session, pointer, "Producer", "me")?;
    let refused = step!(after(
        session,
        REFUSED,
        mark,
        "Add entry was pressed for Producer"
    ));
    report.note(format!("refused: `{}`", refused.raw));
    if session.trace()?.last_after(APPLIED, mark).is_some() {
        return Ok(Some(
            "Producer was written; the engine reserves it.".to_owned(),
        ));
    }

    let saved = save(ctx, session, pointer)?;
    Ok(saved)
}

/// The saved copy holds the entry in `/Info` and in XMP, and a second process
/// lists it on opening.
fn judge_saved(
    ctx: &CheckContext,
    report: &mut CheckReport,
    saved: &std::path::Path,
) -> Result<Option<String>> {
    let bytes = std::fs::read(saved)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", saved.display())))?;
    let text = String::from_utf8_lossy(&bytes);
    let in_info = text.match_indices(INFO_KEY).any(|(at, _)| {
        text[at + INFO_KEY.len()..]
            .trim_start()
            .starts_with(INFO_VALUE)
    });
    if !in_info {
        return Ok(Some(format!(
            "the saved copy {} has no /Info entry `{INFO_KEY} {INFO_VALUE}`.",
            saved.display()
        )));
    }
    if !xmp_holds(&text) {
        return Ok(Some(format!(
            "the saved copy {} has no XMP property `{XMP_OPEN}…>{XMP_BODY}`.",
            saved.display()
        )));
    }
    report.note(format!(
        "the saved bytes hold `{INFO_KEY} {INFO_VALUE}` and `{XMP_OPEN}…>{XMP_BODY}`"
    ));

    let (session, pointer) = launch_path(
        ctx,
        report,
        saved.to_path_buf(),
        INVOKE,
        "info-entries.reopen",
        &[],
    )?;
    let listed = after(&session, LISTED, 0, "the saved copy was opened");
    let _ = pointer.gone(&session);
    let listed = step!(listed);
    report.note(format!("reopened: `{}`", listed.raw));
    let want = format!("{KEY}:{}", VALUE.chars().count());
    if !listed
        .get("keys")
        .is_some_and(|k| k.split('|').any(|e| e == want))
    {
        return Ok(Some(format!(
            "a second process opening the saved copy lists `{}`; `{want}` was wanted.",
            listed.raw
        )));
    }
    Ok(None)
}

/// Whether `text` holds the XMP element with the value, its open tag bare or
/// carrying attributes such as an inline `xmlns:pdfx`.
fn xmp_holds(text: &str) -> bool {
    text.match_indices(XMP_OPEN).any(|(at, _)| {
        let rest = &text[at + XMP_OPEN.len()..];
        let attrs_ok = rest.starts_with('>') || rest.starts_with(char::is_whitespace);
        attrs_ok
            && rest
                .find('>')
                .is_some_and(|close| rest[close + 1..].starts_with(XMP_BODY))
    })
}

#[cfg(test)]
mod tests {
    use super::xmp_holds;

    #[test]
    fn the_xmp_element_is_found_bare_or_with_its_namespace() {
        assert!(xmp_holds("<pdfx:DrawingNo>S-200</pdfx:DrawingNo>"));
        assert!(xmp_holds(
            r#"<pdfx:DrawingNo xmlns:pdfx="http://ns.adobe.com/pdfx/1.3/">S-200</pdfx:DrawingNo>"#
        ));
    }

    #[test]
    fn another_value_or_only_the_info_entry_is_not_the_xmp_element() {
        assert!(!xmp_holds("<pdfx:DrawingNo>S-201</pdfx:DrawingNo>"));
        assert!(!xmp_holds("<pdfx:DrawingNoOld>S-200</pdfx:DrawingNo>"));
        assert!(!xmp_holds("/DrawingNo (S-200)"));
    }
}
