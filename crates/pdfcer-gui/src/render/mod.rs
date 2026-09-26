//! # render — turning a page into pixels, and pixels into a texture
//!
//! Two modules with one seam between them, and the seam is the reason the
//! split exists:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/mod.md`.

//! ## What Phase 4 added, and why it is two modules rather than one
//!
//! Continuous scroll puts several pages on screen. Two things follow, and they
//! are different kinds of thing:
//!
//! | module | subject |
//! |---|---|
//! | [`strip`] | *storage* — the bounded cache of the other visible pages' textures, its pixel budget, and what a page with no texture draws instead of a white rectangle |
//! | [`settle`] | *scheduling* — which page is rasterized next, what waits for a zoom to settle, and how a texture is rehomed when scrolling changes which page is current |
//!
//! Neither touches the single-page path: [`strip::StripRasters`] is empty for
//! the whole of a single-page session, and [`settle`]'s strip pass returns on
//! an `is_empty` check.
//!
//! [`settle`] also holds what used to be the second half of
//! `crate::app::state` — the per-frame staleness decision — moved here when
//! Phase 4 doubled its size. That file's header already named the seam: it
//! answers *"what is open"*, and this answers *"what should the picture be"*.

// A NOTE ON THE ORDER OF WHAT FOLLOWS, because it has already eaten four
// module headers once and the damage is silent.
//
//
// The rule that keeps it fixed: **each `pub mod` sits directly under its own
// doc comment, and the run stays alphabetical.** A doc comment stranded above a
// `pub mod` whose name it does not describe is the tell.

/// **The zoom ceiling this document TAUGHT the shell** — O186's
/// *"zoom should stop at the limit and not end up showing an error"*.
pub use pdfcer_gui_base::rasterceiling as ceiling;

/// **The ground OUTSIDE the sheet** — O23's "see" half: the box to
/// rasterize so that an object placed past the page edge is actually painted,
/// and how far the visible-region tier may look past the sheet.
pub use pdfcer_gui_base::rasterhalo as halo;

/// **Tests only** — the engine properties O23's second half will stand on,
/// asserted here because the engine's own suite has never exercised them.
pub mod offpage;

/// **The blank page nothing reports** — O219's *"the view goes blank"*, given
/// a diagnostic for the first time.
pub mod pressure;

/// Rendered pixmaps into egui textures.
pub use pdfcer_gui_base::raster;

/// **The window's rectangle, in the space the engine documents** — the region
/// tier's one conversion from canvas space to PDF user space.
pub use pdfcer_gui_base::rasterregion as region;

/// **Whole page, or just the window?** — O24's one decision, made from
/// numbers in one place.
pub use pdfcer_gui_base::rasterstrategy as strategy;
// Render-ahead: which page outside the viewport to fill next, and what
// bounds it. Consulted by `settle` only once everything visible is drawn.
mod prefetch;
// The per-frame raster decision, and the strip's scheduling.
pub mod settle;
// Several pages at once: the bounded texture cache, and what an undrawn page
// says about itself.
pub use pdfcer_gui_base::renderstrip as strip;
pub use pdfcer_gui_base::renderworker as worker;
