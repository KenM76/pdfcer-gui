//! # render — turning a page into pixels, and pixels into a texture
//!
//! The raster leaf: the worker, the texture upload, the strip cache and the
//! per-frame GL pressure read. It depends on no other top-level module of this
//! crate; the per-frame scheduling that drives it is `crate::app::settle`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/mod.md`.

// Each `pub mod` / `pub use` sits directly under its own doc comment.

/// **The zoom ceiling this document TAUGHT the shell** — O186's
/// *"zoom should stop at the limit and not end up showing an error"*.
pub use pdfcer_gui_base::rasterceiling as ceiling;

/// **The ground OUTSIDE the sheet** — O23's "see" half: the box to
/// rasterize so that an object placed past the page edge is actually painted,
/// and how far the visible-region tier may look past the sheet.
pub use pdfcer_gui_base::rasterhalo as halo;

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
// Several pages at once: the bounded texture cache, and what an undrawn page
// says about itself.
pub use pdfcer_gui_base::renderstrip as strip;
pub use pdfcer_gui_base::renderworker as worker;
