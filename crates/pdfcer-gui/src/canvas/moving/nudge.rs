//! # `canvas::moving::nudge` — the arrow keys, and the one point they move
//!
//! Precedence is Delete's (`canvas::deleting::route`): selected form fields,
//! then selected annotations, then page content. One unlocked markup moves by
//! `AnnotAction::Move`; several annotations, a ce dimension, or form fields
//! move by one `VectorAction::MoveMixed` with gesture `nudge`, as a group drag
//! does; page content moves by the action a pointer drag of the same
//! displacement commits ([`super::drag`] at [`Phase::Complete`]), with that
//! drag's refusals.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/moving/nudge.md`.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. A keystroke is not a drag, and five
//! of the nine rows are about the interval between press and release that a
//! keystroke does not have. Each is waived with its reason rather than being
//! skipped, because a waiver that is not written down is indistinguishable from
//! an oversight — which is the whole argument of `tools/gates/conventions.list`.
//!
//! - D1 live-preview: WAIVED — there is no interval to preview. The press *is*
//!   the commit; the mark is redrawn one point over, which is the whole change.
//!   A ghost would be a preview of a thing that has already happened.
//! - D2 derived-from-commit: satisfied trivially and for the same reason — there
//!   is one code path, and it is the commit. Nothing is drawn that the commit
//!   did not do.
//! - D3 escape-cancels: WAIVED — there is nothing in flight for Escape to
//!   cancel. Escape's six claimants are listed in [`crate::canvas::keys`] and a
//!   nudge is not among them; the way back is `Ctrl+Z`, which is what every
//!   editor offers for a completed edit.
//! - D4 one-gesture-one-undo: satisfied. One press raises exactly one move
//!   action, which the engine records as one command. See the section above on auto-repeat, which is the one place this
//!   row needed a decision rather than an observation.
//! - D5 modifiers-constrain: **satisfied by having nothing to constrain.** An
//!   arrow is one axis by construction, so Shift is deliberately unbound here
//!   rather than given the drawing programs' 10× meaning — the argument is in
//!   full above, and it is the row this module thought hardest about.
//! - D6 snapping: WAIVED — a nudge is a fixed step in the page's own units, not
//!   a pointer looking for something to land on. Snapping the *result* to a grid
//!   would make the step size depend on where the mark already was, which is the
//!   opposite of what an operator asks a nudge for.
//! - D7 no-op-is-not-an-edit: satisfied by construction. Every accepted press
//!   has a non-zero step, so there is no zero-travel case to filter; a press
//!   that refuses raises no action at all.
//! - D8 grab-point: WAIVED — there is no grab. The delta is relative, so the
//!   mark keeps its relationship to everything around it, which is what this row
//!   protects.
//! - D9 disclosure: **satisfied, and it is the row this module adds to.** A
//!   translation changes no measured value and the operator can see where the
//!   mark went, exactly as [`super`] argues for a drag — but a *refused* nudge
//!   shows nothing at all, so every refusal puts a sentence on the status row
//!   through [`crate::text::arrange`]. A drag that refuses has a pointer the
//!   operator can see not working; a key that refuses has nothing.

use egui::Key;
use pdfcer_core::page_tree::Page;

use crate::app::actions::annot::AnnotAction;
use crate::app::actions::{Action, move_items};
use crate::app::modes::Capabilities;
use crate::app::state::SelectedField;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{AnnotKind, AnnotSelection, SelectionState};
use crate::panels::objects::provider::ObjectModelProvider;
use crate::text::arrange::NudgeRefusal;

/// **The bare-arrow step, in PDF points.**
pub(crate) const STEP_PT: f32 = 1.0;

/// **The Ctrl-arrow step, in PDF points.**
pub(crate) const FINE_STEP_PT: f32 = 0.25;

/// The trace line an annotation nudge writes.
///
/// Suffixed rather than bare, per `tools/gates/check-trace-names.py`: the edit
/// funnel writes `move-annotation …` for the same edit, and two lines sharing a
/// first token means `ui-verify`'s `last(name)` returns the wrong one.
// ui-text-exempt: diagnostic trace name, never displayed
const TRACE: &str = "annot-nudge";

// ui-text-exempt: diagnostic gesture name, never displayed
const GESTURE: &str = "nudge";

/// **Which step this frame's modifiers ask for**, or `None` for a modifier
/// shape this module does not claim.
#[must_use]
fn step_for(modifiers: egui::Modifiers) -> Option<f32> {
    if modifiers.shift || modifiers.alt {
        // Both are spoken for — Shift by the axis constraint, Alt by
        // `pages.move_up`. See the module header.
        return None;
    }
    if modifiers.command {
        Some(FINE_STEP_PT)
    } else {
        Some(STEP_PT)
    }
}

