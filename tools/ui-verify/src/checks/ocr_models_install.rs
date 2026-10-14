//! `ocr_models_install_from_a_zip_and_a_folder` — Manage OCR models…'s
//! Install from file takes a pdfcer OCR add-on zip, refuses one whose files
//! fail the manifest's `sha256` lines and leaves nothing behind, installs a
//! good one byte for byte, refuses a second copy of an installed model, and
//! Install from folder puts a removed bundled `paddle` back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_models_install.md`.

use std::path::Path;

use crate::checks::ocr_models_common::{Drive, LISTED, Program, names};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};

// ui-text-exempt: environment names and trace names, never displayed.
const ZIP_ENV: &str = "PDFCER_DIAG_OCR_ADDON_ZIP";
const FOLDER_ENV: &str = "PDFCER_DIAG_OCR_ADDON_FOLDER";
const INSTALL_FILE: &str = "ocr-models.install-file";
const INSTALL_FOLDER: &str = "ocr-models.install-folder";
const INSTALLED: &str = "ocr-models-installed";
const FAILED: &str = "ocr-models-install-failed";
const RESULT: &str = "ocr-models.result";

/// The add-on zipped: the bundled `ocrs`, whose manifest hashes both files.
const ZIPPED: &str = "ocrs";
const ZIPPED_FILES: [&str; 2] = ["text-detection.rten", "text-rec-checkpoint.rten"];
/// Put back from a folder: the bundled `paddle`, which has no manifest.
const FOLDER: &str = "paddle";
const FOLDER_FILES: [&str; 3] = ["det.onnx", "rec.onnx", "dict.txt"];

/// See the module documentation.
pub struct OcrModelsInstall;

impl Check for OcrModelsInstall {
    fn name(&self) -> &'static str {
        "ocr_models_install_from_a_zip_and_a_folder"
    }

    fn defect(&self) -> &'static str {
        "Install from file or folder does not install, installs files that fail their \
         fingerprints, leaves a half-unpacked add-on behind, or installs a second copy"
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
    let program = Program::new(ctx, "ocr-install", "")?;
    let sources = ctx.out("ocr-install-sources");
    let _ = std::fs::remove_dir_all(&sources);
    let addon = program.copy_reference(ZIPPED, &sources)?;
    let folder = program.copy_reference(FOLDER, &sources.join("folder"))?;
    let zip = std::path::absolute(sources.join("pdfcer-ocr-addon-ocrs.zip"))
        .map_err(|e| Error::new(format!("cannot make the zip path absolute: {e}")))?;
    let env = [
        (ZIP_ENV, zip.display().to_string()),
        (FOLDER_ENV, folder.display().to_string()),
    ];
    let drive = Drive::launch(ctx, &program, "ocr-install", &env, report)?;
    let outcome = steps(&drive, &program, &addon, &zip, report);
    drive.finish(outcome)
}

fn steps(
    drive: &Drive,
    program: &Program,
    addon: &Path,
    zip: &Path,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = drive.path.clone();
    let Some(listed) = drive.open_manager()? else {
        return Ok(Some(format!(
            "★ no Manage OCR models item to open. Trace: {path}."
        )));
    };
    if listed.get("installed") != Some("0") {
        return Ok(Some(format!(
            "the copy's models/ is empty and the manager lists `{}`. Trace: {path}.",
            listed.raw
        )));
    }

    // A tampered add-on: the zip itself is sound, one file fails its sha256.
    write_zip(zip, addon, Some("text-rec-checkpoint.rten"))?;
    let mark = drive.mark()?;
    drive.click(INSTALL_FILE)?;
    let Some(refused) = drive.wait(FAILED, mark, 15)? else {
        return Ok(Some(format!(
            "★★★ an add-on whose file fails its fingerprint was not refused: {:?}. Trace: {path}.",
            drive
                .session
                .trace()?
                .last_after(INSTALLED, mark)
                .map(|l| l.raw.clone())
        )));
    };
    report.note(format!("tampered zip: `{}`", refused.raw));
    if refused.get("why") != Some("verify") || program.models.join(ZIPPED).exists() {
        return Ok(Some(format!(
            "★★★ the tampered add-on: `{}`; models/{ZIPPED} exists: {}. Trace: {path}.",
            refused.raw,
            program.models.join(ZIPPED).exists()
        )));
    }
    if let Some(left) = leftover(program) {
        return Ok(Some(format!(
            "★★ a refused install left {left} behind. Trace: {path}."
        )));
    }

    write_zip(zip, addon, None)?;
    let mark = drive.mark()?;
    drive.click(INSTALL_FILE)?;
    let Some(done) = drive.wait(INSTALLED, mark, 15)? else {
        return Ok(Some(format!(
            "★★★ a good add-on zip did not install: {:?}. Trace: {path}.",
            drive
                .session
                .trace()?
                .last_after(FAILED, mark)
                .map(|l| l.raw.clone())
        )));
    };
    report.note(format!("good zip: `{}`", done.raw));
    let relisted = drive.wait(LISTED, mark, 5)?;
    if done.get("name") != Some(ZIPPED)
        || done.get("checked") != Some("2")
        || !relisted
            .as_ref()
            .is_some_and(|l| names(l, "names") == [ZIPPED])
        || !drive.has(RESULT)?
    {
        return Ok(Some(format!(
            "★★★ the install: `{}`, then the list `{:?}`. Trace: {path}.",
            done.raw,
            relisted.map(|l| l.raw)
        )));
    }
    if let Some(f) = same_files(&program.models.join(ZIPPED), addon, &ZIPPED_FILES) {
        return Ok(Some(f));
    }

    let mark = drive.mark()?;
    drive.click(INSTALL_FILE)?;
    let again = drive.wait(FAILED, mark, 15)?;
    if again.as_ref().and_then(|l| l.get("why")) != Some("exists") {
        return Ok(Some(format!(
            "★★ installing `{ZIPPED}` a second time: {:?}. Trace: {path}.",
            again.map(|l| l.raw)
        )));
    }

    let mark = drive.mark()?;
    drive.click(INSTALL_FOLDER)?;
    let Some(put_back) = drive.wait(INSTALLED, mark, 15)? else {
        return Ok(Some(format!(
            "★★★ `{FOLDER}` from a folder did not install: {:?}. Trace: {path}.",
            drive
                .session
                .trace()?
                .last_after(FAILED, mark)
                .map(|l| l.raw.clone())
        )));
    };
    report.note(format!("folder: `{}`", put_back.raw));
    if put_back.get("name") != Some(FOLDER) {
        return Ok(Some(format!(
            "★★★ the folder installed `{}`. Trace: {path}.",
            put_back.raw
        )));
    }
    Ok(same_files(
        &program.models.join(FOLDER),
        &program.reference.join(FOLDER),
        &FOLDER_FILES,
    ))
}

