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
///
/// Its header carries why this one ceiling cannot be derived the way the two in
/// [`crate::viewer::ceiling`] are: the wall is inside `tiny-skia` and is
/// content-dependent, measured 28x apart on two pages of the same document, so
/// the only honest source of the number is a refusal that has already happened.
pub mod ceiling;

/// **The pixel proof for O137's "line weights off" display mode** — that the
/// mode really thins a drawing, and thins it in the direction the operator
/// asked for rather than the opposite one.
///
/// `#![cfg(test)]`, so it compiles to nothing in a release build. Its own
/// header carries why four passing wiring tests were not enough.
mod hairline;

/// **The ground OUTSIDE the sheet** — O23's "see" half: the box to
/// rasterize so that an object placed past the page edge is actually painted,
/// and how far the visible-region tier may look past the sheet.
///
/// Its header carries the one-sentence cause — `render_page` sizes its pixmap
/// to the `/CropBox`, so nothing culls the content, there are simply no pixels
/// out there — and why a halo that would not fit is declined rather than
/// clamped.
pub mod halo;

/// **Is there anything in this raster?** — the `ink=` field of
/// `render-async-done`, and the reason a blank canvas at deep zoom can be told
/// apart from a lost one.
///
/// Its header is the record of an afternoon spent proving the shell innocent by
/// hand: a near-uniform canvas has two causes, and until this module existed
/// the harness could see only one of them.
pub mod ink;

/// **Tests only** — the engine properties O23's second half will stand on,
/// asserted here because the engine's own suite has never exercised them.
///
/// `render_page_region` accepts a rectangle outside the `/CropBox` by
/// construction and is untested there; a shell feature built on an unexercised
/// engine path is one whose first failure looks like a shell defect.
pub mod offpage;

/// **The blank page nothing reports** — O219's *"the view goes blank"*, given
/// a diagnostic for the first time.
///
/// A texture upload that fails for want of graphics memory raises
/// `GL_OUT_OF_MEMORY` on a flag that `egui_glow` reads only under
/// `debug_assertions`, so in a release build it is completely silent and
/// presents as an empty rectangle drawn at full frame rate. Its header carries
/// the frame boundary the whole module is built around — an upload ordered in
/// one frame is performed at the end of it and its error is first readable at
/// the top of the next — and why attribution refuses to guess.
pub mod pressure;

/// **Screen ⟷ PDF for a RASTER** — the two conversions the region tier
/// needs, kept together because they are inverses and the round trip is the
/// property that matters.
///
/// Its header carries the y flip, which is the half that goes wrong: a missed
/// flip shows the opposite end of the page, which at deep zoom looks like a
/// blank raster rather than a coordinate error.
pub mod raster;

/// **The window's rectangle, in the space the engine documents** — the region
/// tier's one conversion from canvas space to PDF user space.
///
/// Its header carries O174: this module handed `render_page_region` a
/// canvas-space rectangle, which is right for an upright page at the origin and
/// wrong for a turned one, so the operator's `/Rotate 270` sheet jumped and
/// distorted above the whole-page → region crossover and nowhere below it.
pub mod region;

/// **Whole page, or just the window?** — O24's one decision, made from
/// numbers in one place.
///
/// Its header carries the constraint that shaped it: panning at full detail is
/// a property of rasterizing the WHOLE PAGE, and region rendering would cost
/// it. So the region path engages only above the pixmap ceiling, where the
/// whole-page path cannot work at all — nothing is taken away to pay for it.
pub mod strategy;
// Render-ahead: which page outside the viewport to fill next, and what
// bounds it. Consulted by `settle` only once everything visible is drawn.
mod prefetch;
// The per-frame raster decision, and the strip's scheduling.
pub mod settle;
// Several pages at once: the bounded texture cache, and what an undrawn page
// says about itself.
pub mod strip;
pub mod worker;
