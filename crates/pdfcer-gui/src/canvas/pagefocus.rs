//! # `canvas::pagefocus` — the page is a keyboard focus owner
//!
//! `OPERATOR_REQUESTS.md` O204, decision 2. Clicking a page used to focus
//! nothing: `Sense::click_and_drag` is focus*able*, but egui grants focus on a
//! click to no widget — a widget has to ask, as `TextEdit` does. So
//! `Memory::focused()` stayed `None`, Tab took egui's *"nothing is focused,
//! give it to the first widget that wants it"* branch, and the first focusable
//! widget of the frame is the ribbon. That one fact is the operator's whole
//! report.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/pagefocus.md`.

use egui::{Rect, Response, Ui};

use crate::canvas::tabnav::{self, Scope};

/// The stable half of a page's keyboard id.
const PAGE_KEY: &str = "canvas-page-focus"; // ui-text-exempt: internal id seed, never displayed

/// **Give this page a keyboard identity**, and publish it while it is focused.
pub(super) fn seat(ui: &mut Ui, page: usize, rect: Rect, response: &Response) {
    let id = ui.id().with((PAGE_KEY, page));
    let keyboard = ui.interact(rect, id, egui::Sense::focusable_noninteractive());
    if response.clicked() || response.drag_started() {
        keyboard.request_focus();
    }
    if !keyboard.has_focus() {
        return;
    }
    if crate::canvas::tool::active(ui.ctx())
        .measure_kind()
        .is_some()
    {
        // Tab cycles the snap mode while a measure tool is armed, and that
        // gesture predates this one. Ownership is RELEASED rather than merely
        // not published: a stale owner from the frame before the tool was armed
        // still names this same id, so it would pass the identity test and the
        // hook would go on swallowing Tab with nothing to spend it on.
        tabnav::release(ui.ctx());
        return;
    }
    tabnav::publish(ui.ctx(), Scope::Object, id);
}
