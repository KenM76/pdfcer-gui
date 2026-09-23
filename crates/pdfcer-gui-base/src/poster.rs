//! # `poster` — one page across many sheets, with the marks that assemble them
//!
//! The tiling, the band reserved for marks and label, the mark segments, the
//! label's box and its wording are all `pdfcer_print::imposition`'s
//! (`plan_poster`, `PosterLayout::cut_mark_segments`, `label_rect`,
//! `poster_tile_label`). This module flattens a layout into self-contained
//! [`Tile`]s and **rasterises** marks and label into the sheet's bitmap
//! ([`compose`]); drawing the glyphs is the one part the engine leaves to us.
//!
//! # Coordinate spaces
//!
//! - **page** — the source page's rotated extent, points, top-left origin,
//!   `+y` down (`pdfcer_print::imposition::PosterTile::source_pt`'s space).
//! - **sheet** — the printable area, points, top-left origin, `+y` down. A
//!   [`Tile`]'s `content_pt` and `trim_pt` are here, band already included.
//!
//! # Contract
//!
//! - Every rectangle is the engine's, unmoved.
//! - The band shrinks the area the engine tiles, so a job with marks can need
//!   more sheets than one without. That is the cost of not printing over the
//!   drawing.

use std::collections::HashMap;
use std::sync::OnceLock;

use pdfcer_print::imposition::{self, ImpositionError, PosterSpec, Rect};
use pdfcer_render::tiny_skia;

/// Cut-mark stroke width, points; never thinner than one device pixel.
const MARK_WIDTH_PT: f64 = 0.5;
/// The label's baseline, as a fraction of its box's height below the box
/// top: the box height is the type size, and the rest is descender room.
const BASELINE_FRACTION: f64 = 0.8;

/// The operator's poster settings, in engine units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// Magnification before tiling, `1.0` = 100 %.
    pub tile_scale: f64,
    /// Shared border duplicated onto neighbouring sheets, points.
    pub overlap_pt: f64,
    /// Draw cut marks in the band.
    pub cut_marks: bool,
    /// Print each sheet's position in the band.
    pub labels: bool,
    /// Tile only pages that do not fit a sheet at the tile scale.
    pub large_only: bool,
}

impl Options {
    fn spec(self) -> PosterSpec {
        PosterSpec {
            tile_scale: self.tile_scale,
            overlap_pt: self.overlap_pt,
            cut_marks: self.cut_marks,
            labels: self.labels,
            tile_only_large_pages: self.large_only,
            ..PosterSpec::default()
        }
    }
}

/// One sheet of a poster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    /// Zero-based row, counting down.
    pub row: usize,
    /// Zero-based column, counting right.
    pub column: usize,
    /// Rows in the grid.
    pub rows: usize,
    /// Columns in the grid.
    pub columns: usize,
    /// The part of the page this sheet shows — page space.
    pub source_pt: Rect,
    /// Where that part lands — sheet space. Its size is `source_pt`'s times
    /// [`Self::tile_scale`].
    pub content_pt: Rect,
    /// The part of `content_pt` kept when the sheets are trimmed and butted —
    /// sheet space. The cut marks sit on its leading edges.
    pub trim_pt: Rect,
    /// Magnification from page points to sheet points.
    pub tile_scale: f64,
    /// The band this sheet reserves, `(left, top)`, points.
    pub band_pt: (f64, f64),
    /// Draw cut marks.
    pub cut_marks: bool,
    /// Draw the label.
    pub labels: bool,
    /// The engine's cut marks for this sheet — sheet space, points. Empty
    /// when marks are off.
    pub marks: [Option<imposition::MarkSegment>; 2],
    /// The engine's box for the label — sheet space, points. Its height is
    /// the type size. `None` when labels are off.
    pub label_rect: Option<Rect>,
}

impl Tile {
    /// Whether the sheet carries anything besides the page.
    #[must_use]
    pub fn has_band(&self) -> bool {
        self.band_pt.0 > 0.0 || self.band_pt.1 > 0.0
    }

    /// The sheet-space extent the composed bitmap covers: from the printable
    /// origin to the content's far corner, points.
    #[must_use]
    pub fn extent_pt(&self) -> (f64, f64) {
        (self.content_pt.right(), self.content_pt.bottom())
    }

