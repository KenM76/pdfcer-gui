//! # `dialogs::drop_pdf` — **what a PDF dropped on an open document is for**
//!
//! Asked when exactly one PDF lands on the window while a document is open:
//! open it (the default, which Enter takes), insert all of its pages after the
//! page on screen, or place its first page at the drop point: drawn into the
//! page's content where the mode edits content, else as a stamp. A
//! choice the mode does not offer is absent; with neither offered the drop
//! simply opens, as does a file with no readable page.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/drop_pdf.md`.

use std::path::PathBuf;

use egui::Ui;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::pageops::InsertPosition;

use crate::app::actions::pages::PageAction;
use crate::app::actions::{Action, VectorAction};
use crate::canvas::textannot::{
    DEFAULT_STAMP, DEFAULT_STAMP_SIZE, DEFAULT_STICKY_ICON, TextAnnotKind,
};
use crate::stamps::library::CustomStamp;
use crate::text::dropped as t;

/// The window body's published region, for `ui-verify`.
const REGION_BODY: &str = "drop-pdf.body";
/// The insert button's region.
const REGION_INSERT: &str = "drop-pdf.insert";
/// The place button's region.
const REGION_PLACE: &str = "drop-pdf.place";

/// What the window may offer besides Open, as the mode and the drop allow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offer {
    /// The page on screen, 0-based, when inserting after it is offered.
    pub insert_after: Option<usize>,
    /// The drop's page, its point there in PDF user space, and that page's
    /// crop box, when placing is offered.
    pub place_at: Option<(usize, (f64, f64), Rect)>,
    /// Whether Place draws into the page's content rather than adding a
    /// stamp.
    pub as_content: bool,
}

/// Where the first page would go: the target page and the rectangle on it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Placement {
    page: usize,
    rect: Rect,
    as_content: bool,
}

/// The three answers that author something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Open,
    Insert,
    Place,
}

impl Choice {
    /// The trace's spelling.
    const fn word(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Insert => "insert",
            Self::Place => "place",
        }
    }
}

/// The window's state, read once when it opened.
pub struct DropPdfDialog {
    path: PathBuf,
    /// The file's name, for the question.
    name: String,
    source_pages: usize,
    insert_after: Option<usize>,
    place: Option<Placement>,
    chosen: Option<Choice>,
    cancelled: bool,
}

impl DropPdfDialog {
    /// Read `path` for its page count and first page's crop box. `None` when
    /// `offer` offers nothing beyond Open or the file has no readable page:
    /// the drop then simply opens.
    #[must_use]
    pub fn read(path: PathBuf, offer: Offer) -> Option<Self> {
        if offer.insert_after.is_none() && offer.place_at.is_none() {
            return None;
        }
        let doc = pdfcer_core::document::Document::load(&path).ok()?;
        let pages = pdfcer_core::page_tree::pages(&doc).ok()?;
        let first = pages.first()?.crop_box;
        Some(Self::new(path, pages.len(), first, offer))
    }

    /// The window for a file of `source_pages` pages whose first page's crop
    /// box is `first`.
    fn new(path: PathBuf, source_pages: usize, first: Rect, offer: Offer) -> Self {
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        // Natural size, centred on the drop and kept on the page, as a dropped
        // picture lands.
        let place = offer.place_at.map(|(page, point, crop)| Placement {
            page,
            rect: pdfcer_gui_base::clippaste::rect_at(point, (first.width(), first.height()), crop),
            as_content: offer.as_content,
        });
        Self {
            path,
            name,
            source_pages,
            insert_after: offer.insert_after,
            place,
            chosen: None,
            cancelled: false,
        }
    }

    /// The action `choice` raises, or `None` when it was not offered.
    fn action(&self, choice: Choice) -> Option<Action> {
        match choice {
            Choice::Open => Some(Action::Open(self.path.clone())),
            Choice::Insert => self.insert_after.map(|current| {
                Action::Page(PageAction::InsertPagesFromFile {
                    path: self.path.clone(),
                    pages: (0..self.source_pages).collect(),
                    position: InsertPosition::After(current),
                })
            }),
            Choice::Place => self.place.map(|p| self.placed(p)),
        }
    }