/// **Which direction an arrow means, in CANVAS space.**
#[must_use]
const fn direction(key: Key) -> Option<(f32, f32)> {
    match key {
        Key::ArrowUp => Some((0.0, -1.0)),
        Key::ArrowDown => Some((0.0, 1.0)),
        Key::ArrowLeft => Some((-1.0, 0.0)),
        Key::ArrowRight => Some((1.0, 0.0)),
        _ => None,
    }
}

/// Everything a nudge needs that is not the keyboard.
pub(crate) struct Frame<'a> {
    /// The page the selection is on, for the coordinate crossing. `None` for a
    /// frame with no page, which declines rather than guessing.
    pub page: Option<&'a Page>,
    /// The index of [`Self::page`], for a page-content move.
    pub page_index: usize,
    /// The page's decomposition, which a page-content move needs exactly as a
    /// drag does. `canvas::modelneed` asks for it on a frame carrying an arrow
    /// with page content selected; `None` declines as `NoObjectModel`.
    pub targets: Option<&'a ObjectModelProvider>,
    /// What the mode permits: `author_markup` for an annotation, `edit_content`
    /// for page content, matching the pointer drag's gates.
    pub caps: Capabilities,
    /// The revision on screen, for stamping a refusal's sentence. See
    /// [`crate::app::actions::record_note`] on why the *current* epoch is the
    /// right one for a non-edit.
    pub edit_epoch: u64,
    /// The selected form fields, when there are any.
    pub fields: Option<Fields<'a>>,
}

/// The selected form fields a nudge moves: the primary and those selected
/// beside it on the primary's page, as a group drag moves them.
pub(crate) struct Fields<'a> {
    /// The primary field.
    pub primary: &'a SelectedField,
    /// The fields selected beside it.
    pub set: &'a pdfcer_gui_base::fieldset::FieldSet,
    /// The primary's page, for the coordinate crossing.
    pub page: Option<&'a Page>,
}

/// **Read this frame's arrow keys and nudge the selected markup.**
pub(crate) fn keys(
    ctx: &egui::Context,
    frame: &Frame<'_>,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> usize {
    // The canvas caret owns the arrow keys while a draft is in flight. See
    // this function's own table for why the predicate is `composing` and not
    // `text_edit_focused`, and why asking it here is not a second copy of the
    // caller's guard.
    if crate::canvas::textedit::composing(ctx) {
        return 0;
    }

    let modifiers = ctx.input(|i| i.modifiers);
    let Some(step) = step_for(modifiers) else {
        return 0;
    };

    // Counted per key rather than answered yes/no, because `key_pressed`
    // includes key-repeat events and a held arrow must walk the mark rather
    // than move it once. `count_and_consume_key` does both halves in one call:
    // it reports how many presses arrived and takes them out of the queue, so
    // nothing behind this — a scroll area, a focused segment — acts on the same
    // press. `egui-shell`'s mode selector states the rule in its own words:
    // *"an arrow that moved the selector must not also scroll whatever is
    // behind it."*
    let mut raised = 0;
    for key in [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
    ] {
        let Some((ux, uy)) = direction(key) else {
            continue;
        };
        // The pattern is the modifiers actually held, not a constant. See
        // `step_for` for why a constant `NONE` would have matched `Alt+Up`.
        let count = ctx.input_mut(|i| i.count_and_consume_key(modifiers, key));
        if count == 0 {
            continue;
        }
        for _ in 0..count {
            if nudge_once(frame, selection, egui::vec2(ux * step, uy * step), actions) {
                raised += 1;
            } else {
                // A refusal is a property of the selection and the mode, not of
                // this particular press, so the remaining repeats of a held key
                // would refuse identically. Reporting once is the honest count
                // and stops a held arrow writing the same sentence forty times.
                return raised;
            }
        }
    }
    raised
}

/// One press: decide, refuse with a sentence, or raise the move.
///
/// Returns whether an action was raised, which is what lets the caller stop
/// repeating a refusal.
fn nudge_once(
    frame: &Frame<'_>,
    selection: &SelectionState,
    canvas: egui::Vec2,
    actions: &mut Vec<Action>,
) -> bool {
    if frame.caps.edit_content
        && let Some(fields) = &frame.fields
    {
        return nudge_fields(frame, fields, selection, canvas, actions);
    }
    let annots = selection.annots();
    if annots.is_empty() {
        // Nothing selected is silent: the arrow keys are pressed constantly
        // for reasons that are not about a selection.
        if selection.is_empty() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("{TRACE}-declined reason=nothing-selected")
            });
            return false;
        }
        return nudge_content(frame, selection, canvas, actions);
    }
    nudge_annots(frame, selection, &annots, canvas, actions)
}

