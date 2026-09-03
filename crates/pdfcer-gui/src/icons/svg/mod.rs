//! # icons::svg — the SVG-subset parser and the tiny-skia rasterizer
//!
//! Turns one asset's text ([`super::assets`]) into geometry, and geometry
//! into a white-on-transparent coverage mask at a caller-chosen **physical
//! pixel** size. Nothing here knows what any icon means; it knows how to
//! read a path `d` attribute and how to stroke it.
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\icons.rs` (Class A,
//! `SALVAGE.md`). The doc comments below are carried across with their
//! reasoning intact — every one of them records a decision that cost real
//! investigation, and the ones about *refusing* rather than guessing are
//! the reason a hand-rolled parser is safe to rely on at all.
//!
//! This module is the **element scanner**: it walks tags, reads attributes,
//! decides how a shape is painted, and rasterizes the result. The **path
//! `d` grammar** — the lexer, the command state machine and the arc
//! conversion — is [`path`], split out because it is a different subject
//! that grows for different reasons (a new element touches only this file, a
//! new path command touches only that one) and because together they were
//! within a hundred lines of the 1,500-line limit.
//!
//! ## ★ What the parser supports, and what it refuses
//!
//! This is deliberately NOT a general SVG implementation. It reads exactly
//! the shape of file the [`super::assets`] §3 style contract describes:
//!
//! * **Elements:** `<svg>` (opened/closed, attributes ignored), XML
//!   comments, `<path>`, `<rect>`, `<circle>`. **Any other element is an
//!   error** — `<g>`, `<use>`, `<text>`, `<defs>`, gradients, transforms and
//!   CSS are all out of subset and rejected loudly, never skipped, because
//!   silently skipping a `<g transform=…>` would draw a correctly-shaped
//!   glyph in the wrong place.
//! * **Attributes:** `d`, `x`, `y`, `width`, `height`, `rx`, `cx`, `cy`,
//!   `r`, `stroke`, `fill`, `stroke-width`, `stroke-linecap`,
//!   `stroke-linejoin`. Unknown attributes are ignored (they are cosmetic
//!   metadata like `aria-hidden`/`xmlns`, never geometry).
//! * **Paint:** `stroke="currentColor"` strokes; `stroke="none"`/absent does
//!   not. `fill="currentColor"` fills; `fill="none"`/absent does not. The
//!   colour VALUE is discarded — see "Theming" below — but its presence or
//!   absence decides whether the shape is drawn at all, so
//!   `stroke="currentColor"` and `stroke="none"` are not interchangeable.
//!   `stroke-linecap` accepts `butt`/`round`/`square`; `stroke-linejoin`
//!   accepts `miter`/`round`/`bevel`. Any other value is an error rather
//!   than a silent fallback to the default, because a wrong cap on a
//!   2.5-unit stroke at 16 px is a visible defect that would otherwise ship
//!   unnoticed.
//! * **Path commands and numbers:** the complete SVG path grammar
//!   (`M m L l H h V v C c S s Q q T t A a Z z`), the SVG number grammar
//!   with implicit separators, and packed arc flags. All of it lives in
//!   [`path`], whose header explains the two lexing rules that look like
//!   details and are not. Anything outside the grammar is
//!   [`IconError::UnsupportedPathCommand`], never a skip.
//!
//! Every failure mode returns an [`IconError`] carrying enough context to
//! find the offending byte; nothing falls back to "draw something".
//!
//! ## ★ Theming: one raster per icon, tinted at draw time
//!
//! Every asset is `stroke="currentColor"` — a single-colour outline with no
//! palette. So each icon is rasterized ONCE as a **white-on-transparent
//! coverage mask**, and the colour is applied at draw time by the caller.
//! Consequences, all of them deliberate:
//!
//! * Light theme, dark theme, hovered and disabled all share ONE raster.
//!   There are no light/dark asset pairs to keep in sync, and structurally
//!   no way for an icon to end up hardcoded-black on a dark background —
//!   which is exactly the drift `tools/gates/check-theme-colors.sh` exists
//!   to catch, removed here by construction rather than by policing.
//! * The tint is therefore **not** part of the cache key
//!   (`super::cache::CacheKey`) — that is the entire point of the mask.
//!   Re-tinting is free; re-rastering is not.
//!
//! Because the mask is white `(255,255,255,a)` premultiplied to `(a,a,a,a)`,
//! a multiplicative tint yields `(a·Tr, a·Tg, a·Tb, a)` — a correctly
//! premultiplied, correctly antialiased tinted glyph, for any tint.
//! `mask_is_white_so_tinting_is_exact` pins that property, because it is an
//! *arithmetic* precondition of the theming story rather than a convention
//! anyone would notice breaking.

use std::fmt;

use pdfcer_render::tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform,
};

use super::IconWeight;
use path::parse_path_data;

mod path;

/// The viewBox edge length every asset uses (`viewBox="0 0 48 48"`).
///
/// All geometry in the assets is in these units; [`IconArt::rasterize`]
/// scales by `px / VIEWBOX` and lets tiny-skia scale the stroke width with
/// it, which is why a 2.5-unit stroke stays optically identical at every
/// output size.
pub const VIEWBOX: f32 = 48.0;

/// Stroke-width multiplier for [`IconWeight::Bold`].
///
/// 1.35 was chosen as the smallest factor that is unambiguously visible at
/// 16 pt (2.5 → 3.375 viewBox units, ~1.1 physical px heavier at 100% scale)
/// without the glyph starting to blob shut at its tightest interior features
/// (`keyboard.svg`'s 3-unit key gaps, `shape-highlight.svg`'s hatch). It is
/// a *cue*, not a redesign.
const BOLD_STROKE_FACTOR: f32 = 1.35;

/// Circular-arc-to-cubic magic constant: the control-point offset, as a
/// fraction of the radius, that makes a single cubic Bézier approximate a
/// 90° circular arc to within ~0.02%. `4/3 * (sqrt(2) - 1)`.
const KAPPA: f32 = 0.552_284_8;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Why an icon asset could not be turned into geometry.
///
/// Every variant means "this asset is wrong", never "this input was
/// untrusted" — the assets are compiled-in constants, so any of these is an
/// authoring bug that `super::tests::every_icon_parses` is there to catch
/// before it ships. They carry position/context because the alternative (a
/// bare "parse failed") turns a two-minute fix into an afternoon.
///
/// Hand-written `Display`/`Error` impls rather than `thiserror`: this crate
/// does not depend on `thiserror`, and adding a dependency to spell four
/// lines of `match` would be a poor trade against the workspace's
/// "no dependency pdfcer's lockfile does not already carry" rule.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IconError {
    /// An element outside the supported subset (`<g>`, `<use>`, `<defs>`,
    /// …). Refused rather than skipped: skipping a wrapper that carries a
    /// `transform` would draw the right shape in the wrong place.
    UnsupportedElement(String),
    /// A path-data letter that is not one of `MmLlHhVvCcSsQqTtAaZz`.
    UnsupportedPathCommand(char),
    /// Path data that begins with something other than a `MoveTo`, or a
    /// command issued with no current point.
    NoCurrentPoint(char),
    /// A number that could not be lexed at the given byte offset.
    MalformedNumber {
        /// Byte offset into the path data where the number started.
        offset: usize,
    },
    /// An arc flag that was neither `0` nor `1` at the given byte offset.
    MalformedFlag {
        /// Byte offset into the path data of the offending character.
        offset: usize,
    },
    /// Path/attribute data ran out mid-command.
    UnexpectedEnd,
    /// A shape element was missing a geometry attribute it cannot be drawn
    /// without (e.g. `<circle>` with no `r`).
    MissingAttribute {
        /// The element that was missing it.
        element: &'static str,
        /// The attribute name.
        attribute: &'static str,
    },
    /// A `stroke-linecap` / `stroke-linejoin` value outside the subset.
    /// Refused rather than defaulted, because a silently wrong cap is a
    /// visible defect nobody would think to look for.
    UnsupportedPaintValue {
        /// The attribute whose value was rejected.
        attribute: &'static str,
        /// The rejected value.
        value: String,
    },
    /// A tag was opened and never closed (no `>` before end of input).
    UnterminatedTag,
    /// The geometry parsed, but tiny-skia rejected it (an empty or
    /// non-finite path). Practically unreachable for a hand-authored icon;
    /// present so the `Option` from `PathBuilder::finish` is never
    /// `unwrap`ped.
    DegeneratePath,
}

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ui-text-exempt: developer-facing asset diagnostics. Every one of
        // these means a compiled-in constant is malformed, which is a build
        // defect addressed to whoever edited it; none of these strings is
        // ever rendered in the GUI (see `super::cache::IconCache::texture`,
        // whose operator-visible consequence is a blank raster, not text).
        match self {
            Self::UnsupportedElement(tag) => {
                write!(
                    f,
                    "unsupported SVG element <{tag}> (icon subset: svg, path, rect, circle)"
                )
            }
            Self::UnsupportedPathCommand(c) => {
                write!(f, "unsupported SVG path command '{c}'")
            }
            Self::NoCurrentPoint(c) => {
                write!(
                    f,
                    "path command '{c}' issued with no current point (data must start with M/m)"
                )
            }
            Self::MalformedNumber { offset } => {
                write!(f, "malformed number in path data at byte {offset}")
            }
            Self::MalformedFlag { offset } => {
                write!(
                    f,
                    "malformed arc flag at byte {offset} (must be exactly '0' or '1')"
                )
            }
            Self::UnexpectedEnd => write!(f, "SVG data ended mid-command"),
            Self::MissingAttribute { element, attribute } => {
                write!(
                    f,
                    "<{element}> is missing the required '{attribute}' attribute"
                )
            }
            Self::UnsupportedPaintValue { attribute, value } => {
                write!(f, "unsupported {attribute} value '{value}'")
            }
            Self::UnterminatedTag => write!(f, "unterminated SVG tag (no '>')"),
            Self::DegeneratePath => write!(f, "path produced no drawable geometry"),
        }
    }
}

impl std::error::Error for IconError {}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

/// One drawable element of an icon: geometry plus how to paint it.
///
/// Kept separate per element rather than merged into one path because the
/// set mixes paint styles within a single icon — `redact.svg` has stroked
/// outlines *and* one filled bar, and `shape-highlight.svg` deliberately
/// mixes a 2.5-unit contour with a 1-unit hatch.
#[derive(Debug)]
struct Shape {
    /// Geometry in viewBox units (0..48).
    path: pdfcer_render::tiny_skia::Path,
    /// Stroke width in viewBox units, or `None` for `stroke="none"`.
    stroke_width: Option<f32>,
    /// Cap style for stroking (ignored when `stroke_width` is `None`).
    line_cap: LineCap,
    /// Join style for stroking (ignored when `stroke_width` is `None`).
    line_join: LineJoin,
    /// Whether `fill="currentColor"` was set — the one style exception.
    filled: bool,
}

/// A parsed icon: an ordered list of shapes in viewBox units.
///
/// Order is paint order, exactly as written in the asset, because outline
/// icons routinely rely on a later stroke crossing an earlier one.
#[derive(Debug)]
pub struct IconArt {
    shapes: Vec<Shape>,
}

impl IconArt {
    /// Parse an SVG asset into drawable geometry.
    ///
    /// See this module's header for the exact supported subset. This is a
    /// scanner, not an XML parser: it walks the byte stream looking for
    /// `<`, dispatches on the tag name, and reads `name="value"` attribute
    /// pairs out of the tag body. That is sufficient (and safe) because the
    /// only inputs are the crate's own compiled-in constants, which are
    /// mechanically uniform by the style contract — and any input that is
    /// *not* uniform hits [`IconError::UnsupportedElement`] rather than
    /// being interpreted loosely.
    ///
    /// # Errors
    ///
    /// Returns the [`IconError`] describing the first thing it refused to
    /// guess at. It never returns partial geometry: a half-read asset would
    /// draw a half-glyph, which looks like a rendering fault rather than an
    /// authoring one and is therefore attributed to the wrong subsystem.
    pub fn parse(source: &str) -> Result<Self, IconError> {
        let bytes = source.as_bytes();
        let mut shapes = Vec::new();
        let mut i = 0usize;

        while i < bytes.len() {
            // Skip everything that is not a tag opener. Character data
            // between tags is whitespace in every asset; there is no <text>
            // in the subset, so it is simply ignored.
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }

            // XML comment — the style contract requires every asset to carry
            // one naming the concept and disclaiming trademark risk, so this
            // branch is hit by every asset.
            if bytes[i..].starts_with(b"<!--") {
                match find(bytes, i + 4, b"-->") {
                    Some(end) => {
                        i = end + 3;
                        continue;
                    }
                    None => return Err(IconError::UnterminatedTag),
                }
            }

            let tag_end = match bytes[i..].iter().position(|&b| b == b'>') {
                Some(off) => i + off,
                None => return Err(IconError::UnterminatedTag),
            };
            // `get` rather than direct slicing: the indices come from a byte
            // scan, and a stray multi-byte character inside a tag would make
            // them non-char-boundaries. Refuse rather than panic.
            let body = source
                .get(i + 1..tag_end)
                .ok_or(IconError::UnterminatedTag)?;

            // A close tag (`</svg>`) carries no geometry. Skipping it is safe
            // *because* the subset has no container elements: an unsupported
            // opener like `<g>` is rejected below, so no close tag can ever
            // be the end of something whose effect we would be missing.
            if body.starts_with('/') {
                i = tag_end + 1;
                continue;
            }

            let name = tag_name(body);

            match name {
                // Structural, no geometry: the root element (whose own
                // fill="none" is the set-wide default this parser already
                // assumes), plus any XML declaration.
                "svg" | "?xml" => {}
                "path" => shapes.push(parse_path_element(body)?),
                "rect" => shapes.push(parse_rect_element(body)?),
                "circle" => shapes.push(parse_circle_element(body)?),
                other => return Err(IconError::UnsupportedElement(other.to_owned())),
            }
            i = tag_end + 1;
        }

        Ok(Self { shapes })
    }

    /// How many drawable shapes the asset contains.
    ///
    /// Exists so a test can prove a multi-element asset was fully read
    /// rather than truncated at the first element. The renderer never needs
    /// to count shapes.
    #[must_use]
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Whether any shape is filled — the set's one style exception.
    ///
    /// Exposed so a test can assert that redaction's icon really is the
    /// filled one and that nothing else in the set is.
    #[must_use]
    pub fn has_fill(&self) -> bool {
        self.shapes.iter().any(|s| s.filled)
    }

    /// Rasterize to a square white-on-transparent coverage mask of `px`
    /// physical pixels a side (module header, "Theming").
    ///
    /// The colour written is always opaque white; only the alpha channel
    /// carries information, and the caller's tint supplies the hue at draw
    /// time. Antialiasing is on — at 16 pt these glyphs are ~2 px strokes
    /// and aliased diagonals would be immediately obvious.
    ///
    /// Returns a 1×1 transparent image (never panics, never `None`) if the
    /// pixmap allocation fails for an absurd `px`; a missing icon is a
    /// cosmetic defect, a crashed editor holding unsaved edits is not.
    #[must_use]
    pub fn rasterize(&self, px: u32, weight: IconWeight) -> egui::ColorImage {
        let px = px.max(1);
        let Some(mut pixmap) = Pixmap::new(px, px) else {
            return blank_image();
        };

        let scale = px as f32 / VIEWBOX;
        let transform = Transform::from_scale(scale, scale);
        let weight_factor = match weight {
            IconWeight::Regular => 1.0,
            IconWeight::Bold => BOLD_STROKE_FACTOR,
        };

        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        // NOT A THEME COLOUR: opaque WHITE, always, and it is not a look —
        // only the ALPHA channel of the result carries information (module
        // header, "Theming"). The hue arrives later, from the theme-derived
        // tint the ribbon hands the painter. Theming this would break the
        // mask.
        paint.set_color(Color::WHITE);

        for shape in &self.shapes {
            // Fill first, then stroke, matching SVG's own painting order for
            // an element that has both (redact.svg's bar is fill-only, but
            // the ordering must be right if that ever changes).
            if shape.filled {
                pixmap
                    .as_mut()
                    .fill_path(&shape.path, &paint, FillRule::Winding, transform, None);
            }
            if let Some(width) = shape.stroke_width {
                let stroke = Stroke {
                    width: width * weight_factor,
                    line_cap: shape.line_cap,
                    line_join: shape.line_join,
                    ..Stroke::default()
                };
                // tiny-skia strokes in PATH space and transforms the
                // resulting outline, so the stroke width scales with
                // `transform` — which is exactly what a viewBox needs and why
                // no manual width compensation appears here.
                pixmap
                    .as_mut()
                    .stroke_path(&shape.path, &paint, &stroke, transform, None);
            }
        }

        // tiny-skia's buffer is premultiplied RGBA8 and egui's Color32 is
        // premultiplied sRGBA, so this is a straight reinterpretation with no
        // un-premultiply/re-premultiply round trip (which would lose
        // precision in exactly the antialiased edge pixels that are the whole
        // visual quality of a 16 pt glyph).
        let pixels = pixmap
            .pixels()
            .iter()
            .map(|p| {
                // NOT A THEME COLOUR: arithmetic on an existing pixel, not a
                // choice of one — this reassembles a coverage value the
                // rasterizer already produced. The colour a viewer actually
                // sees is the theme's foreground, applied as a tint.
                egui::Color32::from_rgba_premultiplied(p.red(), p.green(), p.blue(), p.alpha())
            })
            .collect();
        egui::ColorImage::new([px as usize, px as usize], pixels)
    }
}

/// A 1×1 fully transparent image — the "could not draw this" result.
///
/// One helper rather than two literals so the degraded case is identical
/// wherever it is reached, and so it is greppable.
#[must_use]
pub(super) fn blank_image() -> egui::ColorImage {
    // NOT A THEME COLOUR: the absence of a colour rather than a choice of
    // one. Nothing is drawn; there is no look here for a theme to own.
    egui::ColorImage::new([1, 1], vec![egui::Color32::TRANSPARENT])
}

// ---------------------------------------------------------------------------
// Element parsing
// ---------------------------------------------------------------------------

/// The tag name at the start of a tag body (everything up to the first
/// whitespace, `/` or end).
fn tag_name(body: &str) -> &str {
    let end = body
        .find(|c: char| c.is_ascii_whitespace() || c == '/')
        .unwrap_or(body.len());
    &body[..end]
}

/// Find `needle` in `haystack` at or after `from`, returning its start.
fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

/// Read a double-quoted attribute value out of a tag body.
///
/// Matches on `name="` with a preceding delimiter check so that looking up
/// `x` does not match `rx`, and `stroke` does not match `stroke-width` — the
/// single most likely silent-wrongness bug in a scanner this simple, and the
/// reason this is one shared helper rather than an inline `find` at each call
/// site. `attribute_lookup_is_not_a_substring_match` pins it.
fn attr<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    let bytes = body.as_bytes();
    let pattern = format!("{name}=\"");
    let mut from = 0usize;
    while let Some(pos) = body[from..].find(pattern.as_str()) {
        let abs = from + pos;
        let preceded_ok = abs == 0 || bytes[abs - 1].is_ascii_whitespace();
        if preceded_ok {
            let start = abs + pattern.len();
            let end = body[start..].find('"')? + start;
            return Some(&body[start..end]);
        }
        from = abs + 1;
    }
    None
}

