//! # render — turning a page into pixels, and pixels into a texture
//!
//! Two modules with one seam between them, and the seam is the reason the
//! split exists:
//!
//! | module | runs on | knows about |
//! |---|---|---|
//! | [`worker`] | a background thread | `pdfcer-render`, never egui |
//! | [`raster`] | the UI thread | egui textures, never rasterization |
//!
//! **Rasterization can happen on any thread; texture upload cannot** — it
//! needs an `egui::Context`, which belongs to the UI thread. That single
//! fact is why [`worker::RenderWorker`] returns a `Pixmap` rather than a
//! `TextureHandle`, and why [`raster::texture_from_pixels`] exists as the
//! other half of it.
//!
//! Both files are Class A salvage from
//! `D:\Dev\pdfce\crates\pdfce-gui\src\`: `render_worker.rs` (466 code lines
//! plus 116 test lines) and `raster.rs` (363 code lines). Their
//! documentation is carried across rather than paraphrased, because it
//! records measured evidence — 28.9 ms to cancel a render against
//! 10,367 ms to let one finish; a real CAD sheet at ~10 s at 1× and ~58 s
//! at 2× — that cannot be re-derived by reading the code and that decides
//! the design.
//!
//! ## What is deliberately NOT here yet
//!
//! - **A thread pool** for thumbnails and adjacent-page prerender. The
//!   worker is single-slot by design (see [`worker::RenderWorker`]); a pool
//!   is a different structure and arrives with the page rail at stage S3.
//! - **The thumbnail cache.** It was part of the salvaged `raster.rs`, and
//!   it belongs with the Pages panel that consumes it, not with a canvas
//!   that has no rail.
//! - **A display list.** `BENCHMARK.md`'s single biggest win, and
//!   explicitly post-fold-in work. It would replace what happens *inside*
//!   the worker, not the worker.

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

/// ★★ **Tests only** — the engine properties O23's second half will stand on,
/// asserted here because the engine's own suite has never exercised them.
///
/// `render_page_region` accepts a rectangle outside the `/CropBox` by
/// construction and is untested there; a shell feature built on an unexercised
/// engine path is one whose first failure looks like a shell defect.
/// **The pixel proof for O137's "line weights off" display mode** — that the
/// mode really thins a drawing, and thins it in the direction the operator
/// asked for rather than the opposite one.
///
/// `#![cfg(test)]`, so it compiles to nothing in a release build. Its own
/// header carries why four passing wiring tests were not enough.
mod hairline;

pub mod offpage;
pub mod raster;
/// ★★★ **Whole page, or just the window?** — O24's one decision, made from
/// numbers in one place.
///
/// Its header carries the constraint that shaped it: panning at full detail is
/// a property of rasterizing the WHOLE PAGE, and region rendering would cost
/// it. So the region path engages only above the pixmap ceiling, where the
/// whole-page path cannot work at all — nothing is taken away to pay for it.
/// ★★ **Screen ⟷ PDF for a RASTER** — the two conversions the region tier
/// needs, kept together because they are inverses and the round trip is the
/// property that matters.
///
/// Its header carries the y flip, which is the half that goes wrong: a missed
/// flip shows the opposite end of the page, which at deep zoom looks like a
/// blank raster rather than a coordinate error.
pub mod region;
pub mod strategy;
// The per-frame raster decision, and the strip's scheduling.
pub mod settle;
// Several pages at once: the bounded texture cache, and what an undrawn page
// says about itself.
pub mod strip;
pub mod worker;
