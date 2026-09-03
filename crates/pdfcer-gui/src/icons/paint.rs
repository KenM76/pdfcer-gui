//! # icons::paint — the seam `egui-shell` asks the application to fill
//!
//! `egui-shell` renders a ribbon it is **forbidden to understand**. An icon
//! set is a licensing decision, a rasterization decision and a look; none of
//! those are a shell's business, so the shell carries an opaque icon **key**
//! (`Command::icon`, a `String`) and calls back into the application to draw
//! it. [`paint_ribbon_icon`] is pdfcer's answer to that callback.
//!
//! ## The shape of the seam, and why it is that shape
//!
//! ```text
//! egui_shell::ribbon::IconPainter<'a> = dyn FnMut(&egui::Painter, &IconRequest<'_>) + 'a
//! ```
//!
//! The painter receives an [`egui::Painter`] and **not** a `&mut egui::Ui`.
//! That is deliberate on the shell's side and it constrains everything here:
//! the icon is painted into a slot *inside a button that is already being
//! laid out*, whose rectangle the button's own layout computed. A `&mut Ui`
//! would let the application allocate layout space inside a widget that has
//! already decided its size, which corrupts the button's geometry in ways
//! that surface as text overlapping its own frame. A `Painter` can draw and
//! cannot allocate, so the seam is safe *by type* rather than by
//! instruction.
//!
//! The consequence for this module is that it must be able to rasterize,
//! upload and draw with nothing but a `Painter` in hand — which is why the
//! texture cache is a thread-local (see [`super::cache`]) rather than
//! something threaded through a call chain that has no room for it.
//!
//! ## ★ Supplying a painter is what turns the ribbon into a ribbon
//!
//! `egui_shell::ribbon::qat`'s `shows_label` draws a QAT control icon-only
//! only when **all three** hold: the command names an icon, it has a tooltip
//! to serve as that icon's accessible name, and *the application actually
//! supplied a painter*. Its doc comment records why the third clause exists:
//!
//! > an application that registers icon keys but supplies no
//! > `Ribbon::with_icon_painter` used to get a row of blank boxes — a
//! > control with no label, no glyph and no explanation.
//!
//! Until a painter is supplied the whole ribbon falls back to text buttons.
//! Supplying one is the difference between a toolbar and a ribbon.
//!
//! ## ★ An unknown key draws a VISIBLE MARK, never nothing
//!
//! This is the decision this module most needs a reader to understand,
//! because the obvious alternative is wrong in a way that is easy to miss.
//!
//! "Draw nothing and let the caller fall back to a label" **cannot work**.
//! The fallback is not downstream of the painter — it is *upstream* of it.
//! `shows_label` is evaluated from `ctx.icons.is_some()` when the control's
//! atoms are assembled, before any key is resolved and before the painter is
//! ever called. By the time this function discovers it does not recognise a
//! key, the label has already been dropped from the button and a square slot
//! has already been reserved. Drawing nothing into that slot produces
//! exactly the row of blank grey boxes that `shows_label`'s third clause was
//! added to prevent, and it produces it *silently*.
//!
//! So an unrecognised key is drawn as [`paint_missing_mark`]: a rounded
//! square with a diagonal slash through it, in the same theme foreground
//! tint as a real glyph. Three properties make that the right answer:
//!
//! * **It is visible.** The control has an identity, a hit target and a
//!   tooltip; it is legibly *wrong* rather than invisibly absent.
//! * **It cannot be mistaken for a real icon.** Nothing in the set is a
//!   plain slashed square — the mark reads as "no glyph for this", which is
//!   the truth, and reads that way at 16 px.
//! * **It is not a placeholder.** The no-placeholders rule forbids drawing a
//!   *guess* at what belongs there. This draws a disclosure that nothing
//!   does. Those are opposites: a placeholder invites the reader to believe
//!   the interface is finished, and this one states that it is not.
//!
//! The alternative of guessing at a near-match key (`"fit_page"` →
//! `Icon::FitPage`) is refused for the same reason
//! [`super::Icon::from_key`] is an exact lookup: a fuzzy resolver draws the
//! *wrong* glyph for a typo, and a wrong glyph is undetectable where a
//! missing one is obvious.
//!
//! The key is additionally reported through [`crate::diag`] so the
//! offending string can be read out of a trace, rather than guessed at from
//! a screenshot of a slashed box.
//!
//! ## The selected cue, and the seam that had to widen to carry it
//!
//! [`super::IconWeight::Bold`] — the "selected state is never colour alone"
//! cue that replaces emboldening a text label on a control that has no text
//! — **is** applied here, for a selected control.
//!
//! It could not be when this module first landed. `IconRequest` carried
//! `enabled` but not `selected`: the shell knew the state (it passes
//! `selected` to `egui::Button::selected`) and did not forward it, so
//! across this seam the only selected cues were the button frame egui
//! paints and the tint the shell derives — both of which a theme can make
//! subtle, and neither of which is the glyph.
//!
//! That was recorded here as a limitation of the seam rather than of the
//! pipeline, with a note that closing it was a one-field addition on the
//! `egui-shell` side and that the consumer was already implemented and
//! waiting. The field was then added for exactly this consumer.
//!
//! The general point is worth more than the fix. A reusable shell can
//! reserve the slot, derive the tint and track the interaction state, but
//! it **cannot** honour "never colour alone" on the application's behalf,
//! because the second cue lives in the glyph and only the application can
//! draw one. Every rule of that shape ends as a field on the request.

