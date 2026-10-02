//! # `app::status::decline::canvas` — what a refused canvas gesture may say
//!
//!
//! ## Why a file and not two more functions in `record`
//!
//! `record.rs`'s own header states its seam: it holds *“a family of twenty-odd
//! functions whose whole content is a one-line write plus the argument for
//! where it is called from”*, and it holds them because that argument is the
//! same argument twenty times over.
//!
//! This is the other kind. `clipboard.rs` and `textedit.rs` were cut out of
//! that family for carrying **an argument of their own**, and so does this: the
//! question here is not *which phase writes* but **what a surface holding
//! `&OpenDoc` is permitted to assert**, which is a question about the shape of
//! the boundary rather than about timing. Among twenty one-line writers it
//! would be unfindable.
//!
//! ## The whole mechanism in four steps
//!
//! 1. a gesture in `crate::canvas` refuses, holding `&OpenDoc` and no `&mut`;
//! 2. it raises [`crate::app::actions::Action::DeclineOnCanvas`] carrying one
//!    [`CanvasDecline`];
//! 3. the apply phase calls [`record_canvas`], the only mapping from that
//!    vocabulary to [`super::Declined`];
//! 4. the status bar reads it through `super::live`, which applies the
//!    retirement filter.
//!
//! ⇒ Nothing here writes a sentence. The wording lives in `crate::text::*`
//! catalogs, per this crate's standing rule, and the two arms below name which
//! catalog entry rather than quoting one.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/canvas.md`.

pub use pdfcer_gui_base::subactions::CanvasDecline;

/// Record what a refused **canvas gesture** had to say — O188, 2026-09-15.
///
/// The one writer behind [`crate::app::actions::Action::DeclineOnCanvas`], and
/// the only place [`CanvasDecline`]'s arms become [`super::Declined`]s.
///
/// # Why this is an apply arm, when [`super::record_inside_form`] argues
/// # at length for the dispatcher
///
/// Because the canvas is not the dispatcher and cannot be made into one. That
/// function's argument is about a *command* refusing, where a dispatcher already
/// holds the answer and an apply phase would have to re-derive it — and both of
/// its callers are exactly that. A canvas **gesture** is not dispatched at all:
/// it happens under `&OpenDoc`, the store is written under `&mut`, and the
/// action is how a read-only surface asks. This is where the ask lands.
///
/// Deliberately not folded into [`super::record_inside_form`], which keeps
/// both of its callers. One of them is `app::dispatch::format` writing directly
/// from a place that has established `NoContainingForm`; the canvas cannot
/// establish that, and folding the two would put a dispatcher's write behind a
/// name that says *canvas*.
///
/// **A selection is still not an edit** — no `vector_edit`, no epoch bump,
/// no cache invalidation. Nothing about the document changed; a sentence was
/// written down.
///
/// # It traces, and the trace is the middle link of a three-stage chain
///
/// The status bar publishes one region for every decline in the application,
/// `status-group:decline`. A driven check that asserted only *that region
/// drew* would be satisfied by a build that raised the **wrong** sentence, and
/// equally by one that left a stale sentence on the bar from an earlier
/// gesture. An assertion both outcomes satisfy is not a measurement of which
/// one shipped.
///
/// So the refusal is observable at three points, in three subsystems, and a
/// build that breaks any one link fails at that link:
///
/// ```text
/// canvas       canvas-move-declined level=Part sel=1 reason=run-has-no-position
/// apply phase  canvas-decline-recorded what=text-run-no-position-of-its-own
/// status bar   ui-rect name=status-group:decline
/// ```
///
/// That is not the application agreeing with itself. The first line is
/// written by `crate::canvas` holding `&OpenDoc`; this one by the apply phase
/// holding `&mut`, on the far side of the `Action` boundary the design exists
/// to keep; the third by the status bar on a later frame. A shell that raised
/// the action and never applied it writes the first and not the second.
///
/// Once per decline, not once per frame. [`super::show`] runs 60 times a
/// second and a line there would bury the channel — the lesson `canvas-pointer`
/// taught, and the reason `canvas::moving::drag` traces its refusal only on
/// release.
pub(crate) fn record_canvas(what: CanvasDecline) {
    let declined = match what {
        CanvasDecline::InsideFormNotAPath => {
            super::Declined::InsideForm(crate::text::status::InsideFormRefusal::NotAPath)
        }
        CanvasDecline::TextRunHasNoPositionOfItsOwn => {
            super::Declined::TextRunHasNoPositionOfItsOwn
        }
        CanvasDecline::TextRunWouldDragTheNextLine => super::Declined::TextRunWouldDragTheNextLine,
        CanvasDecline::TextClick(why) => super::Declined::TextClick(why),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-decline-recorded what={}", what.token())
    });
    super::LAST.with_borrow_mut(|slot| *slot = Some(declined));
}
