//! # `canvas::dimpreview` — a ce dimension drawn as the commit will bake it
//!
//! [`bake`] asks `EditSession::dimension_preview` for the appearance the
//! commit would write for a dragged ce dimension's moved geometry, and
//! [`bake_new`] asks `EditSession::new_dimension_preview` for the one
//! `add_dimension` would write for a ce dimension still being placed; [`paint`] rasterises it with
//! `pdfcer_render::edit_preview::paint_dimension_preview` over the part of its
//! `/Rect` that is on screen, and draws that texture. The pixels are the
//! commit's own, label and extension lines included, so the preview cannot show
//! one dimension and release another.
//!
//! The committed ce dimension is still in the page tiles underneath, and a
//! shortened extension line is a sub-segment of the committed one, so the bake
//! alone would change nothing visible. [`paint`] therefore first covers the
//! committed dimension's `/Rect` with an **underlay**: that region rendered
//! with the dimension's annotation omitted (`RenderOptions::omit_annotations`),
//! once per drag and cached as a texture. Unrotated pages only; on a rotated
//! page the bake is drawn without it. A ce dimension being placed has nothing
//! committed underneath, so it gets no underlay.
//!
//! [`paint`] returning `false` means nothing was drawn and the caller draws
//! the segment outline instead: the bake was refused, the box is off screen,
//! or it would exceed [`MAX_SIDE_PX`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/dimpreview.md`.

use egui::{Color32, Pos2, Rect};
use pdfcer_core::dimension::{DimensionId, DimensionKind, DimensionPreview, GroupId};
use pdfcer_render::tiny_skia;

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;

const TEXTURE: &str = "dimension-drag-ink"; // ui-text-exempt: a texture name, never displayed.
const TRACE: &str = "dim-preview"; // ui-text-exempt: diagnostic trace event name.
const UNDERLAY: &str = "dimension-drag-underlay"; // ui-text-exempt: a texture name, never displayed.

/// The largest side, in pixels, the preview is rasterised at. The raster is
/// already clipped to the visible canvas, so this only binds on a very large
/// monitor.
const MAX_SIDE_PX: f32 = 8192.0;

/// A bake and the ce dimension it is of.
#[derive(Debug, Clone)]
pub struct Baked {
    /// The dragged ce dimension, whose committed annotation the underlay
    /// covers; `None` for one being placed, which has none.
    pub id: Option<DimensionId>,
    /// The engine's bake of the moved geometry.
    pub preview: DimensionPreview,
}

/// The engine's bake of `moved` for dimension `id`, or `None` when it refuses.
#[must_use]
pub fn bake(doc: &OpenDoc, id: DimensionId, moved: &DimensionKind) -> Option<Baked> {
    match doc.session.dimension_preview(id, moved) {
        Ok(preview) => Some(Baked {
            id: Some(id),
            preview,
        }),
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("{TRACE} id={} baked=0 refused={why}", id.0)
            });
            None
        }
    }
}

/// The engine's bake of a ce dimension `kind` not yet added to `group`, or
/// `None` when it refuses.
#[must_use]
pub fn bake_new(doc: &OpenDoc, group: GroupId, kind: &DimensionKind) -> Option<Baked> {
    match doc.session.new_dimension_preview(group, kind) {
        Ok(preview) => Some(Baked { id: None, preview }),
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("{TRACE} new=1 baked=0 refused={why}")
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
    baked: &Baked,
) -> bool {
    let preview = &baked.preview;
    let ctx = painter.ctx();
    let Some(m) = page_to_screen(page, map) else {
        return false;
    };
    if let Some(id) = baked.id {
        underlay(painter, doc, page, clip, m, id);
    }
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

/// What an underlay texture was rendered for; a different key re-renders.
#[derive(Clone, Copy, PartialEq)]
struct UnderlayKey {
    annot: pdfcer_core::object::ObjId,
    body: [u32; 4],
    ppp: u32,
}

/// Cover the committed dimension's `/Rect` with the page as it renders without
/// that annotation. Drawn only for an unrotated page (`m` axis-aligned, y
/// flipped); rendered once per key and cached.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
fn underlay(
    painter: &egui::Painter,
    doc: &OpenDoc,
    page: &pdfcer_core::page_tree::Page,
    clip: Rect,
    m: [f32; 6],
    id: DimensionId,
) {
    if m[1] != 0.0 || m[2] != 0.0 || m[0] <= 0.0 || m[3] >= 0.0 {
        return;
    }
    let model = doc.session.dimension_model();
    let Some(record) = model.dimensions().iter().find(|r| r.id == id) else {
        return;
    };
    let Some(annot) = record.annot else {
        return;
    };
    let Ok(committed) = doc.session.dimension_preview(id, &record.kind) else {
        return;
    };
    let r = committed.appearance.rect;
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point();
    let screen = |x: f64, y: f64| Pos2::new(m[0] * x as f32 + m[4], m[3] * y as f32 + m[5]);
    let clipped = Rect::from_two_pos(screen(r.llx, r.lly), screen(r.urx, r.ury))
        .expand(2.0)
        .intersect(clip);
    if clipped.width() <= 0.0 || clipped.height() <= 0.0 {
        return;
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
        return;
    }
    let key = UnderlayKey {
        annot,
        body: [body.min.x, body.min.y, body.max.x, body.max.y].map(f32::to_bits),
        ppp: ppp.to_bits(),
    };
    let memory = egui::Id::new(UNDERLAY);
    let cached = ctx.data(|d| d.get_temp::<(UnderlayKey, egui::TextureHandle)>(memory));
    let texture = if let Some((_, texture)) = cached.filter(|(k, _)| *k == key) {
        texture
    } else {
        // Page space of the snapped screen rect, so the raster lands 1:1.
        let region = pdfcer_core::page_tree::Rect {
            llx: f64::from((body.min.x - m[4]) / m[0]),
            urx: f64::from((body.max.x - m[4]) / m[0]),
            lly: f64::from((body.max.y - m[5]) / m[3]),
            ury: f64::from((body.min.y - m[5]) / m[3]),
        };
        let options = doc.settings.render_options().with_omit_annotations([annot]);
        let rendered = match pdfcer_render::render_page_region(
            &doc.session.view(),
            page,
            m[0] * ppp,
            region,
            &options,
        ) {
            Ok(rendered) => rendered,
            Err(why) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("{TRACE} underlay=0 refused={why}")
                });
                return;
            }
        };
        let (w, h) = (rendered.pixmap.width(), rendered.pixmap.height());
        let image = egui::ColorImage::from_rgba_premultiplied(
            [w as usize, h as usize],
            rendered.pixmap.data(),
        );
        crate::render::pressure::record_other(
            ctx,
            crate::render::pressure::Surface::DimensionDrag,
            w,
            h,
        );
        let texture = ctx.load_texture(UNDERLAY, image, egui::TextureOptions::NEAREST);
        ctx.data_mut(|d| d.insert_temp(memory, (key, texture.clone())));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("{TRACE} underlay=1 omit={} px={w}x{h}", annot.num)
        });
        texture
    };
    painter.image(
        texture.id(),
        body,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        // NOT A THEME COLOUR: the identity tint.
        Color32::WHITE,
    );
}
