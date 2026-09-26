//! # `panels::pages::import` — **a drawing dropped on the thumbnails becomes
//! # pages in this one**
//!
//!
//! > *"I should be able to drag and drop documents into the thumbnails section
//! > of another pdf to import the pages."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/pages/import.md`.

use std::path::{Path, PathBuf};

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::dropped::{Dropped, classify};

/// One file that is going to be imported, and how many pages it brings.
struct Source {
    path: PathBuf,
    pages: usize,
}

/// **Take a drop that landed on this panel and turn it into insertions.**
///
/// `panel` is the panel body's rectangle in screen points, `gap` the boundary
/// the grid resolved under the pointer (`None` when the pointer was over no
/// tile — see the module header), and `page_count` this document's length.
///
/// Returns `true` when the drop was claimed, which the caller does not need but
/// a test does: it is the difference between *"this panel acted"* and *"the
/// fallback will"*, and that is the property worth asserting.
pub fn claim(
    ctx: &egui::Context,
    panel: egui::Rect,
    gap: Option<usize>,
    page_count: usize,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(landing) = crate::app::filedrag::landed(ctx) else {
        return false;
    };
    // No position means no claim. The operating system declines to give one on
    // a locked workstation, and `(0, 0)` is a real place — see
    // `native_window::cursor_position`.
    let Some(at) = landing.at else {
        return false;
    };
    if !panel.contains(at) {
        return false;
    }

    let sources = readable_documents(&landing.paths);
    if sources.is_empty() {
        // Not a PDF, or not a readable one. Both fall through; see the header.
        return false;
    }

    // The gap the grid resolved, or the end of the document when the pointer
    // was on the panel but over no tile.
    let gap = gap.unwrap_or(page_count);
    let mut position_index = gap;
    let mut inserted = 0usize;
    for source in &sources {
        actions.push(Action::Page(PageAction::InsertPagesFromFile {
            path: source.path.clone(),
            pages: (0..source.pages).collect(),
            position: crate::pagedrag::insert_position(position_index, page_count + inserted),
        }));
        position_index += source.pages;
        inserted += source.pages;
    }

    crate::app::filedrag::claim(ctx);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "pages-import-dropped files={} pages={} gap={gap} of={page_count}",
            sources.len(),
            inserted
        )
    });
    true
}

/// The dropped paths that are PDFs with pages, in order, each with its length.
fn readable_documents(paths: &[PathBuf]) -> Vec<Source> {
    paths
        .iter()
        .filter(|p| matches!(classify(p), Dropped::Document(_)))
        .filter_map(|path| {
            let pages = page_count_of(path);
            (pages > 0).then(|| Source {
                path: path.clone(),
                pages,
            })
        })
        .collect()
}

/// How many pages a file on disk has, or `0` if it cannot be read.
fn page_count_of(path: &Path) -> usize {
    match pdfcer_core::document::Document::load(path) {
        Ok(doc) => pdfcer_core::page_tree::pages(&doc).map_or(0, |p| p.len()),
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("pages-import-unreadable path={path:?} reason={detail}")
            });
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A landing outside the panel is not this panel's business.
    #[test]
    fn a_drop_somewhere_else_is_not_claimed() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        // Nothing landed at all.
        assert!(!claim(
            &ctx,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 400.0)),
            Some(0),
            4,
            &mut actions
        ));
        assert!(actions.is_empty());
    }

    /// **A position of `None` declines**, rather than defaulting.
    #[test]
    fn a_landing_with_no_position_is_not_claimed() {
        let ctx = egui::Context::default();
        crate::app::filedrag::test_land(
            &ctx,
            crate::app::filedrag::Landed {
                paths: vec![PathBuf::from("a.pdf")],
                at: None,
            },
        );
        let mut actions = Vec::new();
        assert!(!claim(
            &ctx,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 400.0)),
            Some(1),
            4,
            &mut actions
        ));
        assert!(actions.is_empty());
        assert!(
            crate::app::filedrag::landed(&ctx).is_some(),
            "and it is left for the fallback, which is what opens the file"
        );
    }

    /// **A file that is not a readable PDF is left for the fallback.**
    #[test]
    fn an_unreadable_document_is_left_for_the_parser_to_explain() {
        let ctx = egui::Context::default();
        crate::app::filedrag::test_land(
            &ctx,
            crate::app::filedrag::Landed {
                paths: vec![PathBuf::from("no-such-file-anywhere.pdf")],
                at: Some(egui::pos2(10.0, 10.0)),
            },
        );
        let mut actions = Vec::new();
        assert!(!claim(
            &ctx,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 400.0)),
            Some(1),
            4,
            &mut actions
        ));
        assert!(actions.is_empty());
        assert!(crate::app::filedrag::landed(&ctx).is_some());
    }

    /// An image dropped on the thumbnails is not a page import.
    ///
    /// It falls through to the placement window, which is where a picture
    /// belongs — a drop on the page grid does not make a JPEG into pages.
    #[test]
    fn an_image_is_not_imported_as_pages() {
        assert!(readable_documents(&[PathBuf::from("logo.png")]).is_empty());
    }

    /// **Two files land in the order they were dragged, not on top of each
    /// other.**
    #[test]
    fn several_files_stack_in_the_order_they_were_dropped() {
        let sources = [
            Source {
                path: PathBuf::from("a.pdf"),
                pages: 3,
            },
            Source {
                path: PathBuf::from("b.pdf"),
                pages: 2,
            },
        ];
        let page_count = 10;
        let mut position_index = 5;
        let mut inserted = 0;
        let mut positions = Vec::new();
        for source in &sources {
            positions.push(crate::pagedrag::insert_position(
                position_index,
                page_count + inserted,
            ));
            position_index += source.pages;
            inserted += source.pages;
        }
        use pdfcer_core::pageops::InsertPosition;
        assert_eq!(
            positions,
            vec![InsertPosition::Before(5), InsertPosition::Before(8)]
        );
    }
}