/// A staging folder left beside `models/`.
fn leftover(program: &Program) -> Option<String> {
    let parent = program.models.parent()?;
    std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with(".pdfcer-installing"))
}

/// Each of `files` in `got` is byte for byte the one in `want`.
fn same_files(got: &Path, want: &Path, files: &[&str]) -> Option<String> {
    for name in files {
        let (a, b) = (
            std::fs::read(got.join(name)),
            std::fs::read(want.join(name)),
        );
        match (a, b) {
            (Ok(a), Ok(b)) if a == b => {}
            (a, _) => {
                return Some(format!(
                    "★★★ {} is {} after the install reported success.",
                    got.join(name).display(),
                    if a.is_ok() {
                        "different from the source"
                    } else {
                        "missing"
                    }
                ));
            }
        }
    }
    None
}

/// Zip `addon` as `<its name>/<file>`, deflated, with `tamper`'s first byte
/// flipped before compression (so the zip's own CRCs still agree).
fn write_zip(zip: &Path, addon: &Path, tamper: Option<&str>) -> Result<()> {
    let top = addon
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut entries = Vec::new();
    for entry in
        std::fs::read_dir(addon).map_err(|e| Error::new(format!("reading the add-on: {e}")))?
    {
        let entry = entry.map_err(|e| Error::new(format!("reading the add-on: {e}")))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let mut data =
            std::fs::read(entry.path()).map_err(|e| Error::new(format!("reading {name}: {e}")))?;
        if tamper == Some(name.as_str())
            && let Some(b) = data.first_mut()
        {
            *b ^= 0xFF;
        }
        entries.push((format!("{top}/{name}"), data));
    }
    std::fs::write(zip, zip_bytes(&entries))
        .map_err(|e| Error::new(format!("writing the zip: {e}")))
}

/// A zip of `entries`, each deflated, as `zipfile` and the engine's packager
/// write them.
fn zip_bytes(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let le16 = |v: &mut Vec<u8>, n: u16| v.extend_from_slice(&n.to_le_bytes());
    let le32 = |v: &mut Vec<u8>, n: u32| v.extend_from_slice(&n.to_le_bytes());
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let packed = miniz_oxide::deflate::compress_to_vec(data, 6);
        let crc = crate::png::crc32(data);
        let offset = u32::try_from(out.len()).unwrap_or(u32::MAX);
        let (size, csize) = (
            u32::try_from(data.len()).unwrap_or(u32::MAX),
            u32::try_from(packed.len()).unwrap_or(u32::MAX),
        );
        let name_len = u16::try_from(name.len()).unwrap_or(u16::MAX);
        // Local header: signature, version 2.0, UTF-8 flag, deflate, 1980-01-01.
        le32(&mut out, 0x0403_4b50);
        for n in [20, 0x0800, 8, 0, 0x21] {
            le16(&mut out, n);
        }
        for n in [crc, csize, size] {
            le32(&mut out, n);
        }
        le16(&mut out, name_len);
        le16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&packed);
        le32(&mut central, 0x0201_4b50);
        for n in [20, 20, 0x0800, 8, 0, 0x21] {
            le16(&mut central, n);
        }
        for n in [crc, csize, size] {
            le32(&mut central, n);
        }
        // Name length, extra, comment, disk, internal attributes.
        for n in [name_len, 0, 0, 0, 0] {
            le16(&mut central, n);
        }
        le32(&mut central, 0);
        le32(&mut central, offset);
        central.extend_from_slice(name.as_bytes());
    }
    let cd_offset = u32::try_from(out.len()).unwrap_or(u32::MAX);
    let cd_size = u32::try_from(central.len()).unwrap_or(u32::MAX);
    let count = u16::try_from(entries.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&central);
    le32(&mut out, 0x0605_4b50);
    for n in [0, 0, count, count] {
        le16(&mut out, n);
    }
    le32(&mut out, cd_size);
    le32(&mut out, cd_offset);
    le16(&mut out, 0);
    out
}
