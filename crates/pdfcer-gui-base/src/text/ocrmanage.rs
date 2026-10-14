//! # `text::ocrmanage` — the copy for File ▸ Recognise ▸ Manage OCR models…

use std::path::Path;

use crate::ocr::addonzip::ZipError;
use crate::ocr::manage::{InstallError, RemoveError};
use crate::text::commands::CommandText;

/// A byte count in the units an operator thinks in.
#[must_use]
pub fn size(n: u64) -> String {
    super::compact::bytes(n)
}

/// `file.ocr_models` — the label and the hover.
#[must_use]
pub const fn file_ocr_models() -> CommandText {
    CommandText::new(
        "Manage OCR models…",
        "See which text recognisers are installed and how much space each takes, choose \
         the one Recognise text starts with, install one from a pdfcer add-on zip or \
         folder, download one, or remove any of them to free the space.",
    )
}

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "OCR models"
}

/// The line naming the folder installs go to and removals act on.
#[must_use]
pub fn folder_line(dir: &Path) -> String {
    format!("Models folder: {}", dir.display())
}

/// The models folder cannot be written.
#[must_use]
pub fn folder_unwritable(dir: &Path) -> String {
    format!(
        "pdfcer cannot change {}: Windows does not let this account write there, which \
         happens when pdfcer is in a protected folder such as Program Files. To install \
         or remove models, run pdfcer from a folder you can write to.",
        dir.display()
    )
}

/// The hover of a button the unwritable folder disables.
#[must_use]
pub const fn unwritable_hover() -> &'static str {
    "The models folder cannot be changed from this account; the line at the top says why."
}

/// The hover of a button disabled while a job runs.
#[must_use]
pub const fn busy_hover() -> &'static str {
    "Wait for the current download, install or removal to finish."
}

/// One installed model: its label, version and size.
#[must_use]
pub fn installed_line(label: &str, version: Option<&str>, bytes: u64) -> String {
    match version {
        Some(v) => format!("{label}, version {v} ({})", size(bytes)),
        None => format!("{label} ({})", size(bytes)),
    }
}

/// Where an installed model is.
#[must_use]
pub fn location_line(folder: &Path) -> String {
    format!("In {}", folder.display())
}

/// An installed model this build can run.
#[must_use]
pub const fn ready() -> &'static str {
    "Ready to use."
}

/// An installed model this build cannot run; `why` is the engine's sentence.
#[must_use]
pub fn cannot_run(why: &str) -> String {
    format!("Installed, but it cannot run here: {why}.")
}

/// The mark on the model Recognise text starts with.
#[must_use]
pub const fn is_default() -> &'static str {
    "Default"
}

/// The hover on the default mark.
#[must_use]
pub const fn is_default_hover() -> &'static str {
    "Recognise text starts with this model. You can choose another there for one run."
}

/// The button making a model the default.
#[must_use]
pub const fn set_default() -> &'static str {
    "Set as default"
}

/// Its hover.
#[must_use]
pub const fn set_default_hover() -> &'static str {
    "Make Recognise text start with this model."
}

/// After Set as default.
#[must_use]
pub fn default_set(label: &str) -> String {
    format!("Recognise text now starts with {label}.")
}

/// A supported recogniser with nothing installed.
#[must_use]
pub const fn not_installed() -> &'static str {
    "Not installed."
}

/// How to get a recogniser there is no download for.
#[must_use]
pub const fn install_how() -> &'static str {
    "Install it from a pdfcer OCR add-on with Install from file or Install from folder below."
}

/// A recogniser this build was made without.
#[must_use]
pub const fn not_in_build() -> &'static str {
    "This build of pdfcer cannot run it, even when installed."
}

/// The Remove button.
#[must_use]
pub const fn remove_button() -> &'static str {
    "Remove…"
}

/// Its hover.
#[must_use]
pub fn remove_hover(bytes: u64) -> String {
    format!(
        "Delete this model's folder to free {}. You can install it again later.",
        size(bytes)
    )
}

/// A model in a folder you added yourself, which is not deleted.
#[must_use]
pub const fn remove_root_hover() -> &'static str {
    "This model is a folder you added in Settings > OCR models; remove it from the list \
     there. pdfcer does not delete folders you added."
}

