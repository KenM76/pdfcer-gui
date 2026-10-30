//! # `imposition` — several pages on one sheet: N-up and booklet
//!
//! The layouts are `pdfcer_print::imposition`'s (`plan_n_up`,
//! `plan_booklet`). This module turns a layout into one [`ImposedSheet`] per
//! printed side, in document page numbers, and **rasterises** a sheet from
//! the pages rendered for its cells ([`compose`]).
//!
//! # Coordinate space
//!
//! Every rectangle is the printable area's: points, top-left origin, `+y`
//! down — the engine's, unmoved.
//!
//! # Contract
//!
//! - Pages are imposed in the order the job sends them (range, odd/even,
//!   reverse), before copies; copies then repeat whole sheets.
//! - A booklet side the fold leaves wholly blank is still a sheet: dropping
//!   it would put every later back side on the wrong front.
//! - A turned page is turned a quarter turn **clockwise**, the engine's one
//!   direction (`CellFit::rotated`).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/imposition.md`.

use pdfcer_print::imposition::{
    self as engine, BookletSide, BookletSpec, CellFit, ImpositionError, NUpSpec, Rect,
};
use pdfcer_render::tiny_skia;

use crate::printspooler::{Collate, JobSpec};

/// The width of a cell border, in points. A hairline at any dpi.
pub const BORDER_WIDTH_PT: f64 = 0.5;

/// One page on an imposed sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// The page, as a document page index.
    pub page: usize,
    /// Where it lands, at what scale, and whether it is turned.
    pub fit: CellFit,
    /// The cell outline to stroke, when borders are on.
    pub border: Option<Rect>,
}

/// One printed side.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImposedSheet {
    /// The pages on it. Empty for a booklet side the fold leaves blank.
    pub cells: Vec<Cell>,
    /// For a booklet: the 1-based physical sheet and which face.
    pub booklet: Option<(usize, BookletSide)>,
}

/// What an imposition made, for the captions and the trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Summary {
    /// Several pages per sheet.
    NUp {
        /// Cells down.
        rows: usize,
        /// Cells across.
        columns: usize,
        /// Pages turned to fit.
        turned: usize,
    },
    /// A folded booklet.
    Booklet {
        /// Physical sheets in the whole booklet.
        total_sheets: usize,
        /// The first and last physical sheet this job prints, 1-based.
        printed: (usize, usize),
        /// Fold-mandated blank positions in the whole booklet.
        blanks: usize,
    },
}

/// A whole imposed job, before copies.
#[derive(Debug, Clone, PartialEq)]
pub struct Imposition {
    /// One entry per printed side, in print order.
    pub sheets: Vec<ImposedSheet>,
    /// What was made.
    pub summary: Summary,
}

/// The pages a job sends, in order, once — the sequence imposition takes.
#[must_use]
pub fn sequence_once(spec: &JobSpec) -> Vec<usize> {
    let mut once = crate::printspooler::to_engine_spec(spec);
    once.copies = 1;
    once.sequence()
}

/// Repeat `sheets` for `copies`, in the job's collation.
#[must_use]
pub fn with_copies<T: Clone>(sheets: &[T], copies: u16, collate: Collate) -> Vec<T> {
    let n = usize::from(copies.max(1));
    match collate {
        Collate::Collated => sheets
            .iter()
            .cycle()
            .take(sheets.len() * n)
            .cloned()
            .collect(),
        Collate::Uncollated => sheets
            .iter()
            .flat_map(|s| std::iter::repeat_n(s.clone(), n))
            .collect(),
    }
}

/// Lay `sequence` (document page indices) out `spec.grid` to a sheet.
///
/// # Errors
///
/// The engine's refusal: no pages, a sheet with no printable area, or a grid
/// past `MAX_CELLS_PER_SHEET`.
pub fn n_up(
    printable_pt: (f64, f64),
    sequence: &[usize],
    page_sizes: &[(f64, f64)],
    spec: &NUpSpec,
) -> Result<Imposition, ImpositionError> {
    let (pages, sizes) = sized(sequence, page_sizes);
    let layout = engine::plan_n_up(printable_pt, &sizes, spec)?;
    let mut sheets = vec![ImposedSheet::default(); layout.sheets];
    let mut turned = 0;
    for slot in &layout.slots {
        turned += usize::from(slot.fit.rotated);
        if let Some(sheet) = sheets.get_mut(slot.sheet) {
            sheet.cells.push(Cell {
                page: pages[slot.source],
                fit: slot.fit,
                border: slot.border,
            });
        }
    }
    Ok(Imposition {
        sheets,
        summary: Summary::NUp {
            rows: layout.rows,
            columns: layout.columns,
            turned,
        },
    })
}

