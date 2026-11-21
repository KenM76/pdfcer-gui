//! # `canvas::textedit::landing` — whether the run a caret lands on can be
//! edited at all, answered before any key is typed
//!
//! Contract: [`refused`] runs two engine questions on an `Anchor::Run`, in
//! order. `RunRepertoire` empty means the font spells nothing pdfcer could put
//! back: no caret. `EditSession::edit_capability` answering
//! `EditError::Unsupported(cause)` means no edit of this run lands: no caret
//! when `UnsupportedCause::workaround` is `None`, and a caret with
//! `DraftNote::CannotEditInPlace` when finishing the edit offers one. Any other
//! answer, or a question that could not be asked, lets the caret land.

use pdfcer_core::text_edit::{EditError, EditTarget};
use pdfcer_gui_base::text::editcause::EditCause;

use super::{Anchor, Refusal};
use crate::app::state::OpenDoc;

/// The refusal for a caret on `anchor`, or the class of cause a draft opens
/// with a note for, or neither.
pub(super) fn refused(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    anchor: &Anchor,
) -> Result<Option<EditCause>, Refusal> {
    let Anchor::Run { run, .. } = anchor else {
        return Ok(None);
    };
    spells_nothing(ctx, doc, page, *run)?;
    capability(doc, page, *run)
}

/// `Err(NoUsableEncoding)` when the run's measured repertoire is empty.
///
/// `of_run` answering `None` means the run could not be pinned or the engine
/// would not answer, and that lets the caret land: refusing on `None` would
/// make every run the engine cannot address untypeable.
fn spells_nothing(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    run: usize,
) -> Result<(), Refusal> {
    let Some(rep) = super::repertoire::of_run(ctx, doc, page, run) else {
        return Ok(());
    };
    if rep.is_editable() {
        return Ok(());
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        // The engine's `reason` is in its own vocabulary; the operator gets
        // `text::textedit::refusal` instead.
        format!(
            "text-edit-refused reason=no-usable-encoding page={page} run={run} font={} said={}",
            rep.base_font,
            rep.reason.as_deref().unwrap_or("none"),
        )
    });
    Err(Refusal::NoUsableEncoding)
}

/// The engine's `edit_capability` answer for the run's show operator.
///
/// A run reached through a form XObject is not asked: `edit_capability`
/// takes a page span and plans against the page's own contents, so its
/// answer for a form's span would be about other bytes.
fn capability(doc: &OpenDoc, page: usize, run: usize) -> Result<Option<EditCause>, Refusal> {
    let Some(pin) = super::pin::resolve(doc, page, run) else {
        return Ok(None);
    };
    if matches!(pin.target, EditTarget::Form { .. }) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-capability page={page} run={run} asked=0 target=form")
        });
        return Ok(None);
    }
    let started = std::time::Instant::now();
    let answer = doc.session.edit_capability(page, pin.span);
    let cause = match &answer {
        Err(EditError::Unsupported(cause)) => Some(cause),
        _ => None,
    };
    let class = cause.map(EditCause::of);
    let offered = cause.is_some_and(|c| c.workaround().is_some());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-capability page={page} run={run} asked=1 ms={} ok={} cause={} offered={}",
            started.elapsed().as_millis(),
            u8::from(answer.is_ok()),
            class.map_or("none", EditCause::name),
            u8::from(offered),
        )
    });
    match class {
        Some(class) if !offered => Err(Refusal::CannotEdit(class)),
        other => Ok(other),
    }
}
