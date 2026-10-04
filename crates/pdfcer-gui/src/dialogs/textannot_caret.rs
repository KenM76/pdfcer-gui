//! # `dialogs::textannot::caret` — the insert-text and replace-text halves of
//! the note dialog
//!
//! The words are typed into the shared field. Inserting adds the
//! new-paragraph choice, and the click rect's upper-left corner is the
//! caret's apex. Replacing carries the selected lines' boxes instead, and the
//! caret follows the last of them.

use egui::Ui;
use pdfcer_core::annot_author::Quad;
use pdfcer_core::page_tree::Rect;

use super::{TextAnnotDialog, TextAnnotKind};
use crate::app::actions::Action;
use crate::canvas::textannot::painted_text;
use crate::text::textannot as t;
use pdfcer_gui_base::newcomment::NewComment;

/// The region the new-paragraph checkbox publishes.
pub const REGION_CARET_PARAGRAPH: &str = "text-annot.caret-paragraph"; // ui-text-exempt: trace region name, never displayed

/// The caret window's own choice.
#[derive(Default)]
pub(super) struct Choice {
    /// Whether a paragraph mark accompanies the caret. Never offered when
    /// replacing.
    paragraph: bool,
    /// The selected lines' boxes when replacing; `None` when inserting.
    struck: Option<Vec<Quad>>,
}

impl Choice {
    /// The new-paragraph checkbox and what it adds; nothing when replacing.
    pub(super) fn show(&mut self, ui: &mut Ui) {
        if self.struck.is_some() {
            return;
        }
        ui.add_space(8.0);
        let r = ui.checkbox(&mut self.paragraph, t::caret_paragraph());
        crate::diag::ui_rect(REGION_CARET_PARAGRAPH, r.rect);
        ui.label(t::caret_paragraph_bound());
    }

    /// Whether Add has something to author. Inserting takes words, a
    /// paragraph break, or both; replacing takes words.
    pub(super) fn ready(&self, typed: &str) -> bool {
        let words = !painted_text(typed).is_empty();
        if self.struck.is_some() {
            words
        } else {
            self.paragraph || words
        }
    }

    /// The action Add raises.
    pub(super) fn action(&self, page: usize, rect: Rect, typed: &str) -> Action {
        let text = painted_text(typed);
        match &self.struck {
            Some(struck) => NewComment::ReplaceText {
                page,
                at: (rect.llx, rect.ury),
                text: text.to_owned(),
                struck: struck.clone(),
            },
            None => NewComment::Caret {
                page,
                at: (rect.llx, rect.ury),
                text: (!text.is_empty()).then(|| text.to_owned()),
                paragraph: self.paragraph,
            },
        }
        .into()
    }
}

/// The caret's height above the line's bottom when it follows struck text,
/// in PDF points; matches the insert caret's own height.
const REPLACE_APEX_RISE_PT: f64 = 10.0;

/// A zero-width rect whose upper-left corner is the replace caret's apex:
/// the right end of the last line, a caret's height above its bottom.
/// `None` for an empty selection.
fn replace_anchor(struck: &[Quad]) -> Option<Rect> {
    let last = struck.last()?;
    let x = last.ur.0.max(last.lr.0);
    let bottom = last.ll.1.min(last.lr.1);
    Some(Rect {
        llx: x,
        lly: bottom,
        urx: x,
        ury: bottom + REPLACE_APEX_RISE_PT,
    })
}

impl TextAnnotDialog {
    /// The replace-text window over `struck` on `page`, or `None` for an
    /// empty selection.
    pub fn replacing(page: usize, struck: Vec<Quad>) -> Option<Self> {
        let anchor = replace_anchor(&struck)?;
        let mut dialog = Self::open(page, TextAnnotKind::Caret, anchor, None);
        dialog.caret = Some(Choice {
            paragraph: false,
            struck: Some(struck),
        });
        Some(dialog)
    }

    fn is_replacing(&self) -> bool {
        self.caret.as_ref().is_some_and(|c| c.struck.is_some())
    }

    /// The window's title.
    pub(super) fn title(&self) -> &'static str {
        if self.is_replacing() {
            t::replace_title()
        } else {
            t::title(self.kind)
        }
    }

    /// The sentence under the title.
    pub(super) fn intro(&self) -> &'static str {
        if self.is_replacing() {
            t::replace_intro()
        } else {
            t::intro(self.kind)
        }
    }

    /// The text field's placeholder.
    pub(super) fn hint(&self) -> &'static str {
        if self.is_replacing() {
            t::replace_hint()
        } else {
            t::hint(self.kind)
        }
    }

    /// Why Add is greyed.
    pub(super) fn accept_disabled(&self) -> &'static str {
        if self.is_replacing() {
            t::replace_accept_disabled()
        } else {
            t::accept_disabled(self.kind)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(llx: f64, lly: f64, urx: f64, ury: f64) -> Quad {
        Quad {
            ul: (llx, ury),
            ur: (urx, ury),
            ll: (llx, lly),
            lr: (urx, lly),
        }
    }

    /// The caret follows the LAST line, at its right end, above its bottom.
    #[test]
    fn the_replace_caret_follows_the_last_line() {
        let struck = [
            quad(10.0, 100.0, 200.0, 112.0),
            quad(10.0, 80.0, 60.0, 92.0),
        ];
        let r = replace_anchor(&struck).expect("a selection");
        assert_eq!((r.llx, r.ury), (60.0, 90.0));
        assert!(replace_anchor(&[]).is_none());
    }

    /// Replacing needs words; a paragraph break alone is an insert's answer.
    #[test]
    fn replacing_needs_words() {
        let c = Choice {
            paragraph: false,
            struck: Some(vec![quad(0.0, 0.0, 1.0, 1.0)]),
        };
        assert!(!c.ready("   "));
        assert!(c.ready("new"));
    }
}
