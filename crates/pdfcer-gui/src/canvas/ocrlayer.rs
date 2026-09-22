//! # `canvas::ocrlayer` — the invisible text a scan carries, drawn
//!
//! A scanned page is a picture. When it has been through OCR it is a picture
//! **plus** a layer of real text, positioned over the marks it was recognised
//! from and drawn in text rendering mode 3 — present, searchable, copyable,
//! and never visible. This module draws it.
//!
//! `OPERATOR_REQUESTS.md` O226 asks for the page and that layer to be
//! comparable through a slider: at one end the scan as it is, at the other end
//! the text alone, and everything between.
//!
//! ## Two layers, two positions in the order
//!
//! | slider | the page raster | the text |
//! |---|---|---|
//! | 0.0 | full opacity | not drawn |
//! | 0.5 | half | half |
//! | 1.0 | not drawn | full opacity |
//!
//! The raster's half is **not a re-render**. Nothing here asks for a pixmap at
//! a different opacity; [`draw_veil`] paints paper over the texture already on
//! screen. Re-rasterizing per slider position would put a render request
//! behind a drag, and `OPERATOR_REQUESTS.md` O24 is this shell's record of
//! what that costs.
//!
//! The text's half is **vector, laid out every frame** by [`draw_text`]. It is
//! never baked into a texture, because it must stay crisp at every zoom and
//! because a cached overlay is a second rendering path for content that has
//! exactly one.
//!
//! The two sit at different places in `painting`'s layer order — the veil
//! under the grid, because it is about the *paper*; the text above the grid
//! and below the find wash, because it is page content and a search answer
//! outranks it. `painting`'s own table carries the argument.
//!
//! ## R8b — this does not mark the canvas
//!
//! The rule forbids styling **applied content** as provisional. Nothing here
//! is applied content and nothing here is provisional: this draws text the
//! file already holds, in a mode the operator turned on and can turn off, and
//! with the mode off the module paints nothing at all. The one-line test
//! answers no — a screenshot of the page with `view.ocr_overlay` at `None` is
//! a screenshot of the saved document.
//!
//! The colour is the operator's (`OPERATOR_REQUESTS.md` O229) precisely
//! because this is a *reading instrument* rather than a rendering of the page:
//! the text has to be told apart from the scan underneath it, and which colour
//! does that depends on the scan.
//!
//! ## Why there is no cap and nothing is skipped
//!
//! `canvas::chunks` bounds its work with `MAX_CHUNK_BOXES` and draws nothing
//! past it. That is right for a *selection* — the operator can select less —
//! and wrong here, because this is a **view** and the pages it exists for are
//! exactly the dense ones. A cap would blank the feature on its own subject.
//!
//! The cost is bounded instead, in two ways that omit nothing visible:
//!
//! * **Runs outside the clip are not laid out.** They are not on screen, so
//!   nothing is lost by not drawing them.
//! * **A run too small to resolve into glyphs is drawn as a filled box** at
//!   the run's own rectangle — see [`MIN_FONT_PX`]. That is not an omission
//!   and not a placeholder: it is the same content at a scale where letters
//!   do not survive, and it says *there is recognised text here* truthfully.
//!   It also costs no layout, which is what makes a whole dense sheet at
//!   fit-page zoom affordable.
//!
//! So this module owes no "some were not shown" disclosure, because there is
//! no state in which it does not show one.
//!
//! ## One page
//!
//! [`crate::app::state::OpenDoc::page_text`] caches the **current page only**.
//! Under a continuous strip the overlay therefore draws on that page and not
//! on its neighbours. The drawing is still routed through that page's own
//! [`crate::canvas::strip::PageView::map`] rather than the acting page's —
//! they are the same page today, and using the acting map would be the exact
//! failure `painting`'s find-wash comment records as the one most likely to
//! ship silently, waiting for the day the cache learns a second page.

use egui::{Color32, CornerRadius, FontId, Painter, Rect};

use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;

/// The smallest size this will ask egui to lay out text at.
///
/// Below it a run is drawn as a filled box instead — see the header. The value
/// is where a proportional face stops resolving into distinguishable letters
/// on a 96 dpi display; under it the glyphs are a smudge that costs a layout
/// and reads as noise, and a solid bar reads as *text, too small*, which is
/// what is true.
pub const MIN_FONT_PX: f32 = 5.0;

