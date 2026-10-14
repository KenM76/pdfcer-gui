//! Installing and removing OCR models in a models folder.
//!
//! Contract:
//! - [`remove`] deletes only a folder that discovery finds again, unchanged,
//!   under the root it was found in; never the root itself. The folder is
//!   first renamed aside, so a refusal from Windows (a protected folder, a
//!   file in use) deletes nothing.
//! - [`install`] unpacks or copies into a staging folder beside `models`,
//!   requires discovery to find exactly one model there, has the engine
//!   verify its `sha256` lines, and only then renames it into `models`.
//!   Staging is deleted whatever happens.
//! - Every change bumps [`generation`], which an open Recognise text dialog
//!   watches to discover again.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/manage.md`.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use pdfcer_core::ocr::addons::{DiscoveryNote, MANIFEST_FILE, OcrModel, discover_ocr_models};

use super::addonzip::{self, ZipError};

/// Every recogniser the model manager lists, in its order: the engine tokens
/// of `pdfcer_core::ocr::addons`.
// ui-text-exempt: engine tokens matched literally, never displayed.
pub const SUPPORTED: [&str; 5] = [
    "paddle",
    "ocrs",
    "ocrcer",
    super::catalog::PADDLE_VL,
    "tesseract",
];

/// Whether this build can run models of `token` at all. A `tesseract` model
/// carries its own program, so the build always can.
#[must_use]
pub fn in_build(token: &str) -> bool {
    super::EngineId::from_key(token).map_or_else(
        || token != super::catalog::PADDLE_VL || cfg!(feature = "ocr-vl"),
        super::EngineId::compiled_in,
    )
}

static GENERATION: AtomicU64 = AtomicU64::new(0);

/// How many times a models folder has been changed by this program.
#[must_use]
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

fn changed() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// The total size of every file under `dir`, following no links.
#[must_use]
pub fn folder_bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => folder_bytes(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map_or(0, |m| m.len()),
            _ => 0,
        })
        .sum()
}

/// Whether this process may create and delete files in `dir`, or, when it
/// does not exist, in its nearest existing ancestor.
#[must_use]
pub fn writable(dir: &Path) -> bool {
    let Some(existing) = dir.ancestors().find(|d| d.is_dir()) else {
        return false;
    };
    let probe = existing.join(format!(".pdfcer-write-probe-{}", std::process::id()));
    match std::fs::File::create_new(&probe) {
        Ok(_) => std::fs::remove_file(&probe).is_ok(),
        Err(_) => false,
    }
}

/// Why a removal did not happen, or did not finish.
#[derive(Debug)]
pub enum RemoveError {
    /// The model folder is a search root the operator added; it is not
    /// deleted.
    IsRoot(PathBuf),
    /// Discovery no longer finds this model in that folder.
    Gone(PathBuf),
    /// The folder now holds a different model or manifest.
    Changed(PathBuf),
    /// Windows refused to move the folder; nothing was deleted.
    Refused(PathBuf, io::Error),
    /// The folder was moved aside and is no longer offered, but deleting it
    /// failed part-way; what is left is at the path.
    Partial(PathBuf, io::Error),
}

/// Delete `model`'s folder after discovering it again. Returns the bytes
/// freed.
///
/// # Errors
///
/// [`RemoveError`]; nothing is deleted except under `Partial`.
pub fn remove(model: &OcrModel) -> Result<u64, RemoveError> {
    let folder = &model.folder;
    if folder == &model.root || folder.parent().is_none() {
        return Err(RemoveError::IsRoot(folder.clone()));
    }
    let again = discover_ocr_models(std::slice::from_ref(&model.root));
    let Some(found) = again.models.iter().find(|m| &m.folder == folder) else {
        return Err(RemoveError::Gone(folder.clone()));
    };
    if found != model {
        return Err(RemoveError::Changed(folder.clone()));
    }
    let bytes = folder_bytes(folder);
    let parent = folder.parent().unwrap_or(folder);
    let aside = parent.join(format!(".pdfcer-removing-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&aside);
    std::fs::rename(folder, &aside).map_err(|e| RemoveError::Refused(folder.clone(), e))?;
    changed();
    // The manifest goes first, so a folder left part-deleted is never
    // discovered as a model.
    let _ = std::fs::remove_file(aside.join(MANIFEST_FILE));
    std::fs::remove_dir_all(&aside).map_err(|e| RemoveError::Partial(aside.clone(), e))?;
    Ok(bytes)
}

/// Where an add-on comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A pdfcer OCR add-on zip.
    Zip(PathBuf),
    /// An add-on folder, as an add-on builder leaves it.
    Folder(PathBuf),
}

/// What [`install`] put in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The model as discovery now finds it in `models`.
    pub model: OcrModel,
    /// Files the engine checked against the manifest's `sha256` lines; 0 when
    /// it lists none.
    pub checked: usize,
    /// Its size on disk.
    pub bytes: u64,
}

