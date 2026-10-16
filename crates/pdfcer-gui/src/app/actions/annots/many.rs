//! Several markups resized or turned by one grip drag, as one undo step: what
//! [`AnnotAction::ResizeMany`] and [`AnnotAction::RotateMany`] run.
//!
//! One `EditSession::resize_annotations` or `rotate_annotations` call about
//! the selection box's shared anchor or pivot, so the group changes as one
//! shape. The engine tries every member, keeps what applied when another is
//! refused, and folds the applied ones into one undo entry; the status line
//! says how many changed. When none changed, the first refusal is worded as
//! the one-markup gesture words it.
//!
//! [`AnnotAction::ResizeMany`]: crate::app::actions::annot::AnnotAction::ResizeMany
//! [`AnnotAction::RotateMany`]: crate::app::actions::annot::AnnotAction::RotateMany

use pdfcer_core::edit::{AnnotationBatchOutcome, EditError};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::annotmany::{ResizeMany, RotateMany};

use crate::app::state::OpenDoc;
use crate::text::markup as t;

/// Resize every markup in `many` about the shared anchor.
pub(in crate::app::actions) fn resize_many(doc: &mut OpenDoc, many: &ResizeMany) {
    let stamps = stamps_on(doc, many.page);
    let targets: Vec<_> = many
        .ids
        .iter()
        .map(|&id| (id, options(many.modifiers, stamps.contains(&id))))
        .collect();
    let mut done = 0;
    super::super::apply::vector_edit(
        doc,
        "resize-annotations",
        many.page,
        many.ids.len(),
        |session| {
            let batch = session.resize_annotations(&targets, many.anchor, many.sx, many.sy);
            let batch = batch.inspect_err(super::record_resize_refusal)?;
            done = batch.applied();
            let told = ok_results(&batch)
                .flat_map(super::resize_notes)
                .collect::<Vec<_>>();
            outcome(
                &batch,
                told,
                false,
                many.left_out,
                super::record_resize_refusal,
            )
        },
    );
    trace("resize", many.page, many.ids.len(), many.left_out, done);
}

/// Turn every markup in `many` about the shared pivot.
pub(in crate::app::actions) fn rotate_many(doc: &mut OpenDoc, many: &RotateMany) {
    let mut done = 0;
    let word = |e: &EditError| crate::app::status::decline::record_rotate(super::refusal_for(e));
    super::super::apply::vector_edit(
        doc,
        "rotate-annotations",
        many.page,
        many.ids.len(),
        |session| {
            let batch = session.rotate_annotations(&many.ids, many.pivot, many.degrees);
            let batch = batch.inspect_err(word)?;
            done = batch.applied();
            outcome(&batch, Vec::new(), true, many.left_out, word)
        },
    );
    trace("rotate", many.page, many.ids.len(), many.left_out, done);
}

fn ok_results<R>(batch: &AnnotationBatchOutcome<R>) -> impl Iterator<Item = &R> {
    batch.results.iter().filter_map(|(_, r)| r.as_ref().ok())
}

/// The batch's sentences, or its first refusal, worded by `word`, when nothing
/// changed.
fn outcome<R>(
    batch: &AnnotationBatchOutcome<R>,
    mut said: Vec<String>,
    turned: bool,
    left_out: usize,
    word: impl Fn(&EditError),
) -> Result<Vec<String>, EditError> {
    let applied = batch.applied();
    if applied == 0
        && let Some((_, e)) = batch.refused().next()
    {
        word(e);
        return Err(e.clone());
    }
    if batch.refused().next().is_some() {
        said.push(t::changed_some_of(turned, applied, batch.results.len()));
    }
    if left_out > 0 {
        said.push(t::left_out_of_group(turned, left_out));
    }
    Ok(said)
}

/// The stamps on `page`: a stamp's picture scales with its box, so both
/// switches are set for it, as the one-markup resize sets them.
fn stamps_on(doc: &OpenDoc, page: usize) -> Vec<ObjId> {
    doc.pages.get(page).map_or_else(Vec::new, |p| {
        pdfcer_core::annot::page_annotations(&doc.session.graph(), p.id)
            .into_iter()
            .filter(|a| a.subtype.as_slice() == b"Stamp")
            .filter_map(|a| a.id)
            .collect()
    })
}

fn options(
    modifiers: crate::canvas::scaling::Modifiers,
    stamp: bool,
) -> pdfcer_core::edit::ResizeOptions {
    let opts = modifiers.to_options();
    if stamp {
        opts.with_scale_stroke_width(true)
            .with_allow_appearance_distortion(true)
    } else {
        opts
    }
}

/// `one_step` is whether anything changed: the engine folds every applied
/// member into one undo entry.
fn trace(verb: &str, page: usize, asked: usize, left_out: usize, done: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "{verb}-annotations-applied page={page} changed={done}/{asked} left_out={left_out} \
             one_step={}",
            done > 0
        )
    });
}
