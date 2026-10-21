//! Conditions for the character and paragraph controls inside a text edit
//! (`OPERATOR_REQUESTS.md` O271, O273).
//!
//! A draft on existing text is a text subject for the Font group exactly as a
//! sweep is, so it enables the group and its tab. Bold, Italic, Underline and
//! Strikethrough render pressed when the letter they would act on carries the
//! axis or line, or, over a selection, when every letter in it does, as a
//! word processor shows a mixed selection unpressed; a pending style for what
//! is typed next flips that.
//!
//! The weight read recognises the page's blocks, so it is cached by position
//! and edit epoch rather than paid every frame.

use egui_shell::commands::ConditionSet;
use egui_shell::ribbon::selected_condition;
use pdfcer_core::text_edit::TextPosition;

use crate::app::state::{OpenDoc, Status};
use crate::canvas::textedit::typing;
use crate::canvas::textedit::weight::{self, Weight};

const CACHE_KEY: &str = "pdfcer-textformat-weight-cache"; // ui-text-exempt: internal memory id, never displayed

/// The position, or range, whose weight was last read, and what it read.
#[derive(Clone)]
struct Cached {
    key: (usize, TextPosition, Option<TextPosition>, u64),
    weight: Option<Weight>,
}

impl crate::app::PdfcerApp {
    /// Publish the draft's text subject and the four toggles' pressed state.
    pub(super) fn text_format_conditions(&self, ctx: &egui::Context, set: &mut ConditionSet) {
        let Status::Open(doc) = &self.status else {
            return;
        };
        let Some(crate::app::fontlook::Subject {
            page,
            at,
            to,
            caret,
        }) = crate::app::fontlook::subject(ctx, doc, Some(set))
        else {
            return;
        };
        let read = weight_cached(ctx, doc, page, at, to);
        let pending = typing::read(ctx).zip(caret);
        let holds = |id: &str| pending.as_ref().is_some_and(|(t, c)| t.holds(id, *c));
        let bold = read.is_some_and(|w| w.bold.present()) != holds("format.bold");
        let italic = read.is_some_and(|w| w.italic.present()) != holds("format.italic");
        let under = read.is_some_and(|w| w.lines.underline) != holds("format.underline");
        let strike = read.is_some_and(|w| w.lines.strikethrough) != holds("format.strikethrough");
        // ui-text-exempt: registered command ids, never displayed.
        for (id, on) in [
            ("format.bold", bold),
            ("format.italic", italic),
            ("format.underline", under),
            ("format.strikethrough", strike),
        ] {
            if on {
                set.set(selected_condition(id));
            }
        }
    }
}

/// [`weight::at`], or [`weight::across`] a range, read once per range and
/// edit.
fn weight_cached(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    at: TextPosition,
    to: Option<TextPosition>,
) -> Option<Weight> {
    let id = egui::Id::new(CACHE_KEY);
    let key = (page, at, to, doc.edit_epoch);
    if let Some(hit) = ctx
        .data(|d| d.get_temp::<Cached>(id))
        .filter(|c| c.key == key)
    {
        return hit.weight;
    }
    let weight = match to {
        Some(to) => weight::across(doc, page, at, to),
        None => weight::at(doc, page, at),
    };
    ctx.data_mut(|d| d.insert_temp(id, Cached { key, weight }));
    weight
}