/// Lay `sequence` out as a folded booklet.
///
/// # Errors
///
/// The engine's refusal: no pages, a sheet with no printable area, a booklet
/// past `MAX_BOOKLET_SHEETS`, or a sheet range that selects nothing.
pub fn booklet(
    printable_pt: (f64, f64),
    sequence: &[usize],
    page_sizes: &[(f64, f64)],
    spec: &BookletSpec,
) -> Result<Imposition, ImpositionError> {
    let (pages, sizes) = sized(sequence, page_sizes);
    let layout = engine::plan_booklet(printable_pt, &sizes, spec)?;
    let mut sheets: Vec<ImposedSheet> = Vec::new();
    for slot in &layout.slots {
        let key = Some((slot.sheet, slot.side));
        if sheets.last().is_none_or(|s| s.booklet != key) {
            sheets.push(ImposedSheet {
                cells: Vec::new(),
                booklet: key,
            });
        }
        if let (Some(source), Some(fit), Some(sheet)) = (slot.source, slot.fit, sheets.last_mut()) {
            sheet.cells.push(Cell {
                page: pages[source],
                fit,
                border: None,
            });
        }
    }
    let numbers = sheets.iter().filter_map(|s| s.booklet.map(|(n, _)| n));
    let printed = (
        numbers.clone().min().unwrap_or(1),
        numbers.max().unwrap_or(1),
    );
    Ok(Imposition {
        sheets,
        summary: Summary::Booklet {
            total_sheets: layout.total_sheets,
            printed,
            blanks: layout.blank_positions,
        },
    })
}

/// The sequence with a stale index dropped, and each page's size.
fn sized(sequence: &[usize], page_sizes: &[(f64, f64)]) -> (Vec<usize>, Vec<(f64, f64)>) {
    sequence
        .iter()
        .filter_map(|&i| page_sizes.get(i).map(|&size| (i, size)))
        .unzip()
}

/// The device pixels a rendered page needs to be, per page point, to land in
/// `fit` on a sheet drawn at `px_per_pt`.
#[must_use]
pub fn render_scale(fit: &CellFit, px_per_pt: f64) -> f64 {
    px_per_pt * fit.scale
}

