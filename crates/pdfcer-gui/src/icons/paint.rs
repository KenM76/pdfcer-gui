//! # icons::paint — the seam `egui-shell` asks the application to fill
//!
//! `egui-shell` renders a ribbon it is **forbidden to understand**. An icon
//! set is a licensing decision, a rasterization decision and a look; none of
//! those are a shell's business, so the shell carries an opaque icon **key**
//! (`Command::icon`, a `String`) and calls back into the application to draw
//! it. [`paint_ribbon_icon`] is pdfcer's answer to that callback.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/icons/paint.md`.

use egui_shell::ribbon::IconRequest;

use super::cache::{Baked, with_cache};
use super::svg::VIEWBOX;
use super::{Icon, IconWeight};

/// The stroke width the assets are authored at, in viewBox units.
const ASSET_STROKE_UNITS: f32 = 2.5;

/// Paint one ribbon icon. **This is the function to hand to
/// `egui_shell::ribbon::Ribbon::with_icon_painter`.**
///
/// ```ignore
/// let mut icons = pdfcer_gui::icons::paint_ribbon_icon;
/// let report = Ribbon::new(&registry, &conditions, &manifest)
///     .with_icon_painter(&mut icons)
///     .render(ui, &mut state);
/// ```
///
/// The `&mut` binding is what the shell's
/// `with_icon_painter(&'a mut (impl FnMut(&egui::Painter, &IconRequest<'_>) + 'a))`
/// asks for; a plain `fn` item satisfies `FnMut`, so no closure and no
/// captured state is needed. That is a property worth keeping: a painter
/// with no state cannot be the thing that goes stale.
///
/// # Behaviour
///
/// * A key in the catalogue is drawn as its glyph, rasterized at the
///   **current physical pixel size** of the reserved rect and tinted with
///   [`IconRequest::tint`] (which the shell derives from the theme and the
///   widget's interaction state, so hover, active and disabled all follow
///   without anything here tracking them).
/// * A key **not** in the catalogue is drawn as a visible missing-icon mark
///   and reported to [`crate::diag`]. See this module's header for why it is
///   emphatically not "draw nothing".
/// * A **selected** control is drawn at [`IconWeight::Bold`].
///
/// # The Bold weight is a second cue, not decoration
///
/// The shell already shows selection with the button's frame, so drawing
/// Regular for everything looks correct and would never be reported as a
/// bug. The rule it would quietly break is **selected state is never
/// colour alone**: a frame is one cue, and in a theme whose selected and
/// unselected frames are close in value it can be a weak one. A heavier
/// stroke is a second, achromatic cue that survives that.
///
/// This could not be done when the icon set landed —
/// `egui_shell::ribbon::IconRequest` carried `enabled` but not `selected`,
/// so the ribbon path had no way to know. The field was added afterwards
/// for exactly this consumer; the shell cannot honour the rule on the
/// application's behalf, because the second cue lives in the glyph, which
/// only the application can draw.
///
/// It never panics, and it never allocates layout.
pub fn paint_ribbon_icon(painter: &egui::Painter, request: &IconRequest<'_>) {
    let weight = if request.selected {
        IconWeight::Bold
    } else {
        IconWeight::Regular
    };
    match Icon::from_key(request.key) {
        Some(icon) => {
            let baked = super::accent::baked(painter.ctx(), icon, request.tint, request.enabled);
            paint_glyph(painter, icon, request.rect, request.tint, weight, baked);
        }
        None => {
            // A diagnostic trace, never displayed in the UI. `trace_changed`
            // rather than `trace` so a key that is missing on every frame is
            // reported once rather than sixty times a second.
            //
            // Both lines carry their own `ui-text-exempt` marker because
            // `check-ui-strings.sh` excludes the body of a `diag::trace(`
            // call by paren depth but matches that name literally, so
            // `trace_changed(` is outside the exclusion — and its block-form
            // marker only reaches the single line after the comment.
            crate::diag::trace_changed("icon-unknown-key", || {
                // ui-text-exempt: diagnostic slot name
                format!("icon-unknown-key key={}", request.key) // ui-text-exempt: diagnostic trace
            });
            paint_missing_mark(painter, request.rect, request.tint);
        }
    }
}

