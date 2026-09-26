//! # `app::dropped` — **files dragged onto the window**
//!
//! ## What this closes
//!
//! The operator: *"also can't drag and drop a jpg file onto a new pdf, and the
//! insert image button doesn't insert it either."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dropped.md`.

use std::path::{Path, PathBuf};

use crate::app::actions::Action;

/// What a dropped file turned out to be.
///
/// Named rather than answered as a `bool` pair, because the four outcomes have
/// four different sentences and a caller reading two booleans would have to
/// re-derive which combination means what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dropped {
    /// A PDF. Open it.
    Document(PathBuf),
    /// A raster image `image_import` should be able to read.
    Image(PathBuf),
    /// Something pdfcer does not take. Carries the extension, lower-cased, for
    /// the sentence — an empty string when the file had none.
    Unknown(String),
}

/// The extensions the image picker offers, which is the list this must agree
/// with.
///
/// ★ Kept in step with `app::files::pick_image_source`'s filter **by this
/// comment and a test**, not by sharing a constant, because the two lists mean
/// different things: that one is what the OS dialog shows, this one is what a
/// drop is willing to try. They happen to be equal and should stay equal, and a
/// shared constant would hide the day they legitimately diverge (a format
/// `image_import` reads but the picker does not advertise).
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "tif", "tiff"];

/// Classify one dropped path by its extension.
///
/// # ★ Why the extension and not the bytes
///
///
/// A `.pdf` that is not a PDF therefore produces the *parser's* error, which is
/// the specific one, rather than "pdfcer does not accept this kind of file",
/// which would be wrong.
#[must_use]
pub fn classify(path: &Path) -> Dropped {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if ext == "pdf" {
        Dropped::Document(path.to_path_buf())
    } else if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        Dropped::Image(path.to_path_buf())
    } else {
        Dropped::Unknown(ext)
    }
}

/// **What a drop means when no surface claimed it**: open it, insert it, or
/// explain the refusal.
///
/// `has_document` decides whether an image can be placed at all; the caller
/// knows it and this module does not need the whole `OpenDoc` to find out.
///
/// Returns the image to insert, if one was dropped and can be — the caller owns
/// the picker-and-dialog path and this module deliberately does not reach into
/// it.
///
/// ★ It is handed the files rather than reading them. See the header: the
/// position-aware half of the feature has to read the input first, and two
/// readers of one `dropped_files` would each see it and each act.
pub fn resolve(
    files: &[PathBuf],
    has_document: bool,
    actions: &mut Vec<Action>,
) -> Option<PathBuf> {
    let first = files.first().cloned()?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("dropped n={} first={:?}", files.len(), first.file_name())
    });

    // ★ Every extra file is NAMED, not silently ignored. An operator who drags
    // four drawings and gets one open has been told something false by the
    // silence — that the other three failed, or that they missed the window.
    if files.len() > 1 {
        crate::app::actions::record_note(
            0,
            crate::text::dropped::only_the_first(files.len()).to_owned(),
        );
    }

    match classify(&first) {
        Dropped::Document(path) => {
            actions.push(Action::Open(path));
            None
        }
        Dropped::Image(path) => {
            if has_document {
                Some(path)
            } else {
                // ★★ The one refusal that has to say what to DO. There is no
                // page to put a picture on, and the remedy — make or open a
                // document first — is not something the operator can guess from
                // "cannot insert".
                crate::app::actions::record_note(
                    0,
                    crate::text::dropped::image_needs_a_document().to_owned(),
                );
                None
            }
        }
        Dropped::Unknown(ext) => {
            crate::app::actions::record_note(0, crate::text::dropped::not_accepted(&ext));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pdf_opens_and_an_image_inserts() {
        assert!(matches!(
            classify(Path::new("C:/x/drawing.pdf")),
            Dropped::Document(_)
        ));
        assert!(matches!(
            classify(Path::new("C:/x/logo.jpg")),
            Dropped::Image(_)
        ));
    }

    /// ★ **Case-insensitive**, which is the property that would ship broken on
    /// Windows and be reported as "it works with some files".
    ///
    /// A camera writes `IMG_0001.JPG`; a scanner writes `.TIF`. Both are what an
    /// operator actually drags, and both would fall to `Unknown` under a
    /// case-sensitive compare — producing the message *"pdfcer does not accept
    /// .JPG files"*, which is both wrong and insulting.
    #[test]
    fn the_extension_is_matched_without_regard_to_case() {
        for name in ["PHOTO.JPG", "Scan.TIF", "DRAWING.PDF", "logo.PnG"] {
            assert!(
                !matches!(classify(Path::new(name)), Dropped::Unknown(_)),
                "{name} must be recognised"
            );
        }
    }

    /// Anything else is named rather than guessed at.
    #[test]
    fn an_unrecognised_file_carries_its_extension() {
        assert_eq!(
            classify(Path::new("C:/x/model.dwg")),
            Dropped::Unknown("dwg".to_owned())
        );
        // No extension at all is an empty string, not a panic, and the sentence
        // handles it — a file called `README` is a plausible mis-drag.
        assert_eq!(
            classify(Path::new("C:/x/README")),
            Dropped::Unknown(String::new())
        );
    }

    /// ★★ **The drop list and the picker's filter must agree.**
    ///
    /// They are two lists in two files, and this is the test that stops them
    /// drifting — the day someone adds `webp` to the file dialog and an operator
    /// discovers that the format they can *choose* is one they cannot *drop*.
    /// The module's own comment says why they are not one constant.
    #[test]
    fn the_drop_list_matches_what_the_picker_offers() {
        // The picker's filter, restated. If this assertion fails, one of the two
        // lists moved and the other did not.
        const PICKER: &[&str] = &["png", "jpg", "jpeg", "bmp", "tif", "tiff"];
        assert_eq!(IMAGE_EXTENSIONS, PICKER);
    }
}
