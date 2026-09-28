//! # `canvas::dimpreview` — a ce dimension drag drawn as the commit will bake it
//!
//! [`bake`] asks `EditSession::dimension_preview` for the appearance the
//! commit would write for the moved geometry; [`paint`] rasterises it with
//! `pdfcer_render::edit_preview::paint_dimension_preview` over the part of its
//! `/Rect` that is on screen, and draws that texture. The pixels are the
//! commit's own, label and extension lines included, so the preview cannot show
//! one dimension and release another.
//!
//! [`paint`] returning `false` means nothing was drawn and the caller draws
//! the segment outline instead: the bake was refused, the box is off screen,
//! or it would exceed [`MAX_SIDE_PX`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/dimpreview.md`.

use egui::{Color32, Pos2, Rect};
use pdfcer_core::dimension::{DimensionId, DimensionKind, DimensionPreview};
use pdfcer_render::tiny_skia;

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;

const TEXTURE: &str = "dimension-drag-ink"; // ui-text-exempt: a texture name, never displayed.
const TRACE: &str = "dim-preview"; // ui-text-exempt: diagnostic trace event name.

/// The largest side, in pixels, the preview is rasterised at. The raster is
/// already clipped to the visible canvas, so this only binds on a very large
/// monitor.
const MAX_SIDE_PX: f32 = 8192.0;

/// The engine's bake of `moved` for dimension `id`, or `None` when it refuses.
#[must_use]
pub fn bake(doc: &OpenDoc, id: DimensionId, moved: &DimensionKind) -> Option<DimensionPreview> {
    match doc.session.dimension_preview(id, moved) {
        Ok(preview) => Some(preview),
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("{TRACE} id={} baked=0 refused={why}", id.0)
            });
            None
        }
    }
}

/// The page-space to screen affine, `[a, b, c, d, e, f]` with
/// `x' = a·x + c·y + e`, read off three mapped points.
fn page_to_screen(page: &pdfcer_core::page_tree::Page, map: &PageMapping) -> Option<[f32; 6]> {
    let at = |x: f64, y: f64| {
        crate::canvas::measure::page_to_screen(pdfcer_core::vector::Point::new(x, y), page, map)
    };
    let o = at(0.0, 0.0)?;
    let ex = at(1.0, 0.0)? - o;
    let ey = at(0.0, 1.0)? - o;
    Some([ex.x, ex.y, ey.x, ey.y, o.x, o.y])
}

/// Draw `preview` on the canvas, clipped to `clip`. `false` when nothing was
/// drawn.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn paint(
    painter: &egui::Painter,
    doc: &OpenDoc,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
    clip: Rect,
    preview: &DimensionPreview,
) -> bool {
    let ctx = painter.ctx();
    let Some(m) = page_to_screen(page, map) else {
        return false;
    };
    let r = preview.appearance.rect;
    let corners = [
        (r.llx, r.lly),
        (r.urx, r.lly),
        (r.llx, r.ury),
        (r.urx, r.ury),
    ]
    .map(|(x, y)| {
        Pos2::new(
            m[0] * x as f32 + m[2] * y as f32 + m[4],
            m[1] * x as f32 + m[3] * y as f32 + m[5],
        )
    });
    let ppp = ctx.pixels_per_point();
    // Snapped outward to whole physical pixels, so the texture is drawn 1:1.
    // Off the grid every texel is resampled across two pixels and thin text
    // comes out grey, lighter than the commit draws it.
    let clipped = Rect::from_points(&corners).expand(2.0).intersect(clip);
    if clipped.width() <= 0.0 || clipped.height() <= 0.0 {
        return false;
    }
    let body = Rect::from_min_max(
        Pos2::new(
            (clipped.min.x * ppp).floor() / ppp,
            (clipped.min.y * ppp).floor() / ppp,
        ),
        Pos2::new(
            (clipped.max.x * ppp).ceil() / ppp,
            (clipped.max.y * ppp).ceil() / ppp,
        ),
    );
    if body.width() * ppp > MAX_SIDE_PX || body.height() * ppp > MAX_SIDE_PX {
        return false;
    }
    let (w, h) = (
        (body.width() * ppp).round().max(1.0) as u32,
        (body.height() * ppp).round().max(1.0) as u32,
    );
    let Some(mut pixmap) = tiny_skia::Pixmap::new(w, h) else {
        return false;
    };
    let to_pixels = tiny_skia::Transform::from_row(
        m[0] * ppp,
        m[1] * ppp,
        m[2] * ppp,
        m[3] * ppp,
        (m[4] - body.min.x) * ppp,
        (m[5] - body.min.y) * ppp,
    );
    let diagnostics = pdfcer_render::edit_preview::paint_dimension_preview(
        &doc.session.view(),
        preview,
        &doc.settings.render_options(),
        to_pixels,
        &mut pixmap,
    );
    let image = egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], pixmap.data());
    crate::render::pressure::record_other(
        ctx,
        crate::render::pressure::Surface::DimensionDrag,
        w,
        h,
    );
    let texture = ctx.load_texture(TEXTURE, image, egui::TextureOptions::NEAREST);
    painter.image(
        texture.id(),
        body,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        // NOT A THEME COLOUR: the identity tint.
        Color32::WHITE,
    );
    let [q0, q1, q2, q3] = preview.appearance.label_quad;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "{TRACE} baked=1 px={w}x{h} faults={} label={:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}",
            diagnostics.sample_ops.len(),
            q0.x,
            q0.y,
            q1.x,
            q1.y,
            q2.x,
            q2.y,
            q3.x,
            q3.y
        )
    });
    true
}
