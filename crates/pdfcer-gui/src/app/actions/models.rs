//! Placing a 3D model on a page, and saving an embedded one's data out to a file.

use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::threed::{
    PlaceholderReason, ThreeDArtwork, ThreeDFormat, ThreeDPoster, ThreeDSpec, extract_3d,
    list_3d_with_notes,
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

/// What [`mesh_bytes`] decoded.
#[cfg(feature = "3d")]
struct Meshed {
    bytes: Vec<u8>,
    meshes: usize,
    triangles: usize,
    skipped: usize,
    placed: bool,
}

/// A PRC model's triangles, each part placed where its assembly puts it.
#[cfg(feature = "3d")]
pub(crate) struct Assembled {
    /// One mesh per placed part.
    pub meshes: Vec<pdfcer_3d::TriangleMesh>,
    pub triangles: usize,
    /// Tessellations that are not triangles pdfcer can rebuild.
    pub skipped: usize,
    /// `false` when the assembly tree could not be read and each mesh is
    /// where its file stores it.
    pub placed: bool,
}

/// Why [`assemble`] produced nothing to draw or save.
#[cfg(feature = "3d")]
pub(crate) enum Unassembled {
    NotPrc,
    Unreadable(String),
    /// No triangles; `compressed` of the parts were compressed meshes the
    /// engine could not rebuild.
    Empty {
        compressed: usize,
    },
}

/// Decode a PRC model and place its parts, as the engine's `3d-render` does.
#[cfg(feature = "3d")]
pub(crate) fn assemble(data: &[u8]) -> Result<Assembled, Unassembled> {
    use pdfcer_3d::{PrcFile, Tessellation};
    if !data.starts_with(b"PRC") {
        return Err(Unassembled::NotPrc);
    }
    let prc = PrcFile::parse(data).map_err(|e| Unassembled::Unreadable(e.to_string()))?;
    let (mut skipped, mut compressed) = (0usize, 0usize);
    let mut by_index = Vec::with_capacity(prc.file_structures.len());
    for structure in &prc.file_structures {
        let found = structure
            .tessellations()
            .map_err(|e| Unassembled::Unreadable(e.to_string()))?;
        let row: Vec<_> = found
            .into_iter()
            .map(|tessellation| match tessellation {
                Tessellation::Mesh(mesh)
                | Tessellation::Compressed {
                    mesh: Some(mesh), ..
                } => Some(mesh),
                Tessellation::Compressed { mesh: None, .. } => {
                    compressed += 1;
                    skipped += 1;
                    None
                }
                _ => {
                    skipped += 1;
                    None
                }
            })
            .collect();
        by_index.push(row);
    }
    let mut meshes = Vec::new();
    let placements = prc.placements().unwrap_or_default();
    for p in &placements {
        let mesh = by_index
            .get(p.file_structure)
            .and_then(|row: &Vec<Option<pdfcer_3d::TriangleMesh>>| row.get(p.tessellation))
            .and_then(Option::as_ref);
        if let Some(mesh) = mesh {
            meshes.push(mesh.transformed(&p.matrix));
        }
    }
    let placed = !meshes.is_empty();
    if !placed {
        meshes = by_index.into_iter().flatten().flatten().collect();
    }
    let triangles: usize = meshes.iter().map(|m| m.triangles.len()).sum();
    if triangles == 0 {
        return Err(Unassembled::Empty { compressed });
    }
    Ok(Assembled {
        meshes,
        triangles,
        skipped,
        placed,
    })
}

/// Decode a PRC model's placed triangle meshes to STL, or OBJ when `obj`.
/// The error is the sentence to show.
#[cfg(feature = "3d")]
fn mesh_bytes(data: &[u8], obj: bool) -> Result<Meshed, String> {
    let model = assemble(data).map_err(|why| match why {
        Unassembled::NotPrc => t::mesh_not_prc().to_owned(),
        Unassembled::Unreadable(detail) => t::mesh_unreadable(&detail),
        Unassembled::Empty { compressed } => t::mesh_empty(compressed),
    })?;
    let bytes = if obj {
        pdfcer_3d::to_obj(&model.meshes).into_bytes()
    } else {
        pdfcer_3d::to_stl(&model.meshes).map_err(|e| t::mesh_unreadable(&e.to_string()))?
    };
    Ok(Meshed {
        bytes,
        meshes: model.meshes.len(),
        triangles: model.triangles,
        skipped: model.skipped,
        placed: model.placed,
    })
}

/// Re-list `artwork`, read its data and assemble it for the viewer. The
/// error is the sentence to show.
#[cfg(feature = "3d")]
pub(crate) fn load_view(doc: &OpenDoc, artwork: &ThreeDArtwork) -> Result<Assembled, String> {
    if !list_3d_with_notes(&*doc.session).0.contains(artwork) {
        return Err(t::gone().to_owned());
    }
    let data = extract_3d(&doc.session.view(), artwork)
        .map_err(|e| t::extract_failed(&e.to_string()))?
        .data;
    assemble(&data).map_err(|why| match why {
        Unassembled::NotPrc => t::view_not_prc().to_owned(),
        Unassembled::Unreadable(detail) => t::mesh_unreadable(&detail),
        Unassembled::Empty { compressed } => t::view_empty(compressed),
    })
}

/// Decode `artwork` and write its triangles as STL or OBJ, chosen by the
/// picked file's ending.
#[cfg(feature = "3d")]
pub(super) fn save_mesh(doc: &mut OpenDoc, artwork: &ThreeDArtwork) {
    let epoch = doc.edit_epoch;
    let data = if list_3d_with_notes(&*doc.session).0.contains(artwork) {
        extract_3d(&doc.session.view(), artwork)
            .map(|e| e.data)
            .map_err(|e| t::extract_failed(&e.to_string()))
    } else {
        Err(t::gone().to_owned())
    };
    let mut suggested = doc.path.clone();
    let stem = doc
        .path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    suggested.set_file_name(format!(
        "{}.stl",
        t::suggested_stem(&stem, artwork.page_index)
    ));
    let data = match data {
        Ok(data) => data,
        Err(said) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("mesh-save-declined page={}", artwork.page_index)
            });
            super::record_note(epoch, said);
            return;
        }
    };
    let crate::app::files::Picked::Path(target) = crate::app::files::pick_mesh_target(&suggested)
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "mesh-save-cancelled".to_owned()
        });
        return;
    };
    let obj = target
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("obj"));
    let meshed = match mesh_bytes(&data, obj) {
        Ok(meshed) => meshed,
        Err(said) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "mesh-save-refused".to_owned()
            });
            super::record_note(epoch, said);
            return;
        }
    };
    match std::fs::write(&target, &meshed.bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "mesh-saved obj={obj} bytes={} meshes={} triangles={} skipped={} placed={}",
                    meshed.bytes.len(),
                    meshed.meshes,
                    meshed.triangles,
                    meshed.skipped,
                    meshed.placed
                )
            });
            let mut notes = vec![
                t::mesh_saved(
                    &target.display().to_string(),
                    meshed.meshes,
                    meshed.triangles,
                ),
                if meshed.placed {
                    t::mesh_placed_note()
                } else {
                    t::mesh_placement_note()
                }
                .to_owned(),
            ];
            if meshed.skipped > 0 {
                notes.push(t::mesh_skipped(meshed.skipped));
            }
            super::record_edit_disclosure(Some(super::EditDisclosure { epoch, notes }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("mesh-save-failed kind={:?}", error.kind())
            });
            super::record_note(epoch, t::save_failed(&error.to_string()));
        }
    }
}

