//! `ui_rect` reporting for context menus — the stable names a
//! verification harness asserts against.
//!
//! # Why menus need this more than the ribbon does
//!
//! [`crate::ribbon::report`]'s header makes the general argument: a
//! harness that wants to assert *"the Delete row is legible"* has to know
//! where that row is, and the only version of that knowledge which does
//! not rot is the one the renderer publishes on the frame it drew.
//!
//! A context menu makes the argument sharper, because it is drawn in a
//! **popup at the pointer**. There is no fraction of the window it can be
//! hard-coded to, and no layout a harness could re-derive: the position
//! depends on where the operator's pointer was, and `egui` may flip the
//! popup to any of several alignments to keep it on screen. Publishing the
//! rectangle is not the best of three options here; it is the only one.
//!
//! # The names
//!
//! | Name | What |
//! |---|---|
//! | `menu.body.<context>` | the whole menu body |
//! | `menu.item.<context>.<command id>` | one command row |
//! | `menu.custom.<context>.<kind>` | one application-drawn row |
//! | `menu.icon.<context>.<command id>` | the icon slot of a row **whose glyph was actually painted** |
//!
//! ## ★★★ Why the icon slot is published, and what its ABSENCE means
//!
//! Added 2026-09-04 with the menu icon column, and it is the one name here
//! whose *absence* is the assertion.
//!
//! A row's own rectangle proves the row was drawn. It proves nothing about
//! the glyph, because a row is justified to the body width and measures
//! the same whether its icon slot holds a picture, holds a blank, or does
//! not exist. So the QAT's trick — assert the SHAPE of the control, since
//! an icon-only button is square and a text button is a word wide — has no
//! menu equivalent, and a harness looking only at
//! `menu.item.<context>.<id>` cannot tell an application that wired an
//! icon painter from one that did not. That was exactly the state this
//! build was in until 2026-09-04, undetectably, for the whole life of the
//! menu engine.
//!
//! This name closes that. It is published **only from the branch that
//! calls the application's painter** — not when the slot is blank, not
//! when there is no slot, and not when no painter was supplied. So:
//!
//! * present ⇒ the row reserved a slot, a painter existed, and it was
//!   handed this rectangle to draw the command's key into;
//! * absent ⇒ one of those three did not happen, which is precisely the
//!   defect class a driven check exists to catch.
//!
//! It is deliberately **not** proof that pixels changed. Whether a key
//! resolves to art is the icon set's own business and is asserted offline
//! by the application's icon tests; the missing fact was never *"does a
//! glyph render"* but *"did anything ask for one"* — the same distinction
//! `tools/ui-verify`'s `qat_controls_are_icon_only` header draws, reached
//! independently on the other surface.
//!
//! ## ★ Why the body is `menu.body.<context>` and not `menu.<context>`
//!
//! Because the shorter spelling makes the two namespaces overlap. A
//! context id is an arbitrary application string; if one were ever called
//! `item`, `menu.item` would be a menu body and `menu.item.canvas.delete`
//! a row inside a different menu, and a harness filtering `menu.item.`
//! would silently match both. The ribbon reached the same conclusion for
//! the same reason with `ribbon.tabs.overflow` — see
//! [`crate::ribbon::report::tab_overflow`].
//!
//! # The sink itself is borrowed, not redefined
//!
//! [`crate::ribbon::report::RectSink`] and
//! [`crate::ribbon::report::Reporter`] carry nothing ribbon-specific — a
//! sink is `FnMut(&str, Rect)` and a reporter is the "do not format the
//! name unless someone is listening" rule. Redefining them here would be
//! two types where one belongs, and would leave an application wiring two
//! different callbacks to the same harness.
//!
//! Only the **names** are this module's, because only the names are about
//! menus.

/// The name prefix every rect this module publishes begins with.
///
/// A harness can therefore separate a menu's reports from the ribbon's or
/// the dock's in one filter.
pub const PREFIX: &str = "menu";

/// The name under which a whole menu **body** is published.
///
/// This is the rect an "is the menu on screen at all" assertion wants, and
/// the one a width assertion measures.
#[must_use]
pub fn body(context_id: &str) -> String {
    format!("{PREFIX}.body.{context_id}")
}

/// The name under which one **command row** is published.
#[must_use]
pub fn item(context_id: &str, command_id: &str) -> String {
    format!("{PREFIX}.item.{context_id}.{command_id}")
}

/// The name under which one **application-drawn row** is published.
///
/// Keyed by `kind` rather than by position, because position is exactly
/// what changes when a command above it is filtered out by the
/// no-placeholders rule.
#[must_use]
pub fn custom(context_id: &str, kind: &str) -> String {
    format!("{PREFIX}.custom.{context_id}.{kind}")
}

/// The name under which one row's **painted icon slot** is published.
///
/// Published only when the application's icon painter was actually called
/// for that row. See this module's header: the absence of this name is the
/// assertion, and it is the only signal a driven check has that a menu
/// surface draws glyphs at all.
#[must_use]
pub fn icon(context_id: &str, command_id: &str) -> String {
    format!("{PREFIX}.icon.{context_id}.{command_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ The published names are a stability contract, and this test is
    /// the tripwire on it.**
    ///
    /// These strings are consumed by a harness in another tool, possibly
    /// in another repository, by literal comparison. A rename is a
    /// breaking change with no compiler to catch it: the harness keeps
    /// building, its assertions simply stop matching anything, and a test
    /// that matches nothing passes.
    #[test]
    fn the_reported_names_are_a_stability_contract() {
        assert_eq!(body("canvas.object"), "menu.body.canvas.object");
        assert_eq!(
            item("canvas.object", "edit.delete"),
            "menu.item.canvas.object.edit.delete"
        );
        assert_eq!(
            custom("pages.thumbnail", "page_scale"),
            "menu.custom.pages.thumbnail.page_scale"
        );
        assert_eq!(
            icon("dock.tab", "view.panel_float"),
            "menu.icon.dock.tab.view.panel_float"
        );
    }

    /// **★ The body namespace and the row namespace cannot collide.**
    ///
    /// The reason the body is `menu.body.` rather than bare `menu.`: a
    /// context id is an arbitrary application string, and one called
    /// `item` would otherwise produce a body name that a harness
    /// filtering for rows would match.
    #[test]
    fn a_body_can_never_be_mistaken_for_a_row() {
        let hostile = body("item");
        assert_eq!(hostile, "menu.body.item");
        assert!(
            !hostile.starts_with("menu.item."),
            "a context id called `item` must not produce a body name that a \
             harness filtering rows would catch; got {hostile}"
        );
        // And the mirror image.
        assert!(!item("x", "y").starts_with("menu.body."));
        assert!(!custom("x", "y").starts_with("menu.item."));
        // The icon slot shares a row's command id, so its namespace is the
        // one most likely to be confused with the row's. It must not be.
        assert!(!icon("x", "y").starts_with("menu.item."));
        assert!(!item("x", "y").starts_with("menu.icon."));
    }

    /// Every name is filterable by [`PREFIX`].
    #[test]
    fn every_name_carries_the_menu_prefix() {
        for name in [
            body("c"),
            item("c", "x.y"),
            custom("c", "k"),
            icon("c", "x.y"),
        ] {
            assert!(
                name.starts_with(PREFIX),
                "`{name}` is not filterable as a menu report"
            );
        }
    }
}