/// Parse an attribute as an `f32`, or report a malformed number.
fn attr_f32(body: &str, name: &str) -> Option<Result<f32, IconError>> {
    attr(body, name).map(|raw| {
        raw.trim()
            .parse::<f32>()
            .map_err(|_| IconError::MalformedNumber { offset: 0 })
    })
}

/// The paint attributes shared by every shape element.
///
/// Absent `stroke` means "not stroked" and absent `fill` means "not filled",
/// matching the root `<svg fill="none">` the style contract mandates.
/// `stroke-width` defaults to the contract's 2.5 so an asset that omits it
/// still draws at the set's weight rather than at tiny-skia's 1.0.
fn parse_paint(body: &str) -> Result<(Option<f32>, LineCap, LineJoin, bool), IconError> {
    let stroked = matches!(attr(body, "stroke"), Some(v) if v != "none");
    let stroke_width = if stroked {
        Some(match attr_f32(body, "stroke-width") {
            Some(v) => v?,
            None => 2.5,
        })
    } else {
        None
    };

    let line_cap = match attr(body, "stroke-linecap") {
        None | Some("butt") => LineCap::Butt,
        Some("round") => LineCap::Round,
        Some("square") => LineCap::Square,
        Some(other) => {
            return Err(IconError::UnsupportedPaintValue {
                attribute: "stroke-linecap",
                value: other.to_owned(),
            });
        }
    };
    let line_join = match attr(body, "stroke-linejoin") {
        None | Some("miter") => LineJoin::Miter,
        Some("round") => LineJoin::Round,
        Some("bevel") => LineJoin::Bevel,
        Some(other) => {
            return Err(IconError::UnsupportedPaintValue {
                attribute: "stroke-linejoin",
                value: other.to_owned(),
            });
        }
    };

    let filled = matches!(attr(body, "fill"), Some(v) if v != "none");
    Ok((stroke_width, line_cap, line_join, filled))
}

