//! # `app::dropped` — **files dragged onto the window**
//!
//! The drop nobody claimed: each PDF opens, each picture lands on the page at
//! the drop point at its natural size (later ones cascading down and to the
//! right), and anything else is named back. Alt held as it lands opens the
//! placement window for the first picture instead.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dropped.md`.

use std::path::{Path, PathBuf};

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::filedrag::Landed;
use crate::app::state::Status;

/// What a dropped file turned out to be.
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
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "tif", "tiff"];

/// How far each further dropped picture sits from the one before, in points,
/// down and to the right.
pub const CASCADE_PT: f64 = 18.0;

/// Classify one dropped path by its extension.
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

/// A drop's files by what each one is, in the order they were dropped.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sorted {
    /// PDFs, to open.
    pub documents: Vec<PathBuf>,
    /// Pictures, to place.
    pub images: Vec<PathBuf>,
    /// The extension of each file pdfcer does not take.
    pub refused: Vec<String>,
}

/// Sort `files` by [`classify`].
#[must_use]
pub fn sort(files: &[PathBuf]) -> Sorted {
    let mut out = Sorted::default();
    for file in files {
        match classify(file) {
            Dropped::Document(p) => out.documents.push(p),
            Dropped::Image(p) => out.images.push(p),
            Dropped::Unknown(ext) => out.refused.push(ext),
        }
    }
    out
}

/// The centre of the `i`th picture of a drop whose first lands at `at`.
#[must_use]
pub fn cascade(at: (f64, f64), i: usize) -> (f64, f64) {
    // ui-text-exempt: a lint reason, never displayed.
    #[allow(clippy::cast_precision_loss, reason = "a count of dropped files")]
    let step = CASCADE_PT * i as f64;
    (at.0 + step, at.1 - step)
}

/// **Act on a drop no surface claimed**: open its PDFs, place or offer its
/// pictures, and name what was refused.
pub fn land(app: &mut PdfcerApp, ctx: &egui::Context, landing: &Landed, actions: &mut Vec<Action>) {
    let first = landing.paths.first();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "dropped n={} first={:?}",
            landing.paths.len(),
            first.and_then(|p| p.file_name())
        )
    });
    let sorted = sort(&landing.paths);
    if let Some(ext) = sorted.refused.first() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("drop-refused ext={ext}")
        });
        crate::app::actions::record_note(0, crate::text::dropped::not_accepted(ext));
    }
    actions.extend(sorted.documents.into_iter().map(Action::Open));
    if sorted.images.is_empty() {
        return;
    }
    if !matches!(app.status, Status::Open(_)) {
        // There is no page to put a picture on, and the remedy is not
        // guessable from "cannot insert".
        crate::app::actions::record_note(
            0,
            crate::text::dropped::image_needs_a_document().to_owned(),
        );
        return;
    }
    if landing.alt {
        if sorted.images.len() > 1 {
            crate::app::actions::record_note(
                0,
                crate::text::dropped::alt_takes_the_first(sorted.images.len()),
            );
        }
        crate::app::dispatch::images::insert_path(&mut app.dialogs, &app.status, &sorted.images[0]);
        return;
    }
    place(app, ctx, landing.at, &sorted.images, actions);
}

/// Each picture at its natural size, the first centred on `at`.
fn place(
    app: &PdfcerApp,
    ctx: &egui::Context,
    at: Option<egui::Pos2>,
    images: &[PathBuf],
    actions: &mut Vec<Action>,
) {
    if !app.capabilities().edit_content {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "drop-declined reason=mode-cannot-place-here".to_owned()
        });
        crate::app::status::decline::record_mode_refusal(
            crate::text::clipboard::ModeRefusal::DropPicture,
        );
        return;
    }
    let Some((page, point, crop)) = crate::app::dispatch::ospaste::target_at(app, ctx, at) else {
        return;
    };
    for (i, path) in images.iter().enumerate() {
        let Some(image) = crate::app::dispatch::images::import(&app.status, path) else {
            continue;
        };
        let rect =
            pdfcer_gui_base::clippaste::rect_at(cascade(point, i), image.natural_size_pt(), crop);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "image-dropped i={i} page={page} llx={:.2} lly={:.2} urx={:.2} ury={:.2}",
                rect.llx, rect.lly, rect.urx, rect.ury
            )
        });
        actions.push(Action::InsertImage {
            page,
            rect,
            fit: pdfcer_core::edit::ImageFit::Contain,
            image: std::sync::Arc::new(image),
        });
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

    /// Case-insensitive: the property that ships broken on Windows.
    #[test]
    fn the_extension_is_matched_without_regard_to_case() {
        for name in ["PHOTO.JPG", "Scan.TIF", "DRAWING.PDF", "logo.PnG"] {
            assert!(
                !matches!(classify(Path::new(name)), Dropped::Unknown(_)),
                "{name} must be recognised"
            );
        }
    }

    #[test]
    fn an_unrecognised_file_carries_its_extension() {
        assert_eq!(
            classify(Path::new("C:/x/model.dwg")),
            Dropped::Unknown("dwg".to_owned())
        );
        assert_eq!(
            classify(Path::new("C:/x/README")),
            Dropped::Unknown(String::new())
        );
    }

    /// The drop list and the picker's filter must agree.
    #[test]
    fn the_drop_list_matches_what_the_picker_offers() {
        const PICKER: &[&str] = &["png", "jpg", "jpeg", "bmp", "tif", "tiff"];
        assert_eq!(IMAGE_EXTENSIONS, PICKER);
    }

    #[test]
    fn a_mixed_drop_keeps_every_file_in_its_order() {
        let files: Vec<PathBuf> = ["a.png", "b.pdf", "c.gif", "d.jpg", "e.pdf"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let s = sort(&files);
        assert_eq!(s.images, [PathBuf::from("a.png"), PathBuf::from("d.jpg")]);
        assert_eq!(
            s.documents,
            [PathBuf::from("b.pdf"), PathBuf::from("e.pdf")]
        );
        assert_eq!(s.refused, ["gif"]);
    }

    #[test]
    fn later_pictures_cascade_down_and_right() {
        assert_eq!(cascade((100.0, 500.0), 0), (100.0, 500.0));
        assert_eq!(
            cascade((100.0, 500.0), 2),
            (100.0 + 2.0 * CASCADE_PT, 500.0 - 2.0 * CASCADE_PT)
        );
    }
}
