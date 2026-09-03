//! # `panels::pages::import` — **a drawing dropped on the thumbnails becomes
//! # pages in this one**
//!
//! The operator, 2026-08-31 (`OPERATOR_REQUESTS.md` **O67**):
//!
//! > *"I should be able to drag and drop documents into the thumbnails section
//! > of another pdf to import the pages."*
//!
//! This is the Pages panel's half of [`crate::app::filedrag`]'s claim
//! protocol: the panel resolves the gap under the pointer with the same code
//! that resolves it for a page drag, and this module decides whether the file
//! that landed there is one it can act on.
//!
//! ## ★★★ Every refusal here is a FALL-THROUGH, never a message
//!
//! There are four reasons this module declines, and not one of them tells the
//! operator anything:
//!
//! | it declines when | and then |
//! |---|---|
//! | the drop was not on this panel | the file opens, or inserts, as always |
//! | the platform gave no cursor position | the file opens — the position-blind behaviour |
//! | the file is **not a PDF** | an image dropped here still goes to the placement window |
//! | the file has **no readable pages** | the file opens, and the *parser* says why |
//!
//! That last row is the one worth stating plainly. A corrupt or encrypted file
//! dropped on the thumbnails produces `pdfcer-core`'s own error, which names the
//! actual problem, rather than this module's guess at one — the same argument
//! `app::dropped::classify` makes for not sniffing bytes it is about to hand to
//! a parser that will.
//!
//! ⇒ **Declining costs a feature and never a file.** That asymmetry is the
//! whole reason the claim protocol exists rather than this module deciding what
//! every drop means.
//!
//! ## ★★ Why the whole panel accepts, not only the tiles
//!
//! The operator wrote *"into the thumbnails section"* — a region, not a
//! target. A drop on the grid's empty space below the last row, or on the
//! *Previews* checkbox, is unambiguously aimed at this panel, and refusing it
//! because it missed a tile by four points would teach that drops work
//! *sometimes*, which `app::dropped`'s header already names as worse than
//! never.
//!
//! So the panel's rectangle accepts, and the **gap** comes from the tile under
//! the pointer when there is one and from the end of the document when there
//! is not. That is also the conventional answer: a file dropped past the last
//! page goes after the last page.
//!
//! ## ★ Several files at once, and why the positions are computed up front
//!
//! *"documents"*, plural. Each file becomes its own
//! [`PageAction::InsertPagesFromFile`] — one undoable command each, which is
//! what the engine's `insert_pages` records — and their positions are worked
//! out **before** any of them is applied:
//!
//! ```text
//! A (3 pages) and B (2 pages) dropped in the gap before page 5
//!   A → Before(5)
//!   B → Before(8)          5 + 3, because A's pages are in front of B's by then
//! ```
//!
//! Deriving each position from the live page count at apply time would be the
//! obvious alternative and is wrong in a way that is hard to see: the actions
//! are applied in sequence within one frame, so the second would read a count
//! that already includes the first, and the two files would interleave.

use std::path::{Path, PathBuf};

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::dropped::{Dropped, classify};

/// One file that is going to be imported, and how many pages it brings.
///
/// ★ The count is read **once**, here, and then used twice — for the page list
/// and for the next file's position. Reading it twice would be two parses of
/// the same file with no guarantee they agreed.
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
///
/// # ★ Why the page count is read here rather than at apply time
///
/// Because a file with **no** readable pages must not be claimed at all — it
/// has to reach `Action::Open` so the parser can say what is wrong with it —
/// and "does this file have pages?" cannot be answered without opening it.
/// `dialogs::insert_pages` opens the file for the same reason and says the same
/// thing about the cost: *"the load is cheap relative to what follows"*, since
/// the insert itself opens it again anyway.
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
///
/// `0` for every failure, deliberately: the caller's only question is *"can
/// this be imported?"*, and a `Result` here would make three different
/// unreadable-ness reasons into three call-site branches that all end in the
/// same fall-through.
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

    /// ★★ **A position of `None` declines**, rather than defaulting.
    ///
    /// The platform gives no cursor position on a locked workstation, and the
    /// tempting default — the panel's own centre, or the end of the document —
    /// would import pages at a place nobody pointed at. Falling through opens
    /// the file instead, which is visible and undoable.
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

    /// ★★★ **A file that is not a readable PDF is left for the fallback.**
    ///
    /// The path does not exist, so it cannot be read — the same outcome as a
    /// corrupt file, and the one that matters: the drop is NOT claimed, so
    /// `app::dropped` still runs and the operator gets the parser's own
    /// sentence rather than silence.
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

    /// ★★ **Two files land in the order they were dragged, not on top of each
    /// other.**
    ///
    /// The arithmetic from the module header, asserted without touching the
    /// disk: A has three pages and goes in the gap before page 5, so B's own
    /// position has to be 8 rather than 5. Deriving each position at apply
    /// time would produce 5 twice and interleave the two documents.
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
