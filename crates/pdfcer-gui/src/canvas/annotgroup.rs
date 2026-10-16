//! A grip or rotate drag with several annotations selected: every unlocked
//! markup on the drag's page scales or turns about the **selection box's**
//! corner or centre, so the group changes as one shape, which is what grips
//! drawn on the union box promise. The others are counted for the status line.
//!
//! `None` when fewer than two annotations are selected: the one-annotation arm
//! of the gesture owns that case.

use pdfcer_core::object::ObjId;
use pdfcer_gui_base::annotmany::{ResizeMany, RotateMany};

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The markups on `page` that can change, and how many selected annotations
/// cannot.
fn split(selection: &SelectionState, page: usize) -> Option<(Vec<ObjId>, usize)> {
    let annots = selection.annots();
    if annots.len() < 2 {
        return None;
    }
    let items: Vec<_> = annots
        .iter()
        .filter(|a| a.target.page == page && a.target.kind == AnnotKind::Markup && !a.target.locked)
        .map(|a| a.target.id)
        .collect();
    let left_out = annots.len() - items.len();
    (!items.is_empty()).then_some((items, left_out))
}

/// The resize of the group about `anchor`, in PDF page space.
#[must_use]
pub fn resize(
    selection: &SelectionState,
    page: usize,
    anchor: (f64, f64),
    (sx, sy): (f32, f32),
    modifiers: crate::canvas::scaling::Modifiers,
) -> Option<Action> {
    let (ids, left_out) = split(selection, page)?;
    trace(
        &format!("resize sx={sx:.4} sy={sy:.4}"), // ui-text-exempt: diagnostic trace measures
        page,
        ids.len(),
        left_out,
    );
    Some(Action::Annot(AnnotAction::ResizeMany(ResizeMany {
        page,
        ids,
        anchor,
        sx: f64::from(sx),
        sy: f64::from(sy),
        uniform: (sx - sy).abs() <= f32::EPSILON,
        modifiers,
        left_out,
    })))
}

/// The turn of the group about `pivot`, in PDF page space.
#[must_use]
pub fn rotate(
    selection: &SelectionState,
    page: usize,
    pivot: (f64, f64),
    degrees: f64,
) -> Option<Action> {
    let (ids, left_out) = split(selection, page)?;
    trace(
        &format!("rotate deg={degrees:.2}"), // ui-text-exempt: diagnostic trace measures
        page,
        ids.len(),
        left_out,
    );
    Some(Action::Annot(AnnotAction::RotateMany(RotateMany {
        page,
        ids,
        pivot,
        degrees,
        left_out,
    })))
}

/// `what` is the verb, then its measures.
fn trace(what: &str, page: usize, n: usize, left_out: usize) {
    let (verb, measures) = what.split_once(' ').unwrap_or((what, ""));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("{verb}-annots-commit page={page} items={n} left_out={left_out} {measures}")
    });
}
