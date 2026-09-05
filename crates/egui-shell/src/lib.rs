//! # `egui-shell` — a reusable application shell for `egui`
//!
//! ## What this crate is
//!
//! A **shell** is everything an application has that is not the
//! application: the ribbon, the panel dock, the mode selector, the theme,
//! the command registry, the layout that persists across sessions, and
//! the diagnostic channel a verification harness drives. It is the part
//! that is identical between a PDF editor, a CAD viewer and a log
//! browser, and it is the part that is normally written three times.
//!
//! This crate is that part, written once.
//!
//! ## The hard boundary
//!
//! > **`egui-shell` knows nothing about any document format, and must
//! > never learn.**
//!
//! It may depend on `egui`, `eframe`, `egui_tiles`, `serde` and small
//! leaf utilities. It may not depend on anything that knows what a page,
//! a glyph, a layer or a signature is. A CI gate
//! (`tools/gates/check-shell-purity.sh`) enforces the negative half; this
//! crate's `Cargo.toml` enforces the positive half.
//!
//! The rule has a diagnostic form that is more useful than the
//! prohibition: **if `egui-shell` needs to know about pages, the
//! abstraction is wrong.** A shell that renders a tab called "Measure"
//! and routes a command called `measure.linear` does not need to know
//! what either means. The moment it does, the thing that should have been
//! a string id or a callback has been inlined into the framework, and the
//! next application cannot take it.
//!
//! Concretely, this is why:
//!
//! - Commands are **string ids** ([`commands`]), not an enum. An enum
//!   would have to enumerate one application's verbs.
//! - Panel *bodies* and command *implementations* stay in the
//!   application. The shell owns where a thing appears; the application
//!   owns what it does.
//! - The theme's palette carries only **chrome** roles ([`theme`]).
//!   Application-specific colour semantics — a vector node mark, a
//!   revision cloud, a snap guide — live in [`theme::Overlays`], an
//!   extension point the shell stores and validates but never interprets.
//!
//! ## The central design decision: the shell is data
//!
//! Two requirements arrived together on this project: *be reusable across
//! projects*, and *be customizable at runtime by the operator*. Both are
//! satisfied by the same decision, recorded in `SHELL_FRAMEWORK.md` §1:
//!
//! > Tabs, groups, commands, panels, layouts, modes and key bindings are
//! > a **serializable document** that the application *supplies* and the
//! > operator *edits* — not code that has to be recompiled to change.
//!
//! A ribbon defined in Rust `match` arms can be neither reused nor
//! customized. A ribbon defined as data can be both, and the same
//! serializer that lets an operator save a customized ribbon lets a
//! different application ship a completely different one.
//!
//! That document is [`manifest::Shell`]. It is inspectable, diffable,
//! testable with no window open, and version-able. Every test in this
//! crate is headless for exactly that reason.
//!
//! ## Module map
//!
//! | Module | Responsibility | Stage |
//! |---|---|---|
//! | [`theme`] | Token palette, three presets, the **rendered-pair contrast gate**, and the [`theme::Overlays`] extension point for application colour semantics. | S0 |
//! | [`verify`] | The `key=value` diagnostic channel a verification harness reads. Off unless asked, never load-bearing. | S0 |
//! | [`manifest`] | The serializable shell definition — tabs, groups, items, modes, keymap, QAT — with validation and the three-layer merge. | S2 |
//! | [`commands`] | The command registry: id → label, tooltip, icon key, enable predicate, handler token. The thing a manifest may only *reference*. | S2 |
//! | [`ribbon`] | Renders a [`manifest::Shell`]: QAT, tab strip, contextual tabs, the N-position mode selector, captioned group bands, overflow, `ui_rect` reporting and accessible names. **Reports intent; executes nothing.** | S2 |
//! | [`dock`] | The panel host: columns per side, vertical stacks, tabbed groups, draggable splitters, and a tab-overflow menu whose space is reserved before any tab is measured. Panels are opaque string ids the application supplies. | S3 |
//! | [`layout`] | Serialization and persistence of a [`dock::DockLayout`], **fail-soft per item**, plus named workspaces and scoped reset. | S3 |
//! | [`menu`] | Context menus, keyed by an application-supplied context id: the same [`manifest::Item`]s a ribbon band holds, resolved through the same registry, with keymap-derived chord hints. **A menu with nothing to offer never opens.** | S4 |
//!
//! Modules named in `SHELL_FRAMEWORK.md` §3 that do **not** exist yet, so
//! their absence is not mistaken for an oversight:
//!
//! | Module | Arrives at | Why not now |
//! |---|---|---|
//! | `modes` | S3b | A mode is a named workspace: a manifest overlay plus a dock layout. [`ribbon`] already renders the mode **selector** and honours a mode's tab list, because a ribbon cannot be drawn without knowing which tabs the mode contains; and [`layout::Workspace`] is already the *panel-layout* half. What arrives at S3b is the piece that binds one to the other. |
//!
//! The build order is deliberate and is stated in `SHELL_FRAMEWORK.md`
//! §7: **`egui-shell` is built *as* its first consumer is built, not
//! before it. A framework designed without a consumer gets the
//! abstractions wrong.**
//!
//! ## What a consuming application does
//!
//! 1. Builds a [`commands::CommandRegistry`] — every verb it can perform,
//!    with a label, a tooltip, an optional icon key, an enable predicate
//!    and an opaque handler token it will dispatch on.
//! 2. Supplies a built-in [`manifest::Shell`] describing where those
//!    commands appear.
//! 3. Loads optional application-override and operator-customization
//!    layers and calls [`manifest::merge`], which overrides **per item**
//!    and reports anything it had to skip as a disclosed
//!    [`manifest::Skip`] rather than failing the load.
//! 4. Calls [`manifest::Shell::validate_against`] on the merged result.
//! 5. Applies a [`theme::Theme`] once per frame.
//! 6. Dispatches on the handler token when the shell reports a command
//!    was invoked.
//!
//! Nothing in that list mentions a document.
//!
//! ## Testing posture
//!
//! Every invariant this crate asserts has a unit test, and every test is
//! headless. That is not a claim that headless tests are sufficient —
//! this project exists partly because two shipped defects were invisible
//! to a green suite and obvious within thirty seconds of using the
//! application. The response is not to write fewer unit tests; it is to
//! make the unit tests measure **what will actually be rendered** rather
//! than what was written down. [`theme::contrast`] is the worked example:
//! it reads back the `egui::Style` the theme produces and measures the
//! foreground/background pairs `egui` will paint, because the defect it
//! exists to prevent was invisible to two adjacent tests that compared
//! palette entries to each other.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod commands;
pub mod dock;
pub mod layout;
pub mod manifest;
pub mod menu;
pub mod peek;
pub mod ribbon;
/// **The document tab strip** — the row of tabs an application draws when the
/// operator has several documents open at once.
///
/// Deliberately separate from [`dock`]'s tab bar: a dock tab names a *panel*
/// and a strip tab names an *operand*. See the module's own header for the
/// table, and for the two things it refuses to know.
pub mod tabstrip;
pub mod theme;
pub mod verify;

pub use commands::{Command, CommandRegistry, ConditionSet, Enable, HandlerToken};
pub use dock::{
    Column, Dock, DockFrameReport, DockLayout, DockSide, DockState, PanelId, PanelInfo,
    PanelRegistry, SideLayout, Stack,
};
pub use layout::{
    LayoutDocument, LayoutSkip, LayoutSkipReason, LoadReport, Loaded, ResetScope, Workspace,
};
pub use manifest::{Group, Item, Keymap, Mode, Qat, Shell, Tab};
pub use menu::{ContextMenu, Menu, MenuLookup, Menus, Shortcuts};
pub use peek::{AutoHide, Peek, Show};
pub use ribbon::{FrameReport, Ribbon, RibbonState};
pub use theme::{Metrics, Overlays, Palette, Preset, Theme};
