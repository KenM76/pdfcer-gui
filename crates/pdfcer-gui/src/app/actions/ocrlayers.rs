//! # `app::actions::ocrlayers` — OCR text layers pdfcer wrote: the re-run policy
//! and File ▸ Remove OCR text
//!
//! The engine marks every layer it writes (`pdfcer_core::ocr::marker`), so a
//! re-run can replace the old one and a removal can find it. Layers written by
//! other software carry no marker and are never touched.
//!
//! Contract:
//! - [`options`] is what [`super::Action::ApplyOcr`] writes with:
//!   `ExistingLayers::Replace`, so a re-run is one copy of every word, and the
//!   recogniser's key in the marker.
//! - [`recognised_disclosures`] adds one sentence totalling `layers_replaced`
//!   ahead of the engine's per-page lines, and traces
//!   `ocr-layer-structure lines= blocks= structure=`: the reading structure
//!   the writer laid the words out in, and whether the engine reported it or
//!   pdfcer inferred it from the word boxes.
//! - [`apply`] is [`super::Action::ApplyOcr`]: the words written on the layer
//!   (optional-content group) named [`t::group_name`], so they are a Layers
//!   panel row; the layer is reused, or made in the same undo step.
//! - [`remove`] is [`super::Action::RemoveOcrLayers`]: every marked layer the
//!   page and engine filters name comes off, and each layer (group) that
//!   leaves with nothing on it is deleted, as ONE undo entry
//!   (`CommandKind::RemoveOcrLayer`), through the funnel.
//! - `LayerPresent`, `LayerNotFound` and "none found" reach the status bar as
//!   [`crate::text::ocr::OcrLayerRefusal`] sentences.

use crate::app::unlock::Refusal;
use pdfcer_core::edit::{
    CommandKind, EditError, EditSession, LayerContentPolicy, LayerEdit, OcrPageLayer,
};
use pdfcer_core::object::ObjId;
use pdfcer_core::ocr::layer::{ExistingLayers, OcrLayerError, OcrLayerOptions, OcrLayerReport};
use pdfcer_core::ocr::{OcrPage, OcrStructureSource};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::ocr::{self as t, OcrLayerRefusal};

/// The options a recognition by `engine` is applied with.
pub(super) fn options(engine: &str) -> OcrLayerOptions {
    OcrLayerOptions::new()
        .with_existing(ExistingLayers::Replace)
        .with_engine(engine)
}

/// Why a recognition was not applied.
enum WriteError {
    Group(EditError),
    Layer(OcrLayerError),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(e) => write!(f, "{e}"),
            Self::Layer(e) => write!(f, "{e}"),
        }
    }
}

impl Refusal for WriteError {
    fn encrypted(&self) -> bool {
        match self {
            Self::Group(e) => e.encrypted(),
            Self::Layer(e) => e.encrypted(),
        }
    }
}

/// [`super::Action::ApplyOcr`]: the run's words written on the
/// [`t::group_name`] layer, made when the document has none, as one undo step.
pub(super) fn apply(doc: &mut OpenDoc, pages: &[(usize, OcrPage)], engine: &str) {
    let first = pages.first().map_or(0, |(index, _)| *index);
    super::apply::vector_edit(doc, "ocr-layer", first, pages.len(), |session| {
        write(session, pages, engine)
    });
}

fn write(
    session: &mut EditSession,
    pages: &[(usize, OcrPage)],
    engine: &str,
) -> Result<Vec<String>, WriteError> {
    let layers: Vec<OcrPageLayer<'_>> = pages
        .iter()
        .map(|(index, recognised)| OcrPageLayer {
            page_index: *index,
            recognised,
        })
        .collect();
    let mut made = false;
    let mut group = match find_group(session) {
        Some(id) => id,
        None => make_group(session, &mut made)?,
    };
    let mut written = session.add_ocr_layer(&layers, &options(engine).on_layer(group));
    // A same-named group outside `/OCGs` is not a layer the writer accepts.
    if !made && matches!(written, Err(OcrLayerError::NotALayerGroup { .. })) {
        group = make_group(session, &mut made)?;
        written = session.add_ocr_layer(&layers, &options(engine).on_layer(group));
    }
    let reports = match written {
        Ok(reports) => reports,
        Err(e) => {
            if made {
                session.undo();
            }
            word_refusal(&e);
            return Err(WriteError::Layer(e));
        }
    };
    if made && !session.coalesce_last(2, CommandKind::AddOcrLayer) {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| "ocr-layer-unfolded n=2".to_owned());
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "ocr-layer-group id={}_{} made={made}",
            group.num, group.generation
        )
    });
    let mut out = recognised_disclosures(&reports);
    if made {
        out.insert(0, t::group_made());
    }
    Ok(out)
}

/// The first layer named [`t::group_name`], if any.
fn find_group(session: &EditSession) -> Option<ObjId> {
    pdfcer_core::layers::read_layers(&session.view())
        .layers
        .into_iter()
        .find(|l| l.name == t::group_name())
        .map(|l| l.id)
}

fn make_group(session: &mut EditSession, made: &mut bool) -> Result<ObjId, WriteError> {
    let id = session
        .add_layer(t::group_name(), &LayerEdit::new())
        .map_err(WriteError::Group)?;
    *made = true;
    Ok(id)
}

