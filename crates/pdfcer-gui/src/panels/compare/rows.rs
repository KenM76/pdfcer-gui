//! # `panels::compare::rows` — a comparison as the panel's list
//!
//! One row per text change and per appearance mark, each carrying the place
//! in the newer document a click reveals. The appearance marks are built here
//! too, so the list and the comments `Mark changes as comments` writes are
//! the same set.

use pdfcer_core::compare::{Change, ChangeKind, CompareReport, RegionMark, Side};
use pdfcer_core::document::Document;
use pdfcer_core::page_tree::Rect;
use pdfcer_render::{PageMatch, VisualCompareReport};

use crate::text::panels::compare as t;

/// What a row reports; it picks the swatch, the label and the hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Inserted,
    Deleted,
    Replaced,
    Formatting,
    Area,
    PageAdded,
    PageRemoved,
}

impl RowKind {
    /// The kind's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Inserted => t::kind_inserted(),
            Self::Deleted => t::kind_deleted(),
            Self::Replaced => t::kind_replaced(),
            Self::Formatting => t::kind_formatting(),
            Self::Area => t::kind_area(),
            Self::PageAdded => t::kind_page_added(),
            Self::PageRemoved => t::kind_page_removed(),
        }
    }

    /// The colour the engine marks this kind in, so the list's swatch and the
    /// comment agree.
    /// `None` only if the engine's palette ever leaves DeviceRGB.
    #[must_use]
    pub fn swatch(self) -> Option<egui::Color32> {
        use pdfcer_core::compare as c;
        let colour = match self {
            Self::Inserted | Self::PageAdded => c::INSERTED_COLOR,
            Self::Replaced | Self::Area => c::REPLACED_COLOR,
            Self::Deleted | Self::PageRemoved => c::DELETED_COLOR,
            Self::Formatting => c::FORMATTING_COLOR,
        };
        rgb(colour)
    }

    /// The token the trace names the kind by.
    #[must_use]
    pub fn token(self) -> &'static str {
        // ui-text-exempt: diagnostic tokens, never displayed
        match self {
            Self::Inserted => "inserted",
            Self::Deleted => "deleted",
            Self::Replaced => "replaced",
            Self::Formatting => "formatting",
            Self::Area => "area",
            Self::PageAdded => "page-added",
            Self::PageRemoved => "page-removed",
        }
    }
}

fn rgb(colour: pdfcer_core::annot_author::Color) -> Option<egui::Color32> {
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match colour {
        pdfcer_core::annot_author::Color::Rgb(r, g, b) => {
            Some(egui::Color32::from_rgb(byte(r), byte(g), byte(b))) // DOCUMENT COLOUR: the colour the engine writes the comparison comment in
        }
        _ => None,
    }
}

/// One line of the list.
#[derive(Debug, Clone)]
pub struct Row {
    pub kind: RowKind,
    /// What changed, in words.
    pub line: String,
    /// Where it is in the newer document: 0-based page and user-space rect.
    /// `None` when the change is only in the older version.
    pub at: Option<(usize, Rect)>,
}

/// The list for a text report and the appearance marks.
#[must_use]
pub fn rows(text: Option<&CompareReport>, visual: Option<&VisualCompareReport>) -> Vec<Row> {
    let mut out: Vec<Row> = text
        .map(|r| r.changes.iter().map(text_row).collect())
        .unwrap_or_default();
    if let Some(v) = visual {
        out.extend(visual_rows(v));
    }
    out
}

fn text_row(change: &Change) -> Row {
    let kind = match change.kind {
        ChangeKind::Inserted => RowKind::Inserted,
        ChangeKind::Deleted => RowKind::Deleted,
        ChangeKind::Formatting => RowKind::Formatting,
        _ => RowKind::Replaced,
    };
    let line = match kind {
        RowKind::Inserted => t::quoted(&clip(&change.new_text)),
        RowKind::Deleted => t::quoted(&clip(&change.old_text)),
        RowKind::Formatting => t::formatting_line(&clip(&change.new_text), &formatting(change)),
        _ => t::replaced_line(&clip(&change.old_text), &clip(&change.new_text)),
    };
    Row {
        kind,
        line,
        at: place_in_newer(change),
    }
}