/// The largest size this will ask egui to lay out text at.
///
/// ⚠ A ceiling on the **font atlas**, not on the design. A run's box grows
/// without bound as the operator zooms, and egui rasterizes a glyph per
/// (face, size): asking for a 4,000 pt face once is a multi-megabyte atlas
/// upload in the middle of a zoom gesture.
///
/// The visible consequence is that past roughly this size the overlay text
/// stops growing with the page while the scan under it keeps growing. That is
/// a real divergence and it is stated here rather than hidden: it begins at a
/// zoom where one run fills the window, which is far past any zoom at which
/// two layers are being compared.
pub const MAX_FONT_PX: f32 = 160.0;

/// Font sizes are rounded to this, in points.
///
/// ★ Not cosmetic. Every distinct size is a separate set of rasterized glyphs
/// in egui's atlas, and a page of OCR runs has as many distinct box heights as
/// it has runs. Rounding collapses a sheet's worth of near-identical sizes
/// onto a few dozen shared ones, so the atlas holds a face-sized set rather
/// than a page-sized one, and a zoom re-uses what the last frame uploaded.
pub const FONT_SIZE_QUANTUM_PX: f32 = 0.5;

/// Where the overlay's colour lives while the program runs. Memory key.
///
/// Two homes, exactly as `canvas::chunks` has: the live answer here, because
/// the painter reaches it with a [`egui::Context`] and nothing else, and the
/// persisted answer on [`crate::app::prefs::Prefs`], because O229 asks for the
/// setting to be remembered. `crate::app::frame` mirrors the second into the
/// first once a frame, in that direction only.
const COLOUR_KEY: &str = "pdfcer.ocr-layer.colour"; // ui-text-exempt: a memory key, never displayed

/// The colour the overlay is drawn in until the operator chooses another.
///
/// ★ Chosen to be a colour a **scan is unlikely to contain**. The overlay's
/// whole job is to be told apart from the marks under it, and a scanned
/// drawing is black, grey and — on a CAD sheet — often blue or red. Magenta is
/// in none of those families, so the default works before anybody has thought
/// about it, which is what a default is for.
pub const DEFAULT_COLOUR: [u8; 3] = [204, 0, 153];

/// The colour the overlay is drawn in, as the operator left it.
#[must_use]
pub fn colour(ctx: &egui::Context) -> [u8; 3] {
    ctx.data(|d| d.get_temp::<[u8; 3]>(egui::Id::new(COLOUR_KEY)))
        .unwrap_or(DEFAULT_COLOUR)
}

/// Mirror the persisted colour into the live one, once per frame.
///
/// The guard is not an optimisation. An unconditional write every frame would
/// make the value impossible to change from anywhere else, which is how a
/// mirror becomes an overwrite — `canvas::chunks::sync` carries the same note.
pub fn sync(ctx: &egui::Context, rgb: [u8; 3]) {
    if colour(ctx) != rgb {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(COLOUR_KEY), rgb));
    }
}

/// [`colour`] as egui sees it.
fn colour32(ctx: &egui::Context) -> Color32 {
    let [r, g, b] = colour(ctx);
    // NOT A THEME COLOUR: the operator's own, set through `OPERATOR_REQUESTS.md`
    // O229 and persisted. A theme must never move it — restyling the
    // application would change a colour chosen to contrast with a particular
    // scan, which the theme has no knowledge of.
    Color32::from_rgb(r, g, b)
}

/// **Is this run part of the OCR layer?**
///
/// `ExtractedGlyph::invisible` is the whole selector: the engine sets it for
/// text rendering modes 3 and 7, which is what an OCR producer writes and what
/// a page's own lettering never is.
///
/// ★ `any`, not `all`, and the difference is a silent omission. A producer
/// that flips the rendering mode mid-run leaves a run with both kinds of
/// glyph. Taking it draws some already-visible letters a second time, which
/// the operator can see and dismiss. Refusing it hides recognised text with
/// nothing on screen to say so — and *nothing on screen* is the failure mode
/// this whole feature exists to end.
#[must_use]
pub fn is_ocr_run(run: &pdfcer_core::text_extract::TextRun) -> bool {
    run.glyphs.iter().any(|glyph| glyph.invisible)
}

