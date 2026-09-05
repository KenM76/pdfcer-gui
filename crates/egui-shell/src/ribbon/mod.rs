//! The ribbon renderer — draws a [`crate::manifest::Shell`] and reports
//! what the operator asked for.
//!
//! This file is the module **root**: what the ribbon is made of, which
//! file holds which decision, and the one rule that ties them together.
//! The reasoning for each part lives with that part — see the map below —
//! because a header that explained twelve files would be the file nobody
//! updates when one of them changes.
//!
//! # What is drawn, top to bottom
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │ [QAT…] │ File View Pages ⏷ 2 more         ( Read │ Review │ Edit )   │  ← tab-strip row
//! ├──────────────────────────────────────────────────────────────────────┤
//! │  [ ][ ][ ]  │  [ ][ ]   │  [ ][ ][ ]                        ⏷ 2 more  │  ← band
//! │  Page display│  Render   │  Window                                    │
//! └──────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # The map: which file holds which decision
//!
//! | File | Owns | The rule it holds |
//! |---|---|---|
//! | [`render`] | the builder and the one entry point | **The shell reports intent; the application dispatches.** Reads the ribbon's true width before anything is drawn. |
//! | [`state`] | [`RibbonState`] | The two facts the ribbon itself decides: which tab, which mode. |
//! | [`frame_report`] | [`FrameReport`] | What the last frame *contained* — and the distinction between what was planned and what was drawn. |
//! | [`plan`] | all layout arithmetic, with no `egui` in it | The overflow affordance is reserved **before** content is measured against the remainder. |
//! | [`strip`] | the whole tab-strip row | QAT → mode selector → tab overflow → tabs, in that order, into rectangles computed before anything is drawn. |
//! | [`tabs`] | which tabs exist, and how one looks | Mode's tabs, then contextual tabs; R84 redundant cues. |
//! | [`mode_selector`] | the N-position segmented control | Every position labelled and reachable; a roving keyboard tab stop. |
//! | [`qat`] | the quick-access toolbar | Never behind a tab switch; icon-only is earned, not assumed. |
//! | [`trailing`] | the controls past the mode selector | Registered commands only — the region an application extends without the shell learning what it extended it with. |
//! | [`band`] | the row of captioned groups | **One closure, every group, caption cannot be omitted.** |
//! | [`ctx`] | the per-frame render context | One condition set for the whole ribbon, so no two surfaces can disagree about what is true. |
//! | [`report`] | published rectangles | Stable names, zero cost when unused. |
//! | [`a11y`] | accessible names | Tooltips become names; the `egui` tab-role ceiling is stated rather than papered over. |
//!
//! # Why the split is where it is
//!
//! Three seams, each of which is a different *kind* of thing rather than a
//! line count:
//!
//! - **A builder that draws one frame** ([`render`]) is the crate's API
//!   surface and carries the architectural seam — intent out, no dispatch
//!   in. It is the file an application author reads.
//! - **State that survives between frames** ([`state`]) and **a report
//!   about the frame just drawn** ([`frame_report`]) look similar and are
//!   opposites: one is an input to the next frame, one is an output of the
//!   last. Filing them together is how "what the plan decided" and "what
//!   was drawn" got confused once already; see [`frame_report`]'s header.
//! - **Arithmetic with no `egui` in it** ([`plan`]) versus **drawing**
//!   (everything else). That boundary is not organisational, it is what
//!   makes the overflow invariant testable without a window.
//!
//! # ★ Layout order is the enforcement mechanism
//!
//! This is the one rule that spans every file above, and it is why the
//! ribbon is split the way it is.
//!
//! (The [`trailing`] region is the one reserved region that MAY be squeezed
//! out — see [`plan::plan_strip_row`]. It is an optional extra whose absence
//! costs nothing, where the four below are load-bearing.)
//!
//! Four things on this ribbon must never be squeezed out by content: the
//! **mode selector**, the **QAT**, the **band's overflow affordance** and
//! the **tab strip's overflow affordance**. All four are handled the same
//! way — their space is subtracted from the row **first**, and the content
//! is given the remainder — and all four subtractions live in [`plan`],
//! which has no `egui` in it and can therefore be tested without a window.
//!
//! The alternative spelling ("draw the content, then draw the control in
//! what is left") is the obvious immediate-mode one, reads correctly, and
//! is the source of `MODES_AND_PANELS.md` Part 2's failure mode #8:
//! *"past ~6 tabs the overflow button itself gets hidden, leaving no route
//! to the hidden tabs."* Reserving first makes the failure unreachable
//! rather than merely unlikely.
//!
//! It is not sufficient on its own, and that is worth knowing before
//! editing any of this: **`egui` does not clip a `Ui`'s children to its
//! `max_rect`**. A widget that does not fit is still laid out, still
//! allocated, still given a `Response` and a `Rect` — it is simply painted
//! where nobody can see or click it. So a row that merely *nests* a
//! reserved island inside a layout has protected the island and nothing
//! else. Both rows therefore compute explicit rectangles and draw into
//! them; see [`strip`]'s header for the measurements that made that
//! necessary, and [`render`]'s for the `entitled` rectangle both rows
//! depend on.
//!
//! # What is *not* here yet
//!
//! - **Key bindings.** [`crate::manifest::Keymap`] is parsed and
//!   validated, but the ribbon does not consume input for it. Chord
//!   handling belongs with the application's own input pass, because the
//!   application owns the question of what has focus and what a chord
//!   should mean while a text field is open.
//! - **Drag-to-customize.** `SHELL_FRAMEWORK.md` §5 permits an operator to
//!   move a command between groups; doing it *by dragging on the ribbon*
//!   is a later stage. The manifest already expresses the result.

