//! Several markups resized or turned by one grip drag, as one undo step: what
//! [`AnnotAction::ResizeMany`] and [`AnnotAction::RotateMany`] run.
//!
//! The engine resizes and turns one annotation per command, so this is one
//! `resize_annotation` or `rotate_annotation` per markup, every one tried, then
//! one `coalesce_last` (G202). All share one anchor or pivot, so the group
//! changes as one shape. What was applied stays applied when another is
//! refused, and the status line says how many changed. When none changed, the
//! first refusal is worded as the one-markup gesture words it.
//!
//! [`AnnotAction::ResizeMany`]: crate::app::actions::annot::AnnotAction::ResizeMany
//! [`AnnotAction::RotateMany`]: crate::app::actions::annot::AnnotAction::RotateMany

use pdfcer_core::edit::{CommandKind, EditError};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::annotmany::{ResizeMany, RotateMany};

use crate::app::actions::foldeach::{Folded, OnRefusal, fold_each};
use crate::app::state::OpenDoc;
use crate::text::markup as t;

/// Resize every markup in `many` about the shared anchor.
pub(in crate::app::actions) fn resize_many(doc: &mut OpenDoc, many: &ResizeMany) {
    let stamps = stamps_on(doc, many.page);
    let mut done = (0, false);
    super::super::apply::vector_edit(
        doc,
        "resize-annotations",
        many.page,
        many.items.len(),
        |session| {
            let folded = fold_each(
                session,
                many.items.iter().copied(),
                CommandKind::ResizeAnnotation,
                OnRefusal::Continue,
                |session, (id, anchor)| {
                    let opts = options(many.modifiers, stamps.contains(&id));
                    session
                        .resize_annotation(id, anchor, many.sx, many.sy, &opts)
                        .map(|outcome| Some(super::resize_notes(&outcome)))
                },
            )?;
            done = (folded.run.pushed, folded.folded);
            outcome(
                folded,
                false,
                many.items.len(),
                many.left_out,
                super::record_resize_refusal,
            )
        },
    );
    trace("resize", many.page, many.items.len(), many.left_out, done);
}

/// Turn every markup in `many` about the shared pivot.
pub(in crate::app::actions) fn rotate_many(doc: &mut OpenDoc, many: &RotateMany) {
    let mut done = (0, false);
    super::super::apply::vector_edit(
        doc,
        "rotate-annotations",
        many.page,
        many.items.len(),
        |session| {
            let folded = fold_each(
                session,
                many.items.iter().copied(),
                CommandKind::RotateAnnotation,
                OnRefusal::Continue,
                |session, (id, pivot)| {
                    session
                        .rotate_annotation(id, pivot, many.degrees)
                        .map(|_| Some(Vec::new()))
                },
            )?;
            done = (folded.run.pushed, folded.folded);
            outcome(folded, true, many.items.len(), many.left_out, |e| {
                crate::app::status::decline::record_rotate(super::refusal_for(e));
            })
        },
    );
    trace("rotate", many.page, many.items.len(), many.left_out, done);
}

/// The run's sentences, or its first refusal, worded by `word`, when nothing
/// changed.
fn outcome(
    folded: Folded<EditError>,
    turned: bool,
    asked: usize,
    left_out: usize,
    word: impl Fn(&EditError),
) -> Result<Vec<String>, EditError> {
    let Folded { run, folded } = folded;
    if run.pushed == 0
        && let Some(e) = run.failure
    {
        word(&e);
        return Err(e);
    }
    let mut said = run.told;
    if run.failure.is_some() {
        said.push(t::changed_some_of(turned, run.pushed, asked));
    }
    if left_out > 0 {
        said.push(t::left_out_of_group(turned, left_out));
    }
    if !folded && run.pushed > 1 {
        said.push(t::group_not_one_undo(turned).to_owned());
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

fn trace(verb: &str, page: usize, asked: usize, left_out: usize, (done, folded): (usize, bool)) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "{verb}-annotations-applied page={page} changed={done}/{asked} left_out={left_out} \
             one_step={folded}"
        )
    });
}