/// The question before one removal.
#[must_use]
pub fn confirm_remove(label: &str, folder: &Path, bytes: u64) -> String {
    format!(
        "Remove {label}? This deletes {} and frees {}.",
        folder.display(),
        size(bytes)
    )
}

/// The question before removing every model.
#[must_use]
pub fn confirm_remove_all(count: usize, bytes: u64) -> String {
    format!(
        "Remove all {count} model(s)? This deletes their folders and frees {}. Recognise \
         text will have nothing to use until you install one again.",
        size(bytes)
    )
}

/// The confirming button.
#[must_use]
pub const fn confirm_button() -> &'static str {
    "Remove"
}

/// The declining button.
#[must_use]
pub const fn keep_button() -> &'static str {
    "Keep"
}

/// The line under the list: how many and how much.
#[must_use]
pub fn total_line(count: usize, bytes: u64) -> String {
    format!("{count} model(s) installed, {} in total.", size(bytes))
}

/// Nothing is installed.
#[must_use]
pub const fn none_installed() -> &'static str {
    "No OCR model is installed, so Recognise text cannot read words from images. \
     Install or download one here when you need it."
}

/// The Remove all button.
#[must_use]
pub const fn remove_all_button() -> &'static str {
    "Remove all…"
}

/// Its hover.
#[must_use]
pub const fn remove_all_hover() -> &'static str {
    "Delete every installed model's folder to free the space. Folders you added in \
     Settings are left alone."
}

/// The install-from-file button.
#[must_use]
pub const fn install_file_button() -> &'static str {
    "Install from file…"
}

/// Its hover.
#[must_use]
pub const fn install_file_hover() -> &'static str {
    "Install a pdfcer OCR add-on zip. Its files are checked against the add-on's \
     fingerprints before it is put in the models folder."
}

/// The install-from-folder button.
#[must_use]
pub const fn install_folder_button() -> &'static str {
    "Install from folder…"
}

/// Its hover.
#[must_use]
pub const fn install_folder_hover() -> &'static str {
    "Install an unpacked pdfcer OCR add-on folder, such as a PaddleOCR-VL add-on. It is \
     copied, checked against its fingerprints, then put in the models folder."
}

/// The file picker's title.
#[must_use]
pub const fn pick_zip_title() -> &'static str {
    "Choose a pdfcer OCR add-on zip"
}

/// The folder picker's title.
#[must_use]
pub const fn pick_folder_title() -> &'static str {
    "Choose a pdfcer OCR add-on folder"
}

/// The file picker's filter name.
#[must_use]
pub const fn zip_filter() -> &'static str {
    "pdfcer OCR add-on (*.zip)"
}

/// While a removal or install runs.
#[must_use]
pub const fn working() -> &'static str {
    "Working…"
}

/// After a removal; `then` is what Recognise text starts with now, if that
/// changed.
#[must_use]
pub fn removed(label: &str, bytes: u64, then: Option<&str>) -> String {
    let freed = format!("Removed {label} and freed {}.", size(bytes));
    then.map_or_else(|| freed.clone(), |t| format!("{freed} {t}"))
}

/// After Remove all; `then` as for [`removed`].
#[must_use]
pub fn removed_all(count: usize, bytes: u64, then: Option<&str>) -> String {
    let freed = format!("Removed {count} model(s) and freed {}.", size(bytes));
    then.map_or_else(|| freed.clone(), |t| format!("{freed} {t}"))
}

/// What was removed, then what was refused, as one paragraph.
#[must_use]
pub fn both(done: Option<String>, failed: Option<String>) -> String {
    match (done, failed) {
        (Some(d), Some(f)) => format!("{d} {f}"),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => String::new(),
    }
}

/// The default was removed and another model takes its place.
#[must_use]
pub fn new_default(label: &str) -> String {
    format!("Recognise text now starts with {label}.")
}

/// The default was removed and nothing runnable is left.
#[must_use]
pub const fn no_default_left() -> &'static str {
    "No OCR model that can run is left, so Recognise text has nothing to use until you \
     install one here."
}

