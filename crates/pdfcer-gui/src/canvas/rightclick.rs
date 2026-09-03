//! # `canvas::rightclick` — which menu a secondary click opens
//!
//! One question, and it takes three hit tests plus one frame-ordering rule to
//! answer honestly. Split out of [`super::interact`] under **R2** on
//! 2026-08-28, when the fourth canvas menu took that file past 1,500 lines.
//!
//! ## ★★★ The frame-ordering hazard, which is the whole reason this is subtle
//!
//! **`egui` opens a popup ON the secondary click.** There is no later frame on
//! which a wrong answer could be corrected — the menu that appears is the menu
//! decided by this frame's evaluation, and if that evaluation reads state which
//! the click itself is about to change, the operator sees the *previous*
//! answer, permanently.
//!
//! Two pieces of state are one frame behind here, for two different reasons:
//!
//! | what | why it is stale | how it is answered |
//! |---|---|---|
//! | the object selection | a right-click over an unselected object **selects it**, and the ribbon's condition snapshot was taken at the top of the frame | [`crate::shell::menus::MenuHost::with_conditions`] corrects the conditions |
//! | `doc.selected_field` | `canvas::forms::select_click` raises `FieldAction::Select`; the queue applies it at the **end** of the frame | a **hit test**, not a state read — see [`Click::field_menu`] |
//!
//! ⇒ Both are the same defect shape and both had to be found the same way: by
//! asking *"what does this read, and who writes it, and when?"* Neither is
//! visible from a unit test, because a unit test constructs the state directly
//! rather than letting a click produce it.
//!
//! ## The four menus, in the order they are asked
//!
//! | # | condition | menu | why it outranks the next |
//! |---|---|---|---|
//! | 1 | a caret is in existing page text | `canvas.text` | the operator is *in* the words; a text run is not a hit-testable object, so deciding by hit test first would give them the view menu |
//! | 2 | a form field is under the pointer or selected | `canvas.field` | a widget sits on top of whatever page content is beneath it |
//! | 3 | an object is under the pointer | `canvas.object` | there is a thing to act *on* |
//! | 4 | otherwise | `canvas.empty` | no thing, so the menu is about the *view* |
//!
//! ★ 1 and 2 are mutually exclusive by construction — `canvas::forms` owns
//! `/Widget` presses and only Edit mode offers field selection, while a caret
//! belongs to the text tool — so their order is documentation of that fact
//! rather than a precedence anybody has to enforce.
//!
//! ## ★ Called on EVERY frame, not only on the frame of the click
//!
//! `egui` draws an open popup until it is dismissed, and the popup exists only
//! while something is attached to the response. On a frame with no secondary
//! click and nothing open this does nothing at all.

use egui_shell::commands::HandlerToken;

use crate::app::state::OpenDoc;
use crate::canvas::menus;
use crate::canvas::selection::SelectionState;
use crate::shell::menus::MenuHost;

/// Everything one frame's secondary-click decision needs.
///
/// ★ A struct rather than eleven arguments, and it crossed clippy's threshold
/// on its way here — the same conversion `Press`, `Keys`, `Frame`, `Drag` and
/// `Swept` all made in this crate. What it buys beyond satisfying a lint is
/// that each field can carry its own note, which eleven positional arguments
/// cannot.
pub struct Click<'a> {
    /// The canvas response the popup attaches to.
    pub response: &'a egui::Response,
    /// For the caret draft and the memoised widget census.
    pub ctx: &'a egui::Context,
    /// For `selected_field` and the form census.
    pub doc: &'a OpenDoc,
    /// The mode's capabilities — a field is selectable only where it is
    /// offered, and a menu offered where selection is not is a menu whose
    /// Delete acts on nothing.
    pub caps: &'a crate::app::modes::Capabilities,
    /// Mutated: a right-click over an unselected object selects it.
    pub selection: &'a mut SelectionState,
    /// The object model, for the object hit test. `None` before one is built.
    pub targets: Option<&'a crate::panels::objects::provider::ObjectModelProvider>,
    /// Where the pointer is, in screen points. `None` when it is off-window.
    pub screen_pos: Option<egui::Pos2>,
    /// Screen ↔ page for this page.
    pub map: &'a crate::canvas::mapping::PageMapping,
    /// The page the canvas is showing.
    pub page_index: usize,
    /// Whether this frame carries a secondary click **that the mode allows**.
    pub secondary_clicked: bool,
    /// `None` when the built-in manifest failed to validate, in which case
    /// nothing happens at all — including no selection change.
    pub host: Option<&'a MenuHost<'a>>,
}

