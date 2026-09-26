//! # `text::files` — the copy the open/close/recent surface owns
//!
//! The strings `pdfcer_gui::app::files` and `pdfcer_gui::app::recent` show:
//! the file dialog's own title and filter names, and everything the Recent
//! control draws.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/files.md`.

use std::path::Path;

/// The file dialog's title bar.
#[must_use]
pub fn open_dialog_title() -> &'static str {
    "Open a PDF document"
}

/// The name of the dialog's PDF filter. The pattern (`*.pdf`) is appended by
/// the caller, which is the convention every platform picker follows.
#[must_use]
pub fn filter_pdf() -> &'static str {
    "PDF documents"
}

/// The picker filter for a raster image.
#[must_use]
pub fn filter_image() -> &'static str {
    "Images (PNG, JPEG, BMP, TIFF)"
}

/// The picker filter for a plain text file.
#[must_use]
pub fn filter_text() -> &'static str {
    "Text files"
}

/// The picker filter for a form-data file.
#[must_use]
pub fn filter_form_data() -> &'static str {
    "Form data (FDF, XFDF, CSV)"
}

/// The name of the dialog's everything filter.
#[must_use]
pub fn filter_all() -> &'static str {
    "All files"
}

/// The title bar of the dialog `file.save_copy` opens.
#[must_use]
pub fn save_copy_dialog_title() -> &'static str {
    "Save a copy of this document"
}

/// The picker's heading for **Save As**, and the wording carries the difference.
#[must_use]
pub fn save_as_dialog_title() -> &'static str {
    "Save this document as"
}

/// The receipt for a completed Save As, naming the file that is now open.
#[must_use]
pub fn save_as_receipt(name: &str) -> String {
    format!("Saved as {name}. You are now editing that file — the original is untouched.")
}

/// The receipt for a completed Save-in-place, naming the file it went into.
#[must_use]
pub fn saved_in_place(path: &std::path::Path) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    format!("Saved {name}.")
}

/// The suffix `file.save_copy` appends to suggest a name for the copy.
#[must_use]
pub fn save_copy_suffix() -> &'static str {
    "-copy"
}

/// The title bar of the dialog `pages.extract` opens.
#[must_use]
pub fn extract_pages_dialog_title() -> &'static str {
    "Save the extracted pages as a new document"
}

/// The heading on the picker that chooses what to combine —
/// `OPERATOR_REQUESTS.md` O68.
#[must_use]
pub fn merge_dialog_title() -> &'static str {
    "Choose several PDFs to combine"
}

/// The heading on the picker that chooses where the combined file goes.
#[must_use]
pub fn merge_target_dialog_title() -> &'static str {
    "Save the combined document as a new file"
}

/// The name `tools.merge_files` suggests for the combined document.
#[must_use]
pub fn merge_target_name() -> &'static str {
    "Combined.pdf"
}

/// The suffix `pages.extract` appends to suggest a name for the new document.
#[must_use]
pub fn extract_pages_suffix() -> &'static str {
    "-pages"
}

// ---------------------------------------------------------------------------
// The Recent control's own LABEL and TOOLTIP are deliberately not here.
//
// It is a control for a registered command — `file.recent` — and a command's
// words live in `crate::text::commands`, whichever surface draws it. The
// custom item reads `crate::text::commands::file_recent()` for exactly the
// reason `pdfcer_gui::shell::menus`' header gives for a context-menu row reading
// its command's text: "a second copy of 'Delete' is a second copy that can
// drift". What IS here is everything the command's text cannot cover — the
// rows, which are file names, and the empty state, which is not a verb.
// ---------------------------------------------------------------------------

/// Shown inside the Recent menu when it has nothing to offer.
#[must_use]
pub fn recent_empty() -> &'static str {
    "No recent documents"
}

/// One row of the Recent menu: the file's name.
#[must_use]
pub fn recent_entry_label(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// **What a document made by `file.new` is called before it is saved.**
#[must_use]
pub fn untitled(ordinal: u32) -> String {
    format!("Untitled {ordinal}.pdf")
}

/// One row of the Recent menu, on hover: where the file actually is.
#[must_use]
pub fn recent_entry_tooltip(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// **No dialog string can break out of the PowerShell script.**
    #[test]
    fn the_dialog_strings_cannot_break_out_of_the_script() {
        for text in [
            open_dialog_title(),
            save_copy_dialog_title(),
            filter_pdf(),
            filter_all(),
        ] {
            assert!(
                !text.contains('\''),
                "`{text}` carries an apostrophe, which ends the single-quoted literal it is \
                 interpolated into (see this module's header)"
            );
            assert!(!text.is_empty());
        }
    }

    /// A row shows the file's name and hovers its whole path.
    #[test]
    fn a_row_names_the_file_and_hovers_where_it_is() {
        let path = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
        assert_eq!(recent_entry_label(&path), "Sheet 1.pdf");
        assert_eq!(recent_entry_tooltip(&path), "D:\\jobs\\4471\\Sheet 1.pdf");
    }

    /// **Two created documents are told apart by their names.**
    #[test]
    fn each_created_document_gets_its_own_name() {
        assert_eq!(untitled(1), "Untitled 1.pdf");
        assert_ne!(untitled(1), untitled(2));
        assert!(
            untitled(7).ends_with(".pdf"),
            "a save suggestion is built from this name; without a suffix it would \
             offer to write an extensionless file"
        );
        // The label surface must be able to draw it, which for a bare name
        // means `file_name()` answering rather than falling through.
        assert_eq!(
            recent_entry_label(Path::new(&untitled(3))),
            "Untitled 3.pdf"
        );
    }

    /// A path with no file name still renders something the operator can see.
    #[test]
    fn a_path_without_a_file_name_still_draws_a_row() {
        let label = recent_entry_label(Path::new("D:\\"));
        assert!(!label.is_empty(), "an empty row is an invisible control");
    }
}