/// Draw one known icon into `rect`, tinted `tint`.
///
/// The primitive [`paint_ribbon_icon`] is built from, exposed because menus,
/// the status bar and any hand-drawn control need the same thing and must
/// not re-derive the DPI arithmetic.
///
/// # Rasterized at the physical pixel size, drawn at the logical one
///
/// This is the load-bearing decision of the whole pipeline, and the reason
/// the set is SVG path data rather than pre-baked PNGs.
///
/// `rect` is in **logical points**. The raster is built at
/// `side * ctx.pixels_per_point()` **physical pixels** and then drawn back
/// into the logical rect. Rasterizing at the logical size instead would make
/// every icon visibly soft on any HiDPI display — a 16 px raster stretched
/// over 32 device pixels at 200% Windows scaling — and a pre-baked PNG has
/// that stretch permanently baked in, wrong again for any future "larger
/// toolbar icons" accessibility option.
///
/// Because the physical size is part of the cache key, dragging the window
/// between a 100% and a 150% monitor re-rasterizes automatically rather than
/// reusing a stale, wrongly-sized texture. Nothing has to notice the change
/// and tell the cache about it; asking for the right size *is* the
/// invalidation.
///
/// # Geometry
///
/// The glyph is drawn into the largest centred **square** that fits `rect`.
/// The shell always reserves a square (`Vec2::splat(metrics.icon_pts)`), so
/// in practice this is the identity — but the assets are authored in a
/// square viewBox and a non-square rect would stretch them, which is the
/// kind of thing that shows up as "the magnifier looks like an egg" and gets
/// attributed to the artwork.
pub fn paint_icon(
    painter: &egui::Painter,
    icon: Icon,
    rect: egui::Rect,
    tint: egui::Color32,
    weight: IconWeight,
) {
    paint_glyph(painter, icon, rect, tint, weight, None);
}

/// [`paint_icon`], or with `baked` the coloured glyph, drawn at `tint`'s alpha.
fn paint_glyph(
    painter: &egui::Painter,
    icon: Icon,
    rect: egui::Rect,
    tint: egui::Color32,
    weight: IconWeight,
    baked: Option<Baked>,
) {
    let square = centred_square(rect);
    if square.width() <= 0.0 {
        return;
    }
    let ctx = painter.ctx();
    let px = (square.width() * ctx.pixels_per_point()).round().max(1.0) as u32;
    let handle = with_cache(|cache| cache.texture_baked(ctx, icon, px, weight, baked));
    // NOT A THEME COLOUR: a baked glyph carries its colours; this is its alpha.
    let tint = if baked.is_some() {
        egui::Color32::from_white_alpha(tint.a())
    } else {
        tint
    };
    painter.image(handle.id(), square, FULL_UV, tint);
}

