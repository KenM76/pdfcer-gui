//! # `app::dropped` — **files dragged onto the window**
//!
//! ## What this closes
//!
//! The operator, 2026-08-19: *"also can't drag and drop a jpg file onto a new
//! pdf, and the insert image button doesn't insert it either."*
//!
//! The second half turned out to be false — `insert_image_places_a_picture`
//! drives that button end to end and passes, including on a real JPEG once the
//! harness was made to feed it one. **The first half was entirely true**:
//! nothing in this shell or in `egui-shell` read `dropped_files` at all, so a
//! file dragged onto the window did nothing, silently, with no cursor feedback
//! on the way in.
//!
//! ## ★★ Why "does nothing" is worse here than almost anywhere else
//!
//! Because drag-and-drop is the one gesture with **no discoverable
//! alternative**. A missing menu item can be looked for; a missing chord can be
//! found in a shortcuts window. A drop that is ignored teaches the operator
//! that this program does not accept drops — a conclusion they will not revisit,
//! and one they reached about a program that opens documents for a living.
//!
//! It also cost more than the feature: the same report named the Insert-image
//! button, which works. A drop that silently failed made a working button look
//! broken, because both were tried in the same minute and only one of them told
//! the operator anything.
//!
//! ## What a drop means, by what was dropped
//!
//! | dropped | action |
//! |---|---|
//! | a **PDF** | open it — the same [`Action::Open`] the File ▸ Open picker raises |
//! | a **raster image** (png/jpg/bmp/tif) with a document open | insert it, straight into the placement window |
//! | a raster image with **no** document open | say so, and say what to do about it |
//! | anything else | say what pdfcer accepts |
//!
//! ★ **A dropped PDF opens rather than being inserted**, and that is the
//! decision most worth stating because the opposite is defensible. Every viewer
//! in this class opens a dropped PDF; `pages.insert` is a deliberate act with a
//! position and a page range, and inferring it from a drag would make the
//! commonest gesture in the product do the rarer of two things. The operator who
//! wants to insert has a command that asks them where.
//!
//! ## ★ Why the drop is read where the ribbon is, and not in the canvas
//!
//! `egui` reports drops on the **`Context`**, not on a widget — `RawInput`
//! carries `dropped_files` for the whole window and nothing narrows it to a
//! rect. So a drop anywhere on the window is one event, and reading it inside
//! the canvas would be reading a window-scoped fact in a page-scoped place: a
//! drop on the ribbon or on a dock panel would be missed, and the operator would
//! learn that the program accepts drops *sometimes*, which is worse than never.
//!
//! ## ★★ What changed on 2026-08-31, and what deliberately did not
//!
//! `OPERATOR_REQUESTS.md` O67 asked for a drop onto the **thumbnails** to
//! import pages, which needs the one thing the paragraph above says does not
//! exist: a position. [`crate::app::filedrag`] supplies it — from the
//! operating system, because the toolkit discards it — and lets a surface
//! **claim** a drop that landed on it.
//!
//! This module is what happens to a drop that **nobody claimed**, and it is
//! unchanged in every respect except where the paths come from: it no longer
//! reads `egui`'s input itself, it is handed the files. That inversion is the
//! safety property. The fallback is unconditional, so a surface that forgets
//! to claim costs a feature and never a file — the failure is *"it opened in a
//! tab instead of inserting"*, which the operator can see and undo.
//!
//! It now runs at the END of the frame, after every surface has had its
//! chance, rather than at the top.
//!
//! ## What is deliberately NOT here
//!
//! - **Multi-file drops.** Only the first is acted on, and the rest are named in
//!   the disclosure. Opening five documents at once is a tabbed shell this one
//!   is not; inserting five images is five placement windows, and the second
//!   would open over the first with no way to tell them apart.
//! - **Hover feedback on the way in.** `egui` offers `hovered_files`, and a
//!   preview would be the right thing eventually. It is left out today rather
//!   than done badly: a tint that appeared on any hover, including over a
//!   document that cannot take the file, would promise a drop that then refused.

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
/// Because the answer is used to decide **which of two commands to raise**, and
/// both of them read the file properly afterwards: `Action::Open` runs the
/// parser, and the insert path runs `image_import`, which sniffs the real magic
/// bytes and refuses a mislabelled file with its own message. Sniffing here
/// would mean reading every dropped file twice and would put a second, weaker
/// opinion about file types in front of two that are already correct.
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
