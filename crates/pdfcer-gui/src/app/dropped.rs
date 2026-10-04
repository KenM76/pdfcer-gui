//! # `app::dropped` — **files dragged onto the window**
//!
//! The drop nobody claimed: each PDF opens (one dropped alone on an open
//! document asks whether to open, insert or place it), each picture lands on the page at
//! the drop point at its natural size (later ones cascading down and to the
//! right), each text file becomes pages after the one on screen, and anything
//! else is named back. Alt held as it lands opens the
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
    /// A plain-text file. Set it as new pages.
    Text(PathBuf),
    /// Something pdfcer does not take. Carries the extension, lower-cased, for
    /// the sentence — an empty string when the file had none.
    Unknown(String),
}

/// The extensions the image picker offers, read from the one list.
const IMAGE_EXTENSIONS: &[&str] = pdfcer_gui_base::picture::EXTENSIONS;

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
    } else if ext == "txt" {
        Dropped::Text(path.to_path_buf())
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
    /// Text files, to set as pages.
    pub texts: Vec<PathBuf>,
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
            Dropped::Text(p) => out.texts.push(p),
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
    documents(app, ctx, landing.at, &sorted, actions);
    pages(app, &sorted.texts, actions);
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

/// Each PDF opens, except one dropped alone on an open document, which asks
/// what it is for when the mode offers more than opening it.
fn documents(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    at: Option<egui::Pos2>,
    sorted: &Sorted,
    actions: &mut Vec<Action>,
) {
    if let [only] = sorted.documents.as_slice()
        && sorted.images.is_empty()
        && sorted.texts.is_empty()
        && let Status::Open(doc) = &app.status
    {
        let caps = app.capabilities();
        let offer = crate::dialogs::drop_pdf::Offer {
            insert_after: caps.edit_content.then_some(doc.view.page_index),
            place_at: caps
                .author_markup
                .then(|| crate::app::dispatch::ospaste::target_at(app, ctx, at))
                .flatten(),
        };
        if let Some(dialog) = crate::dialogs::drop_pdf::DropPdfDialog::read(only.clone(), offer) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "drop-pdf-asked insert={} place={}",
                    offer.insert_after.is_some(),
                    offer.place_at.is_some()
                )
            });
            app.dialogs.open_drop_pdf(dialog);
            return;
        }
    }
    actions.extend(sorted.documents.iter().cloned().map(Action::Open));
}

/// Each text file as pages after the one on screen, in the order dropped,
/// with Import text as pages' defaults.
fn pages(app: &PdfcerApp, texts: &[PathBuf], actions: &mut Vec<Action>) {
    if texts.is_empty() {
        return;
    }
    let Status::Open(doc) = &app.status else {
        crate::app::actions::record_note(
            0,
            crate::text::dropped::text_needs_a_document().to_owned(),
        );
        return;
    };
    let current = doc.view.page_index;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-dropped n={} page={current}", texts.len())
    });
    // Each lands directly after `current`, so pushing them last-first leaves
    // the first dropped first.
    actions.extend(
        texts.iter().rev().map(|path| {
            crate::dialogs::import_text::ImportTextDialog::dropped(path.clone(), current)
        }),
    );
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
        let Some(picture) = crate::app::dispatch::images::import(&app.status, path) else {
            continue;
        };
        let rect =
            pdfcer_gui_base::clippaste::rect_at(cascade(point, i), picture.natural_size_pt(), crop);
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
            image: std::sync::Arc::new(picture),
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

    #[test]
    fn a_drawing_drops_as_a_picture() {
        let names: &[&str] = if cfg!(feature = "svg-import") {
            &["logo.svg", "PLAN.EMF"]
        } else {
            &["PLAN.EMF"]
        };
        for &name in names {
            assert!(
                matches!(classify(Path::new(name)), Dropped::Image(_)),
                "{name}"
            );
        }
    }

    #[test]
    fn a_mixed_drop_keeps_every_file_in_its_order() {
        let files: Vec<PathBuf> = ["a.png", "b.pdf", "c.dwg", "d.jpg", "e.pdf", "f.TXT"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let s = sort(&files);
        assert_eq!(s.images, [PathBuf::from("a.png"), PathBuf::from("d.jpg")]);
        assert_eq!(
            s.documents,
            [PathBuf::from("b.pdf"), PathBuf::from("e.pdf")]
        );
        assert_eq!(s.refused, ["dwg"]);
        assert_eq!(s.texts, [PathBuf::from("f.TXT")]);
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