/// Draw the "there is no glyph for this key" mark into `rect`.
///
/// A rounded square with a diagonal slash, stroked in `tint` at the set's
/// own weight so it sits at the same optical density as the real glyphs
/// beside it.
///
/// # Why this shape
///
/// It has to satisfy three constraints at once, and the intersection is
/// small:
///
/// 1. **Not confusable with any real icon.** The set has squares
///    ([`Icon::ShapeRect`]) and it has diagonals
///    ([`Icon::ShapeArrow`], [`Icon::Close`]), but nothing is a square with
///    a single diagonal through it, and nothing else is drawn to the slot's
///    full extent.
/// 2. **Legible at 16 px.** Two strokes, no interior detail, no text — a
///    "?" glyph would be the obvious choice and is rejected: at 16 px it is
///    a blob, and it would have to come from a font, which is exactly the
///    dependency on glyph coverage that half this icon set exists to escape.
/// 3. **Reads as a report, not as art.** Deliberately geometric and
///    deliberately plain. An operator who sees one should think "something
///    is missing here", which is true, rather than "what does that mean".
///
/// It is drawn in the ordinary foreground tint rather than in an alarm
/// colour. A missing icon is a defect in the *application's* wiring, not an
/// error the operator caused or can act on; colouring it as danger would put
/// an alarm in their interface about somebody else's mistake, and the
/// colour would have to be a raw one anyway.
pub fn paint_missing_mark(painter: &egui::Painter, rect: egui::Rect, tint: egui::Color32) {
    let square = centred_square(rect);
    if square.width() <= 0.0 {
        return;
    }
    // Inset by one stroke so the outline sits inside the reserved slot
    // rather than straddling its edge, matching how every asset insets its
    // content from the viewBox edge.
    let width = (square.width() * ASSET_STROKE_UNITS / VIEWBOX).max(1.0);
    let body = square.shrink(width);
    if body.width() <= 0.0 {
        return;
    }
    let stroke = egui::Stroke::new(width, tint);

    painter.rect_stroke(
        body,
        // A small radius, in the same spirit as the set's `rx="1"`/`rx="2"`
        // rects. Not a circle and not a hard square: both of those are
        // shapes the set uses for real meanings.
        egui::CornerRadius::same((width * 1.5).round().clamp(1.0, 255.0) as u8),
        stroke,
        egui::StrokeKind::Inside,
    );
    // The slash runs corner to corner of the INNER box, so it terminates on
    // the outline rather than crossing it — a slash that overshot would read
    // as a "no entry" prohibition sign, which claims something stronger than
    // "this key is unknown".
    let inner = body.shrink(width);
    painter.line_segment([inner.left_top(), inner.right_bottom()], stroke);
}

/// The whole texture, in normalised texture coordinates.
const FULL_UV: egui::Rect = egui::Rect {
    min: egui::Pos2 { x: 0.0, y: 0.0 },
    max: egui::Pos2 { x: 1.0, y: 1.0 },
};

