//! `ocr_models_set_default_and_remove_down_to_none` — File > Recognise >
//! Manage OCR models… lists every supported recogniser, installed or not;
//! Set as default is what Recognise text starts with; Remove asks first,
//! deletes the folder, frees what it says and names the new default; Remove
//! all leaves no model; and Recognise text, reached from the ribbon and from
//! Find's offer on a page with no text, then says no OCR engine is installed
//! and opens the manager.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_models_manage.md`.

use crate::checks::ocr_models_common::{Drive, LISTED, Program, bytes_under, names};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::Result;
use crate::trace::TraceLine;

/// Planted in the copy's `models/`, from the binary's own.
const PLANTED: [&str; 3] = ["paddle", "ocrs", "ocrcer"];
/// Supported recognisers the copy has no model of.
const ABSENT: [&str; 2] = ["paddle-vl", "tesseract"];

// ui-text-exempt: trace region and event names, never displayed.
const DEFAULT_SET: &str = "ocr-models-default";
const ASKED: &str = "ocr-models-remove-asked";
const REMOVED: &str = "ocr-models-removed";
const CONFIRM: &str = "ocr-models.confirm";
const REMOVE_ALL: &str = "ocr-models.remove-all";
const RESULT: &str = "ocr-models.result";
const CLOSE: &str = "ocr-models.close";
const RECOGNISE: &str = "ribbon.item.file.ocr";
const NO_ENGINE: &str = "ocr-no-engine";
const OPEN_MANAGER: &str = "ocr-open-manager";
const OCR_CLOSE: &str = "ocr-close";
const FIND_OFFER: &str = "find-ocr-offer.accept";

/// See the module documentation.
pub struct OcrModelsSetDefaultAndRemove;

impl Check for OcrModelsSetDefaultAndRemove {
    fn name(&self) -> &'static str {
        "ocr_models_set_default_and_remove_down_to_none"
    }

    fn defect(&self) -> &'static str {
        "the OCR model manager lists only what can be downloaded, cannot set the default, \
         removes without asking or removes nothing, frees a different amount than it says, \
         or leaves Recognise text silent or broken with no engine installed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let program = Program::new(ctx, "ocr-manage", "")?;
    for name in PLANTED {
        program.copy_reference(name, &program.models)?;
    }
    let drive = Drive::launch(ctx, &program, "ocr-manage", &[], report)?;
    let outcome = steps(&drive, &program, report);
    drive.finish(outcome)
}

fn steps(drive: &Drive, program: &Program, report: &mut CheckReport) -> Result<Option<String>> {
    let path = drive.path.clone();
    let Some(listed) = drive.open_manager()? else {
        return Ok(Some(format!(
            "★ File > Recognise has no Manage OCR models item, or it lists nothing. Trace: {path}."
        )));
    };
    report.note(format!("opened: `{}`", listed.raw));
    if let Some(f) = the_list(drive, &listed)? {
        return Ok(Some(f));
    }
    let Some(chosen) = set_default(drive, program, &listed, report)? else {
        return Ok(Some(format!(
            "★★ Set as default did not take. Trace: {path}."
        )));
    };
    if let Some(f) = remove_one(drive, program, &chosen, report)? {
        return Ok(Some(f));
    }
    if let Some(f) = remove_all(drive, program, report)? {
        return Ok(Some(f));
    }
    if let Some(f) = no_engine(drive, report)? {
        return Ok(Some(f));
    }
    find_offer(drive, report)
}

/// Every planted model has a row, every absent recogniser a "not installed".
fn the_list(drive: &Drive, listed: &TraceLine) -> Result<Option<String>> {
    let path = &drive.path;
    let installed = names(listed, "names");
    for name in PLANTED {
        if !installed.iter().any(|n| n == name) {
            return Ok(Some(format!(
                "★ `{name}` is in models/ and the manager does not list it: `{}`. Trace: {path}.",
                listed.raw
            )));
        }
        if !drive.has(&format!("ocr-models.row.{name}"))? {
            return Ok(Some(format!(
                "★ `{name}` is listed and draws no row. Trace: {path}."
            )));
        }
    }
    let missing = names(listed, "missing");
    for token in ABSENT {
        if !missing.iter().any(|n| n == token)
            || !drive.has(&format!("ocr-models.missing.{token}"))?
        {
            return Ok(Some(format!(
                "★ `{token}` is supported and not installed, and the manager does not say so: \
                 `{}`. Trace: {path}.",
                listed.raw
            )));
        }
    }
    Ok(None)
}