/// Why a removal did not happen.
#[must_use]
pub fn remove_failed(label: &str, error: &RemoveError, writable: bool) -> String {
    match error {
        RemoveError::IsRoot(_) => format!("{label} was not removed. {}", remove_root_hover()),
        RemoveError::Gone(dir) => format!(
            "{label} was not removed: it is no longer in {}. The list has been refreshed.",
            dir.display()
        ),
        RemoveError::Changed(dir) => format!(
            "{label} was not removed: {} changed since the list was made. The list has \
             been refreshed; check it and try again.",
            dir.display()
        ),
        RemoveError::Refused(dir, e) if writable => format!(
            "{label} was not removed, and nothing was deleted: Windows refused to change {} \
             ({e}). A file in it may be in use; close any recognition that is running and \
             try again.",
            dir.display()
        ),
        RemoveError::Refused(dir, _) => format!(
            "{label} was not removed, and nothing was deleted. {}",
            folder_unwritable(dir.parent().unwrap_or(dir))
        ),
        RemoveError::Partial(left, e) => format!(
            "{label} is no longer offered, but deleting it stopped part-way ({e}). What is \
             left is in {}; you can delete that folder yourself.",
            left.display()
        ),
    }
}

/// After an install; `checked` is how many files the engine verified.
#[must_use]
pub fn installed(label: &str, bytes: u64, checked: usize) -> String {
    let check = if checked == 0 {
        "It lists no fingerprints, so its files could not be checked.".to_owned()
    } else {
        format!("All {checked} of its file(s) matched the add-on's fingerprints.")
    };
    format!("Installed {label} ({}). {check}", size(bytes))
}

/// Why an install did not happen.
#[must_use]
pub fn install_failed(error: &InstallError) -> String {
    let why = match error {
        InstallError::Zip(z) => zip_failed(z),
        InstallError::Io(path, e) => format!("{}: {e}", path.display()),
        InstallError::NoModel(notes) if notes.is_empty() => {
            "it holds no pdfcer OCR model (no pdfcer-ocr-model.txt and no ocrs, ocrcer or \
             paddle folder)"
                .to_owned()
        }
        InstallError::NoModel(notes) => format!("it holds no usable model: {}", notes.join("; ")),
        InstallError::Several(names) => format!(
            "it holds {} models ({}); install them one at a time",
            names.len(),
            names.join(", ")
        ),
        InstallError::Verify(why) => format!("a file does not match its fingerprint: {why}"),
        InstallError::Exists(name, folder) => format!(
            "a model named {name} is already installed in {}; remove it first to replace it",
            folder.display()
        ),
    };
    format!("Nothing was installed: {why}.")
}

fn zip_failed(error: &ZipError) -> String {
    match error {
        ZipError::Io(path, e) => format!("{}: {e}", path.display()),
        ZipError::NotZip => "the file is not a zip archive, or it is cut short".to_owned(),
        ZipError::Zip64 => "the archive uses the ZIP64 format, which pdfcer cannot unpack; \
                            unpack it yourself and use Install from folder"
            .to_owned(),
        ZipError::TooManyEntries => "the archive holds more files than any add-on does".to_owned(),
        ZipError::Encrypted(name) => format!("{name} in the archive is encrypted"),
        ZipError::Method(name, m) => {
            format!(
                "{name} in the archive is compressed with method {m}, which pdfcer cannot unpack"
            )
        }
        ZipError::UnsafePath(name) => {
            format!("{name} in the archive would be written outside the add-on's folder")
        }
        ZipError::Corrupt(name) => format!("{name} in the archive is damaged or repeated"),
    }
}

/// Recognise text with no model installed at all.
#[must_use]
pub const fn no_engine_installed() -> &'static str {
    "No OCR engine is installed, so pdfcer cannot read words from images. Open Manage OCR \
     models to install or download one."
}

/// The button that opens the manager from Recognise text.
#[must_use]
pub const fn open_manager_button() -> &'static str {
    "Manage OCR models…"
}

/// Its hover.
#[must_use]
pub const fn open_manager_hover() -> &'static str {
    "Open the window that installs, downloads and removes OCR models."
}

/// The close button.
#[must_use]
pub const fn close_button() -> &'static str {
    "Close"
}
