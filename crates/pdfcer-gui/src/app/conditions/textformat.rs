//! Conditions for the character and paragraph controls inside a text edit
//! (`OPERATOR_REQUESTS.md` O271, O273).
//!
//! A draft on existing text is a text subject for the Font group exactly as a
//! sweep is, so it enables the group and its tab. Bold, Italic, Underline and
//! Strikethrough render pressed when the letter they would act on carries the
//! axis or line; a pending style for what is typed next flips that.
//!
//! The weight read recognises the page's blocks, so it is cached by position
//! and edit epoch rather than paid every frame.

use egui_shell::commands::ConditionSet;
use egui_shell::ribbon::selected_condition;
use pdfcer_core::text_edit::TextPosition;

use crate::app::state::{OpenDoc, Status};
use crate::canvas::textedit::weight::{self, Weight};
use crate::canvas::textedit::{self, Anchor, typing};

const CACHE_KEY: &str = "pdfcer-textformat-weight-cache"; // ui-text-exempt: internal memory id, never displayed

/// The position whose weight was last read, and what it read.
#[derive(Clone)]
struct Cached {
    key: (usize, TextPosition, u64),
    weight: Option<Weight>,
}

impl crate::app::PdfcerApp {
    /// Publish the draft's text subject and the four toggles' pressed state.
    pub(super) fn text_format_conditions(&self, ctx: &egui::Context, set: &mut ConditionSet) {
        let Status::Open(doc) = &self.status else {
            return;
        };
        let draft = textedit::read(ctx);
        let (page, at, caret) = match &draft {
            Some(d) => {
                let Anchor::Run { run, .. } = d.anchor else {
                    return;
                };
                set.set("selection.text_runs");
                set.set("selection.formattable");
                let first = textedit::caret::range(d.mark, d.caret)
                    .map_or(d.caret.saturating_sub(1), |(a, _)| a);
                let byte = d
                    .text
                    .char_indices()
                    .nth(first)
                    .map_or(d.text.len(), |(b, _)| b);
                (d.page, TextPosition::new(run, byte), Some(d.caret))
            }
            None => match doc
                .text_selection
                .as_ref()
                .filter(|s| s.live(doc.edit_epoch) && !s.is_empty())
            {
                Some(s) => {
                    let (from, _) = pdfcer_gui_base::textselection::ordered(s.anchor(), s.focus());
                    (s.page, from, None)
                }
                None => return,
            },
        };
        let read = weight_cached(ctx, doc, page, at);
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

/// [`weight::at`], read once per position and edit.
fn weight_cached(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    at: TextPosition,
) -> Option<Weight> {
    let id = egui::Id::new(CACHE_KEY);
    let key = (page, at, doc.edit_epoch);
    if let Some(hit) = ctx
        .data(|d| d.get_temp::<Cached>(id))
        .filter(|c| c.key == key)
    {
        return hit.weight;
    }
    let weight = weight::at(doc, page, at);
    ctx.data_mut(|d| d.insert_temp(id, Cached { key, weight }));
    weight
}