/// Draw one sheet: white, each `(rendered page, cell)` at its fit, turned
/// where the fit says, then the borders. `px_per_pt` is the sheet's density;
/// each page must have been rendered at [`render_scale`]. `None` when the
/// sheet is too large to allocate.
#[must_use]
pub fn compose(
    printable_pt: (f64, f64),
    px_per_pt: f64,
    pages: &[(&tiny_skia::Pixmap, Cell)],
) -> Option<tiny_skia::Pixmap> {
    let px = |v: f64| (v * px_per_pt).round();
    let mut sheet = tiny_skia::Pixmap::new(
        px(printable_pt.0).max(1.0) as u32,
        px(printable_pt.1).max(1.0) as u32,
    )?;
    sheet.fill(tiny_skia::Color::WHITE);
    for (page, cell) in pages {
        let (x, y) = (px(cell.fit.rect.x) as f32, px(cell.fit.rect.y) as f32);
        let place = if cell.fit.rotated {
            // Clockwise in a `+y`-down space: (u, v) → (h − v, u), then moved
            // to the footprint's corner. The page's top edge lands on the right.
            tiny_skia::Transform::from_row(0.0, 1.0, -1.0, 0.0, x + page.height() as f32, y)
        } else {
            tiny_skia::Transform::from_translate(x, y)
        };
        sheet.draw_pixmap(
            0,
            0,
            page.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            place,
            None,
        );
    }
    let mut ink = tiny_skia::Paint::default();
    ink.set_color(tiny_skia::Color::BLACK);
    let stroke = tiny_skia::Stroke {
        width: (BORDER_WIDTH_PT * px_per_pt).max(1.0) as f32,
        ..tiny_skia::Stroke::default()
    };
    for r in pages.iter().filter_map(|(_, c)| c.border) {
        let outline = tiny_skia::Rect::from_xywh(
            (r.x * px_per_pt) as f32,
            (r.y * px_per_pt) as f32,
            (r.width * px_per_pt) as f32,
            (r.height * px_per_pt) as f32,
        )
        .map(tiny_skia::PathBuilder::from_rect);
        if let Some(path) = outline {
            sheet.stroke_path(&path, &ink, &stroke, tiny_skia::Transform::identity(), None);
        }
    }
    Some(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printspooler::{PageSubset, ScaleMode};
    use pdfcer_print::imposition::{Binding, NUpGrid};

    const LETTER: (f64, f64) = (612.0, 792.0);
    const LANDSCAPE: (f64, f64) = (792.0, 612.0);

    fn spec(pages: Vec<usize>, reverse: bool, copies: u16) -> JobSpec {
        JobSpec {
            pages,
            mode: ScaleMode::Fit,
            max_dpi: 300,
            subset: PageSubset::All,
            reverse,
            copies,
            collate: Collate::Collated,
        }
    }

    #[test]
    fn the_sequence_is_imposed_once_and_copies_repeat_sheets() {
        assert_eq!(sequence_once(&spec(vec![0, 1, 2], true, 3)), vec![2, 1, 0]);
        assert_eq!(with_copies(&[1, 2], 2, Collate::Collated), vec![1, 2, 1, 2]);
        assert_eq!(
            with_copies(&[1, 2], 2, Collate::Uncollated),
            vec![1, 1, 2, 2]
        );
    }

    #[test]
    fn two_up_puts_document_pages_side_by_side() {
        let sizes = [LETTER; 5];
        let spec = NUpSpec {
            grid: NUpGrid::Count(2),
            ..NUpSpec::default()
        };
        let imposed = n_up(LANDSCAPE, &[4, 3, 2, 1, 0], &sizes, &spec).expect("imposes");
        assert_eq!(imposed.sheets.len(), 3);
        assert_eq!(
            imposed.summary,
            Summary::NUp {
                rows: 1,
                columns: 2,
                turned: 0
            }
        );
        let first: Vec<usize> = imposed.sheets[0].cells.iter().map(|c| c.page).collect();
        assert_eq!(
            first,
            vec![4, 3],
            "cells carry DOCUMENT pages, not sequence positions"
        );
        assert_eq!(imposed.sheets[2].cells.len(), 1);
    }

    #[test]
    fn a_five_page_booklet_keeps_its_blank_sides() {
        let sizes = [LETTER; 5];
        let spec = BookletSpec {
            binding: Binding::Left,
            ..BookletSpec::default()
        };
        let imposed = booklet(LANDSCAPE, &[0, 1, 2, 3, 4], &sizes, &spec).expect("imposes");
        assert_eq!(
            imposed.sheets.len(),
            4,
            "two physical sheets, two sides each"
        );
        assert_eq!(
            imposed.summary,
            Summary::Booklet {
                total_sheets: 2,
                printed: (1, 2),
                blanks: 3
            }
        );
        let placed: usize = imposed.sheets.iter().map(|s| s.cells.len()).sum();
        assert_eq!(placed, 5);
    }

    /// A turned page's top-left pixel lands at the footprint's top-RIGHT.
    #[test]
    fn a_turned_page_is_turned_clockwise() {
        let mut page = tiny_skia::Pixmap::new(4, 6).expect("pixmap");
        page.fill(tiny_skia::Color::WHITE);
        page.pixels_mut()[0] =
            tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255).expect("rgba");
        let cell = Cell {
            page: 0,
            fit: CellFit {
                rect: Rect::new(0.0, 0.0, 6.0, 4.0),
                scale: 1.0,
                rotated: true,
            },
            border: None,
        };
        let sheet = compose((6.0, 4.0), 1.0, &[(&page, cell)]).expect("sheet");
        let dark = |x: u32, y: u32| sheet.pixel(x, y).expect("in range").red() < 128;
        assert!(
            dark(5, 0),
            "the page's top-left goes to the sheet's top-right"
        );
        assert!(!dark(0, 0));
    }

    #[test]
    fn a_border_is_drawn_round_its_cell() {
        let page = tiny_skia::Pixmap::new(2, 2).expect("pixmap");
        let cell = Cell {
            page: 0,
            fit: CellFit {
                rect: Rect::new(4.0, 4.0, 2.0, 2.0),
                scale: 1.0,
                rotated: false,
            },
            border: Some(Rect::new(1.0, 1.0, 10.0, 10.0)),
        };
        let sheet = compose((12.0, 12.0), 1.0, &[(&page, cell)]).expect("sheet");
        assert!(
            sheet.pixel(1, 5).expect("in range").red() < 200,
            "left edge inked"
        );
        assert_eq!(
            sheet.pixel(8, 8).expect("in range").red(),
            255,
            "inside stays white"
        );
    }
}
