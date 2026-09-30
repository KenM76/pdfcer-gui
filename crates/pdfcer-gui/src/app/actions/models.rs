//! Placing a 3D model on a page, and saving an embedded one's data out to a file.

use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::threed::{
    ThreeDArtwork, ThreeDFormat, ThreeDSpec, extract_3d, list_3d_with_notes,
};

use crate::app::state::OpenDoc;
use crate::text::panels::models as t;

/// Write `artwork`'s data, as stored, to a file the operator picks.
///
/// The row is re-listed first and acted on only if it is still identical, so
/// an edit since the click is refused rather than saving a different model.
pub(super) fn save(doc: &mut OpenDoc, artwork: &ThreeDArtwork) {
    let epoch = doc.edit_epoch;
    let found = if list_3d_with_notes(&*doc.session).0.contains(artwork) {
        extract_3d(&doc.session.view(), artwork).map_err(|e| t::extract_failed(&e.to_string()))
    } else {
        Err(t::gone().to_owned())
    };
    let extracted = match found {
        Ok(extracted) => extracted,
        Err(said) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("model-save-declined page={}", artwork.page_index)
            });
            super::record_note(epoch, said);
            return;
        }
    };

    let format = extracted.sniffed.as_ref().or(artwork.declared.as_ref());
    let extension = format.map_or("bin", ThreeDFormat::extension);
    let stem = doc
        .path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let mut suggested = doc.path.clone();
    suggested.set_file_name(format!(
        "{}.{extension}",
        t::suggested_stem(&stem, artwork.page_index)
    ));

    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_attachment_target(&suggested)
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "model-save-cancelled".to_owned()
        });
        return;
    };

    match std::fs::write(&target, &extracted.data) {
        Ok(()) => {
            let contradicts = extracted.contradicts(artwork.declared.as_ref());
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-saved bytes={} ext={extension} contradicts={contradicts}",
                    extracted.data.len()
                )
            });
            let mut notes = vec![t::saved(&target.display().to_string())];
            match (&extracted.sniffed, &artwork.declared) {
                (Some(found), Some(declared)) if contradicts => {
                    notes.push(t::mismatch(&declared.label(), &found.label()));
                }
                (None, _) => notes.push(t::unrecognised().to_owned()),
                _ => {}
            }
            super::record_edit_disclosure(Some(super::EditDisclosure { epoch, notes }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("model-save-failed kind={:?}", error.kind())
            });
            super::record_note(epoch, t::save_failed(&error.to_string()));
        }
    }
}

/// Pick a U3D or PRC file and place it, centred, on `page`: one undo entry.
pub(super) fn insert(doc: &mut OpenDoc, page: usize) {
    let crate::app::files::Picked::Path(source) = crate::app::files::pick_model_source() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "model-insert-cancelled".to_owned()
        });
        return;
    };
    let data = match std::fs::read(&source) {
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
    let Some(crop) = doc.pages.get(page).map(|p| p.crop_box) else {
        return;
    };
    let spec = match ThreeDSpec::new(centred(crop), data) {
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
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "model-insert-requested page={page} format={:?} bytes={}",
            spec.format,
            spec.data.len()
        )
    });
    let format = spec.format.clone();
    super::apply::vector_edit(doc, "add-3d", page, 1, |session| {
        session
            .add_3d_annotation(page, &spec, &MarkupOptions::default())
            .map(|outcome| {
                let mut notes = vec![t::inserted(&format, page)];
                if outcome.below_required_version() {
                    notes.push(t::below_version(
                        &outcome.required_version.to_string(),
                        &outcome.document_version.to_string(),
                    ));
                }
                notes
            })
    });
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

#[cfg(test)]
mod tests {
    use pdfcer_core::edit::{EditSession, MarkupOptions};
    use pdfcer_core::page_tree::Rect;
    use pdfcer_core::threed::{ThreeDFormat, ThreeDSpec, extract_3d, list_3d_with_notes};

    /// **A model put in is listed once and reads back byte for byte** — the
    /// pair the section and the save verb rely on.
    #[test]
    fn an_inserted_model_is_listed_and_extracts_unchanged() {
        let (doc, _pages) = crate::app::blank::document().expect("the template parses");
        let mut session = EditSession::new(doc);
        let data = b"PRC\x08\x00 not a real model".to_vec();
        let rect = Rect {
            llx: 72.0,
            lly: 72.0,
            urx: 288.0,
            ury: 288.0,
        };
        let spec = ThreeDSpec::new(rect, data.clone()).expect("the PRC magic is recognised");
        session
            .add_3d_annotation(0, &spec, &MarkupOptions::default())
            .expect("a blank page takes a model");

        let (listed, notes) = list_3d_with_notes(&session);
        assert_eq!(listed.len(), 1);
        assert_eq!(notes, Default::default());
        assert_eq!(listed[0].declared, Some(ThreeDFormat::Prc));
        let extracted = extract_3d(&session.view(), &listed[0]).expect("it reads back");
        assert_eq!(extracted.data, data);
        assert!(!extracted.contradicts(listed[0].declared.as_ref()));
    }
}
