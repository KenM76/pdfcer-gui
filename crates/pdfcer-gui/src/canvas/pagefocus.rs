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
//! ## Contract
//!
//! [`seat`] is called once per drawn page, with that page's rect and the
//! response the strip already allocated. It adds a second, **keyboard-only**
//! widget over the same rect, asks for focus when the page is clicked or
//! dragged, and publishes the page as [`crate::canvas::tabnav`]'s owner while
//! it holds focus.
//!
//! ## Why a second widget rather than a focus request on the page's own
//!
//! The strip's response comes from `Ui::allocate_rect`, whose id is
//! `Id::new(self.next_auto_id_salt)` — a counter bumped once per allocation,
//! not a hash of the rect. The rect moving is therefore not the problem;
//! **what is allocated before it** is. The strip draws only the pages the
//! viewport reaches, so the counter a page receives depends on how many pages
//! happened to be visible ahead of it. Scroll one page off the top and the id
//! page 2 held becomes page 3's: focus would not be dropped, it would
//! silently move to another sheet, and Tab would walk the wrong page's objects
//! with nothing on screen to say so.
//!
//! This widget's id is seeded from the page index instead, so it is stable for
//! as long as the document is open and belongs to one page whatever else is on
//! screen. It senses nothing but the keyboard, so it cannot take a press away
//! from the page beneath it.
//!
//! ## Why D1 does not come back
//!
//! `DEFECTS.md` D1 was a **guard** that asked whether *anything* held the
//! keyboard, which a focused page satisfies — so Delete stopped working after
//! any canvas click. This shell's guard asks whether text is being *composed*,
//! which a focused page is not, and `tools/gates/check-typing-guard.sh` refuses
//! the old predicate. Making the page a focus owner is safe here precisely
//! because that guard was already written the other way round.

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
