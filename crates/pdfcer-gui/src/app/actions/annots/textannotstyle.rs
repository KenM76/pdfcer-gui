//! # `app::actions::annots::textannotstyle` — restyling a text-bearing
//! annotation, and recolouring or fading any annotation
//!
//! `set_text_annot_style` (a sticky note, a stamp, a text box),
//! `set_annot_opacity` (any annotation) and `set_marker_style` (a caret,
//! attachment, sound or screen mark), with the disclosures they owe. A refusal
//! that a redraw answers is kept on [`OpenDoc::redraw_offer`] for the
//! Properties panel to offer.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/annots/textannotstyle.md`.

use pdfcer_core::annot_author::{Color, StampLabelFit};
use pdfcer_core::edit::{EditError, MarkerStyle, StyleEdit, TextAnnotStyle};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::redrawoffer::{RedrawOffer, Refused};

use crate::app::state::OpenDoc;
use crate::text::panels::textannotstyle as ts;

/// **Restyle a text-bearing annotation.** `EditSession::set_text_annot_style`.
pub(in crate::app::actions) fn set_text_annot_style(
    doc: &mut OpenDoc,
    id: ObjId,
    style: &TextAnnotStyle,
) {
    let mut refused: Option<EditError> = None;
    crate::app::actions::apply::vector_edit(doc, "set-text-annot-style", 0, 1, |session| {
        session
            .set_text_annot_style(id, style)
            .inspect_err(|e| refused = Some(e.clone()))
            .map(|change| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "set-text-annot-style-applied id={} subtype={} icon={} colour={} \
                         size={} fit={} was_foreign={} ap={:?} opacity={} frame={} text={} \
                         label={} rich_dropped={} redraw={}",
                        id.num,
                        change.subtype,
                        change.icon_written,
                        change.color_written,
                        change.font_size_written,
                        change
                            .stamp_label_fit
                            .as_ref()
                            .map_or("none", StampLabelFit::token),
                        u8::from(change.appearance_was_foreign),
                        change.appearance,
                        change.opacity_written,
                        change.frame_written,
                        change.text_style_written,
                        change.label_written,
                        change.rich_text_dropped.len(),
                        style.redraw_as_plain,
                    )
                });
                disclosures(&change)
            })
    });
    settle_offer(doc, id, refused, || {
        Refused::TextStyle(Box::new(style.clone()))
    });
}

/// What a text-annotation restyle owes the operator off-canvas: a fit the
/// engine chose for a stamp's words, a look that was not pdfcer's and was
/// replaced, and rich-text formatting dropped by a plain redraw.
fn disclosures(change: &pdfcer_core::edit::TextAnnotStyleChange) -> Vec<String> {
    let mut out = Vec::new();
    // `is_inference()` is the engine's own test: a label drawn at the size
    // asked for decided nothing and is not reported back.
    if let Some(fit) = change.stamp_label_fit.as_ref()
        && fit.is_inference()
    {
        out.push(match fit {
            StampLabelFit::LabelShrunk { size, requested } => {
                ts::stamp_label_shrunk(*size, *requested)
            }
            StampLabelFit::LabelClipped { hidden_chars, .. } => {
                ts::stamp_label_clipped(*hidden_chars)
            }
            StampLabelFit::BoxGrown { width, .. } => ts::stamp_label_box_grown(*width),
            _ => ts::stamp_label_fit_unknown().to_owned(),
        });
    }
    if change.appearance_was_foreign {
        out.push(ts::appearance_replaced().to_owned());
    }
    if !change.rich_text_dropped.is_empty() {
        out.push(ts::rich_text_dropped(&change.rich_text_dropped));
    }
    out
}

/// **Set or remove an annotation's opacity.** `EditSession::set_annot_opacity`,
/// which takes any subtype and leaves the appearance bytes alone.
pub(in crate::app::actions) fn set_annot_opacity(
    doc: &mut OpenDoc,
    id: ObjId,
    opacity: StyleEdit<f64>,
) {
    crate::app::actions::apply::vector_edit(doc, "set-annot-opacity", 0, 1, |session| {
        session.set_annot_opacity(id, opacity).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "set-annot-opacity-applied id={} previous={} current={} clamped={}",
                    id.num,
                    change
                        .previous
                        .map_or_else(|| "none".to_owned(), |a| a.to_string()),
                    change
                        .current
                        .map_or_else(|| "none".to_owned(), |a| a.to_string()),
                    change.clamped
                )
            });
            Vec::new()
        })
    });
}

/// **Recolour a marker's icon.** `EditSession::set_marker_style`.
pub(in crate::app::actions) fn set_marker_colour(
    doc: &mut OpenDoc,
    id: ObjId,
    color: Color,
    redraw_as_plain: bool,
) {
    let mut refused: Option<EditError> = None;
    let mut style = MarkerStyle::new(color);
    style.redraw_as_plain = redraw_as_plain;
    crate::app::actions::apply::vector_edit(doc, "set-marker-colour", 0, 1, |session| {
        session
            .set_marker_style(id, &style)
            .inspect_err(|e| refused = Some(e.clone()))
            .map(|change| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "set-marker-colour-applied id={} subtype={} was_foreign={} redraw={}",
                        id.num,
                        change.subtype,
                        u8::from(change.appearance_was_foreign),
                        redraw_as_plain
                    )
                });
                if change.appearance_was_foreign {
                    vec![ts::appearance_replaced().to_owned()]
                } else {
                    Vec::new()
                }
            })
    });
    settle_offer(doc, id, refused, || Refused::Marker(color));
}

/// Keep a refusal a redraw answers as the annotation's offer; drop the
/// annotation's offer once a restyle of it succeeds.
fn settle_offer(
    doc: &mut OpenDoc,
    id: ObjId,
    refused: Option<EditError>,
    asked: impl FnOnce() -> Refused,
) {
    match refused {
        Some(e) => {
            if let Some(offer) = RedrawOffer::of(&e, id, asked()) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("redraw-offer id={} why={}", id.num, offer.why.token())
                });
                doc.redraw_offer = Some(offer);
            }
        }
        None => {
            if doc.redraw_offer.as_ref().is_some_and(|o| o.id == id) {
                doc.redraw_offer = None;
            }
        }
    }
}