/// Every page's engine disclosures, preceded by one run-wide total of the
/// earlier layers the write replaced (none when it replaced nothing).
pub(super) fn recognised_disclosures(reports: &[OcrLayerReport]) -> Vec<String> {
    let layers: usize = reports.iter().map(|r| r.layers_replaced).sum();
    let pages = reports.iter().filter(|r| r.layers_replaced > 0).count();
    crate::diag::trace(|| {
        let structure = reports
            .iter()
            .map(|r| structure_key(r.structure))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "ocr-layer-structure lines={} blocks={} structure={structure}",
            reports.iter().map(|r| r.lines_written).sum::<usize>(),
            reports.iter().map(|r| r.blocks_written).sum::<usize>(),
        )
    });
    let mut out = Vec::new();
    if layers > 0 {
        out.push(t::layers_replaced(layers, pages));
    }
    out.extend(reports.iter().flat_map(OcrLayerReport::disclosures));
    out
}

/// The trace token for where a page's lines and blocks came from.
const fn structure_key(source: OcrStructureSource) -> &'static str {
    match source {
        OcrStructureSource::Reported => "reported",
        OcrStructureSource::BlocksInferred => "blocks-inferred",
        OcrStructureSource::Inferred => "inferred",
        _ => "other",
    }
}

/// Word the refusals this shell can name; the rest take the funnel's generic
/// sentence and keep the engine's words on the trace.
pub(super) fn word_refusal(error: &OcrLayerError) {
    let why = match error {
        OcrLayerError::LayerPresent { .. } => OcrLayerRefusal::AlreadyPresent,
        OcrLayerError::LayerNotFound { .. } => OcrLayerRefusal::LayerGone,
        _ => return,
    };
    decline::record_ocr_layer(why);
}

/// Why a removal run stopped with nothing removed.
enum RemoveError {
    PageTree(pdfcer_core::page_tree::PageTreeError),
    Layer(OcrLayerError),
    NoneFound,
}

impl Refusal for RemoveError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Layer(e) if e.encrypted())
    }
}

impl std::fmt::Display for RemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PageTree(e) => write!(f, "{e}"),
            Self::Layer(e) => write!(f, "{e}"),
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::NoneFound => f.write_str("no pdfcer OCR layer in the document"),
        }
    }
}

/// File ▸ Remove OCR text: remove the layers pdfcer wrote on `pages` by
/// `engines` (`None`: no filter), as one undo step.
pub(super) fn remove(
    doc: &mut OpenDoc,
    pages: Option<&[usize]>,
    engines: Option<&[Option<String>]>,
) {
    super::apply::vector_edit(doc, "remove-ocr-layers", 0, 0, |session| {
        remove_in(session, pages, engines).inspect_err(|e| match e {
            RemoveError::NoneFound => decline::record_ocr_layer(OcrLayerRefusal::NoneFound),
            RemoveError::Layer(e) => word_refusal(e),
            RemoveError::PageTree(_) => {}
        })
    });
}

fn remove_in(
    session: &mut EditSession,
    on: Option<&[usize]>,
    by: Option<&[Option<String>]>,
) -> Result<Vec<String>, RemoveError> {
    let layers: Vec<_> = session
        .find_ocr_layers()
        .map_err(RemoveError::PageTree)?
        .into_iter()
        .filter(|l| on.is_none_or(|p| p.contains(&l.page_index)))
        .filter(|l| by.is_none_or(|e| e.contains(&l.engine)))
        .collect();
    if layers.is_empty() {
        return Err(RemoveError::NoneFound);
    }
    let mut pages: Vec<usize> = Vec::new();
    let mut emptied: Vec<ObjId> = Vec::new();
    let mut removed = 0usize;
    let mut failure = None;
    for layer in &layers {
        match session.remove_ocr_layer(layer) {
            Ok(removal) => {
                removed += 1;
                if let Some(group) = removal.optional_content.filter(|_| removal.group_emptied)
                    && !emptied.contains(&group)
                {
                    emptied.push(group);
                }
                if !pages.contains(&layer.page_index) {
                    pages.push(layer.page_index);
                }
            }
            // Nothing committed yet: an ordinary refusal.
            Err(e) if removed == 0 => return Err(RemoveError::Layer(e)),
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    let groups = delete_groups(session, &emptied);
    // Committed commands must reach the epoch bump, so a partial run is an
    // `Ok` that says what stayed. The fold is checked; a `false` leaves the
    // removals applied as separate undo steps.
    if !session.coalesce_last(removed + groups.len(), CommandKind::RemoveOcrLayer) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("remove-ocr-layers-unfolded n={removed}")
        });
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "remove-ocr-layers-applied removed={removed} pages={} groups-deleted={}",
            pages.len(),
            groups.len()
        )
    });
    let mut out = vec![match failure {
        None => t::layers_removed(removed, pages.len()),
        Some(e) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "remove-ocr-layers-partial removed={removed} of={} why={e}",
                    layers.len()
                )
            });
            t::layers_removed_partly(removed, layers.len())
        }
    }];
    if !groups.is_empty() {
        out.push(t::groups_deleted(&groups));
    }
    Ok(out)
}

/// Delete each layer the removal left with nothing on it, returning the names
/// of those deleted. A refusal leaves that layer, traced.
fn delete_groups(session: &mut EditSession, emptied: &[ObjId]) -> Vec<String> {
    let names = pdfcer_core::layers::read_layers(&session.view()).layers;
    let mut deleted = Vec::new();
    for &group in emptied {
        let name = names
            .iter()
            .find(|l| l.id == group)
            .map_or_else(String::new, |l| l.name.clone());
        match session.delete_layer(group, LayerContentPolicy::KeepUnlayered) {
            Ok(_) => deleted.push(name),
            Err(e) => crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "remove-ocr-layers-group-kept id={}_{} why={e}",
                    group.num, group.generation
                )
            }),
        }
    }
    deleted
}