/// The first newer-side place, else a newer-side anchor.
fn place_in_newer(change: &Change) -> Option<(usize, Rect)> {
    if let Some(p) = change.new_places.first() {
        return Some((p.page, p.rect));
    }
    match &change.anchor {
        Some((Side::New, p)) => Some((p.page, p.rect)),
        _ => None,
    }
}

fn formatting(change: &Change) -> Vec<String> {
    let Some(delta) = &change.formatting else {
        return Vec::new();
    };
    let mut what = Vec::new();
    if let Some((a, b)) = delta.size {
        what.push(t::fmt_size(a, b));
    }
    if let Some((a, b)) = delta.weight {
        what.push(t::fmt_weight(a, b));
    }
    if delta.color.is_some() {
        what.push(t::fmt_colour().to_owned());
    }
    what
}

/// At most 160 characters of a stretch of words, as the CLI shows them.
fn clip(text: &str) -> String {
    const MAX: usize = 160;
    if text.chars().count() <= MAX {
        return text.to_owned();
    }
    let mut s: String = text.chars().take(MAX).collect();
    s.push('\u{2026}');
    s
}

fn visual_rows(report: &VisualCompareReport) -> Vec<Row> {
    let mut out = Vec::new();
    for pair in &report.pairs {
        match (pair.old, pair.new) {
            (None, Some(n)) => out.push(Row {
                kind: RowKind::PageAdded,
                line: t::page_added_line().to_owned(),
                at: Some((
                    n,
                    Rect {
                        llx: 0.0,
                        lly: 0.0,
                        urx: 0.0,
                        ury: 0.0,
                    },
                )),
            }),
            (Some(o), None) => out.push(Row {
                kind: RowKind::PageRemoved,
                line: t::page_removed_line(o + 1),
                at: None,
            }),
            (Some(o), Some(n)) => out.extend(pair.regions.iter().map(|g| Row {
                kind: RowKind::Area,
                line: t::area_line(o + 1, g.changed_pixels),
                at: Some((n, g.rect)),
            })),
            (None, None) => {}
        }
    }
    out
}

/// The appearance comments, as `pdfcer compare --visual --output` writes
/// them: a cloud round each changed area and a frame inside each added page.
#[must_use]
pub fn region_marks(report: &VisualCompareReport, new: &Document) -> Vec<RegionMark> {
    let boxes: Vec<Rect> = pdfcer_core::page_tree::pages(new)
        .map(|p| p.iter().map(|p| p.crop_box).collect())
        .unwrap_or_default();
    let mut out = Vec::new();
    for pair in &report.pairs {
        let Some(n) = pair.new else { continue };
        let Some(o) = pair.old else {
            if let Some(b) = boxes.get(n) {
                out.push(RegionMark::inserted_page(n, *b, t::page_added_note()));
            }
            continue;
        };
        for g in &pair.regions {
            out.push(RegionMark::changed(
                n,
                g.rect,
                t::area_note(o + 1, g.changed_pixels),
            ));
        }
    }
    out
}

/// What the appearance comparison inferred and the operator cannot see.
#[must_use]
pub fn visual_notes(report: &VisualCompareReport) -> Vec<String> {
    let mut notes = vec![t::appearance_basis(report.scale * 72.0, report.threshold)];
    if report.nothing_matched() {
        notes.push(t::nothing_matched().to_owned());
    }
    for pair in &report.pairs {
        if let (Some(o), Some(n)) = (pair.old, pair.new) {
            if pair.how == PageMatch::Moved {
                notes.push(t::page_moved(o + 1, n + 1));
            }
            if pair.size_differs {
                notes.push(t::size_differs(n + 1));
            }
        }
        if let (Some(n), Some(why)) = (pair.new, &pair.error) {
            notes.push(t::pair_failed(n + 1, why));
        }
    }
    notes
}
