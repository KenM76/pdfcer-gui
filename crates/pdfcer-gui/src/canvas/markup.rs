//! # `canvas::markup` — what a markup annotation IS, and the pen it is drawn with
//!
//! ## The defect this module exists so that we never ship again
//!
//! The old shell's `canvas.rs` records it in the doc comment of the tool
//! variant this one is modelled on, and it is worth carrying across verbatim
//! because it is the reason a markup *substrate* exists at all rather than
//! eight commands that each insert a shape:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup.md`.

use egui::{Color32, Pos2};
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec, Quad, TextMarkupKind};
use pdfcer_core::page_tree::Rect as PageRect;

use crate::app::actions::Action;
use crate::canvas::mapping::PageMapping;

/// The two-point rubber band: Rectangle, Ellipse, Arrow, Highlight.
pub mod band;
/// Freehand: press, follow the pointer, release. `/Ink`.
pub mod ink;

/// **Solid or dashed** — `/BS` `/S` and `/D` (§12.5.4, Table 166), as the
/// four choices this shell offers and the one reading it can only report.
///
pub use pdfcer_gui_base::linestyle;
/// Which markup gesture one drag reaches — band, freehand trail, or the
/// line-grouped quads of a highlight that found text. Split out of
/// `canvas::interact` under R2; its header carries the fallback ordering.
pub mod route;

/// **Acrobat's own markup colours, measured** — the ten values Adobe
/// authors comments in, and the grid the Style swatch offers them from.
pub use pdfcer_gui_base::markuppalette as palette;
#[cfg(test)]
mod palette_tests;

/// The colour and width the next markup is authored with (`RIBBON_IA.md` §5.5 Style).
pub use pdfcer_gui_base::markuppen as pen;
/// The Markup ▸ Style group's `colour_swatch` control.
pub use pdfcer_gui_base::markupswatch as swatch;

/// Underline, strikeout and squiggly — the kinds whose operand is a text
/// selection rather than a pointer gesture. See this module's header for why
/// they are not [`MarkupKind`] variants.
pub mod text;
/// The click-shaped kinds: PolyLine and Polygon, and the two endings that
/// finish them.
pub mod vertex;

/// Which markup annotation the markup tool is currently drawing.
pub use pdfcer_gui_base::markupkind::MarkupKind;

/// The default border/stroke width, in PDF points, every geometric markup is
/// authored with.
pub const PEN_WIDTH_PTS: f64 = 2.0;

pub use pdfcer_gui_base::markupkind::Geometry;

/// Why a markup gesture committed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The gesture ended where it began: no extent on either axis, or a vertex
    /// run every point of which is the same point. See the module docs on
    /// degenerate input.
    NoExtent,
    /// A coordinate was not finite. Refused rather than authored, because the
    /// alternative is a NaN in an annotation's `/Rect`.
    NotFinite,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space position for the gesture. Declining is the only
    /// honest answer; authoring fabricated geometry is not.
    DegeneratePage,
    /// The frame has no page to author onto — a strip whose visible window fell
    /// outside every page, or a document whose pages have not loaded.
    NoPage,
    /// A vertex run too short for its kind: fewer than two for a `/PolyLine`,
    /// fewer than **three** for a `/Polygon`.
    ///
    /// **This is the one place the shell is deliberately stricter than the
    /// engine.** `pdfcer-core`'s `validate_geometry` refuses `vertices.len() < 2`
    /// for both, so a two-vertex `/Polygon` would be accepted and authored: a
    /// closed shape drawn from A to B and back to A, which renders as a single
    /// line and is not a polygon anybody meant. The engine is right to accept it
    /// — it is legal PDF and a CLI caller may have reason — and the shell is
    /// right to refuse it, because a *gesture* that produced it is an operator
    /// who double-clicked one click early.
    TooFewVertices,
    /// The geometry does not describe the kind — a `Band` for `/Ink`, a
    /// `Vertices` for a `/Square`.
    ///
    /// **Structurally unreachable from any gesture**: each family builds its own
    /// [`Geometry`] variant beside the kind it belongs to, and the two are
    /// written on adjacent lines. Refused explicitly anyway, for the reason
    /// [`text::Refusal::NoQuads`] is: an [`Action`] is plain data, it can be
    /// constructed by a test or replayed by a future undo/redo surface, and the
    /// alternative to a named refusal is a panic or a silently wrong annotation.
    Mismatched,
}

/// The `/BE /I` intensity every revision cloud this shell authors carries.
const CLOUD_INTENSITY: f64 = 1.0;

