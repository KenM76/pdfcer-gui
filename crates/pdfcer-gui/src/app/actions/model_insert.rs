//! Placing a 3D model on a page with Insert 3D model's choices: when it
//! starts, its poster, and the opening view's lighting, in one undo entry.
//!
//! The lit opening view is the engine's `EditSession::set_3d_opening_view`:
//! automatic lighting writes CAD lights for a model converted from glTF and
//! no view for any other; a chosen scheme always writes one, framed as the
//! poster frames the model where the engine can mesh it.

use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::threed::{PlaceholderReason, ThreeDLighting, ThreeDPoster, ThreeDSpec};
use pdfcer_gui_base::editactions::ModelPlacement;

use crate::app::state::OpenDoc;
use crate::text::panels::models as t;

/// Read `placement.source` and place it, centred, on `page`.
pub(super) fn insert(doc: &mut OpenDoc, page: usize, placement: &ModelPlacement) {
    let data = match std::fs::read(&placement.source) {
        Ok(data) => data,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("model-insert-unreadable kind={:?}", error.kind())
            });
            super::record_note(doc.edit_epoch, t::insert_unreadable(&error.to_string()));
            return;
        }
    };
    let Some((data, notes, converted)) = converted(doc, placement, data) else {
        return;
    };
    let Some(crop) = doc.pages.get(page).map(|p| p.crop_box) else {
        return;
    };
    let mut spec = match ThreeDSpec::new(centred(crop), data) {
        Ok(spec) => spec,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("model-insert-refused error={error:?}")
            });
            super::record_note(doc.edit_epoch, t::insert_refused(&error));
            return;
        }
    };
    spec.activation = placement.activation;
    spec.render_poster = placement.render_poster;
    trace_requested(page, &spec, placement);
    place(doc, page, &spec, placement.lighting, (notes, converted));
}

/// The bytes to embed and the conversion's notes, glTF converted to PRC
/// with the window's switches, and whether it was converted. `None` when the
/// conversion was refused, with the reason recorded.
#[cfg(feature = "3d")]
fn converted(
    doc: &OpenDoc,
    placement: &ModelPlacement,
    data: Vec<u8>,
) -> Option<(Vec<u8>, Vec<String>, bool)> {
    let mut options = pdfcer_3d::GltfOptions::default();
    if !placement.gltf_linear {
        options.colours = pdfcer_3d::GltfColours::AsStored;
    }
    if !placement.gltf_z_up {
        options.up_axis = pdfcer_3d::GltfUpAxis::AsStored;
    }
    match super::gltf::to_embed(&placement.source, data, options) {
        Ok(done) => Some(done),
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "model-insert-gltf-refused".to_owned()
            });
            super::record_note(doc.edit_epoch, why);
            None
        }
    }
}

/// Without the `3d` feature nothing is converted.
#[cfg(not(feature = "3d"))]
fn converted(
    _doc: &OpenDoc,
    _placement: &ModelPlacement,
    data: Vec<u8>,
) -> Option<(Vec<u8>, Vec<String>, bool)> {
    Some((data, Vec::new(), false))
}

fn trace_requested(page: usize, spec: &ThreeDSpec, placement: &ModelPlacement) {
    crate::diag::trace(|| {
        let poster = if spec.render_poster {
            "rendered"
        } else {
            "plain"
        }; // ui-text-exempt: diagnostic trace
        let lighting = placement
            .lighting
            .map_or_else(|| "auto".to_owned(), |l| format!("{l:?}")); // ui-text-exempt: diagnostic trace
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "model-insert-requested page={page} format={:?} bytes={} activation={:?} poster={poster} lighting={lighting}",
            spec.format,
            spec.data.len(),
            spec.activation,
        )
    });
}

/// Add the annotation and, when there is one, its opening view: one entry.
fn place(
    doc: &mut OpenDoc,
    page: usize,
    spec: &ThreeDSpec,
    lighting: Option<ThreeDLighting>,
    (converted, was_converted): (Vec<String>, bool),
) {
    let format = spec.format.clone();
    let Ok((options, receipt)) = super::drawlayer::onto(doc, "add-3d", MarkupOptions::default())
    else {
        return;
    };
    super::apply::vector_edit(doc, "add-3d", page, 1, |session| {
        let outcome = session.add_3d_annotation(page, spec, &options)?;
        let mut notes = vec![t::inserted(&format, page)];
        notes.extend(converted);
        notes.extend(poster_note(&outcome.poster));
        let opened =
            session.set_3d_opening_view(page, outcome.annot_id, spec, lighting, was_converted)?;
        if let Some(opened) = opened {
            let (scheme, framed) = (Some(opened.lighting), opened.framed);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-insert-lit page={page} scheme={scheme:?} framed={framed} views={}",
                    opened.outcome.views_after
                )
            });
            notes.push(opened.disclosure());
            notes.extend(opened.outcome.disclosures);
        }
        if outcome.below_required_version() {
            notes.push(t::below_version(
                &outcome.required_version.to_string(),
                &outcome.document_version.to_string(),
            ));
        }
        notes.extend(receipt);
        Ok::<_, pdfcer_core::edit::EditError>(notes)
    });
}

/// The poster the engine drew, traced and worded for the status line: a
/// rendered one is an inference about the model's look and is disclosed.
fn poster_note(poster: &ThreeDPoster) -> Option<String> {
    let (drawn, reason) = match poster {
        ThreeDPoster::Supplied => ("supplied", "-"),
        ThreeDPoster::Rendered(_) => ("rendered", "-"),
        ThreeDPoster::Placeholder(why) => ("placeholder", placeholder_token(why)),
        _ => ("other", "-"),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        let uncoloured = match poster {
            ThreeDPoster::Rendered(r) => r.uncoloured_meshes,
            _ => 0,
        };
        format!("model-insert-poster drawn={drawn} reason={reason} uncoloured={uncoloured}")
    });
    match poster {
        ThreeDPoster::Rendered(r) => Some(t::poster_rendered(
            r.compressed_skipped > 0 || r.markups_skipped > 0 || r.unplaced.is_some(),
            r.uncoloured_meshes,
            r.meshes,
        )),
        ThreeDPoster::Placeholder(why) => Some(t::poster_placeholder(&why.to_string())),
        _ => None,
    }
}

/// A trace token for `why`.
fn placeholder_token(why: &PlaceholderReason) -> &'static str {
    match why {
        PlaceholderReason::Requested => "requested",
        PlaceholderReason::NotDecoded { .. } => "not-decoded",
        PlaceholderReason::NoDecoder => "no-decoder",
        PlaceholderReason::Undecodable { .. } => "undecodable",
        _ => "other",
    }
}

/// A 4:3 box, half the page's width (capped by its height), centred on it.
fn centred(page: Rect) -> Rect {
    let width = (page.urx - page.llx).abs();
    let height = (page.ury - page.lly).abs();
    let w = (width / 2.0).min(height * 0.75 * 4.0 / 3.0);
    let h = w * 0.75;
    let cx = (page.llx + page.urx) / 2.0;
    let cy = (page.lly + page.ury) / 2.0;
    Rect {
        llx: cx - w / 2.0,
        lly: cy - h / 2.0,
        urx: cx + w / 2.0,
        ury: cy + h / 2.0,
    }
}
