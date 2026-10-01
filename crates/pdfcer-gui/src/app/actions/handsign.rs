//! # `app::actions::handsign` — write a hand-drawn signature into the page, inside its box
//!
//! Contract: one `EditSession::add_markup_as_content` Ink call through the
//! funnel, so one undo step and content that renders exactly as it saves; on
//! success the field enters `OpenDoc::hand_signed`. The `/Sig` field is not
//! touched, so a certificate signature can still be added to it later.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/handsign.md`.

use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_gui_base::handsign::{self, Mark};

use super::funnel::vector_edit;
use crate::app::state::OpenDoc;

/// The ink: a dark blue-black, the colour of a ballpoint signature, so it
/// reads as handwriting beside black form text. A document colour, not a
/// theme role: it is written into the page.
const INK: Color = Color::Rgb(0.05, 0.10, 0.35);

/// Fit `mark` into `rect` (canvas space) on `page` and write it as content.
pub(super) fn place(doc: &mut OpenDoc, field: &str, page: usize, rect: egui::Rect, mark: &Mark) {
    let refuse = |reason: &str| {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("hand-sign-refused page={page} reason={reason}"));
    };
    let Some(fitted) = handsign::fit(mark, rect) else {
        refuse("no-extent");
        return;
    };
    let Some(sheet) = doc.pages.get(page) else {
        refuse("no-page");
        return;
    };
    // Canvas space is display-oriented, so mapping each point back through
    // the page transform keeps the signature upright on a rotated page.
    let strokes: Option<Vec<Vec<(f64, f64)>>> = fitted
        .strokes
        .iter()
        .map(|stroke| {
            stroke
                .iter()
                .map(|p| {
                    crate::viewer::canvas_to_pdf_space(*p, sheet)
                        .map(|q| (f64::from(q.x), f64::from(q.y)))
                })
                .collect()
        })
        .collect();
    let Some(strokes) = strokes else {
        refuse("unmappable");
        return;
    };
    let points: usize = strokes.iter().map(Vec::len).sum();
    let count = strokes.len();
    // One canvas unit is one point at scale 1, so the pen width carries over.
    let spec = MarkupSpec::Ink {
        strokes,
        color: INK,
        width: f64::from(fitted.width),
    };
    let mut applied = None;
    vector_edit(doc, "place-hand-signature", page, 1, |session| {
        session
            .add_markup_as_content(page, &spec, &MarkupOptions::default())
            .map(|outcome| {
                applied = Some(outcome.objects.clone());
                outcome.paste.disclosures
            })
    });
    let Some(objects) = applied else {
        return;
    };
    let depth = doc.session.undo_depth();
    doc.hand_signed.placed(field, depth);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. No field name: it
        // is text from the operator's own document.
        format!(
            "hand-sign-placed page={page} strokes={count} points={points} objects={}..{} undo_depth={depth} signed={}",
            objects.start,
            objects.end,
            doc.hand_signed.signed_count()
        )
    });
}