impl Click<'_> {
    /// **Is this right-click about a form field?**
    ///
    /// ★★★ A **hit test**, not a read of `doc.selected_field`, and the module
    /// header's table says why: the selection this very click raises is applied
    /// at the end of the frame, and the popup opens now.
    ///
    /// ★ OR'd with the state, deliberately. A right-click on a field that is
    /// *already* selected still opens its menu, and on that frame the two
    /// answers agree anyway. The disjunction is what keeps the menu available
    /// when the pointer is a few points outside the box of the field the
    /// operator selected a moment ago — the same forgiveness
    /// [`menus::select_under_right_click`]'s rule 3 gives an object selection,
    /// where a mis-aimed right-click must not destroy work.
    fn field_menu(&self) -> bool {
        // ★ Nothing is computed on a frame with no secondary click. `attach`
        // uses this only inside its own `if response.secondary_clicked()`, and
        // the hit test below is a linear scan over every widget on the page —
        // cheap once, wasteful sixty times a second on a form-heavy sheet.
        //
        // ★★ `secondary_clicked` already carries the mode gate (it is `&&
        // caps.edit_content` at its source), and `right_click_hits_a_field`
        // asks the same question again for its own callers. Two guards for one
        // rule is tolerable here because the second is the function's own
        // contract rather than a copy of this one.
        if !self.secondary_clicked {
            return false;
        }
        self.doc.selected_field.is_some()
            || self.screen_pos.is_some_and(|at| {
                crate::canvas::forms::right_click_hits_a_field(
                    self.ctx,
                    self.doc,
                    self.caps,
                    self.page_index,
                    self.map.to_page(at),
                )
            })
    }
}

/// Decide the menu, attach it, and report the commands the operator chose.
///
/// **Nothing is executed here.** The returned tokens are *intent*; the
/// application dispatches them at the one choke point a ribbon click and a
/// keyboard chord also reach, which is what makes it impossible for a menu item
/// and a button that share a command to do different things.
#[must_use]
pub fn attach(click: Click<'_>) -> Vec<HandlerToken> {
    // The object hit test is `menus`' own — see `menus::right_clicked_object`
    // for why it is the Object rung only and why it takes a screen position.
    let object = menus::right_clicked_object(
        click.secondary_clicked,
        click.targets,
        click.screen_pos,
        click.map,
        click.page_index,
    );
    let field_menu = click.field_menu();
    // ★★★ The DOCUMENT's half of `selection.delete_permitted`, corrected here
    // because the frame-top condition set could not have known.
    //
    // `field_menu()` above opens the field menu for a widget merely **under the
    // pointer** — `right_click_hits_a_field`, the second disjunct — so on a
    // first right-click over an unselected widget `doc.selected_field` was
    // still `None` when `PdfcerApp::conditions()` ran, and the published answer
    // came from the annotation arm of that ladder rather than the forms one.
    // Left stale, `format.delete` would be drawn on that frame over a certified
    // form: the *drawn and silently inert* control R83 exists to remove.
    //
    // ★ `document_refuses_delete` and not `refuses_delete`, and the difference
    // is the whole reason the scope-free entry point exists — see its doc.
    // `EditSession::deletion_refusal` names no field, so the honest question
    // about a widget that is not yet selected is the document's.
    //
    // ★ Computed unconditionally rather than behind `field_menu`: it is one
    // `Option` test over a census the session already holds, `attach` reads it
    // only inside its own `secondary_clicked` guard, and a `then()` here would
    // make the value's meaning depend on which of two booleans was false.
    let field_delete_permitted =
        !crate::panels::properties::formfield::document_refuses_delete(click.doc);
    // ★★ **Whether this is a READER's right-click** — O71.
    //
    // Read from the same `Capabilities` every other gate in this frame reads,
    // rather than from a mode id: `edit_content` is derived from the mode's tab
    // list, so a mode added later that happens to include the Edit tab gets the
    // editing menu without anybody editing this line, and one that does not
    // gets the reader's.
    let reading = !click.caps.edit_content;
    menus::attach(
        click.response,
        click.selection,
        click.page_index,
        object,
        field_menu,
        field_delete_permitted,
        reading,
        click.host,
    )
}
