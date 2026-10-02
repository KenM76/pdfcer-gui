//! # `canvas::textedit::sweep` — a text-tool drag that starts on text selects it
//!
//! Contract: [`over_text`] answers whether a text-tool drag belongs to the
//! caret rather than to a new text box. A drag pressed inside the open editor
//! box is already a selection sweep (`keys::pointer` makes it one). A drag
//! pressed on a page run with the edit caret armed opens a draft on that run
//! at the press, and `keys::pointer` sweeps from there on the following
//! frames. Anything else is the box gesture, unchanged.

use egui::Pos2;

use super::{Click, TextEditKind};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// Whether the drag pressed at `from` (canvas space) selects text instead of
/// drawing a box; opens the run's draft on the drag's first frame.
pub fn over_text(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    kind: Option<TextEditKind>,
    from: Pos2,
    actions: &mut Vec<Action>,
) -> bool {
    if super::hit::owns_canvas(ctx, from) {
        return true;
    }
    if kind != Some(TextEditKind::Edit) || !super::place::run_under(doc, page_index, from) {
        return false;
    }
    let click = Click {
        doc,
        page_index,
        kind: TextEditKind::Edit,
        canvas_point: from,
        on_image: false,
    };
    if let Err(refusal) = super::click(ctx, &click, actions) {
        super::route::decline(actions, refusal, " via=sweep");
    }
    true
}
