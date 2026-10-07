//! # `panels::comments::rich` — the formatting a note carries, named
//!
//! A comment authored in Acrobat stores its note twice: plain in `/Contents`,
//! formatted in `/RC` (§12.7.3.4). The row shows the plain words; this line
//! names what the formatted copy holds, so a note that reads flat here is not
//! mistaken for one with no formatting. Off-canvas, per R8b.

use pdfcer_core::richtext::{Run, parse};

use super::model::{CommentRow, RichNote};
use crate::text::panels::comments as t;

/// What a row's `/RC` resolves to.
enum Reading {
    Runs(Vec<Run>),
    Unreadable(Option<String>),
}

fn read(rich: &RichNote) -> Reading {
    match rich {
        RichNote::Body {
            xhtml,
            default_style,
        } => match parse(xhtml, default_style.as_deref()) {
            Ok(runs) => Reading::Runs(runs),
            Err(e) => Reading::Unreadable(Some(e.to_string())),
        },
        RichNote::Unreadable => Reading::Unreadable(None),
    }
}

/// The line under the note, when the row carries `/RC`.
pub(super) fn line(ui: &mut egui::Ui, comment: &CommentRow) {
    let Some(rich) = &comment.rich else {
        return;
    };
    let (text, hover, state, words) = match read(rich) {
        Reading::Runs(runs) => {
            let words = crate::text::richtext::formatting(&runs);
            let state = if words.is_empty() {
                "empty"
            } else {
                "formatted"
            };
            (
                t::comment_row_rich_note(&words),
                Some(t::comment_row_rich_note_breakdown(&runs)),
                state,
                words.join(", "), // ui-text-exempt: the trace field separator, never displayed
            )
        }
        Reading::Unreadable(reason) => (
            t::comment_row_rich_note_unreadable(reason.as_deref()),
            None,
            "unreadable",
            String::new(),
        ),
    };
    let label = ui.label(egui::RichText::new(text).small().weak());
    if let Some(h) = hover {
        label.on_hover_text(h);
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: a trace line, never displayed
            "comment-rich-note id={} state={state} words=\"{words}\"",
            comment.id.map_or(0, |id| id.num)
        )
    });
}
