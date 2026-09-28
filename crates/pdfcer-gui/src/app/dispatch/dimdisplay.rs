//! # `app::dispatch::dimdisplay` — show a circular ce dimension as a radius or
//! a diameter, from the canvas menu

use pdfcer_core::dimension::{DimensionId, DimensionKind};

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::{AnnotKind, SelectionState};

/// Switch the selected circular ce dimension to its diameter.
pub(crate) const DIAMETER: &str = "format.dimension_diameter"; // ui-text-exempt: a command id, never displayed

/// Switch the selected circular ce dimension to its radius.
pub(crate) const RADIUS: &str = "format.dimension_radius"; // ui-text-exempt: a command id, never displayed

/// The selected ce dimension and whether it shows its diameter, when the
/// selection is one circular ce dimension. Read from the session's model, so
/// it includes unsaved edits.
///
/// `selection` is passed rather than read from `doc`: while the canvas runs its
/// frame it holds the selection taken off the document, and `doc.selection` is
/// then empty.
#[must_use]
pub(crate) fn circular(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, bool)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    match record.kind {
        DimensionKind::Circular { show_diameter, .. } => Some((record.id, show_diameter)),
        _ => None,
    }
}

/// Raise the switch. A chord or a stale menu can arrive with no circular ce
/// dimension selected, or with it already showing the asked-for measure; both
/// are traced and raise nothing.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    if !app.capabilities().author_measure {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-author-measure")
        });
        return;
    }
    let Status::Open(doc) = &app.status else {
        return;
    };
    let want = match id {
        DIAMETER => true,
        RADIUS => false,
        _ => return,
    };
    match circular(doc, &doc.selection) {
        Some((dimension, shown)) if shown != want => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "dimension-display id={} diameter={}",
                    dimension.0,
                    u8::from(want)
                )
            });
            actions.push(Action::Dimension(DimensionAction::SetDisplay {
                dimension,
                show_diameter: want,
            }));
        }
        _ => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-circular-dimension-to-switch")
        }),
    }
}