/// Make a model that is not the default the default; its name, or `None`
/// when that did not take.
fn set_default(
    drive: &Drive,
    program: &Program,
    listed: &TraceLine,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let before = listed.get("default").unwrap_or("none").to_owned();
    let Some(chosen) = PLANTED.iter().find(|n| {
        **n != before
            && drive
                .has(&format!("ocr-models.default.{n}"))
                .unwrap_or(false)
    }) else {
        return Ok(None);
    };
    let mark = drive.mark()?;
    drive.click(&format!("ocr-models.default.{chosen}"))?;
    let Some(set) = drive.wait(DEFAULT_SET, mark, 5)? else {
        return Ok(None);
    };
    let relisted = drive.wait(LISTED, mark, 5)?;
    let now = relisted
        .as_ref()
        .and_then(|l| l.get("default"))
        .unwrap_or("none");
    let saved = program
        .prefs()
        .lines()
        .any(|l| l.trim() == format!("ocr_model = {chosen}"));
    report.note(format!(
        "default {before} -> {now}; `{}`; preferences hold it: {saved}",
        set.raw
    ));
    Ok((set.get("name") == Some(chosen) && now == *chosen && saved).then(|| (*chosen).to_owned()))
}

/// Remove the default: asked first, then gone, freed as measured, and a new
/// default named.
fn remove_one(
    drive: &Drive,
    program: &Program,
    name: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = &drive.path;
    let folder = program.models.join(name);
    let measured = bytes_under(&folder);
    let mark = drive.mark()?;
    drive.click(&format!("ocr-models.remove.{name}"))?;
    let Some(asked) = drive.wait(ASKED, mark, 3)? else {
        return Ok(Some(format!(
            "★★ Remove on `{name}` asked nothing. Trace: {path}."
        )));
    };
    if !folder.is_dir() {
        return Ok(Some(format!(
            "★★★ {} was deleted before the operator confirmed. Trace: {path}.",
            folder.display()
        )));
    }
    if asked.get("bytes") != Some(measured.to_string().as_str()) {
        return Ok(Some(format!(
            "★★ the question says {:?} bytes; the folder holds {measured}. Trace: {path}.",
            asked.get("bytes")
        )));
    }
    let mark = drive.mark()?;
    drive.click(CONFIRM)?;
    let Some(removed) = drive.wait(REMOVED, mark, 15)? else {
        return Ok(Some(format!(
            "★★ confirming removed nothing. Trace: {path}."
        )));
    };
    report.note(format!("removed one: `{}`", removed.raw));
    let new_default = removed.get("default").unwrap_or("unchanged");
    if folder.exists()
        || removed.get("names") != Some(name)
        || removed.get("freed") != Some(measured.to_string().as_str())
    {
        return Ok(Some(format!(
            "★★★ removing `{name}` ({measured} bytes): `{}`, and the folder is still there: {}. \
             Trace: {path}.",
            removed.raw,
            folder.exists()
        )));
    }
    if new_default == "unchanged" || new_default == "none" || new_default == name {
        return Ok(Some(format!(
            "★★★ the default was removed and two runnable models remain, yet the new default \
             is `{new_default}`. Trace: {path}."
        )));
    }
    if !drive.has(RESULT)? {
        return Ok(Some(format!(
            "★★ the removal leaves no sentence. Trace: {path}."
        )));
    }
    Ok(None)
}