/// Assemble a [`Shape`] from a finished builder plus the element's paint.
fn finish_shape(builder: PathBuilder, body: &str) -> Result<Shape, IconError> {
    let path = builder.finish().ok_or(IconError::DegeneratePath)?;
    let (stroke_width, line_cap, line_join, filled) = parse_paint(body)?;
    Ok(Shape {
        path,
        stroke_width,
        line_cap,
        line_join,
        filled,
    })
}

/// `<path d="…"/>`.
fn parse_path_element(body: &str) -> Result<Shape, IconError> {
    let d = attr(body, "d").ok_or(IconError::MissingAttribute {
        element: "path",
        attribute: "d",
    })?;
    let mut builder = PathBuilder::new();
    parse_path_data(d, &mut builder)?;
    finish_shape(builder, body)
}

/// `<rect x y width height rx?/>`, including the rounded-corner form.
fn parse_rect_element(body: &str) -> Result<Shape, IconError> {
    let need = |name: &'static str| -> Result<f32, IconError> {
        attr_f32(body, name).unwrap_or(Err(IconError::MissingAttribute {
            element: "rect",
            attribute: name,
        }))
    };
    let x = need("x")?;
    let y = need("y")?;
    let w = need("width")?;
    let h = need("height")?;
    let rx = match attr_f32(body, "rx") {
        Some(v) => v?,
        None => 0.0,
    };

    let mut builder = PathBuilder::new();
    push_round_rect(&mut builder, x, y, w, h, rx);
    finish_shape(builder, body)
}

