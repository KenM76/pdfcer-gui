//! # `app::actions::handsign` — write a hand signature into the page, inside its box
//!
//! Contract: a drawn signature is one `EditSession::add_markup_as_content`
//! Ink call and a typed one is one `EditSession::add_text` call in an
//! embedded subset of its handwriting face, each through the funnel, so one
//! undo step and content that renders exactly as it saves. Both are tagged
//! with the field's name, which is how `app::handsigned` later finds the box
//! signed. The `/Sig` field is not touched, so a
//! certificate signature can still be added to it later.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/handsign.md`.

use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::text_edit::{AddTextRequest, FontProvenance, NewTextColor};
use pdfcer_gui_base::handsign::typed::{self, Typed};
use pdfcer_gui_base::handsign::{self, Mark, Signature};

use super::funnel::vector_edit;
use crate::app::state::OpenDoc;

/// The ink: a dark blue-black, the colour of a ballpoint signature, so it
/// reads as handwriting beside black form text. A document colour, not a
/// theme role: it is written into the page.
const INK: (f64, f64, f64) = (0.05, 0.10, 0.35);

/// Fit `signature` into `rect` (canvas space) on `page` and write it as content.
pub(super) fn place(
    doc: &mut OpenDoc,
    field: &str,
    page: usize,
    rect: egui::Rect,
    signature: &Signature,
) {
    match signature {
        Signature::Drawn(mark) => place_drawn(doc, field, page, rect, mark),
        Signature::Typed(typed) => place_typed(doc, field, page, rect, typed),
    }
}

fn refuse(page: usize, via: &str, reason: &str) {
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("hand-sign-refused via={via} page={page} reason={reason}"));
}

fn place_drawn(doc: &mut OpenDoc, field: &str, page: usize, rect: egui::Rect, mark: &Mark) {
    let Some(fitted) = handsign::fit(mark, rect) else {
        refuse(page, "draw", "no-extent");
        return;
    };
    let Some(sheet) = doc.pages.get(page) else {
        refuse(page, "draw", "no-page");
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
        refuse(page, "draw", "unmappable");
        return;
    };
    let points: usize = strokes.iter().map(Vec::len).sum();
    let count = strokes.len();
    // One canvas unit is one point at scale 1, so the pen width carries over.
    let spec = MarkupSpec::Ink {
        strokes,
        color: Color::Rgb(INK.0, INK.1, INK.2),
        width: f64::from(fitted.width),
    };
    let options = MarkupOptions {
        hand_signature: Some(field.to_owned()),
        ..MarkupOptions::default()
    };
    let mut applied = None;
    vector_edit(doc, "place-hand-signature", page, 1, |session| {
        session
            .add_markup_as_content(page, &spec, &options)
            .map(|outcome| {
                applied = Some(outcome.objects.clone());
                outcome.paste.disclosures
            })
    });
    let Some(objects) = applied else {
        return;
    };
    let depth = doc.session.undo_depth();
    let tagged = u8::from(tagged(doc, field, page));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. No field name: it
        // is text from the operator's own document.
        format!(
            "hand-sign-placed via=draw page={page} strokes={count} points={points} objects={}..{} undo_depth={depth} tagged={tagged}",
            objects.start, objects.end
        )
    });
}

fn place_typed(doc: &mut OpenDoc, field: &str, page: usize, rect: egui::Rect, typed: &Typed) {
    let Some(face) = typed::faces().iter().find(|f| f.label == typed.face) else {
        refuse(page, "type", "no-face");
        return;
    };
    let plan = match typed::plan(face, &typed.name) {
        Ok(plan) => plan,
        Err(_) => {
            refuse(page, "type", "not-in-face");
            return;
        }
    };
    let metrics = plan.metrics;
    let fit = typed::advance(&plan, &typed.name).and_then(|advance| {
        typed::fit_typed(
            advance,
            metrics.ascent as f32 / 1000.0,
            metrics.descent as f32 / 1000.0,
            rect,
        )
    });
    let Some(fit) = fit else {
        refuse(page, "type", "no-extent");
        return;
    };
    let Some(sheet) = doc.pages.get(page) else {
        refuse(page, "type", "no-page");
        return;
    };
    if !typed::writes_along(sheet) {
        // The window greys the Type tab on such a page; this is its backstop.
        refuse(page, "type", "turned-page");
        return;
    }
    let Some(origin) = crate::viewer::canvas_to_pdf_space(fit.origin, sheet) else {
        refuse(page, "type", "unmappable");
        return;
    };
    let size = fit.size;
    let req = AddTextRequest::new(
        page,
        (f64::from(origin.x), f64::from(origin.y)),
        typed.name.clone(),
    )
    .with_embedded_face(plan)
    .with_provenance(FontProvenance::Supplied)
    .with_size(f64::from(size))
    .with_color(NewTextColor::Rgb(INK.0, INK.1, INK.2))
    .with_hand_signature(field);
    let before = doc.edit_epoch;
    vector_edit(doc, "place-typed-signature", page, 1, |session| {
        session.add_text(&req).map(|report| report.disclosures)
    });
    if doc.edit_epoch == before {
        return;
    }
    let depth = doc.session.undo_depth();
    let tagged = u8::from(tagged(doc, field, page));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. No field name or
        // typed name: both are the operator's own text.
        format!(
            "hand-sign-placed via=type page={page} face={} chars={} size={size:.1} undo_depth={depth} tagged={tagged}",
            face.label.replace(' ', "_"),
            typed.name.chars().count()
        )
    });
}

/// Whether `page` now reads back a hand-signature tag naming `field`.
fn tagged(doc: &OpenDoc, field: &str, page: usize) -> bool {
    doc.pages.get(page).is_some_and(|sheet| {
        pdfcer_core::hand_sig::hand_signatures(&doc.session.view(), sheet)
            .is_ok_and(|marks| marks.iter().any(|m| m.field == field))
    })
}
