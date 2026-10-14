//! `ocr_models_unwritable_folder_is_stated` — when pdfcer may not write in
//! its `models/` folder, Manage OCR models… says so in the window, and Remove
//! neither asks nor deletes.
//!
//! The folder is made unwritable with a deny on creating files and folders in
//! it alone, which is what the program's probe tests; it is lifted before the
//! verdict, and a deny left behind by an interrupted run still lets the next
//! run delete the folder.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_models_manage.md`.

use std::path::Path;
use std::process::Command;

use crate::checks::ocr_models_common::{Drive, Program};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};

// ui-text-exempt: trace region and event names, never displayed.
const UNWRITABLE: &str = "ocr-models.unwritable";
const REMOVE: &str = "ocr-models.remove.ocrs";
const ASKED: &str = "ocr-models-remove-asked";
/// Everyone: deny creating files (WD) and folders (AD) in the folder itself.
const DENY: &str = "*S-1-1-0:(WD,AD)";

/// See the module documentation.
pub struct OcrModelsUnwritable;

impl Check for OcrModelsUnwritable {
    fn name(&self) -> &'static str {
        "ocr_models_unwritable_folder_is_stated"
    }

    fn defect(&self) -> &'static str {
        "with a models folder pdfcer may not write in, the manager does not say so, or offers \
         a Remove that asks and then fails"
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
    let program = Program::new(ctx, "ocr-unwritable", "")?;
    program.copy_reference("ocrs", &program.models)?;
    icacls(&program.models, &["/deny", DENY])?;
    let outcome = Drive::launch(ctx, &program, "ocr-unwritable", &[], report).and_then(|drive| {
        let outcome = steps(&drive, &program, report);
        drive.finish(outcome)
    });
    let lifted = icacls(&program.models, &["/remove:d", "*S-1-1-0"]);
    let outcome = outcome?;
    lifted?;
    Ok(outcome)
}

fn steps(drive: &Drive, program: &Program, report: &mut CheckReport) -> Result<Option<String>> {
    let path = &drive.path;
    let Some(listed) = drive.open_manager()? else {
        return Ok(Some(format!(
            "★ no Manage OCR models item to open. Trace: {path}."
        )));
    };
    report.note(format!("opened: `{}`", listed.raw));
    if listed.get("writable") != Some("false") || !drive.has(UNWRITABLE)? {
        return Ok(Some(format!(
            "★★★ models/ refuses new files and the manager does not say so: `{}`, the sentence \
             drawn: {}. Trace: {path}.",
            listed.raw,
            drive.has(UNWRITABLE)?
        )));
    }
    if drive.has(REMOVE)? {
        let mark = drive.mark()?;
        drive.click(REMOVE)?;
        if drive.wait(ASKED, mark, 2)?.is_some() {
            return Ok(Some(format!(
                "★★ Remove asked to delete from a folder it may not write in. Trace: {path}."
            )));
        }
    }
    if !program.models.join("ocrs").is_dir() {
        return Ok(Some(format!("★★★ models/ocrs is gone. Trace: {path}.")));
    }
    Ok(None)
}

fn icacls(dir: &Path, args: &[&str]) -> Result<()> {
    let out = Command::new("icacls")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| Error::new(format!("icacls could not run: {e}")))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::new(format!(
            "icacls {} {}: {}",
            dir.display(),
            args.join(" "),
            String::from_utf8_lossy(&out.stdout).trim()
        )))
    }
}