/// `<circle cx cy r/>`.
fn parse_circle_element(body: &str) -> Result<Shape, IconError> {
    let need = |name: &'static str| -> Result<f32, IconError> {
        attr_f32(body, name).unwrap_or(Err(IconError::MissingAttribute {
            element: "circle",
            attribute: name,
        }))
    };
    let cx = need("cx")?;
    let cy = need("cy")?;
    let r = need("r")?;

    let mut builder = PathBuilder::new();
    builder.push_circle(cx, cy, r);
    finish_shape(builder, body)
}

/// Emit a rounded rectangle as an explicit path.
///
/// tiny-skia's `push_rect` has no corner radius, and the set uses `rx` on
/// most rects, so the corners are drawn as four 90° cubic arcs. Radius is
/// clamped to half the shorter side, which is what SVG requires and what
/// stops a hand-edited `rx="99"` from turning into a self-intersecting mess
/// instead of a stadium.
fn push_round_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, rx: f32) {
    let r = rx.min(w / 2.0).min(h / 2.0).max(0.0);
    if r <= 0.0 {
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        return;
    }
    let k = r * KAPPA;
    let (x1, y1) = (x + w, y + h);
    pb.move_to(x + r, y);
    pb.line_to(x1 - r, y);
    pb.cubic_to(x1 - r + k, y, x1, y + r - k, x1, y + r);
    pb.line_to(x1, y1 - r);
    pb.cubic_to(x1, y1 - r + k, x1 - r + k, y1, x1 - r, y1);
    pb.line_to(x + r, y1);
    pb.cubic_to(x + r - k, y1, x, y1 - r + k, x, y1 - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_unsupported_element() {
        let svg = r#"<svg viewBox="0 0 48 48"><g transform="translate(4,4)"><path d="M0 0L1 1"/></g></svg>"#;
        assert_eq!(
            IconArt::parse(svg).unwrap_err(),
            IconError::UnsupportedElement("g".to_owned())
        );
    }

    #[test]
    fn refuses_unsupported_linecap() {
        let svg = r#"<svg><path d="M0 0L1 1" stroke="currentColor" stroke-linecap="flat"/></svg>"#;
        assert!(matches!(
            IconArt::parse(svg).unwrap_err(),
            IconError::UnsupportedPaintValue {
                attribute: "stroke-linecap",
                ..
            }
        ));
    }

    #[test]
    fn refuses_shape_missing_geometry() {
        let svg = r#"<svg><circle cx="4" cy="4" stroke="currentColor"/></svg>"#;
        assert_eq!(
            IconArt::parse(svg).unwrap_err(),
            IconError::MissingAttribute {
                element: "circle",
                attribute: "r"
            }
        );
    }

    #[test]
    fn refuses_unterminated_tag() {
        assert_eq!(
            IconArt::parse("<svg><path d=\"M0 0\"").unwrap_err(),
            IconError::UnterminatedTag
        );
    }

    /// Every refusal must be able to say what it refused. A parse error that
    /// prints nothing useful is the "two-minute fix, afternoon of searching"
    /// case [`IconError`]'s own docs warn about, and `Display` is the only
    /// place that promise is kept.
    #[test]
    fn every_error_renders_a_non_empty_message() {
        let all = [
            IconError::UnsupportedElement("g".to_owned()),
            IconError::UnsupportedPathCommand('X'),
            IconError::NoCurrentPoint('L'),
            IconError::MalformedNumber { offset: 7 },
            IconError::MalformedFlag { offset: 9 },
            IconError::UnexpectedEnd,
            IconError::MissingAttribute {
                element: "circle",
                attribute: "r",
            },
            IconError::UnsupportedPaintValue {
                attribute: "stroke-linecap",
                value: "flat".to_owned(),
            },
            IconError::UnterminatedTag,
            IconError::DegeneratePath,
        ];
        for e in all {
            let msg = e.to_string();
            assert!(!msg.is_empty(), "{e:?} rendered an empty message");
        }
    }

    // -- attribute scanning -------------------------------------------------

    /// `attr` must not confuse `x` with `rx`, nor `stroke` with
    /// `stroke-width`. Getting this wrong would move every rounded rect in
    /// the set.
    #[test]
    fn attribute_lookup_is_not_a_substring_match() {
        let body =
            r#"rect x="10" y="4" width="28" rx="2" stroke="currentColor" stroke-width="2.5""#;
        assert_eq!(attr(body, "x"), Some("10"));
        assert_eq!(attr(body, "rx"), Some("2"));
        assert_eq!(attr(body, "stroke"), Some("currentColor"));
        assert_eq!(attr(body, "stroke-width"), Some("2.5"));
        assert_eq!(attr(body, "height"), None);
    }

    // -- paint --------------------------------------------------------------

    #[test]
    fn stroke_none_is_not_drawn_but_fill_is() {
        let svg = r#"<svg><rect x="0" y="0" width="10" height="10" fill="currentColor" stroke="none"/></svg>"#;
        let art = IconArt::parse(svg).expect("parses");
        assert!(art.has_fill());
        assert_eq!(art.shape_count(), 1);
        let img = art.rasterize(32, IconWeight::Regular);
        assert!(img.pixels.iter().any(|p| p.a() > 0), "fill drew nothing");
    }

    // -- rasterization ------------------------------------------------------

    #[test]
    fn rasterizes_at_the_requested_physical_size() {
        let art = IconArt::parse(super::super::assets::UNDO).expect("parses");
        assert_eq!(art.rasterize(16, IconWeight::Regular).size, [16, 16]);
        assert_eq!(art.rasterize(48, IconWeight::Regular).size, [48, 48]);
    }

    /// A zero physical size is clamped, not a panic and not a zero-area
    /// image. `px` is derived from a display scale the application does not
    /// control, and `Pixmap::new(0, 0)` is `None`.
    #[test]
    fn a_zero_size_raster_is_clamped_rather_than_empty() {
        let art = IconArt::parse(super::super::assets::UNDO).expect("parses");
        assert_eq!(art.rasterize(0, IconWeight::Regular).size, [1, 1]);
    }

    #[test]
    fn bold_weight_covers_more_pixels_than_regular() {
        let art = IconArt::parse(super::super::assets::CHEVRON_LEFT).expect("parses");
        let regular = art.rasterize(48, IconWeight::Regular);
        let bold = art.rasterize(48, IconWeight::Bold);
        let lit = |img: &egui::ColorImage| img.pixels.iter().filter(|p| p.a() > 128).count();
        assert!(
            lit(&bold) > lit(&regular),
            "bold weight must be a visible cue: regular={} bold={}",
            lit(&regular),
            lit(&bold)
        );
    }

    /// ★ The arithmetic precondition of the whole theming story.
    ///
    /// Every non-transparent pixel must be premultiplied WHITE, i.e.
    /// `r == g == b == a`. That property is what makes a multiplicative tint
    /// produce a correctly premultiplied tinted glyph for **any** tint
    /// colour, which is in turn what lets one raster serve every theme
    /// preset and every widget state (module header, "Theming").
    #[test]
    fn mask_is_white_so_tinting_is_exact() {
        let art = IconArt::parse(super::super::assets::SHAPE_RECT).expect("parses");
        let img = art.rasterize(32, IconWeight::Regular);
        for p in &img.pixels {
            let a = i32::from(p.a());
            for c in [p.r(), p.g(), p.b()] {
                // +/-1 tolerance for tiny-skia's integer premultiply
                // rounding; anything larger would mean a coloured (i.e.
                // untintable) mask.
                assert!(
                    (i32::from(c) - a).abs() <= 1,
                    "mask pixel is not white: rgba=({},{},{},{})",
                    p.r(),
                    p.g(),
                    p.b(),
                    p.a()
                );
            }
        }
    }

    #[test]
    fn the_blank_image_is_one_transparent_pixel() {
        let img = blank_image();
        assert_eq!(img.size, [1, 1]);
        assert_eq!(img.pixels[0].a(), 0);
    }
}
