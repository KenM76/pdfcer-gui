//! The worded decline: telling the operator that a command did *not* run.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline.md`.

use std::cell::RefCell;

use crate::app::state::OpenDoc;
use crate::canvas::zoom::{self, ZoomOutcome};

/// Named region: the worded decline, when one is live.
const REGION_DECLINE: &str = "status-group:decline"; // ui-text-exempt: trace region name, never displayed

/// Named region: the button beside a decline that runs its remedy.
const REGION_DECLINE_REMEDY: &str = "status-group:decline.remedy"; // ui-text-exempt: trace region name, never displayed

// ---------------------------------------------------------------------------
// What was declined
// ---------------------------------------------------------------------------

pub(crate) use pdfcer_gui_base::declined::{Declined, History};

/// The decline in a framing zoom's outcome, if it is one.
#[must_use]
pub(crate) fn of(outcome: ZoomOutcome) -> Option<Declined> {
    match outcome {
        ZoomOutcome::NoBounds => Some(Declined::NothingToFrame),
        ZoomOutcome::NoCanvas => Some(Declined::CanvasNotDrawn),
        ZoomOutcome::Zoomed { .. } => None,
    }
}

thread_local! {
    /// The most recent declined command, waiting to be read by the status
    /// bar. See the module docs for why a thread-local, and why that is sound
    /// rather than smuggled.
    static LAST: RefCell<Option<Declined>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// The store — written by the dispatcher, read by the bar
// ---------------------------------------------------------------------------

/// Forget any live decline — **the operator's next act**.
pub(crate) fn retire() {
    LAST.with_borrow_mut(|slot| *slot = None);
}

/// The live decline, if there is one and it still describes what the operator
/// is looking at.
#[must_use]
pub(super) fn live(ctx: &egui::Context, doc: &OpenDoc) -> Option<Declined> {
    let has_bounds = zoom::can_zoom_to_selection(doc);
    let canvas_has_drawn = zoom::last_frame(ctx).is_some();
    let history = History::of(doc);
    // The same accessor `crate::app::conditions` publishes `selection.in_form`
    // from, asked here in the same words, so that the greyed control and the
    // sentence explaining why it is greyed cannot come from two questions that
    // drift apart.
    let selection_in_form = !doc
        .selection
        .leaf_indices_on(doc.view.page_index)
        .is_empty();
    LAST.with_borrow(|slot| {
        slot.clone()
            .filter(|d| d.still_true(has_bounds, canvas_has_drawn, history, selection_in_form))
    })
}

/// Record that a verb refused because what is selected lives inside a form
/// XObject.
pub(crate) fn record_inside_form(reason: crate::text::status::InsideFormRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::InsideForm(reason)));
}

/// **The raw store, for tests only.**
#[cfg(test)]
#[must_use]
pub(crate) fn recorded_for_test() -> Option<Declined> {
    LAST.with_borrow(Clone::clone)
}

/// What the BAR would draw — [`live`] under a test-visible name.
#[cfg(test)]
pub(crate) fn live_for_test(ctx: &egui::Context, doc: &OpenDoc) -> Option<Declined> {
    live(ctx, doc)
}

// ---------------------------------------------------------------------------
// The line
// ---------------------------------------------------------------------------

/// Draw the worded decline into the bar's single row, if one is live, and
/// beside it the button for its remedy when that command is registered.
pub(super) fn show(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    commands: &egui_shell::CommandRegistry,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    let Some(declined) = live(ui.ctx(), doc) else {
        return;
    };
    super::disclosure::disclosure_line(ui, REGION_DECLINE, &declined.line());
    let Some(command) = declined.remedy().and_then(|id| commands.get(id)) else {
        return;
    };
    let mut button = ui.button(&command.label);
    if let Some(tip) = &command.tooltip {
        button = button.on_hover_text(tip);
    }
    crate::diag::ui_rect(REGION_DECLINE_REMEDY, button.rect);
    if button.clicked() {
        actions.push(crate::app::actions::Action::Command(command.id.clone()));
    }
}