/// Without the `3d` feature the button is not drawn, so this is unreachable
/// from the panel.
#[cfg(not(feature = "3d"))]
pub(super) fn save_mesh(_doc: &mut OpenDoc, _artwork: &ThreeDArtwork) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        "mesh-save-declined reason=built-without-3d".to_owned()
    });
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
                notes.extend(poster_note(&outcome.poster));
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
        format!("model-insert-poster drawn={drawn} reason={reason}")
    });
    match poster {
        ThreeDPoster::Rendered(r) => Some(
            t::poster_rendered(
                r.compressed_skipped > 0 || r.wires_skipped > 0 || r.unplaced.is_some(),
            )
            .to_owned(),
        ),
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

    /// **The engine's square fixture meshes to a well-formed binary STL**, and
    /// a non-PRC model is refused rather than written empty.
    #[cfg(feature = "3d")]
    #[test]
    fn a_prc_square_meshes_and_u3d_is_refused() {
        let Ok(square) = std::fs::read("D:/Dev/pdfcer/fixtures/synthetic/prc/square.prc") else {
            return; // The engine corpus is absent on this machine.
        };
        let stl = super::mesh_bytes(&square, false).expect("the square decodes");
        assert!(stl.triangles > 0);
        let count = u32::from_le_bytes(stl.bytes[80..84].try_into().expect("four bytes"));
        assert_eq!(count as usize, stl.triangles);
        assert_eq!(stl.bytes.len(), 84 + 50 * stl.triangles);
        let obj = super::mesh_bytes(&square, true).expect("the square decodes");
        assert!(String::from_utf8_lossy(&obj.bytes).contains("\nf "));
        assert!(super::mesh_bytes(b"U3D\0 not a prc", false).is_err());
    }

    /// **An assembly's parts are placed**, so its two copies of one part do
    /// not land on top of each other.
    #[cfg(feature = "3d")]
    #[test]
    fn an_assembly_is_placed_part_by_part() {
        let Ok(model) = std::fs::read("D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc") else {
            return; // The engine corpus is absent on this machine.
        };
        let Ok(assembled) = super::assemble(&model) else {
            panic!("the assembly decodes");
        };
        assert!(assembled.placed);
        assert!(
            assembled.meshes.len() >= 2,
            "{} parts",
            assembled.meshes.len()
        );
        let bounds = |m: &pdfcer_3d::TriangleMesh| pdfcer_3d::Bounds::of(std::slice::from_ref(m));
        assert_ne!(
            bounds(&assembled.meshes[0]),
            bounds(&assembled.meshes[1]),
            "two placed parts sit in the same place"
        );
    }

    /// A compressed PRC mesh the engine rebuilds is saved with the rest.
    #[cfg(feature = "3d")]
    #[test]
    fn a_compressed_prc_triangle_is_rebuilt() {
        let Ok(model) =
            std::fs::read("D:/Dev/pdfcer/fixtures/synthetic/prc/compressed_triangle.prc")
        else {
            return; // The engine corpus is absent on this machine.
        };
        let stl = super::mesh_bytes(&model, false).expect("the compressed triangle rebuilds");
        assert_eq!(stl.triangles, 1);
        assert_eq!(stl.skipped, 0);
        assert_eq!(stl.bytes.len(), 84 + 50);
    }
}
