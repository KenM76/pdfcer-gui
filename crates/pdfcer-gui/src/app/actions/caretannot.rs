//! # `app::actions::caretannot` — mark where words are to be inserted
//!
//! `/Caret` (§12.5.6.11) through `EditSession::add_caret_annotation`, one
//! undo entry. The words travel as the caret's note; the page's content is
//! not changed. Replace Text pairs that caret with a `/StrikeOut` over the
//! selected text through `EditSession::add_replace_text`.

use pdfcer_core::annot_author::{CaretSpec, CaretSymbol, Color, Quad};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::page_tree::Rect;

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::text::textannot as t;

/// The caret's width, in PDF points.
const CARET_W_PT: f64 = 8.0;
/// The caret's height, in PDF points; a paragraph mark doubles the box.
const CARET_H_PT: f64 = 10.0;

/// The caret's rectangle with its apex at `at`. The engine draws the caret
/// in the lower half of the box when a paragraph mark rides above it.
#[must_use]
pub(super) fn caret_rect(at: (f64, f64), paragraph: bool) -> Rect {
    let (x, y) = at;
    let height = if paragraph {
        2.0 * CARET_H_PT
    } else {
        CARET_H_PT
    };
    Rect {
        llx: x - CARET_W_PT / 2.0,
        lly: y - CARET_H_PT,
        urx: x + CARET_W_PT / 2.0,
        ury: y - CARET_H_PT + height,
    }
}

/// What the dialog accepted.
pub(super) struct Placed<'a> {
    /// The 0-based page.
    pub page: usize,
    /// The point clicked, in PDF user space.
    pub at: (f64, f64),
    /// The trimmed words, or `None` for a paragraph break alone.
    pub text: Option<&'a str>,
    /// Whether a paragraph mark accompanies the caret.
    pub paragraph: bool,
}

/// Author the caret. With no words no note is passed, so no empty
/// `/Contents` is written.
pub(super) fn place(
    doc: &mut OpenDoc,
    prefs: &Prefs,
    placed: &Placed<'_>,
    ink: (f64, f64, f64),
    opacity: Option<f64>,
) {
    let (r, g, b) = ink;
    let mut spec = CaretSpec::new(caret_rect(placed.at, placed.paragraph));
    spec.color = Color::Rgb(r, g, b);
    if placed.paragraph {
        spec.symbol = CaretSymbol::Paragraph;
    }
    let options = MarkupOptions {
        note: placed
            .text
            .map(|w| super::annots::signed_note(w, Some(&prefs.author_name))),
        opacity,
        ..Default::default()
    };
    let page = placed.page;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "caret-annot-read page={page} chars={} paragraph={} x={:.1} y={:.1}",
            placed.text.map_or(0, |w| w.chars().count()),
            placed.paragraph,
            placed.at.0,
            placed.at.1,
        )
    });
    super::apply::vector_edit(doc, "add-caret-annot", page, 1, |session| {
        let id = session.add_caret_annotation(page, &spec, &options)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("caret-annot-placed page={page} id={}", id.num)
        });
        Ok::<_, pdfcer_core::edit::EditError>(vec![t::caret_placed(page)])
    });
}

/// What the replace-text dialog accepted.
pub(super) struct Replaced<'a> {
    /// The 0-based page.
    pub page: usize,
    /// The caret's apex, in PDF user space.
    pub at: (f64, f64),
    /// The replacement words, trimmed.
    pub text: &'a str,
    /// The selected lines' boxes.
    pub struck: &'a [Quad],
}

/// Author the strike-out and its caret as one group, one undo entry. Both
/// take the caret pen's colour; the words are the caret's note.
pub(super) fn replace(
    doc: &mut OpenDoc,
    prefs: &Prefs,
    replaced: &Replaced<'_>,
    ink: (f64, f64, f64),
    opacity: Option<f64>,
) {
    let (r, g, b) = ink;
    let mut spec = CaretSpec::new(caret_rect(replaced.at, false));
    spec.color = Color::Rgb(r, g, b);
    let options = MarkupOptions {
        note: Some(super::annots::signed_note(
            replaced.text,
            Some(&prefs.author_name),
        )),
        opacity,
        ..Default::default()
    };
    let page = replaced.page;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "replace-text-read page={page} chars={} quads={} x={:.1} y={:.1}",
            replaced.text.chars().count(),
            replaced.struck.len(),
            replaced.at.0,
            replaced.at.1,
        )
    });
    super::apply::vector_edit(doc, "add-replace-text", page, 1, |session| {
        let added = session.add_replace_text(page, &spec, replaced.struck, &options)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "replace-text-placed page={page} caret={} strike={}",
                added.caret_id.num, added.strike_out_id.num
            )
        });
        Ok::<_, pdfcer_core::edit::EditError>(vec![t::replace_placed(page)])
    });
}

#[cfg(test)]
mod tests {
    use super::caret_rect;

    /// The apex is at the click: centred across it, and at the top of the
    /// caret's own part of the box with or without a paragraph mark.
    #[test]
    fn the_apex_sits_at_the_click() {
        for paragraph in [false, true] {
            let r = caret_rect((100.0, 200.0), paragraph);
            assert!((r.llx + r.urx - 200.0).abs() < 1e-9);
            let caret_top = if paragraph {
                r.lly + (r.ury - r.lly) * 0.5
            } else {
                r.ury
            };
            assert!((caret_top - 200.0).abs() < 1e-9, "paragraph={paragraph}");
        }
    }
}