/// **The funnel's floor**, split out under R2 when this file reached 1,530
/// lines. See `decline/floor.rs`'s header for why that particular seam: it is
/// the one part of this module that answers a question about somebody else's
/// protocol rather than about what a decline is.
///
mod floor;
/// Re-exported so that the one caller — `crate::app::actions::funnel` — still
/// says `decline::before_the_verb()`. The split is about where the code lives; a
/// call site should not have to learn that a submodule exists.
pub(crate) use floor::before_the_verb;

/// **The two declines the text caret raises**, split out under R2 on
/// 2026-09-04 when `OPERATOR_REQUESTS.md` O127 took this file past 1,500 lines
/// for the second time. See `decline/textedit.rs`'s header for the seam and for
/// the argument both of them share — that a sentence in the wrong slot is
/// indistinguishable, from the operator's chair, from no sentence at all.
mod textedit;
/// Re-exported so the four call sites still say `decline::record_reflow(..)`
/// and `decline::record_enter_cannot_split(..)`. `floor`'s rule: the split is
/// about where the code lives, and a call site should not have to learn that a
/// submodule exists.
pub(crate) use textedit::{
    record_edit_text_refusal, record_enter_cannot_split, record_key_refused, record_reflow,
};

/// **Every writer of the decline slot**, split out under R2 on 2026-09-05
/// when this file reached 1,497 lines against the ceiling for the third time —
/// see `decline/record.rs`'s header for the seam. It is the same seam `floor`
/// and `textedit` already stand on: this file answers *what a decline is and
/// how long it owes its sentence*, and a recorder answers *who says one*.
mod record;
/// Re-exported with a glob, uniquely among the three submodules, and that is a
/// deliberate exception rather than a shortcut. `floor` and `textedit` name
/// their two or three items because each is a small, closed set with an
/// argument attached; this is the whole recording surface — twenty-odd
/// constructors that grow by one every time a verb learns to decline — and a
/// hand-written list of them here would be a second register of the same
/// family, free to fall out of step with the file it mirrors. Every name it
/// exports is `pub(crate) fn record_*` and nothing else, so the glob cannot
/// leak anything a reader would not expect to find under `decline::`.
pub(crate) use record::*;

/// **The mode's refusal of a cut or a paste**, 2026-09-05. Its own file
/// rather than a function in `record` for `textedit`'s reason: it carries an
/// argument of its own — why a chord pushed blind at the gate obliges the
/// dispatcher to word every refusal it can now meet — and that argument would
/// be buried among twenty siblings.
mod clipboard;
/// Re-exported so the two call sites in `app::dispatch::clipboard` say
/// `decline::record_mode_refusal(..)`. `floor`'s rule.
pub(crate) use clipboard::record_mode_refusal;

/// **What a refused CANVAS GESTURE may say, and the one writer that
/// says it** — O188, 2026-09-15.
mod canvas;
/// Re-exported so `app::actions` says `decline::CanvasDecline` and
/// `decline::record_canvas(..)`. `floor`'s rule: the split is about where the
/// code lives, and a call site should not have to learn that a submodule exists.
///
/// **The two halves are re-exported at different visibilities, and
/// that asymmetry is the design rather than an oversight.** The TYPE is `pub`
/// because it is half of a `pub` variant's signature (`private_interfaces`;
/// `canvas::CanvasDecline`'s own docs carry the argument, and the reason
/// `app::prefs`'s way out was not available). The WRITER stays `pub(crate)`,
/// behind this module's `pub(super)` path, so it remains reachable only from
/// inside `crate::app` — which is the boundary that matters: *a decline is
/// written by the one dispatcher and read by the one bar*. Naming a sentence
/// is not writing one.
pub use canvas::CanvasDecline;
pub(crate) use canvas::record_canvas;

/// See `decline/tests.rs`.
#[cfg(test)]
mod tests;