/// **How much paint, from a slider position** — the one answer, and the one
/// the status bar must quote.
///
/// ★★ A **non-finite input paints nothing**, where
/// [`crate::viewer::normalise_ocr_overlay`] answers the same corruption with
/// the default position. The two are not inconsistent: that one answers *where
/// did the operator leave the slider*, and a lost preference should land
/// somewhere useful; this one answers *how opaque is this stroke*, and a
/// number nobody can account for must not end up drawn over the document.
///
/// ★★★ Which is exactly why this is public and `app::status::ocrlayer` reads
/// it rather than the raw field. A disclosure that quoted the other normaliser
/// would report 65 % on the one input where the painter draws nothing — a
/// sentence describing a blend that is not on screen, produced by two
/// functions that each behave correctly.
#[must_use]
pub fn painted_fraction(raw: f32) -> f32 {
    if raw.is_finite() {
        raw.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// How opaque the paper veil over the raster is, out of 255.
///
/// Rises with the slider: at the right-hand stop the veil is opaque and the
/// scan is gone, which is the design's *blank field behind the text*.
#[must_use]
pub fn veil_alpha(strength: f32) -> u8 {
    // `round` rather than a truncating cast: 1.0 must reach 255 exactly, or
    // the right-hand stop leaves one part in 255 of the scan showing through
    // and the "only the text layer is visible" end of O226 is not reached.
    (painted_fraction(strength) * 255.0).round() as u8
}

/// How opaque the overlay text is, out of 255.
///
/// Equal to [`veil_alpha`] by construction rather than by coincidence: the two
/// halves of one slider are one number, and the left stop has to draw *no*
/// text rather than faint text for the same reason the right stop has to blank
/// the scan.
#[must_use]
pub fn text_alpha(strength: f32) -> u8 {
    veil_alpha(strength)
}

/// Clamp a wanted font size into what the atlas will bear, and round it.
#[must_use]
pub fn quantise_font_size(raw: f32) -> f32 {
    if !raw.is_finite() {
        return MIN_FONT_PX;
    }
    let clamped = raw.clamp(MIN_FONT_PX, MAX_FONT_PX);
    (clamped / FONT_SIZE_QUANTUM_PX).round() * FONT_SIZE_QUANTUM_PX
}

/// The colour a fully-drawn veil leaves behind.
///
/// The same value and the same argument as `canvas::shapes`' — PDF has no page
/// background, and `render_page` composited this raster onto an opaque white
/// backdrop per §11.4.7. Fading the raster towards anything else would fade it
/// towards a colour the renderer never used, and the page would change hue on
/// its way to blank.
fn paper() -> Color32 {
    super::shapes::paper()
}

/// **Fade the page raster**, by painting paper over it.
///
/// Called before the grid: this is about the sheet, and everything drawn on
/// the sheet has to win.
///
/// Only over pages that actually have a raster this frame, and only over
/// [`PageView::paint_rect`] — the rectangle the texture is a picture of. A
/// page still waiting on its pixmap is left alone rather than veiled, because
/// veiling nothing would put a white rectangle over whatever the strip is
/// showing in its place.
pub(super) fn draw_veil(painter: &Painter, pages: &[PageView], strength: f32) {
    let alpha = veil_alpha(strength);
    if alpha == 0 {
        return;
    }
    let veil = super::overlay::at_alpha(paper(), alpha);
    for view in pages {
        if view.raster.is_none() {
            continue;
        }
        painter.rect_filled(view.paint_rect, CornerRadius::ZERO, veil);
    }
}

/// **Draw the recognised text**, in the operator's colour.
///
/// `clip` culls: a run whose screen rectangle misses it is never laid out.
///
/// The colour is read from the painter's own context rather than passed in, so
/// the `NOT A THEME COLOUR:` argument stays beside the value it is about and
/// `painting` does not have to carry a colour it makes no decision on.
pub(super) fn draw_text(
    painter: &Painter,
    doc: &OpenDoc,
    pages: &[PageView],
    clip: Rect,
    strength: f32,
) {
    let alpha = text_alpha(strength);
    if alpha == 0 {
        return;
    }
    let ink = super::overlay::at_alpha(colour32(painter.ctx()), alpha);
    let Some(text) = doc.page_text() else {
        return;
    };
    // ★ The page the CACHE describes, found among the pages drawn — never
    // `pages[0]` and never the acting page's map. See the header.
    let Some(view) = pages.iter().find(|view| view.page == text.page_index) else {
        return;
    };
    let Some(page) = doc.pages.get(text.page_index) else {
        return;
    };
    for run in &text.runs {
        if !is_ocr_run(run) {
            continue;
        }
        // A run with no geometry — derived whitespace, or an `/ActualText`
        // replacement that covered no glyphs — has nowhere to be drawn. It
        // carries no letters an operator could be looking for either.
        let Some(bbox) = run.bbox else {
            continue;
        };
        let Some(canvas) =
            super::geometry::pdf_rect_to_canvas((bbox.llx, bbox.lly, bbox.urx, bbox.ury), page)
        else {
            continue;
        };
        let screen = view.map.rect_to_screen(canvas);
        if !screen.intersects(clip) {
            continue;
        }
        draw_run(painter, screen, run.text.trim(), ink);
    }
}

/// One run, fitted to its own box.
///
/// The size is taken from the box's **height** and then corrected by a
/// measurement of the laid-out width, which is at most two layouts and usually
/// one. Taking it from the glyph metrics instead would mean reproducing the
/// text matrix here; taking it from the width alone would make a two-word run
/// and a twenty-word run in equal boxes render at wildly different sizes.
fn draw_run(painter: &Painter, screen: Rect, text: &str, ink: Color32) {
    if text.is_empty() || screen.width() <= 0.0 || screen.height() <= 0.0 {
        return;
    }
    let wanted = screen.height();
    if !wanted.is_finite() || wanted < MIN_FONT_PX {
        painter.rect_filled(screen, CornerRadius::ZERO, ink);
        return;
    }
    let first = quantise_font_size(wanted);
    let galley = painter.layout_no_wrap(text.to_owned(), FontId::proportional(first), ink);
    let measured = galley.rect.width();
    let (size, galley) = if measured > screen.width() && measured > 0.0 {
        let fitted = quantise_font_size(first * (screen.width() / measured));
        if fitted < first {
            (
                fitted,
                painter.layout_no_wrap(text.to_owned(), FontId::proportional(fitted), ink),
            )
        } else {
            (first, galley)
        }
    } else {
        (first, galley)
    };
    if size < MIN_FONT_PX {
        painter.rect_filled(screen, CornerRadius::ZERO, ink);
        return;
    }
    // Left-aligned and vertically centred. The box is the run's own extent, so
    // its left edge is where the first glyph starts; centring on the other
    // axis is what keeps a line sitting on its marks when the fitted face's
    // ascent does not match the one the producer measured the box from.
    let at = egui::pos2(
        screen.left(),
        screen.center().y - galley.rect.height() * 0.5,
    );
    painter.galley(at, galley, ink);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both stops, and the fact that they are the two ends.**
    ///
    /// A build that inverted the slider satisfies neither: it would paint the
    /// veil at the left stop, which is the position that means *show me the
    /// scan*.
    #[test]
    fn the_slider_stops_are_no_paint_and_full_paint() {
        assert_eq!(veil_alpha(0.0), 0, "the left stop must not veil the scan");
        assert_eq!(veil_alpha(1.0), 255, "the right stop must blank the scan");
        assert_eq!(text_alpha(0.0), 0, "the left stop must draw no text");
        assert_eq!(text_alpha(1.0), 255, "the right stop must draw text fully");
    }

    /// Out-of-range and corrupt positions paint nothing rather than paint
    /// wrongly — the argument is on [`painted_fraction`].
    #[test]
    fn a_position_outside_the_range_is_clamped_and_a_corrupt_one_paints_nothing() {
        assert_eq!(veil_alpha(-1.0), 0);
        assert_eq!(veil_alpha(2.0), 255);
        assert_eq!(veil_alpha(f32::NAN), 0);
        assert_eq!(veil_alpha(f32::INFINITY), 0);
    }

    /// The midpoint is genuinely between the two, not at one of them.
    ///
    /// A cast that truncated, or a scale that used 256, would still pass the
    /// stops test above; this is what tells them apart.
    #[test]
    fn the_middle_of_the_slider_is_half_of_each() {
        let mid = veil_alpha(0.5);
        assert!(
            (120..=135).contains(&mid),
            "half the slider should be about half the paint, got {mid}"
        );
    }

    /// The quantum is applied and both ends of the atlas guard hold.
    #[test]
    fn a_font_size_is_rounded_and_bounded() {
        assert_eq!(quantise_font_size(12.3), 12.5);
        assert_eq!(quantise_font_size(12.1), 12.0);
        assert_eq!(quantise_font_size(0.001), MIN_FONT_PX);
        assert_eq!(quantise_font_size(9_000.0), MAX_FONT_PX);
        assert_eq!(quantise_font_size(f32::NAN), MIN_FONT_PX);
    }

    /// ★ Every size this yields is a multiple of the quantum.
    ///
    /// The property the atlas argument rests on, asserted over a walk rather
    /// than at two chosen points: a rounding that worked at 12.3 and failed
    /// near the clamps would pass the test above.
    #[test]
    fn every_size_is_a_multiple_of_the_quantum() {
        let mut raw = 0.0_f32;
        while raw < 200.0 {
            let size = quantise_font_size(raw);
            let steps = size / FONT_SIZE_QUANTUM_PX;
            assert!(
                (steps - steps.round()).abs() < 1e-3,
                "{raw} quantised to {size}, which is not a multiple of the quantum"
            );
            raw += 0.17;
        }
    }
}