/// Build the `pdfcer-core` spec one markup gesture authors.
#[must_use]
pub fn spec(kind: MarkupKind, geometry: &Geometry, pen: pen::Pen) -> Option<MarkupSpec> {
    // The pen is a PARAMETER as of 2026-08-17, and this is the seam
    // `MarkupKind::rgb`'s own doc comment named in advance: *"give it a colour
    // and a width from the document's markup state and nothing else in the
    // module changes."* Nothing else in this module did.
    let (r, g, b) = pen.colour_for(kind);
    let color = Color::Rgb(r, g, b);
    let width = pen.width_pts;
    match (kind, geometry) {
        (
            MarkupKind::Rectangle | MarkupKind::Ellipse | MarkupKind::Arrow | MarkupKind::Highlight,
            Geometry::Band { start, end },
        ) => {
            let rect = PageRect::from_corners(
                start.0.min(end.0),
                start.1.min(end.1),
                start.0.max(end.0),
                start.1.max(end.1),
            );
            Some(match kind {
                MarkupKind::Rectangle => MarkupSpec::Square {
                    rect,
                    border: Some(color),
                    // No fill. A filled comment shape hides the drawing it is a
                    // comment about, which on a CAD sheet is the whole content
                    // under it.
                    interior: None,
                    border_width: width,
                    //
                    // Named explicitly rather than absorbed by a struct update,
                    // for the reason `to_engine_settings` states about the same
                    // situation in the print adapter: a `..Default::default()`
                    // would have taken this field silently and would take the
                    // NEXT one too, which is how a shell comes to ignore a
                    // capability the engine grew for it. The compile error this
                    // replaced is the whole value of naming every field.
                    //
                    // The cloud is a SEPARATE control — `RIBBON_IA.md` §5.5's
                    // revision-cloud row — and giving Rectangle a cloudy border
                    // would change what a shipped control draws without asking.
                    border_effect: None,
                },
                MarkupKind::Ellipse => MarkupSpec::Circle {
                    rect,
                    border: Some(color),
                    interior: None,
                    border_width: width,
                },
                // RAW `start` and `end` — see this function's docs.
                MarkupKind::Arrow => MarkupSpec::Line {
                    start: *start,
                    end: *end,
                    color,
                    width,
                    // Tail then head, in the operator's own words. `None` at the
                    // start is what makes the raw-endpoint rule above
                    // load-bearing rather than decorative.
                    endings: (LineEnding::None, LineEnding::OpenArrow),
                },
                // Exactly one quad, always, so `validate_geometry`'s empty-quad
                // refusal is structurally unreachable from this path.
                _ => MarkupSpec::TextMarkup {
                    kind: TextMarkupKind::Highlight,
                    quads: vec![Quad::from_rect(rect)],
                    color,
                },
            })
        }
        (MarkupKind::PolyLine, Geometry::Vertices(vertices)) => Some(MarkupSpec::PolyLine {
            vertices: vertices.clone(),
            color,
            width,
        }),
        (MarkupKind::Polygon, Geometry::Vertices(vertices)) => Some(MarkupSpec::Polygon {
            vertices: vertices.clone(),
            border: Some(color),
            interior: None,
            width,
        }),
        // **The revision cloud**, and the ONLY line in this module that
        // distinguishes it from the arm above.
        //
        // `intensity` is the whole of the difference in the file. Table 167's
        // `I` row is typed `number` and constrained *"in the range 0 to 2"* —
        // a CONTINUOUS range, not the enumeration `{0, 1, 2}` it is routinely
        // mis-read as; `pdfcer-core`'s `MarkupSpec::Cloud` carries the evidence
        // (the sibling `S` row in the same four-line table uses the standard's
        // enumeration idiom, and `I` does not). So a shell may pick any value
        // in the range and 1.0 is a choice rather than the only legal one.
        //
        // **1.0 is Acrobat's own default cloud**, which is the whole argument.
        // The standing tie-breaker for anything an operator will compare
        // against the program they are replacing is *make it work the way the
        // other program does*, and a reviewer who draws a cloud in pdfcer and a
        // cloud in Acrobat over the same drawing must not be able to tell which
        // is which by the size of the scallop.
        //
        // It is deliberately **not** exposed as a control. `markup.line_width`,
        // `markup.fill` and `markup.opacity` are all in
        // `crate::shell::manifest::PLANNED` for the same reason — Style sets the
        // NEXT markup's properties and only colour has a control today — and a
        // cloud-intensity slider arriving before a line-width one would be this
        // shell offering the ninth-most-wanted property first.
        (MarkupKind::Cloud, Geometry::Vertices(vertices)) => Some(MarkupSpec::Cloud {
            vertices: vertices.clone(),
            border: Some(color),
            // No fill, for the reason every other arm here gives: a filled
            // comment shape hides the drawing it is a comment about, and on a
            // revision cloud that is the *revision* — the thing the cloud was
            // drawn to draw attention to.
            interior: None,
            width,
            intensity: CLOUD_INTENSITY,
        }),
        (MarkupKind::Ink, Geometry::Strokes(strokes)) => Some(MarkupSpec::Ink {
            strokes: strokes.clone(),
            color,
            width,
        }),
        // Every remaining pair is a kind holding another family's geometry. See
        // `Refusal::Mismatched`: unreachable from a gesture, refused rather than
        // guessed.
        _ => None,
    }
}

/// [`spec`] with the shipped pen — **for tests and for `apply`'s own
/// falsifier only.**
#[must_use]
pub fn spec_default_pen(kind: MarkupKind, geometry: &Geometry) -> Option<MarkupSpec> {
    spec(kind, geometry, pen::Pen::default())
}