/// Why an install did not happen. Nothing is left in `models` by any of
/// these.
#[derive(Debug)]
pub enum InstallError {
    /// The zip was refused.
    Zip(ZipError),
    /// Copying, creating or moving failed at this path.
    Io(PathBuf, io::Error),
    /// No model in it; the engine's notes on what it skipped.
    NoModel(Vec<String>),
    /// More than one model in it; their names.
    Several(Vec<String>),
    /// A file failed the engine's check: its sentence.
    Verify(String),
    /// A model of this name is already installed, at this folder.
    Exists(String, PathBuf),
}

/// Install `source` into `models` (created if absent).
///
/// # Errors
///
/// [`InstallError`].
pub fn install(source: &Source, models: &Path) -> Result<Installed, InstallError> {
    std::fs::create_dir_all(models).map_err(|e| InstallError::Io(models.to_path_buf(), e))?;
    let staging = staging_for(models);
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| InstallError::Io(staging.clone(), e))?;
    let result = stage_and_move(source, &staging, models);
    let _ = std::fs::remove_dir_all(&staging);
    if result.is_ok() {
        changed();
    }
    result
}

/// The staging folder: beside `models`, never inside it, so discovery never
/// finds a half-unpacked add-on.
fn staging_for(models: &Path) -> PathBuf {
    let name = format!(".pdfcer-installing-{}", std::process::id());
    models
        .parent()
        .map_or_else(|| models.with_extension("installing"), |p| p.join(name))
}

fn stage_and_move(
    source: &Source,
    staging: &Path,
    models: &Path,
) -> Result<Installed, InstallError> {
    match source {
        Source::Zip(zip) => {
            addonzip::unpack(zip, staging).map_err(InstallError::Zip)?;
        }
        Source::Folder(dir) => {
            let name = dir
                .file_name()
                .map_or_else(|| "addon".into(), ToOwned::to_owned);
            copy_tree(dir, &staging.join(name))?;
        }
    }
    let found = discover_ocr_models(&[staging.to_path_buf()]);
    let model = match found.models.as_slice() {
        [one] => one.clone(),
        [] => {
            let notes = found
                .notes
                .iter()
                .filter(|n| !matches!(n, DiscoveryNote::RootMissing(_)))
                .map(ToString::to_string)
                .collect();
            return Err(InstallError::NoModel(notes));
        }
        many => {
            return Err(InstallError::Several(
                many.iter().map(|m| m.name.clone()).collect(),
            ));
        }
    };
    let checked = model
        .verify()
        .map_err(|e| InstallError::Verify(e.to_string()))?;
    let folder_name = if model.folder == staging {
        model.name.clone()
    } else {
        model
            .folder
            .file_name()
            .map_or_else(|| model.name.clone(), |n| n.to_string_lossy().into_owned())
    };
    let dest = models.join(&folder_name);
    let existing = discover_ocr_models(&[models.to_path_buf()]);
    if let Some(m) = existing.by_name(&model.name) {
        return Err(InstallError::Exists(model.name.clone(), m.folder.clone()));
    }
    if dest.exists() {
        return Err(InstallError::Exists(model.name.clone(), dest));
    }
    std::fs::rename(&model.folder, &dest).map_err(|e| InstallError::Io(dest.clone(), e))?;
    let now = discover_ocr_models(&[models.to_path_buf()]);
    let model = now
        .models
        .into_iter()
        .find(|m| m.folder == dest)
        .ok_or_else(|| InstallError::NoModel(Vec::new()))?;
    Ok(Installed {
        bytes: folder_bytes(&dest),
        model,
        checked,
    })
}

fn io_at(p: &Path) -> impl FnOnce(io::Error) -> InstallError + '_ {
    move |e| InstallError::Io(p.to_path_buf(), e)
}

/// Copy `from` to `to`, files and folders only; links are not followed.
fn copy_tree(from: &Path, to: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(to).map_err(io_at(to))?;
    for entry in std::fs::read_dir(from).map_err(io_at(from))? {
        let entry = entry.map_err(io_at(from))?;
        let path = entry.path();
        let kind = entry.file_type().map_err(io_at(&path))?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&path, &target)?;
        } else if kind.is_file() {
            std::fs::copy(&path, &target).map_err(io_at(&path))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "manage_tests.rs"]
mod tests;
