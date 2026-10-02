//! # `editmodel::installedfaces` — the operator's font folders as the source
//! an embedded subset is extended from
//!
//! A typed key an embedded TrueType subset has no outline for is appended to
//! the subset from the installed face it was cut from, when one of the font
//! folders holds a face whose PostScript name is the subset's
//! (`EditOptions::with_subset_augment`; the engine proves the face is the same
//! font before it appends anything).
//!
//! Contract: [`for_folders`] answers one `'static` source per distinct folder
//! list and indexes the folders' face names on a background thread. Until the
//! index is built every request refuses and `addable` is empty, so the
//! keystroke query can only ever under-promise against the commit. A file's
//! bytes are read again only when a subset of its name asks.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/installedfaces.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use pdfcer_core::text_edit::{
    AugmentRefusal, AugmentRequest, AugmentedProgram, EditOptions, SubsetAugment, SubsetAugmenter,
};
use pdfcer_render::font::InstalledFaceAugmenter;
use pdfcer_render::{FontData, FontEnvironment};

/// The faces of one folder list.
#[derive(Debug)]
pub struct InstalledFaces {
    folders: Vec<PathBuf>,
    /// Face name → the files advertising it, in folder then file order.
    index: OnceLock<BTreeMap<String, Vec<PathBuf>>>,
    /// The faces read so far, by subset name without its tag.
    loaded: Mutex<BTreeMap<String, Arc<InstalledFaceAugmenter>>>,
}

/// Every source made so far; one per distinct folder list, never freed.
static SOURCES: Mutex<Vec<&'static InstalledFaces>> = Mutex::new(Vec::new());

/// The source for `folders`, indexing it in the background the first time.
#[must_use]
pub fn for_folders(folders: &[PathBuf]) -> &'static InstalledFaces {
    let mut held = SOURCES.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(source) = held.iter().find(|s| s.folders == folders) {
        return source;
    }
    let source: &'static InstalledFaces = Box::leak(Box::new(InstalledFaces {
        folders: folders.to_vec(),
        index: OnceLock::new(),
        loaded: Mutex::new(BTreeMap::new()),
    }));
    held.push(source);
    let spawned = std::thread::Builder::new()
        .name("installed-faces".to_owned())
        .spawn(move || source.build_index());
    if spawned.is_err() {
        source.build_index();
    }
    source
}

/// `options` with `faces` as the subset augmenter, at the engine's safe
/// defaults (every shared glyph compared; hinting stripped when it differs).
#[must_use]
pub fn augmenting(options: EditOptions, faces: &'static InstalledFaces) -> EditOptions {
    options.with_subset_augment(SubsetAugment::new(faces))
}

impl InstalledFaces {
    /// Whether the folders have been indexed.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.index.get().is_some()
    }

    fn build_index(&self) {
        let started = std::time::Instant::now();
        let mut index: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        let mut files = 0usize;
        for path in self.folders.iter().flat_map(|f| font_files(f)) {
            let Some(names) = face_names(&path) else {
                continue;
            };
            files += 1;
            for name in names {
                index.entry(name).or_default().push(path.clone());
            }
        }
        let names = index.len();
        if self.index.set(index).is_ok() {
            crate::diag::trace(|| {
                format!(
                    "installed-faces-indexed folders={} files={files} names={names} ms={}", // ui-text-exempt: diagnostic trace
                    self.folders.len(),
                    started.elapsed().as_millis()
                )
            });
        }
    }

    /// The faces named like `base_font`, read on first ask; `None` while the
    /// index is being built.
    fn faces_for(&self, base_font: &str) -> Option<Arc<InstalledFaceAugmenter>> {
        let index = self.index.get()?;
        let stem = FontEnvironment::subset_stem(base_font);
        let mut loaded = self.loaded.lock().unwrap_or_else(PoisonError::into_inner);
        let faces = loaded.entry(stem.to_owned()).or_insert_with(|| {
            let mut faces = InstalledFaceAugmenter::new();
            for path in index.get(stem).into_iter().flatten() {
                if let Ok(bytes) = std::fs::read(path) {
                    let label = path.file_name().unwrap_or(path.as_os_str());
                    faces.insert(&label.to_string_lossy(), FontData::new(bytes));
                }
            }
            Arc::new(faces)
        });
        Some(Arc::clone(faces))
    }
}

impl SubsetAugmenter for InstalledFaces {
    fn augment(&self, request: &AugmentRequest<'_>) -> Result<AugmentedProgram, AugmentRefusal> {
        match self.faces_for(request.base_font) {
            Some(faces) => faces.augment(request),
            None => Err(AugmentRefusal {
                reason: crate::text::fonts::folders_still_indexing(),
            }),
        }
    }

    fn addable(&self, request: &AugmentRequest<'_>) -> Vec<char> {
        self.faces_for(request.base_font)
            .map_or_else(Vec::new, |faces| faces.addable(request))
    }
}

/// The font files directly in `folder`, sorted; none when it cannot be read.
fn font_files(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && crate::fontlibrary::has_font_extension(p))
        .collect();
    files.sort();
    files
}

/// The names `path` advertises, or `None` when it is too large, unreadable
/// or not a font.
fn face_names(path: &Path) -> Option<Vec<String>> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > crate::fontlibrary::MAX_FONT_FILE_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let program = pdfcer_render::font::program::FontProgram::parse(&bytes).ok()?;
    Some(program.face_names())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_folder_list_is_one_source() {
        let folders = vec![PathBuf::from("no-such-folder-for-installedfaces")];
        let a = for_folders(&folders);
        let b = for_folders(&folders);
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn an_unreadable_folder_indexes_to_nothing() {
        let folders = vec![PathBuf::from("no-such-folder-for-installedfaces-2")];
        let source = for_folders(&folders);
        while !source.is_ready() {
            std::thread::yield_now();
        }
        assert!(
            source
                .faces_for("ABCDEF+Arial-BoldMT")
                .is_some_and(|f| f.is_empty())
        );
    }
}