    /// The assembly label, in the engine's wording.
    #[must_use]
    pub fn label(&self, document: &str) -> String {
        imposition::poster_tile_label(self.row, self.column, self.rows, self.columns, document)
    }

    /// Cut-mark segments, sheet space, points: `(x0, y0, x1, y1)`.
    #[must_use]
    pub fn mark_segments(&self) -> Vec<(f64, f64, f64, f64)> {
        self.marks
            .iter()
            .flatten()
            .map(|m| (m.from.0, m.from.1, m.to.0, m.to.1))
            .collect()
    }

    /// Where the label's baseline starts, and its type size — sheet space,
    /// points. `None` when labels are off.
    #[must_use]
    pub fn label_baseline(&self) -> Option<(f64, f64, f64)> {
        self.label_rect
            .map(|r| (r.x, r.y + r.height * BASELINE_FRACTION, r.height))
    }
}

/// What poster mode does with one page.
#[derive(Debug, Clone, PartialEq)]
pub enum Imposed {
    /// The page fits a sheet and "tile only large pages" is on: it prints as
    /// an ordinary page at the tile scale.
    Untiled,
    /// The page is tiled.
    Tiled {
        /// The sheets, row-major.
        tiles: Vec<Tile>,
        /// The assembled poster, points.
        poster_pt: (f64, f64),
    },
}

/// Tile one page onto sheets whose printable area is `printable_pt`.
///
/// # Errors
///
/// The engine's [`ImpositionError`], unchanged: an empty sheet, a degenerate
/// page, a bad scale or overlap, or more tiles than the engine's ceiling.
pub fn impose(
    printable_pt: (f64, f64),
    page_pt: (f64, f64),
    options: Options,
) -> Result<Imposed, ImpositionError> {
    let spec = options.spec();
    // The whole sheet, not the sheet less the band: a page left untiled
    // prints as an ordinary page and carries no band.
    if !spec.tiles_page(page_pt, printable_pt) {
        return Ok(Imposed::Untiled);
    }
    let layout = imposition::plan_poster(printable_pt, page_pt, &spec)?;
    let band = layout.mark_band_pt();
    let tiles = layout
        .tiles
        .iter()
        .map(|t| {
            let mut marks = [None; 2];
            for (slot, mark) in marks.iter_mut().zip(layout.cut_mark_segments(t)) {
                *slot = Some(mark);
            }
            Tile {
                row: t.row,
                column: t.column,
                rows: layout.rows,
                columns: layout.columns,
                source_pt: t.source_pt,
                content_pt: t.sheet_pt,
                trim_pt: t.trim_pt,
                tile_scale: options.tile_scale,
                band_pt: band,
                cut_marks: options.cut_marks,
                labels: options.labels,
                marks,
                label_rect: layout.label_rect(t),
            }
        })
        .collect();
    Ok(Imposed::Tiled {
        tiles,
        poster_pt: layout.poster_pt,
    })
}

