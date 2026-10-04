//! # `editmodel::replacementfaces` — the operator's font folders offered to
//! the engine's replacement-face ladder
//!
//! Contract: [`for_folders`] answers one `'static` source per distinct folder
//! list. It keeps no font bytes: [`ReplacementFaces::candidates`] reads every
//! font file in the folders once per call, and [`ReplacementFaces::plan`] reads
//! the picked file again. The ladder runs only when the operator presses a
//! workaround offer, so a read of the folders is paid per press, never held
//! (engine request G111 asks for a provider that makes this module redundant).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use pdfcer_core::font_embed::FontEmbedPlan;
use pdfcer_core::text_edit::{EditOptions, FaceCandidate, ReplacementFaces};
use pdfcer_render::FontData;
use pdfcer_render::font::InstalledFaces;

/// The faces of one folder list, read on each ladder call.
#[derive(Debug)]
pub struct FolderFaces {
    folders: Vec<PathBuf>,
    /// The last `candidates` call's ids: index → (file, id within the file).
    offered: Mutex<Vec<(PathBuf, usize)>>,
}

static SOURCES: Mutex<Vec<&'static FolderFaces>> = Mutex::new(Vec::new());

/// The source for `folders`; one per distinct list, never freed.
#[must_use]
pub fn for_folders(folders: &[PathBuf]) -> &'static FolderFaces {
    let mut held = SOURCES.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(source) = held.iter().find(|s| s.folders == folders) {
        return source;
    }
    let source: &'static FolderFaces = Box::leak(Box::new(FolderFaces {
        folders: folders.to_vec(),
        offered: Mutex::new(Vec::new()),
    }));
    held.push(source);
    source
}

/// `options` with `faces` as the replacement-face ladder's source.
#[must_use]
pub fn laddered(options: EditOptions, faces: &'static FolderFaces) -> EditOptions {
    options.with_replacement_faces(faces)
}

/// `path` as a one-file provider, or `None` when it cannot be read or is too
/// large to be a font this shell reads.
fn one_file(path: &Path) -> Option<InstalledFaces> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > crate::fontlibrary::MAX_FONT_FILE_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let mut faces = InstalledFaces::new();
    faces.insert(&path.display().to_string(), FontData::new(bytes));
    (!faces.is_empty()).then_some(faces)
}

impl ReplacementFaces for FolderFaces {
    fn candidates(&self, chars: &[char]) -> Vec<FaceCandidate> {
        let started = std::time::Instant::now();
        let mut offered = Vec::new();
        let mut out = Vec::new();
        for path in self
            .folders
            .iter()
            .flat_map(|f| crate::editmodel::installedfaces::font_files(f))
        {
            let Some(faces) = one_file(&path) else {
                continue;
            };
            for mut candidate in faces.candidates(chars) {
                offered.push((path.clone(), candidate.id));
                candidate.id = offered.len() - 1;
                out.push(candidate);
            }
        }
        crate::diag::trace(|| {
            format!(
                "replacement-faces folders={} faces={} ms={}", // ui-text-exempt: diagnostic trace
                self.folders.len(),
                out.len(),
                started.elapsed().as_millis()
            )
        });
        *self.offered.lock().unwrap_or_else(PoisonError::into_inner) = offered;
        out
    }

    fn plan(&self, candidate: &FaceCandidate, chars: &[char]) -> Result<FontEmbedPlan, String> {
        let (path, local) = self
            .offered
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(candidate.id)
            .cloned()
            .ok_or_else(crate::text::fonts::face_no_longer_offered)?;
        let faces = one_file(&path).ok_or_else(crate::text::fonts::face_unreadable)?;
        let mut own = candidate.clone();
        own.id = local;
        faces.plan(&own, chars)
    }
}
