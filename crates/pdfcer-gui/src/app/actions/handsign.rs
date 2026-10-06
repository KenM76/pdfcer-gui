//! # `app::actions::handsign` — write a hand signature into the page, inside its box
//!
//! Contract: a drawn signature is one `EditSession::add_markup_as_content`
//! Ink call, a typed one is one `EditSession::add_text` call in an embedded
//! subset of its handwriting face, and a picture is one
//! `EditSession::add_image` call, each through the funnel, so one undo step
//! and content that renders exactly as it saves. Each lands where the
//! operator put it in the window, or by the fit rule when he did not
//! (`handsign::place::ink_rect`). All three are tagged with the field's name,
//! which is how `app::handsigned` later finds the box signed. The `/Sig` field is not touched, so a
//! certificate signature can still be added to it later.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/handsign.md`.

use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::edit::{MarkupOptions, NewImage};
use pdfcer_core::text_edit::{AddTextRequest, FontProvenance, NewTextColor};
use pdfcer_gui_base::handsign::picture::SigPicture;
use pdfcer_gui_base::handsign::place::{self, Placement};
use pdfcer_gui_base::handsign::typed::{self, Typed};
use pdfcer_gui_base::handsign::{self, Mark, Signature};

use super::funnel::vector_edit;

mod adjust;
use crate::app::state::OpenDoc;
pub(in crate::app::actions) use adjust::adjust;

/// The ink: a dark blue-black, the colour of a ballpoint signature, so it
/// reads as handwriting beside black form text. A document colour, not a
/// theme role: it is written into the page.
const INK: (f64, f64, f64) = (0.05, 0.10, 0.35);

/// Write `signature` as content into the box `rect` (canvas space) on
/// `page`, where `placement` puts it, or by the fit rule when it is `None`.
pub(super) fn place(
    doc: &mut OpenDoc,
    field: &str,
    page: usize,
    rect: egui::Rect,
    signature: &Signature,
    placement: Option<Placement>,
) {
    let at = Target {
        field,
        page,
        rect,
        placement,
    };
    match signature {
        Signature::Drawn(mark) => place_drawn(doc, &at, mark),
        Signature::Typed(typed) => place_typed(doc, &at, typed),
        Signature::Picture(picture) => place_picture(doc, &at, picture),
    }
}

/// The box a signature goes into and where in it.
struct Target<'a> {
    field: &'a str,
    page: usize,
    rect: egui::Rect,
    placement: Option<Placement>,
}

fn refuse(page: usize, via: &str, reason: &str) {
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("hand-sign-refused via={via} page={page} reason={reason}"));
}

fn place_drawn(doc: &mut OpenDoc, at: &Target, mark: &Mark) {
    let (field, page) = (at.field, at.page);
    let fitted = mark
        .has_extent()
        .then(|| mark.bounds())
        .flatten()
        .and_then(|ink| place::ink_rect(ink.size(), at.rect, at.placement))
        .and_then(|ink| handsign::fit_into(mark, ink));
    let Some(fitted) = fitted else {
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

fn place_typed(doc: &mut OpenDoc, at: &Target, typed: &Typed) {
    let (field, page) = (at.field, at.page);
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
    let (ascent, descent) = (
        metrics.ascent as f32 / 1000.0,
        metrics.descent as f32 / 1000.0,
    );
    let fit = typed::advance(&plan, &typed.name)
        .filter(|advance| *advance > 0.0 && ascent > descent)
        .and_then(|advance| {
            place::ink_rect(egui::vec2(advance, ascent - descent), at.rect, at.placement)
        })
        .map(|ink| typed::typed_in(ascent, descent, ink));
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

/// A picture signature: one `add_image`, stretched to the ink rectangle,
/// whose proportions are the picture's own unless the operator chose others.
fn place_picture(doc: &mut OpenDoc, at: &Target, picture: &SigPicture) {
    let (field, page) = (at.field, at.page);
    let image = match picture.image() {
        Ok(image) => image,
        Err(_) => {
            refuse(page, "picture", "unreadable");
            return;
        }
    };
    let Some(ink) = place::ink_rect(
        pdfcer_gui_base::handsign::picture::ink_size(&image),
        at.rect,
        at.placement,
    ) else {
        refuse(page, "picture", "no-extent");
        return;
    };
    let Some(sheet) = doc.pages.get(page) else {
        refuse(page, "picture", "no-page");
        return;
    };
    if !typed::writes_along(sheet) {
        // `add_image` places a picture along the page's own axes; the window
        // greys the Picture tab on a turned page and this is its backstop.
        refuse(page, "picture", "turned-page");
        return;
    }
    let corners = crate::viewer::canvas_to_pdf_space(ink.min, sheet)
        .zip(crate::viewer::canvas_to_pdf_space(ink.max, sheet));
    let Some((a, b)) = corners else {
        refuse(page, "picture", "unmappable");
        return;
    };
    let rect = pdfcer_core::page_tree::Rect {
        llx: f64::from(a.x.min(b.x)),
        lly: f64::from(a.y.min(b.y)),
        urx: f64::from(a.x.max(b.x)),
        ury: f64::from(a.y.max(b.y)),
    };
    let alpha = u8::from(image.soft_mask.is_some());
    let before = doc.edit_epoch;
    vector_edit(doc, "place-picture-signature", page, 1, |session| {
        let spec = NewImage::new(page, rect, &image)
            .stretching()
            .as_hand_signature(field);
        session.add_image(&spec).map(|outcome| {
            let d = &outcome.disclosures;
            let mut notes = crate::text::images::placement_disclosures(
                d.effective_dpi,
                d.below_screen_resolution,
                d.letterboxed,
                d.aspect_distorted,
                d.recompressed,
                d.source_bytes,
                d.stored_bytes,
            );
            notes.extend(crate::text::images::source_decoding_notes(d));
            notes
        })
    });
    if doc.edit_epoch == before {
        return;
    }
    let depth = doc.session.undo_depth();
    let tagged = u8::from(tagged(doc, field, page));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "hand-sign-placed via=picture page={page} undo_depth={depth} tagged={tagged} clear_white={} alpha={alpha}",
            u8::from(picture.clear_white)
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