pub mod a11y;
pub mod band;
pub(crate) mod collapsed;
pub(crate) mod control;
pub mod ctx;
pub mod frame_report;
pub mod mode_selector;
// The four width primitives every ribbon row plans with -- button padding,
// the truncation floor, a separator's cost and text measurement. Split from
// `band` under R2 on 2026-09-04; see that module's header for the seam.
pub(crate) mod measure;
pub(crate) mod overflow;
pub mod plan;
pub mod qat;
pub mod render;
pub mod report;
pub mod rhythm;
// How much room one control asks for, and what it shows -- the three item
// sizes, the earned-icon-only rule, and the visible_when filter.
// RIBBON_SCALING.md.
pub mod sizing;
pub mod state;
pub mod strip;
pub mod tabs;
// The trailing controls at the far right of the tab-strip row, past the mode
// selector -- the seam for a control whose presence is a property of the
// machine. See `crate::manifest::Trailing`.
pub mod trailing;

// Test-only, and separate files rather than one `mod tests` for three
// reasons: R2 caps a source file at 1,500 lines; the width tests need a
// fixture (a synthetic font) whose construction has nothing to do with the
// ribbon and should not be read as if it did; and structural tests and
// geometric tests are different kinds of claim that should not be filed
// together. `height_tests` splits off the *vertical* geometry — how many
// rows a group uses and whether the band is the same height on two tabs
// (R128) — from `width_tests`' horizontal one, and borrows that file's
// synthetic-face harness rather than standing up a second one. See each
// file's header.
#[cfg(test)]
mod height_tests;
#[cfg(test)]
mod testfont;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod width_tests;

#[cfg(test)]
mod scroll_tests;
// Rendered geometry for the three item sizes and the visible_when filter.
// Separate from `width_tests` only because that file is at the R2 limit.
#[cfg(test)]
mod sizing_tests;

pub use band::{SELECTED_CONDITION_PREFIX, selected_condition};
pub use ctx::{CustomItem, CustomItemRenderer, IconPainter, IconRequest};
pub use frame_report::FrameReport;
pub use render::Ribbon;
pub use report::RectSink;
pub use state::RibbonState;
