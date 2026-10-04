//! # `editmodel::replacementfaces` — the operator's font folders offered to
//! the engine's replacement-face ladder
//!
//! Contract: [`for_folders`] answers one `'static` source per distinct folder
//! list. It holds a `pdfcer_render::font::FaceCatalog` (face descriptions and
//! coverage, never font bytes), built on the first ladder call and rebuilt
//! when a folder's listing (name, size, modified time) changes. `plan` reads
//! only the picked file, through the catalogue's loader.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/replacementfaces.md`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use pdfcer_core::font_embed::FontEmbedPlan;
use pdfcer_core::text_edit::{EditOptions, FaceCandidate, ReplacementFaces};
use pdfcer_render::font::FaceCatalog;

/// One file as listed: what a rebuild is keyed on.
type Listed = (PathBuf, u64, Option<SystemTime>);

/// A catalogue and the listing it was built from.
struct Built {
    listing: Vec<Listed>,
    catalog: FaceCatalog,
}

/// The faces of one folder list.
pub struct FolderFaces {
    folders: Vec<PathBuf>,
    /// The catalogue the last `candidates` call answered from. Only
    /// `candidates` rebuilds it, so the ladder's following `plan` reads the
    /// catalogue its candidate's id was numbered in.
    built: Mutex<Option<Arc<Built>>>,
}

impl std::fmt::Debug for FolderFaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FolderFaces")
            .field("folders", &self.folders)
            .finish_non_exhaustive()
    }
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
        built: Mutex::new(None),
    }));
    held.push(source);
    source
}

/// `options` with `faces` as the replacement-face ladder's source.
#[must_use]
pub fn laddered(options: EditOptions, faces: &'static FolderFaces) -> EditOptions {
    options.with_replacement_faces(faces)
}

/// The bytes of `path`, or `None` when it cannot be read or is too large to
/// be a font this shell reads.
fn read_font(path: &Path) -> Option<Vec<u8>> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > crate::fontlibrary::MAX_FONT_FILE_BYTES {
        return None;
    }
    std::fs::read(path).ok()
}

impl FolderFaces {
    fn listing(&self) -> Vec<Listed> {
        self.folders
            .iter()
            .flat_map(|f| crate::editmodel::installedfaces::font_files(f))
            .map(|path| {
                let meta = std::fs::metadata(&path).ok();
                let size = meta.as_ref().map_or(0, std::fs::Metadata::len);
                let modified = meta.and_then(|m| m.modified().ok());
                (path, size, modified)
            })
            .collect()
    }

    /// The catalogue for the folders as they are now.
    fn current(&self) -> Arc<Built> {
        let listing = self.listing();
        let mut held = self.built.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(built) = held.as_ref().filter(|b| b.listing == listing) {
            return Arc::clone(built);
        }
        let mut catalog = FaceCatalog::new(|label: &str| {
            read_font(Path::new(label)).ok_or_else(crate::text::fonts::face_unreadable)
        });
        for (path, _, _) in &listing {
            if let Some(bytes) = read_font(path) {
                catalog.add(&path.display().to_string(), &bytes);
            }
        }
        let built = Arc::new(Built { listing, catalog });
        *held = Some(Arc::clone(&built));
        built
    }
}

impl ReplacementFaces for FolderFaces {
    fn candidates(&self, chars: &[char]) -> Vec<FaceCandidate> {
        let started = std::time::Instant::now();
        let out = self.current().catalog.candidates(chars);
        crate::diag::trace(|| {
            format!(
                "replacement-faces folders={} faces={} ms={}", // ui-text-exempt: diagnostic trace
                self.folders.len(),
                out.len(),
                started.elapsed().as_millis()
            )
        });
        out
    }

    fn plan(&self, candidate: &FaceCandidate, chars: &[char]) -> Result<FontEmbedPlan, String> {
        let built = self
            .built
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .ok_or_else(crate::text::fonts::face_no_longer_offered)?;
        built.catalog.plan(candidate, chars)
    }
}
