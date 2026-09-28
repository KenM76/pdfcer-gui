//! # `canvas::rightclick` — which menu a secondary click opens
//!
//!
//! ## The frame-ordering hazard, which is the whole reason this is subtle
//!
//! **`egui` opens a popup ON the secondary click.** There is no later frame on
//! which a wrong answer could be corrected — the menu that appears is the menu
//! decided by this frame's evaluation, and if that evaluation reads state which
//! the click itself is about to change, the operator sees the *previous*
//! answer, permanently.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/rightclick.md`.

use egui_shell::commands::HandlerToken;

use crate::app::state::OpenDoc;
use crate::canvas::menus;
use crate::canvas::selection::SelectionState;
use crate::shell::menus::MenuHost;

/// Everything one frame's secondary-click decision needs.
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
    fn field_menu(&self) -> bool {
        // Nothing is computed on a frame with no secondary click. `attach`
        // uses this only inside its own `if response.secondary_clicked()`, and
        // the hit test below is a linear scan over every widget on the page —
        // cheap once, wasteful sixty times a second on a form-heavy sheet.
        //
        // `secondary_clicked` already carries the mode gate (it is `&&
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
    // The DOCUMENT's half of `selection.delete_permitted`, corrected here
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
    // `document_refuses_delete` and not `refuses_delete`, and the difference
    // is the whole reason the scope-free entry point exists — see its doc.
    // `EditSession::deletion_refusal` names no field, so the honest question
    // about a widget that is not yet selected is the document's.
    //
    // Computed unconditionally rather than behind `field_menu`: it is one
    // `Option` test over a census the session already holds, `attach` reads it
    // only inside its own `secondary_clicked` guard, and a `then()` here would
    // make the value's meaning depend on which of two booleans was false.
    let field_delete_permitted =
        !crate::panels::properties::formfield::document_refuses_delete(click.doc);
    // **Whether this is a READER's right-click** — O71.
    //
    // Read from the same `Capabilities` every other gate in this frame reads,
    // rather than from a mode id: `edit_content` is derived from the mode's tab
    // list, so a mode added later that happens to include the Edit tab gets the
    // editing menu without anybody editing this line, and one that does not
    // gets the reader's.
    let reading = !click.caps.edit_content;
    menus::attach(menus::Attach {
        response: click.response,
        selection: click.selection,
        page: click.page_index,
        object,
        // The same provider `right_clicked_object` was just handed, passed
        // on so the menu can ask one rung deeper — which LINE of a text block
        // the pointer is on (O188(A)). Nothing extra is computed here.
        targets: click.targets,
        field_selected: field_menu,
        field_delete_permitted,
        reading,
        // **`author_markup`, and it is deliberately not `!reading`.**
        //
        // Review has `edit_content == false` and `author_markup == true` — it
        // edits no page content and authors every comment there is. Gating the
        // markup menu on the reader flag above would take the shape's own menu
        // away in the one mode whose entire subject is shapes, and would hand
        // that operator `canvas.read-object`'s *Copy image* instead.
        //
        // ⇒ One capability per question. The two flags exist separately on
        // `Capabilities` for exactly this, and `app::conditions`' delete ladder
        // already reads them apart the same way.
        author_markup: click.caps.author_markup,
        author_measure: click.caps.author_measure,
        doc: click.doc,
        map: click.map,
        screen_pos: click.screen_pos,
        host: click.host,
    })
}