use egui_shell::ribbon::IconRequest;

use super::cache::with_cache;
use super::svg::VIEWBOX;
use super::{Icon, IconWeight};

/// The stroke width the assets are authored at, in viewBox units.
///
/// Only [`paint_missing_mark`] needs this — real glyphs carry their own
/// `stroke-width` — but it is the set's weight, and the missing mark has to
/// look like it belongs to the same family or it reads as a rendering
/// artefact rather than as a deliberate report.
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
        Some(icon) => paint_icon(painter, icon, request.rect, request.tint, weight),
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
/// # ★ Rasterized at the physical pixel size, drawn at the logical one
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
    let square = centred_square(rect);
    if square.width() <= 0.0 {
        return;
    }
    let ctx = painter.ctx();
    let px = (square.width() * ctx.pixels_per_point()).round().max(1.0) as u32;
    let handle = with_cache(|cache| cache.texture(ctx, icon, px, weight));
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
///
/// Named rather than rebuilt at each call site so that "draw the entire
/// glyph" is stated once; a subtly wrong UV rect would crop a glyph in a way
/// that looks like bad artwork.
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
    ///
    /// Tests must not invent a colour: the whole point of the theming story
    /// is that no colour is chosen outside the theme module, and a test that
    /// reached for a literal would be modelling something the application
    /// never does.
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
    ///
    /// Shape count is a coarse instrument, and deliberately so: asserting on
    /// `epaint`'s internal shape *variants* would pin this module to an
    /// implementation detail of a dependency, and the property that actually
    /// matters here is "did anything get drawn at all", because the failure
    /// being guarded against is *nothing* getting drawn.
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
    ///
    /// A bare egui frame is not guaranteed to emit zero shapes — plugins and
    /// the root `Ui` may contribute — so every assertion below is a
    /// *difference* against a frame that painted no icon. Asserting on the
    /// raw total would make these tests depend on egui's internals rather
    /// than on this module's behaviour.
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

    /// ★ An unknown key must NOT be a blank slot.
    ///
    /// The failure this guards against is precise: by the time the painter
    /// is called, `shows_label` has already dropped the control's text label
    /// on the strength of a painter existing. A painter that silently draws
    /// nothing therefore produces a control with no label, no glyph and no
    /// explanation — the exact row of blank boxes the shell's third
    /// `shows_label` clause was added to prevent, reintroduced from the
    /// application's side.
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
    ///
    /// `fit_page` is one character away from a real key. Resolving it fuzzily
    /// would draw a plausible glyph and hide the typo forever; the mark makes
    /// it visible on the first frame.
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

    /// ★ The function really does satisfy the shell's painter bound.
    ///
    /// The wiring is one line in another module, and if the bound did not
    /// hold it would fail there rather than here — in a file this module's
    /// author does not own, during a build somebody else is running. This
    /// coerces `paint_ribbon_icon` to the shell's own
    /// `IconPainter` type alias, which is the same check the ribbon
    /// performs, done here where it is this module's problem.
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