/// Remove all: asked first, then nothing left and no default.
fn remove_all(
    drive: &Drive,
    program: &Program,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = &drive.path;
    let mark = drive.mark()?;
    drive.click(REMOVE_ALL)?;
    if drive.wait(ASKED, mark, 3)?.is_none() {
        return Ok(Some(format!("★★ Remove all asked nothing. Trace: {path}.")));
    }
    let mark = drive.mark()?;
    drive.click(CONFIRM)?;
    let Some(removed) = drive.wait(REMOVED, mark, 15)? else {
        return Ok(Some(format!(
            "★★ confirming Remove all removed nothing. Trace: {path}."
        )));
    };
    let relisted = drive.wait(LISTED, mark, 5)?;
    report.note(format!("removed all: `{}`", removed.raw));
    let left: Vec<String> = PLANTED
        .iter()
        .filter(|n| program.models.join(n).exists())
        .map(|n| (*n).to_owned())
        .collect();
    let count = relisted.as_ref().and_then(|l| l.get("installed"));
    if !left.is_empty() || removed.get("default") != Some("none") || count != Some("0") {
        return Ok(Some(format!(
            "★★★ after Remove all: folders left {left:?}, `{}`, listed installed={count:?}. \
             Trace: {path}.",
            removed.raw
        )));
    }
    Ok(None)
}

/// With nothing installed, Recognise text says so and opens the manager.
fn no_engine(drive: &Drive, report: &mut CheckReport) -> Result<Option<String>> {
    let path = &drive.path;
    drive.click(CLOSE)?;
    let mark = drive.mark()?;
    drive.click(RECOGNISE)?;
    let Some(said) = drive.wait(NO_ENGINE, mark, 5)? else {
        return Ok(Some(format!(
            "★★★ with no model installed, Recognise text does not say so. Trace: {path}."
        )));
    };
    report.note(format!("Recognise text: `{}`", said.raw));
    if said.get("none-installed") != Some("true") || !drive.has(OPEN_MANAGER)? {
        return Ok(Some(format!(
            "★★★ Recognise text with nothing installed: `{}`, and the manager button is drawn: \
             {}. Trace: {path}.",
            said.raw,
            drive.has(OPEN_MANAGER)?
        )));
    }
    let mark = drive.mark()?;
    drive.click(OPEN_MANAGER)?;
    let Some(listed) = drive.wait(LISTED, mark, 10)? else {
        return Ok(Some(format!(
            "★★★ Recognise text's Manage OCR models button opened nothing. Trace: {path}."
        )));
    };
    if drive.session.has_exited()? {
        return Ok(Some(format!("★★★ the program ended. Trace: {path}.")));
    }
    report.note(format!("the button reopened the manager: `{}`", listed.raw));
    Ok(None)
}

/// Find on the image-only page offers Recognise text, and the offer reaches
/// the same no-engine sentence.
fn find_offer(drive: &Drive, report: &mut CheckReport) -> Result<Option<String>> {
    let path = &drive.path;
    drive.click(CLOSE)?;
    drive.click(OCR_CLOSE)?;
    drive.pointer.key(&drive.session, None, "F", Some("ctrl"))?;
    drive.pointer.type_text(&drive.session, None, "e")?;
    drive.pointer.key(&drive.session, None, "Enter", None)?;
    drive.session.settle(20);
    if !drive.has(FIND_OFFER)? {
        return Ok(Some(format!(
            "★★ Find on a page with no text offers no Recognise text. Trace: {path}."
        )));
    }
    let mark = drive.mark()?;
    drive.click(FIND_OFFER)?;
    let Some(said) = drive.wait(NO_ENGINE, mark, 5)? else {
        return Ok(Some(format!(
            "★★★ with no model installed, Find's Recognise text offer does not say so. Trace: {path}."
        )));
    };
    report.note(format!("from Find: `{}`", said.raw));
    if said.get("none-installed") != Some("true") || drive.session.has_exited()? {
        return Ok(Some(format!(
            "★★★ Find's offer reached `{}`. Trace: {path}.",
            said.raw
        )));
    }
    Ok(None)
}
