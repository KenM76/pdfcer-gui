//! # `render::raster` — the bridge from `pdfcer-render`'s pixmaps to egui textures
//!
//! One job, kept in one place: take a [`tiny_skia::Pixmap`] out of
//! [`pdfcer_render::render_page`] and hand egui a
//! [`egui::TextureHandle`] it can draw, plus the [`Diagnostics`] that
//! came with it. Everything about GPU-texture lifetimes and pixel
//! formats is confined here so the canvas deals only in "do I have a
//! current texture for this page at this zoom."
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/raster.md`.

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use pdfcer_render::{Diagnostics, tiny_skia};

use crate::render::worker::RenderKey;

/// A rasterized page, uploaded and ready to draw.
#[derive(Clone)]
pub struct PageTexture {
    /// The uploaded raster. Freed when this struct drops.
    pub texture: TextureHandle,
    /// Everything this is a picture *of* — page, raster scale, annotation
    /// visibility, layer-override generation.
    ///
    /// The scale inside it is in **device pixels** per PDF user-space unit,
    /// i.e. the operator-visible zoom already multiplied by the display's
    /// `pixels_per_point` ([`crate::viewer::raster_scale`]). Staleness is
    /// compared against that, not against the logical zoom, so dragging the
    /// window to a monitor with a different density re-rasterizes rather
    /// than leaving a soft picture behind.
    pub key: RenderKey,
    /// The honesty report that came with these pixels — which glyphs were
    /// substituted, which features were skipped. Displayed in the status
    /// bar; never discarded.
    pub diagnostics: Diagnostics,
    /// How long the rasterization that produced these pixels took.
    ///
    /// Carried straight across from [`crate::render::worker::RenderedPixels`],
    /// whose field documents what is and is not inside the measurement. Read by
    /// the `tools.render_diagnostics` dialog, which is the one surface that
    /// answers *"what did the renderer do with the last page?"* — and which
    /// needs the scale in [`Self::key`] beside it, because on this project's
    /// documents ~99 % of render cost is resolution-independent and a duration
    /// with no scale beside it invites exactly the wrong conclusion.
    pub elapsed: std::time::Duration,
}

impl std::fmt::Debug for PageTexture {
    /// Hand-written because `egui::TextureHandle`'s own `Debug` prints
    /// the whole texture-manager state, which is noise in a panic
    /// message. The fields below are what identifies a cached raster.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageTexture")
            .field("key", &self.key)
            .field("diagnostics", &self.diagnostics)
            .field("elapsed", &self.elapsed)
            .finish_non_exhaustive()
    }
}

/// The egui texture name the page raster is uploaded under.
const PAGE_TEXTURE_ID: &str = "pdfcer-page"; // ui-text-exempt: internal texture id, never displayed

/// Convert a `tiny-skia` pixmap into an egui image.
fn pixmap_to_color_image(pixmap: &tiny_skia::Pixmap) -> ColorImage {
    ColorImage::from_rgba_premultiplied(
        [pixmap.width() as usize, pixmap.height() as usize],
        pixmap.data(),
    )
}

/// The most pixels a whole-page raster may have and still be kept as the
/// backdrop.
pub const BASE_MAX_PIXELS: u32 = 4_000_000;

/// Whether this raster is small enough to keep as the page's backdrop.
#[must_use]
pub fn within_base_budget(pixels: &crate::render::worker::RenderedPixels) -> bool {
    pixels
        .pixmap
        .width()
        .checked_mul(pixels.pixmap.height())
        .is_some_and(|n| n <= BASE_MAX_PIXELS)
}

/// Upload pixels a background worker produced, as a [`PageTexture`].
#[must_use]
pub fn texture_from_pixels(
    ctx: &Context,
    surface: crate::render::pressure::Surface,
    pixels: &crate::render::worker::RenderedPixels,
) -> PageTexture {
    let image = pixmap_to_color_image(&pixels.pixmap);
    // Recorded BEFORE the upload is ordered, and from the key rather than
    // from the image: this is the provenance a `GL_OUT_OF_MEMORY` drained at
    // the top of the next frame is matched against, and GL's error flag
    // carries none of its own. See `crate::render::pressure`.
    //
    // Here rather than at the call site because this is the one function every
    // page-raster upload passes through — the same reason the key is carried
    // across rather than recomputed below.
    crate::render::pressure::record_raster(
        ctx,
        surface,
        &pixels.key,
        pixels.pixmap.width(),
        pixels.pixmap.height(),
    );
    // LINEAR is what makes a stale texture drawn at a new zoom read as
    // *soft* rather than blocky, which is the free staleness signal the
    // canvas relies on while a background render is in flight.
    let texture = ctx.load_texture(PAGE_TEXTURE_ID, image, TextureOptions::LINEAR);
    PageTexture {
        texture,
        // Carried straight across from the pixels, which took it from the
        // request they were rendered from. Nothing here recomputes it: a
        // second derivation of the same key is how a texture comes to be
        // labelled with inputs it was not drawn with.
        key: pixels.key,
        diagnostics: pixels.diagnostics.clone(),
        // Carried, not re-measured. The upload happens on the UI thread and
        // the render did not; a clock started here would time the wrong thing
        // and would do it plausibly, which is the worst combination available.
        elapsed: pixels.elapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The premultiplied convention must survive the conversion.
    #[test]
    fn a_pixmap_is_read_as_premultiplied_not_unmultiplied() {
        let mut pixmap = tiny_skia::Pixmap::new(1, 1).expect("1x1 pixmap");
        pixmap.data_mut().copy_from_slice(&[128, 0, 0, 128]);
        let image = pixmap_to_color_image(&pixmap);
        assert_eq!(image.size, [1, 1]);
        let px = image.pixels[0];
        // epaint's Color32 is itself premultiplied, so a correct read is
        // byte-for-byte identity. `from_rgba_unmultiplied` would instead
        // multiply the already-multiplied red by the alpha again and give
        // r = 64.
        assert_eq!((px.r(), px.g(), px.b(), px.a()), (128, 0, 0, 128));
    }

    /// A fully opaque pixel — the normal case for a page raster — must
    /// come through untouched under either reading.
    #[test]
    fn an_opaque_pixel_survives_the_conversion_unchanged() {
        let mut pixmap = tiny_skia::Pixmap::new(1, 1).expect("1x1 pixmap");
        pixmap.data_mut().copy_from_slice(&[10, 200, 30, 255]);
        let px = pixmap_to_color_image(&pixmap).pixels[0];
        assert_eq!((px.r(), px.g(), px.b(), px.a()), (10, 200, 30, 255));
    }
}