/// The ONE action a completed markup gesture becomes.
pub fn action(
    kind: MarkupKind,
    page: usize,
    geometry: Geometry,
    pen: pen::Pen,
) -> Result<Action, Refusal> {
    if !geometry.coordinates().all(f64::is_finite) {
        return Err(Refusal::NotFinite);
    }
    match (&geometry, kind) {
        (Geometry::Band { start, end }, k) if k.is_band() => {
            // No second threshold, and none in page space. egui's own drag
            // threshold has already separated a click from a drag in SCREEN
            // space; all that is refused here is a drag that ended exactly where
            // it began, which would author a 1-point mark nobody can see. See
            // the module docs.
            if start == end {
                return Err(Refusal::NoExtent);
            }
        }
        (Geometry::Vertices(points), MarkupKind::PolyLine) => {
            if points.len() < 2 {
                return Err(Refusal::TooFewVertices);
            }
            if all_the_same(points) {
                return Err(Refusal::NoExtent);
            }
        }
        // Polygon and Cloud share this arm, and sharing it is the assertion.
        //
        // A cloud IS a polygon in the file — `MarkupSpec::Cloud` writes
        // `/Subtype /Polygon` and differs only by `/BE` — so a vertex run that
        // is too short for one is too short for the other, by exactly the same
        // argument. Two arms with the same body would be two places for that to
        // stop being true.
        //
        // The engine agrees on the floor for the cloud and not for the polygon:
        // `validate_geometry` refuses `vertices.len() < 3` for `Cloud`
        // (Pass 82.1's *"a two-vertex cloud is a line pretending to be an
        // area"*) and `< 2` for `Polygon`. So this arm is redundant for one
        // kind and load-bearing for the other, and it is written once because
        // the SHELL's reason — the module header's "stricter than the engine"
        // note — is the same for both: a *gesture* that produced two vertices
        // is an operator who double-clicked one click early.
        (Geometry::Vertices(points), MarkupKind::Polygon | MarkupKind::Cloud) => {
            // Three, not two — the module header's "stricter than the engine"
            // note, and `Refusal::TooFewVertices`' own docs.
            if points.len() < 3 {
                return Err(Refusal::TooFewVertices);
            }
            if all_the_same(points) {
                return Err(Refusal::NoExtent);
            }
        }
        (Geometry::Strokes(strokes), MarkupKind::Ink) => {
            // A stroke of one point draws nothing at all: `ink`'s builder emits
            // a `move_to` and then paints, which strokes zero length. The engine
            // would accept it (its guard is `strokes.iter().all(Vec::is_empty)`)
            // and the operator would get an invisible annotation and an undo
            // step. Refused as NoExtent, which is the same fact the band kinds'
            // zero-length drag reports.
            if strokes.iter().all(|s| s.len() < 2) {
                return Err(Refusal::NoExtent);
            }
            if strokes.iter().all(|s| all_the_same(s)) {
                return Err(Refusal::NoExtent);
            }
        }
        // A kind holding another family's geometry — see `Refusal::Mismatched`.
        _ => return Err(Refusal::Mismatched),
    }
    Ok(Action::CommitMarkup {
        page,
        kind,
        geometry,
        pen,
    })
}

/// Whether every point in a run is the same point.
fn all_the_same(points: &[(f64, f64)]) -> bool {
    points.first().is_none_or(|first| {
        points
            .iter()
            .all(|p| (p.0 - first.0).abs() < f64::EPSILON && (p.1 - first.1).abs() < f64::EPSILON)
    })
}

/// Report a markup gesture that committed nothing, with the reason.
pub(crate) fn decline(kind: MarkupKind, page: usize, reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-declined kind={kind:?} page={page} reason={reason:?}")
    });
}

/// Report a markup that is about to be authored, with its geometry.
pub(crate) fn trace_commit(kind: MarkupKind, page: usize, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-commit kind={kind:?} page={page} {detail}")
    });
}

/// The pen's width in **screen** points at this frame's magnification.
pub(crate) fn pen_px(mapping: &PageMapping, pen: pen::Pen) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    let width = pen.width_pts as f32;
    let scale = mapping.to_screen(Pos2::new(1.0, 0.0)).x - mapping.to_screen(Pos2::ZERO).x;
    if scale.is_finite() && scale > 0.0 {
        (width * scale).max(1.0)
    } else {
        width.max(1.0)
    }
}

/// The pen colour, as egui sees it.
pub(crate) fn pen_color(kind: MarkupKind, pen: pen::Pen) -> Color32 {
    let (r, g, b) = pen.colour_for(kind);
    let byte = |v: f64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let out = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        out
    };
    // DOCUMENT COLOUR: the operator's pen, converted for the preview from the
    // exact components `spec` writes into `/C`. Deriving it from one source
    // rather than naming a second is what keeps the band the colour of the
    // thing it is previewing.
    Color32::from_rgb(byte(r), byte(g), byte(b))
}

#[cfg(test)]
mod tests;
