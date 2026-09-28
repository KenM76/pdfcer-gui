//! # `dialogs::page_crop` — **the visible area of the picked sheets**
//!
//! `pages.crop`, Pages ▸ Transform ▸ Crop…. Four margins, measured in from the
//! edges **as the operator sees the sheet** (after `/Rotate`), become one
//! `CropBoxEdit::Set`; all four at zero become `CropBoxEdit::Reset`, so the
//! whole sheet shows again without a crop box being written.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/page_crop.md`.

use egui::Ui;
use pdfcer_core::edit::CropBoxEdit;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::state::OpenDoc;
use crate::text::page_crop as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "page-crop.body";
/// One margin box; the suffix is `left`, `right`, `top` or `bottom`.
const REGION_MARGIN_PREFIX: &str = "page-crop.margin.";
/// The Show whole sheet button.
const REGION_WHOLE: &str = "page-crop.whole";
/// The commit button.
const REGION_APPLY: &str = "page-crop.apply";

/// The on-screen edges, in the order the window lists them.
const EDGES: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];

/// An edge of the sheet as the operator sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    /// The label beside its box.
    const fn label(self) -> &'static str {
        match self {
            Self::Left => t::left(),
            Self::Right => t::right(),
            Self::Top => t::top(),
            Self::Bottom => t::bottom(),
        }
    }

    /// The machine name, for regions and traces.
    const fn key(self) -> &'static str {
        // ui-text-exempt: region suffixes, never displayed.
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    /// Which **page-space** edge this screen edge is on a sheet turned
    /// `rotate` degrees clockwise: an index into `[left, right, bottom, top]`
    /// of the unrotated page.
    const fn pdf_index(self, rotate: u16) -> usize {
        // Turning a page 90° clockwise carries its left edge to the top, its
        // top to the right, its right to the bottom and its bottom to the left.
        let (l, r, b, t) = (0, 1, 2, 3);
        match (rotate, self) {
            (90, Self::Left) | (180, Self::Top) | (0, Self::Bottom) => b,
            (90, Self::Right) | (180, Self::Bottom) | (0, Self::Top) => t,
            (90, Self::Top) | (180, Self::Right) | (0, Self::Left) => l,
            (90, Self::Bottom) | (180, Self::Left) | (0, Self::Right) => r,
            (_, Self::Left) => t,
            (_, Self::Right) => b,
            (_, Self::Top) => r,
            (_, Self::Bottom) => l,
        }
    }
}

/// The crop window's state.
pub struct PageCropDialog {
    /// The operand pages, 0-based, ascending.
    pages: Vec<usize>,
    /// The sheet every operand shares, with its rotation; `None` when the
    /// pick mixes sizes or rotations, and then only the whole sheet is offered.
    sheet: Option<(Rect, u16)>,
    /// Margins in whole millimetres, in page space: `[left, right, bottom,
    /// top]` of the unrotated page.
    margins_mm: [i64; 4],
    /// Set by the commit button, consumed after the window closure returns.
    apply_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
}

