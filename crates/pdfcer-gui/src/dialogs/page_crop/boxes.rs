//! The crop window's choice of box, its picture of the page, and the buttons
//! under the margins.
//!
//! Choosing a box reseeds the margins from that box on the first page the
//! window was raised on, so the numbers always describe a box that exists.
//! The picture is the window's own diagram of the sheet as the operator sees
//! it (after `/Rotate`) with the box the margins describe; it is not the page
//! view and draws nothing onto the document.

use egui::{Sense, Stroke, Ui, vec2};
use pdfcer_core::edit::PageBox;

use super::{PageCropDialog, REGION_WHOLE};
use crate::text::page_crop as t;

/// The boxes, in the order the window offers them and the text catalogue
/// numbers them.
pub(super) const BOXES: [PageBox; 4] = [PageBox::Crop, PageBox::Bleed, PageBox::Trim, PageBox::Art];

/// One box radio; the suffix is [`key`].
const REGION_BOX_PREFIX: &str = "page-crop.box."; // ui-text-exempt: trace region name, never displayed
/// Set to zero, for a bleed, trim or art box.
const REGION_ZERO: &str = "page-crop.zero"; // ui-text-exempt: trace region name, never displayed
/// Remove the box, for a bleed, trim or art box.
const REGION_REMOVE: &str = "page-crop.remove"; // ui-text-exempt: trace region name, never displayed
/// Same margin on every edge.
const REGION_EQUAL: &str = "page-crop.equal"; // ui-text-exempt: trace region name, never displayed
/// Back to the marquee region.
const REGION_BACK: &str = "page-crop.region"; // ui-text-exempt: trace region name, never displayed
/// The picture of the page.
const REGION_PICTURE: &str = "page-crop.picture"; // ui-text-exempt: trace region name, never displayed

/// The picture's longer side, in points of screen.
const PICTURE: f32 = 112.0;

/// `which`'s place in [`BOXES`].
pub(super) fn index(which: PageBox) -> usize {
    BOXES.iter().position(|&b| b == which).unwrap_or(0)
}

/// The machine name, for regions and traces.
pub(super) const fn key(which: PageBox) -> &'static str {
    // ui-text-exempt: region suffixes and trace values, never displayed.
    match which {
        PageBox::Crop => "crop",
        PageBox::Bleed => "bleed",
        PageBox::Trim => "trim",
        PageBox::Art => "art",
        _ => "other",
    }
}

impl PageCropDialog {
    /// The first page the window was raised on: its sheet and rotation.
    fn first_sheet(&self) -> Option<(pdfcer_core::page_tree::Rect, u16)> {
        self.sheets.get(*self.operands.first()?).copied()
    }

    /// The four box radios. Choosing one reseeds the margins from it.
    pub(super) fn box_group(&mut self, ui: &mut Ui) {
        let before = self.which;
        ui.horizontal(|ui| {
            ui.label(t::box_heading());
            for (i, which) in BOXES.into_iter().enumerate() {
                let radio = ui
                    .radio_value(&mut self.which, which, t::box_label(i))
                    .on_hover_text(t::box_tooltip(i));
                crate::diag::ui_rect(&format!("{REGION_BOX_PREFIX}{}", key(which)), radio.rect);
            }
        });
        if self.which == before {
            return;
        }
        if let Some((media, rotate)) = self.first_sheet() {
            self.margins_mm = Self::margins_of(media, rotate, self.seeds[index(self.which)]);
        }
        let m = self.margins_mm;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-crop-box box={} margins_mm={},{},{},{}",
                key(self.which),
                m[0],
                m[1],
                m[2],
                m[3]
            )
        });
    }

    /// Show whole sheet (crop) or Set to zero and Remove (the others), Same
    /// margin on every edge, and Back to the region when opened on one.
    pub(super) fn margin_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if self.which == PageBox::Crop {
                let whole = ui.button(t::whole_sheet());
                crate::diag::ui_rect(REGION_WHOLE, whole.rect);
                if whole.on_hover_text(t::whole_sheet_tooltip()).clicked() {
                    self.margins_mm = [0; 4];
                }
            } else {
                let zero = ui.button(t::set_zero());
                crate::diag::ui_rect(REGION_ZERO, zero.rect);
                if zero.on_hover_text(t::set_zero_tooltip()).clicked() {
                    self.margins_mm = [0; 4];
                }
                let i = index(self.which);
                let remove = ui.button(t::remove_box(i));
                crate::diag::ui_rect(REGION_REMOVE, remove.rect);
                if remove.on_hover_text(t::remove_box_tooltip(i)).clicked() {
                    self.remove_requested = true;
                }
            }
            if let Some(region) = self.region {
                let back = ui.button(t::back_to_region());
                crate::diag::ui_rect(REGION_BACK, back.rect);
                if back.on_hover_text(t::back_to_region_tooltip()).clicked() {
                    self.margins_mm = region;
                }
            }
        });
        let equal = ui
            .checkbox(&mut self.equal, t::equal())
            .on_hover_text(t::equal_tooltip());
        crate::diag::ui_rect(REGION_EQUAL, equal.rect);
        if equal.changed() && self.equal {
            self.margins_mm = [self.margins_mm[0]; 4];
        }
    }

    /// The sheet of the first page in scope as the operator sees it, with
    /// the box the margins describe outlined on it.
    pub(super) fn picture(&self, ui: &mut Ui) {
        let sheet = self
            .pages()
            .first()
            .and_then(|&i| self.sheets.get(i).copied())
            .or_else(|| self.first_sheet());
        let Some((media, rotate)) = sheet else {
            return;
        };
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a sheet's size in points fits f32 for drawing" // ui-text-exempt: lint justification
        )]
        let (w, h) = if rotate % 180 == 90 {
            (media.height() as f32, media.width() as f32)
        } else {
            (media.width() as f32, media.height() as f32)
        };
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let scale = PICTURE / w.max(h);
        let (rect, _) = ui.allocate_exact_size(vec2(w * scale, h * scale), Sense::hover());
        crate::diag::ui_rect(REGION_PICTURE, rect);
        let painter = ui.painter_at(rect.expand(1.0));
        let visuals = ui.visuals();
        let (accent, _) = egui_shell::theme::Theme::accent_pair(ui.ctx());
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, visuals.weak_text_color()),
            egui::StrokeKind::Inside,
        );
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            reason = "a millimetre count drawn at screen precision" // ui-text-exempt: lint justification
        )]
        let at = |i: usize| crate::units::points_from_mm(self.margins_mm[i] as f64) as f32 * scale;
        let inner =
            egui::Rect::from_min_max(rect.min + vec2(at(0), at(2)), rect.max - vec2(at(1), at(3)));
        if inner.width() > 0.0 && inner.height() > 0.0 {
            painter.rect_stroke(
                inner,
                0.0,
                Stroke::new(1.5, accent),
                egui::StrokeKind::Middle,
            );
        }
    }
}