/// The selected annotations. A locked one stays in a group's action so the
/// engine refuses it by name and the status line counts it, as a group drag
/// does; a selection with nothing unlocked refuses here, so a key and a drag
/// agree on what locked means (§12.5.3 Table 165 bit 8).
///
/// The gate is `author_markup`, not `edit_content`: Review is the markup
/// stance, and Delete's rung learned that an `edit_content` gate leaves Review
/// unable to touch a comment.
fn nudge_annots(
    frame: &Frame<'_>,
    selection: &SelectionState,
    annots: &[&AnnotSelection],
    canvas: egui::Vec2,
    actions: &mut Vec<Action>,
) -> bool {
    if annots.iter().all(|a| a.target.locked) {
        refuse(frame, selection, NudgeRefusal::Locked);
        return false;
    }
    if !frame.caps.author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{TRACE}-declined reason=mode-cannot-author-markup")
        });
        return false;
    }
    // Everything about the flip and the page's `/Rotate` is inside
    // `page_delta`; a page whose transform will not invert declines.
    let Some(delta) = frame.page.and_then(|page| super::page_delta(canvas, page)) else {
        refuse(frame, selection, NudgeRefusal::DegeneratePage);
        return false;
    };
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "{TRACE} id={} n={} dx={:.3} dy={:.3}",
            annots[0].target.id.num,
            annots.len(),
            delta.dx,
            delta.dy
        )
    });
    // One markup keeps the single verb. A ce dimension is placed by
    // `dimdrag::placed`, as its pointer drag is, which `MoveMixed` does.
    if let [one] = annots
        && matches!(one.target.kind, AnnotKind::Markup)
    {
        actions.push(Action::Annot(AnnotAction::Move {
            id: one.target.id,
            dx: delta.dx,
            dy: delta.dy,
        }));
        return true;
    }
    let page = annots[0].target.page;
    let targets = annots.iter().map(|a| a.target.clone());
    actions.push(move_items(
        page,
        Vec::new(),
        targets,
        (delta.dx, delta.dy),
        GESTURE,
    ));
    true
}

/// The selected form fields on the primary's page; the gate is
/// `edit_content`, Delete's for fields.
fn nudge_fields(
    frame: &Frame<'_>,
    fields: &Fields<'_>,
    selection: &SelectionState,
    canvas: egui::Vec2,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(delta) = fields.page.and_then(|page| super::page_delta(canvas, page)) else {
        refuse(frame, selection, NudgeRefusal::DegeneratePage);
        return false;
    };
    let page = fields.primary.page;
    let widgets: Vec<SelectedField> = fields
        .set
        .all(Some(fields.primary))
        .into_iter()
        .filter(|f| f.page == page)
        .collect();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "field-nudge page={page} n={} dx={:.3} dy={:.3}",
            widgets.len(),
            delta.dx,
            delta.dy
        )
    });
    actions.push(move_items(
        page,
        widgets,
        Vec::new(),
        (delta.dx, delta.dy),
        GESTURE,
    ));
    true
}

/// Page content: the move a drag of `canvas` would commit, and its refusals.
fn nudge_content(
    frame: &Frame<'_>,
    selection: &SelectionState,
    canvas: egui::Vec2,
    actions: &mut Vec<Action>,
) -> bool {
    if !frame.caps.edit_content {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{TRACE}-declined reason=mode-cannot-edit-content")
        });
        return false;
    }
    let before = actions.len();
    // The hold preview is dropped: a one-point step needs no ghost while the
    // raster catches up, and `keys` has no document to hand it to.
    let _ = super::drag(
        canvas,
        Phase::Complete,
        selection,
        frame.page_index,
        frame.targets,
        frame.page,
        actions,
    );
    actions[before..]
        .iter()
        .any(|action| !matches!(action, Action::DeclineOnCanvas(_)))
}

/// Say why a nudge did nothing — on the status row **and** on the trace.
fn refuse(frame: &Frame<'_>, selection: &SelectionState, why: NudgeRefusal) {
    if let Some(sentence) = crate::text::arrange::nudge_refusal(why) {
        // The CURRENT epoch, not a new one. The sentence stands from now until
        // the next real edit moves past it, and is retired without anything
        // having to remember to — `app::actions::disclosure::record_note`'s
        // contract for a non-edit.
        crate::app::actions::record_note(frame.edit_epoch, sentence.to_owned());
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "{TRACE}-declined level={:?} sel={} reason={why:?}",
            selection.level(),
            selection.len(),
        )
    });
}

#[cfg(test)]
mod tests;