/// The sheet's bitmap: `content` (the tile's part of the page, rendered at
/// `px_per_pt × tile_scale`) placed at its sheet position on white, with the
/// cut marks and `label` drawn in the band. Covers [`Tile::extent_pt`] at
/// `px_per_pt` device pixels per sheet point.
///
/// `None` when the extent is too large for a pixmap.
#[must_use]
pub fn compose(
    content: &tiny_skia::Pixmap,
    tile: &Tile,
    px_per_pt: f64,
    label: Option<&str>,
) -> Option<tiny_skia::Pixmap> {
    let (w, h) = tile.extent_pt();
    let px = |v: f64| (v * px_per_pt).round();
    let mut sheet = tiny_skia::Pixmap::new(px(w).max(1.0) as u32, px(h).max(1.0) as u32)?;
    sheet.fill(tiny_skia::Color::WHITE);
    sheet.draw_pixmap(
        px(tile.content_pt.x) as i32,
        px(tile.content_pt.y) as i32,
        content.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    let mut ink = tiny_skia::Paint::default();
    ink.set_color(tiny_skia::Color::BLACK);
    ink.anti_alias = true;
    let stroke = tiny_skia::Stroke {
        width: (MARK_WIDTH_PT * px_per_pt).max(1.0) as f32,
        ..tiny_skia::Stroke::default()
    };
    for (x0, y0, x1, y1) in tile.mark_segments() {
        let mut path = tiny_skia::PathBuilder::new();
        path.move_to((x0 * px_per_pt) as f32, (y0 * px_per_pt) as f32);
        path.line_to((x1 * px_per_pt) as f32, (y1 * px_per_pt) as f32);
        if let Some(path) = path.finish() {
            sheet.stroke_path(&path, &ink, &stroke, tiny_skia::Transform::identity(), None);
        }
    }
    if let (Some((x, y, size)), Some(text)) = (tile.label_baseline(), label) {
        draw_text(
            &mut sheet,
            text,
            x * px_per_pt,
            y * px_per_pt,
            size * px_per_pt,
            &ink,
        );
    }
    Some(sheet)
}

/// Fill `text` in pdfcer's bundled fixed-pitch face, baseline starting at
/// `(x, y)` device pixels, `size` device pixels per em.
///
/// Characters outside `WinAnsiEncoding` print as `?`: the face is addressed
/// by glyph name, and WinAnsi is the name table the engine exposes.
fn draw_text(
    sheet: &mut tiny_skia::Pixmap,
    text: &str,
    x: f64,
    y: f64,
    size: f64,
    ink: &tiny_skia::Paint<'_>,
) {
    let faces = pdfcer_render::font::bundled::faces();
    let Some(data) = faces.get(&pdfcer_render::FallbackKey::Fixed) else {
        return;
    };
    let Ok(font) = pdfcer_render::font::program::FontProgram::parse(data.bytes()) else {
        return;
    };
    let em = f64::from(font.upem());
    if em <= 0.0 {
        return;
    }
    let s = size / em;
    let names = glyph_names();
    let mut pen = x;
    for ch in text.chars() {
        let name = names.get(&ch).copied().unwrap_or("question");
        if let Some(gid) = font.glyph_for_name(name)
            && let Ok(Some(path)) = font.outline(gid)
        {
            let at =
                tiny_skia::Transform::from_row(s as f32, 0.0, 0.0, -s as f32, pen as f32, y as f32);
            sheet.fill_path(&path, ink, tiny_skia::FillRule::Winding, at, None);
        }
        pen += FIXED_ADVANCE * size;
    }
}

/// The fixed face's advance, in ems.
const FIXED_ADVANCE: f64 = 0.6;

/// Character → glyph name, over `WinAnsiEncoding`.
fn glyph_names() -> &'static HashMap<char, &'static str> {
    static NAMES: OnceLock<HashMap<char, &'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        use pdfcer_core::fontdata::{BaseEncoding, encoding_glyph_name, glyph_name_to_unicode};
        (0..=u8::MAX)
            .filter_map(|code| encoding_glyph_name(BaseEncoding::WinAnsi, code))
            .filter_map(|name| glyph_name_to_unicode(name).map(|ch| (ch, name)))
            .collect()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use pdfcer_print::imposition::POSTER_MARK_BAND_PT as BAND_PT;

    const LETTER: (f64, f64) = (612.0, 792.0);

    fn options(cut_marks: bool, labels: bool) -> Options {
        Options {
            tile_scale: 2.0,
            overlap_pt: 0.0,
            cut_marks,
            labels,
            large_only: false,
        }
    }

    fn tiles(imposed: Imposed) -> Vec<Tile> {
        match imposed {
            Imposed::Tiled { tiles, .. } => tiles,
            Imposed::Untiled => panic!("expected tiles"),
        }
    }

    /// Every tile is the engine's, unmoved, with or without a band — the
    /// band is the engine's to reserve, and reserving it twice shrinks the
    /// drawing twice.
    #[test]
    fn the_tiles_are_the_engines_unmoved() {
        for (marks, labels) in [(false, false), (true, true), (false, true)] {
            let o = options(marks, labels);
            let ours = tiles(impose(LETTER, LETTER, o).unwrap());
            let engine = imposition::plan_poster(LETTER, LETTER, &o.spec()).unwrap();
            assert_eq!(ours.len(), engine.tiles.len());
            for (a, b) in ours.iter().zip(&engine.tiles) {
                assert_eq!(a.content_pt, b.sheet_pt);
                assert_eq!(a.trim_pt, b.trim_pt);
                assert_eq!(a.source_pt, b.source_pt);
                assert_eq!(a.label_rect, engine.label_rect(b));
            }
        }
    }

    /// Marks move the drawing off the top and left edges by the band, so no
    /// mark is drawn over it.
    #[test]
    fn marks_move_the_drawing_clear_of_the_band() {
        for tile in tiles(impose(LETTER, LETTER, options(true, true)).unwrap()) {
            assert!(tile.content_pt.x >= BAND_PT - 1e-6 && tile.content_pt.y >= BAND_PT - 1e-6);
            assert_eq!(tile.mark_segments().len(), 2);
            assert!(tile.content_pt.right() <= LETTER.0 + 1e-6);
            assert!(tile.content_pt.bottom() <= LETTER.1 + 1e-6);
            for (x0, y0, x1, y1) in tile.mark_segments() {
                let inside =
                    |x: f64, y: f64| x > tile.content_pt.x + 1e-6 && y > tile.content_pt.y + 1e-6;
                assert!(!inside(x0, y0) && !inside(x1, y1), "a mark on the drawing");
            }
        }
    }

    /// A label alone reserves only the top band.
    #[test]
    fn a_label_alone_reserves_only_the_top() {
        let band = |m, l| tiles(impose(LETTER, LETTER, options(m, l)).unwrap())[0].band_pt;
        assert_eq!(band(false, true), (0.0, BAND_PT));
        assert_eq!(band(true, false), (BAND_PT, BAND_PT));
    }

    /// "Tile only large pages" passes a page that fits through untiled.
    #[test]
    fn a_page_that_fits_is_untiled_when_asked() {
        let small = Options {
            tile_scale: 0.5,
            large_only: true,
            ..options(false, false)
        };
        assert_eq!(impose(LETTER, LETTER, small).unwrap(), Imposed::Untiled);
    }

    /// The composed sheet has the drawing where the tile says and ink in the
    /// band where the marks and label go.
    #[test]
    fn compose_places_the_content_and_inks_the_band() {
        let tile = tiles(impose(LETTER, LETTER, options(true, true)).unwrap())[0];
        let px_per_pt = 0.5;
        let mut content = tiny_skia::Pixmap::new(
            (tile.content_pt.width * px_per_pt) as u32,
            (tile.content_pt.height * px_per_pt) as u32,
        )
        .unwrap();
        content.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        let sheet = compose(&content, &tile, px_per_pt, Some(&tile.label("plan.pdf"))).unwrap();
        let at = |x: f64, y: f64| {
            sheet
                .pixel((x * px_per_pt) as u32, (y * px_per_pt) as u32)
                .unwrap()
        };
        let red = at(tile.content_pt.x + 20.0, tile.content_pt.y + 20.0);
        assert_eq!((red.red(), red.green()), (255, 0));
        let dark_in_band = (0..sheet.width())
            .flat_map(|x| (0..(BAND_PT * px_per_pt) as u32).map(move |y| (x, y)))
            .filter(|&(x, y)| sheet.pixel(x, y).unwrap().green() < 128)
            .count();
        assert!(dark_in_band > 10, "no ink in the band: {dark_in_band}");
    }

    /// With marks off, the only ink in the band is the label's — so the face
    /// loaded and its glyphs drew.
    #[test]
    fn the_label_draws_glyphs() {
        let tile = tiles(impose(LETTER, LETTER, options(false, true)).unwrap())[0];
        let px_per_pt = 2.0;
        let content = tiny_skia::Pixmap::new(4, 4).unwrap();
        let sheet = compose(&content, &tile, px_per_pt, Some("ABC")).unwrap();
        let dark = (0..sheet.width())
            .flat_map(|x| (0..(BAND_PT * px_per_pt) as u32).map(move |y| (x, y)))
            .filter(|&(x, y)| sheet.pixel(x, y).unwrap().green() < 128)
            .count();
        assert!(dark > 30, "the label drew {dark} dark pixels");
    }

    /// The label names the sheet's place in the grid, 1-based with totals.
    #[test]
    fn the_label_names_row_and_column_of_the_grid() {
        let all = tiles(impose(LETTER, LETTER, options(false, true)).unwrap());
        let last = all.last().unwrap();
        let text = last.label("plan.pdf");
        assert!(text.contains("plan.pdf"), "{text}");
        assert!(
            text.contains(&format!("row {} of {}", last.rows, last.rows)),
            "{text}"
        );
    }
}