/// The largest square centred inside `rect`.
///
/// See [`paint_icon`], "Geometry": the assets are authored in a square
/// viewBox, so a non-square slot must letterbox rather than stretch.
fn centred_square(rect: egui::Rect) -> egui::Rect {
    let side = rect.width().min(rect.height());
    if side <= 0.0 {
        return egui::Rect::from_center_size(rect.center(), egui::Vec2::ZERO);
    }
    egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(side))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::theme::{Preset, Theme};

    /// A tint taken from the theme, exactly as the shell derives one.
    fn theme_tint() -> egui::Color32 {
        Theme::new(Preset::Dark).palette.text
    }

    /// The slot the ribbon reserves: square, `metrics.icon_pts` a side.
    fn slot() -> egui::Rect {
        let side = Theme::new(Preset::Dark).metrics.icon_pts;
        egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::Vec2::splat(side))
    }

    /// Run one frame, calling `f` with the frame's `Painter` — the same
    /// thing the shell hands an `egui_shell::ribbon::IconPainter` — and
    /// report how many shapes reached the frame's output.
    fn shapes_from(f: impl FnOnce(&egui::Painter)) -> usize {
        let ctx = egui::Context::default();
        let mut once = Some(f);
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(f) = once.take() {
                f(ui.painter());
            }
        });
        output.shapes.len()
    }

    /// Shapes attributable to `f`, with the frame's own baseline removed.
    fn painted(f: impl FnOnce(&egui::Painter)) -> usize {
        shapes_from(f).saturating_sub(shapes_from(|_| {}))
    }

    /// The control, stated as a test rather than left implicit: two frames
    /// that paint no icon agree. If this ever fails, every "it drew
    /// something" assertion below is measuring noise.
    #[test]
    fn the_baseline_frame_is_stable() {
        assert_eq!(shapes_from(|_| {}), shapes_from(|_| {}));
        assert_eq!(painted(|_| {}), 0);
    }

    /// Every key a command can name resolves and draws.
    ///
    /// This is the whole-catalogue integration of the seam: parse, rasterize,
    /// upload and draw, once per icon, through the real thread-local cache.
    #[test]
    fn every_catalogue_key_paints_something() {
        for &icon in Icon::ALL {
            let drawn = painted(|painter| {
                paint_ribbon_icon(
                    painter,
                    &IconRequest {
                        key: icon.name(),
                        rect: slot(),
                        tint: theme_tint(),
                        enabled: true,
                        selected: false,
                    },
                );
            });
            assert!(drawn > 0, "icon '{}' painted nothing", icon.name());
        }
    }

    /// An unknown key must NOT be a blank slot.
    #[test]
    fn an_unknown_key_draws_a_visible_mark_rather_than_nothing() {
        let drawn = painted(|painter| {
            paint_ribbon_icon(
                painter,
                &IconRequest {
                    key: "no-such-icon",
                    rect: slot(),
                    tint: theme_tint(),
                    enabled: true,
                    selected: false,
                },
            );
        });
        assert!(
            drawn > 0,
            "an unknown key drew nothing — that is a blank box in the ribbon"
        );
    }

    /// The empty key is an unknown key, not a special case. A command whose
    /// icon field somehow arrived empty must still leave a visible control.
    #[test]
    fn an_empty_key_is_treated_as_unknown() {
        let drawn = painted(|painter| {
            paint_ribbon_icon(
                painter,
                &IconRequest {
                    key: "",
                    rect: slot(),
                    tint: theme_tint(),
                    enabled: true,
                    selected: false,
                },
            );
        });
        assert!(drawn > 0);
    }

    /// A near-miss key is reported, not silently repaired.
    #[test]
    fn a_near_miss_key_gets_the_mark_rather_than_the_nearest_glyph() {
        assert_eq!(Icon::from_key("fit_page"), None);
        let drawn = painted(|painter| {
            paint_ribbon_icon(
                painter,
                &IconRequest {
                    key: "fit_page",
                    rect: slot(),
                    tint: theme_tint(),
                    enabled: true,
                    selected: false,
                },
            );
        });
        assert!(drawn > 0);
    }

    /// A disabled control still gets a glyph. The shell expresses disabled
    /// through the tint (and egui's own opacity multiplier); an icon set that
    /// answered `enabled: false` by drawing nothing would leave a hole where
    /// a greyed-out control belongs.
    #[test]
    fn a_disabled_control_still_gets_its_glyph() {
        let drawn = painted(|painter| {
            paint_ribbon_icon(
                painter,
                &IconRequest {
                    key: "open",
                    rect: slot(),
                    tint: theme_tint(),
                    enabled: false,
                    selected: false,
                },
            );
        });
        assert!(drawn > 0);
    }

    /// A degenerate slot is survived rather than drawn into. A zero-width
    /// rect can be handed over by a layout that ran out of room, and the
    /// correct response is to draw nothing at all — there is no space in
    /// which a mark could be seen, so a mark would only be a stray pixel.
    #[test]
    fn a_zero_sized_slot_draws_nothing_and_does_not_panic() {
        for key in ["open", "no-such-icon"] {
            let drawn = painted(|painter| {
                paint_ribbon_icon(
                    painter,
                    &IconRequest {
                        key,
                        rect: egui::Rect::from_min_size(egui::pos2(4.0, 4.0), egui::Vec2::ZERO),
                        tint: theme_tint(),
                        enabled: true,
                        selected: false,
                    },
                );
            });
            assert_eq!(drawn, 0, "key '{key}' drew into a zero-sized slot");
        }
    }

    /// A non-square slot letterboxes rather than stretching.
    #[test]
    fn a_non_square_slot_yields_a_centred_square() {
        let wide = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(40.0, 16.0));
        let square = centred_square(wide);
        assert_eq!(square.width(), 16.0);
        assert_eq!(square.height(), 16.0);
        assert_eq!(square.center(), wide.center());
    }

    /// The function really does satisfy the shell's painter bound.
    #[test]
    fn the_painter_satisfies_the_shell_seam() {
        let mut f = paint_ribbon_icon;
        let painter: &mut egui_shell::ribbon::IconPainter<'_> = &mut f;
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            painter(
                ui.painter(),
                &IconRequest {
                    key: "save",
                    rect: slot(),
                    tint: theme_tint(),
                    enabled: true,
                    selected: false,
                },
            );
        });
    }
}