    /// Page 0 of the file drawn into the page's content, or, where the mode
    /// only authors markup, as a custom stamp exactly as a stamp collection's
    /// page is.
    fn placed(&self, p: Placement) -> Action {
        if p.as_content {
            return Action::Vector(VectorAction::PlacePageContent {
                page: p.page,
                rect: p.rect,
                file: self.path.clone(),
                source_page: 0,
            });
        }
        Action::CommitTextAnnot {
            page: p.page,
            kind: TextAnnotKind::Stamp,
            rect: p.rect,
            text: String::new(),
            stamp: DEFAULT_STAMP,
            stamp_size: DEFAULT_STAMP_SIZE,
            icon: DEFAULT_STICKY_ICON,
            custom: Some(CustomStamp {
                label: self.name.clone(),
                category: String::new(),
                file: self.path.clone(),
                page_index: 0,
                dynamic: false,
            }),
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "drop-pdf", // ui-text-exempt: a viewport key, never displayed.
            t::drop_pdf_title(),
            egui::vec2(460.0, 230.0),
            egui::vec2(360.0, 180.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if let Some(choice) = self.chosen.take() {
            self.chosen_trace(choice);
            actions.extend(self.action(choice));
            return false;
        }
        !frame.closed && !self.cancelled
    }

    /// The one trace a choice writes, with the placement's rectangle.
    fn chosen_trace(&self, choice: Choice) {
        let at = self
            .place
            .filter(|_| choice == Choice::Place)
            .map_or_else(String::new, |p| {
                format!(
                    " page={} llx={:.2} lly={:.2} urx={:.2} ury={:.2}", // ui-text-exempt: diagnostic trace
                    p.page, p.rect.llx, p.rect.lly, p.rect.urx, p.rect.ury
                )
            });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "drop-pdf-chosen choice={} n={}{at}",
                choice.word(),
                self.source_pages
            )
        });
    }

    /// The question, the offered answers, and the footer.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::drop_pdf_question(&self.name, self.source_pages));
        ui.add_space(8.0);
        if let Some(current) = self.insert_after {
            let button = ui.button(t::drop_pdf_insert(self.source_pages, current + 1));
            crate::diag::ui_rect(REGION_INSERT, button.rect);
            if button.clicked() {
                self.chosen = Some(Choice::Insert);
            }
        }
        if self.place.is_some() {
            let button = ui.button(t::drop_pdf_place());
            crate::diag::ui_rect(REGION_PLACE, button.rect);
            if button.clicked() {
                self.chosen = Some(Choice::Place);
            }
        }
        ui.separator();
        let (open, cancel, _) = crate::dialogs::host::Host::footer(
            ui,
            (t::drop_pdf_open(), ""),
            (t::drop_pdf_cancel(), ""),
            None,
        );
        if open {
            self.chosen = Some(Choice::Open);
        }
        self.cancelled |= cancel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(llx: f64, lly: f64, urx: f64, ury: f64) -> Rect {
        Rect { llx, lly, urx, ury }
    }

    fn dialog(offer: Offer) -> DropPdfDialog {
        DropPdfDialog::new(
            PathBuf::from("C:/x/logo.pdf"),
            3,
            rect(0.0, 0.0, 144.0, 72.0),
            offer,
        )
    }

    const BOTH: Offer = Offer {
        insert_after: Some(6),
        place_at: Some((
            2,
            (200.0, 500.0),
            Rect {
                llx: 0.0,
                lly: 0.0,
                urx: 612.0,
                ury: 792.0,
            },
        )),
        as_content: false,
    };

    /// Insert takes every page, after the page on screen.
    #[test]
    fn insert_takes_every_page_after_this_one() {
        let Some(Action::Page(PageAction::InsertPagesFromFile {
            pages, position, ..
        })) = dialog(BOTH).action(Choice::Insert)
        else {
            panic!("insert must raise InsertPagesFromFile");
        };
        assert_eq!(pages, vec![0, 1, 2]);
        assert_eq!(position, InsertPosition::After(6));
    }

    /// Place puts page 1 at its natural size, centred on the drop point.
    #[test]
    fn place_centres_the_first_page_on_the_drop() {
        let Some(Action::CommitTextAnnot {
            page,
            rect: r,
            custom: Some(c),
            ..
        }) = dialog(BOTH).action(Choice::Place)
        else {
            panic!("place must raise a custom stamp");
        };
        assert_eq!((page, c.page_index), (2, 0));
        assert_eq!(r, rect(128.0, 464.0, 272.0, 536.0));
    }

    /// Where the mode edits content, Place draws page 1 into the page.
    #[test]
    fn place_in_edit_draws_into_the_page() {
        let offer = Offer {
            as_content: true,
            ..BOTH
        };
        let Some(Action::Vector(VectorAction::PlacePageContent {
            page,
            rect: r,
            source_page,
            ..
        })) = dialog(offer).action(Choice::Place)
        else {
            panic!("place in Edit must raise PlacePageContent");
        };
        assert_eq!((page, source_page), (2, 0));
        assert_eq!(r, rect(128.0, 464.0, 272.0, 536.0));
    }

    /// A choice the mode did not offer raises nothing, and an offer of
    /// nothing opens no window.
    #[test]
    fn an_unoffered_choice_raises_nothing() {
        let none = Offer {
            insert_after: None,
            place_at: None,
            as_content: false,
        };
        assert!(dialog(none).action(Choice::Insert).is_none());
        assert!(dialog(none).action(Choice::Place).is_none());
        assert!(matches!(
            dialog(none).action(Choice::Open),
            Some(Action::Open(_))
        ));
        assert!(DropPdfDialog::read(PathBuf::from("C:/x/logo.pdf"), none).is_none());
    }
}