impl PageCropDialog {
    /// Open the window over `pages`, seeded with the first sheet's crop.
    #[must_use]
    pub fn open(doc: &OpenDoc, pages: &[usize]) -> Option<Self> {
        let picked: Vec<_> = pages.iter().filter_map(|&i| doc.pages.get(i)).collect();
        let first = picked.first()?;
        let tol = pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE;
        let same = |a: Rect, b: Rect| {
            (a.llx - b.llx).abs() <= tol
                && (a.lly - b.lly).abs() <= tol
                && (a.urx - b.urx).abs() <= tol
                && (a.ury - b.ury).abs() <= tol
        };
        let uniform = picked
            .iter()
            .all(|p| same(p.media_box, first.media_box) && p.rotate == first.rotate);
        let sheet = uniform.then_some((first.media_box, first.rotate));

        let (m, c) = (first.media_box, first.crop_box);
        let mm = crate::units::whole_mm_from_points;
        let margins_mm = if uniform {
            [
                mm(c.llx - m.llx),
                mm(m.urx - c.urx),
                mm(c.lly - m.lly),
                mm(m.ury - c.ury),
            ]
        } else {
            [0; 4]
        };

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-crop-opened sheets={} uniform={} rotate={} margins_mm={:?}",
                pages.len(),
                u8::from(uniform),
                first.rotate,
                margins_mm,
            )
        });

        Some(Self {
            pages: pages.to_vec(),
            sheet,
            margins_mm,
            apply_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "page-crop", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(420.0, 340.0),
            egui::vec2(340.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.apply_requested)
            && let Some(edit) = self.edit()
        {
            crate::diag::trace(|| {
                let (kind, r) = match edit {
                    CropBoxEdit::Set(r) => ("set", r),
                    _ => ("reset", Rect::from_corners(0.0, 0.0, 0.0, 0.0)),
                };
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-crop-commit n={} edit={kind} llx={:.2} lly={:.2} urx={:.2} ury={:.2}",
                    self.pages.len(),
                    r.llx,
                    r.lly,
                    r.urx,
                    r.ury,
                )
            });
            actions.push(Action::Page(PageAction::SetCropBox {
                pages: self.pages.clone(),
                edit,
            }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The crop box the margins describe, in page space, or `None` for a
    /// mixed pick or margins that leave nothing.
    fn rect(&self) -> Option<Rect> {
        let (m, _) = self.sheet?;
        #[allow(
            clippy::cast_precision_loss,
            reason = "a millimetre count is exact in f64" // ui-text-exempt: lint justification
        )]
        let pt = |i: usize| crate::units::points_from_mm(self.margins_mm[i] as f64);
        let r = Rect::from_corners(m.llx + pt(0), m.lly + pt(2), m.urx - pt(1), m.ury - pt(3));
        (m.llx + pt(0) < m.urx - pt(1) - 1.0 && m.lly + pt(2) < m.ury - pt(3) - 1.0).then_some(r)
    }

    /// What committing asks of the engine: the whole sheet when every margin
    /// is zero, otherwise the rectangle. `None` when nothing valid is asked.
    fn edit(&self) -> Option<CropBoxEdit> {
        if self.margins_mm == [0; 4] {
            return Some(CropBoxEdit::Reset);
        }
        self.rect().map(CropBoxEdit::Set)
    }

    /// The controls and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        match self.sheet {
            Some((media, rotate)) => {
                ui.label(t::margins_heading());
                egui::Grid::new("page-crop.grid") // ui-text-exempt: widget id
                    .num_columns(2)
                    .show(ui, |ui| {
                        for edge in EDGES {
                            ui.label(edge.label());
                            let slot = &mut self.margins_mm[edge.pdf_index(rotate)];
                            let (widget, refusal) = entry::drag_value(
                                ui,
                                slot,
                                entry::Kind::Length(entry::LengthUnit::Of(
                                    pdfcer_core::dimension::Unit::Millimeter,
                                )),
                            );
                            let response = refusal.show(ui.add(widget.range(0..=5080)));
                            crate::diag::ui_rect(
                                &format!("{REGION_MARGIN_PREFIX}{}", edge.key()),
                                response.rect,
                            );
                            ui.end_row();
                        }
                    });
                let whole = ui.button(t::whole_sheet());
                crate::diag::ui_rect(REGION_WHOLE, whole.rect);
                if whole.on_hover_text(t::whole_sheet_tooltip()).clicked() {
                    self.margins_mm = [0; 4];
                }
                ui.add_space(6.0);
                let shown = self.rect().unwrap_or(media);
                if self.edit().is_some() {
                    let (w, h) = if rotate % 180 == 90 {
                        (shown.height(), shown.width())
                    } else {
                        (shown.width(), shown.height())
                    };
                    let mm = crate::units::whole_mm_from_points;
                    ui.label(
                        egui::RichText::new(t::visible_summary(mm(w), mm(h)))
                            .small()
                            .weak(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(t::nothing_left())
                            .small()
                            .color(ui.visuals().error_fg_color),
                    );
                }
            }
            None => {
                ui.label(t::mixed_sizes(self.pages.len()));
                let whole = ui.button(t::whole_sheet());
                crate::diag::ui_rect(REGION_WHOLE, whole.rect);
                if whole.on_hover_text(t::whole_sheet_tooltip()).clicked() {
                    self.apply_requested = true;
                }
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // Absent, not greyed, when the margins leave nothing: the refusal
            // line above already says why (R9).
            if self.sheet.is_some() && self.edit().is_some() {
                let apply = ui.button(t::apply());
                crate::diag::ui_rect(REGION_APPLY, apply.rect);
                if apply.on_hover_text(t::apply_tooltip()).clicked() {
                    self.apply_requested = true;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog(rotate: u16, margins_mm: [i64; 4]) -> PageCropDialog {
        PageCropDialog {
            pages: vec![0],
            sheet: Some((Rect::from_corners(0.0, 0.0, 595.0, 842.0), rotate)),
            margins_mm,
            apply_requested: false,
            close_requested: false,
        }
    }

    /// Every screen edge lands on a different page edge, at every rotation.
    #[test]
    fn each_rotation_maps_the_four_edges_onto_four_page_edges() {
        for rotate in [0, 90, 180, 270] {
            let mut seen: Vec<usize> = EDGES.iter().map(|e| e.pdf_index(rotate)).collect();
            seen.sort_unstable();
            assert_eq!(seen, vec![0, 1, 2, 3], "rotate={rotate}");
        }
        // A sheet turned a quarter clockwise shows its page-space left edge on top.
        assert_eq!(Edge::Top.pdf_index(90), 0);
        assert_eq!(Edge::Top.pdf_index(270), 1);
    }

    /// Zero margins ask for the whole sheet, not for a crop box equal to it.
    #[test]
    fn zero_margins_reset_rather_than_write_the_sheet() {
        assert_eq!(dialog(0, [0; 4]).edit(), Some(CropBoxEdit::Reset));
    }

    /// Margins become a rectangle measured in from the sheet's edges.
    #[test]
    fn margins_become_the_crop_rectangle() {
        let Some(CropBoxEdit::Set(r)) = dialog(0, [10, 0, 0, 20]).edit() else {
            panic!("non-zero margins crop");
        };
        assert!((r.llx - 28.35).abs() < 0.01, "{r:?}");
        assert!((r.ury - (842.0 - 56.69)).abs() < 0.01, "{r:?}");
        assert!((r.urx - 595.0).abs() < 0.01, "{r:?}");
    }

    /// Margins that meet leave nothing, and nothing is offered.
    #[test]
    fn margins_that_cross_offer_no_crop() {
        assert_eq!(dialog(0, [150, 150, 0, 0]).edit(), None);
    }
}
