//! Context menus — the third surface, carrying the same commands again.
//!
//! # Why this module exists
//!
//! A selection that can be made and not acted on is worse than no selection,
//! and right-click is the path most users try after the keyboard.
//! `RIBBON_IA.md` §6 puts it as *"the other half of making selection mean
//! something"*: every selectable kind needs a menu carrying the same
//! commands as its Format tab section plus the clipboard verbs, and no
//! amount of ribbon design substitutes for it.
//!
//! `RIBBON_IA.md` §5 places this surface among the others, which are not
//! redundant with it:
//!
//! | Surface | Answers | Lives |
//! |---|---|---|
//! | The **Format** contextual tab | "what do I change mid-gesture?" | in the ribbon, on selection |
//! | The **properties panel** | "what *is* this thing?" | in the dock, always |
//! | The **context menu** | "act on *this*, now" | at the pointer |
//!
//! > A third surface, the context menu, carries the same commands again
//! > for the user who right-clicks. **That is not duplication in the P1
//! > sense — context menus are not tabs** — and it is the path most users
//! > try after the keyboard.
//!
//! That sentence is load-bearing twice over. It is why
//! [`Menus::validate`] does **not** extend the one-command-one-tab rule
//! over menus, and it is why a menu reuses [`crate::manifest::Item`]
//! rather than getting a vocabulary of its own: "the same commands again"
//! has to mean the same *ids*, resolved through the same registry, into
//! the same handler tokens, or the third surface is a third thing to
//! maintain.
//!
//! # The map: which file holds which decision
//!
//! | File | Owns | The rule it holds |
//! |---|---|---|
//! | [`model`] | [`Menu`], [`Menus`], [`MenuLookup`] | A menu is a [`crate::manifest::Group`] keyed by an application-supplied **context id**. Serializable, mergeable, customizable. |
//! | [`shortcut`] | [`Shortcuts`] | The chord is **derived from the keymap**, never written down twice. When one command has several chords, the easiest one is taught. |
//! | [`plan`] | resolution, punctuation, width, the icon column — no `egui` | Unregistered is **absent**; disabled is **greyed**; nothing enabled means **the menu does not open**; the **column** belongs to the menu and the **glyph** to the command. |
//! | [`render`] | [`ContextMenu`] and the entry points | The shell reports intent; the application dispatches. The decision not to open is taken **before** `egui` is asked for a popup. |
//! | [`a11y`] | announced names | The chord travels *in the accessible name*, because `egui` 0.35 has no field that carries it anywhere else. |
//! | [`report`] | published rectangles | A popup at the pointer cannot be found any other way — and a painted icon slot publishes one too, because a justified row measures the same with a glyph in it or without. |
//! | [`ctx`] | the per-menu render context | Icon and rect seams are the **ribbon's own types**; only the custom-row seam differs, and it differs because a menu has no tab and no group. |
//!
//! # What an application does
//!
//! ```no_run
//! # use egui_shell::{CommandRegistry, ConditionSet, Shell, menu::Menu};
//! # fn dispatch(_: egui_shell::HandlerToken) {}
//! # fn draw(ui: &mut egui::Ui, shell: &Shell, registry: &CommandRegistry,
//! #         conditions: &ConditionSet) {
//! // 1. Draw something right-clickable.
//! let object = ui.add(egui::Label::new("an object").sense(egui::Sense::click()));
//!
//! // 2. Hand the shell the context id for what was clicked.
//! for token in Menu::attach(&object, shell, registry, "canvas.object", conditions) {
//!     // 3. Dispatch at the same choke point the ribbon uses.
//!     dispatch(token);
//! }
//! # }
//! ```
//!
//! The context id (`"canvas.object"`, `"dock.tab"`, `"pages.thumbnail"`)
//! is the application's; the shell never interprets it. See [`model`]'s
//! header for why that is the only key that keeps this crate free of the
//! application's document model.
//!
//! # Where the menus come from
//!
//! [`crate::Shell::menus`] carries them, and [`model::menus_of`] is the
//! single function that reads that field — everything else in this module
//! goes through it, so the manifest is the one source. An application that
//! is not using the manifest yet can hold a [`Menus`] itself and pass it in;
//! the document, the customization overlay, the resolution rules, the width
//! arithmetic, the rendering and the accessibility layer all work either
//! way, and the tests here exercise both.

pub mod a11y;
pub mod ctx;
pub mod model;
pub mod plan;
pub mod render;
pub mod report;
pub mod shortcut;

// Test-only, and split into separate files rather than one `mod tests` for
// the reasons the ribbon gives: the width tests need a fixture (a synthetic
// font) whose construction has nothing to do with menus and should not be
// read as if it did, and structural tests and geometric tests are different
// kinds of claim that should not be filed together.
#[cfg(test)]
mod tests;
#[cfg(test)]
mod width_tests;

// The synthetic proportional face, borrowed rather than duplicated.
//
// `crate::ribbon::testfont` is `mod testfont;` — private to the ribbon —
// so it cannot be reached by a path from here, and `ribbon/` is not this
// module's to edit. Including the file by `#[path]` under `#[cfg(test)]`
// compiles the same source a second time, in test builds only, with no
// edit to the ribbon and no 600-line copy in this tree. The dock does
// exactly this, for exactly this reason.
//
// Why it must be here at all is not a detail:
// `D:/dev/rag/rust/a_crate_tested_alone_and_in_a_workspace_gets_different_features_so_layout_tests_can_be_vacuous.md`
// records this crate's own suite passing over a width layer that had never
// been shown a non-zero width, because `egui` with
// `default-features = false` supplies no font data and every galley
// measures ~0. Every assertion about a menu's width, its column gap and
// its clamp would be satisfied by nothing at all without this.
#[cfg(test)]
#[path = "../ribbon/testfont.rs"]
// `clippy::duplicate_mod` is exactly right in general and wrong here: it
// warns that one file compiled as two modules gives two distinct types
// that look identical. That is the intent. Nothing crosses between the
// ribbon's copy and this one — each is used only by its own sibling tests
// — and the alternative clippy would prefer, a shared module, is what
// `ribbon/` not being this module's to edit rules out.
#[allow(clippy::duplicate_mod)]
mod testfont;

pub use ctx::{MenuCustomItem, MenuCustomRenderer};
pub use model::{Menu, MenuError, MenuLookup, Menus};
pub use plan::{BodyWidth, RowWidths, Slot};
pub use render::ContextMenu;
pub use shortcut::Shortcuts;
