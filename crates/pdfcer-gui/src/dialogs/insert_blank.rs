//! # `dialogs::insert_blank` — how many blank sheets, what size, where
//!
//! The dialog behind `pages.insert_blank`. It raises
//! `PageAction::InsertBlankPages`; the sheets themselves are made by
//! `app::actions::insertruns::insert_blank`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/insert_blank.md`.

use egui::Ui;
use pdfcer_core::paper::{Orientation, PaperSize};

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::dialogs::insert_where::Where;
use crate::text::insertblank as t;
use crate::text::new_document as sheets;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "insert-blank.body";

/// The size combo, closed.
const REGION_SIZE: &str = "insert-blank.size";

/// One region per entry in the OPEN size list: `0` is "Same as page N",
/// then `PaperSize::ALL` in order from `1`.
const REGION_SIZE_ITEM_PREFIX: &str = "insert-blank.size.item.";

/// The count field.
const REGION_COUNT: &str = "insert-blank.count";

/// The Landscape radio (present only for a standard size).
const REGION_LANDSCAPE: &str = "insert-blank.landscape";

/// The commit button.
const REGION_INSERT: &str = "insert-blank.insert";

/// The most sheets one press inserts. Each is a separate engine call on the
/// UI thread (see `app::actions::insertruns`), so a dragged count is capped
/// where the wait stays short.
const MAX_COUNT: usize = 500;

/// Which sheet the operator has picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    /// The page the dialog was opened on, as it displays.
    SameAsCurrent,
    /// One of `PaperSize::ALL`, turned by the orientation radios.
    Standard(PaperSize),
}

/// The Insert-blank-pages dialog's state.
pub struct InsertBlankDialog {
    /// The 0-based page the dialog was opened on. Frozen at open, for
    /// `insert_pages`' reason: "after page 7" must not become "after page 9".
    current_page: usize,
    /// That page's displayed size in points, rotation applied.
    current_size: (f64, f64),
    choice: Choice,
    /// Applies to a standard size only; the current page's shape is its own.
    landscape: bool,
    count: usize,
    position: Where,
    /// Set by the commit button, consumed after the window closure returns
    /// (the action rebuilds document state egui is mid-way through drawing).
    insert_requested: bool,
    close_requested: bool,
}

impl InsertBlankDialog {
    /// Open it on one sheet the size of `current_size`, after `current_page`.
    #[must_use]
    pub const fn open(current_page: usize, current_size: (f64, f64)) -> Self {
        Self {
            current_page,
            current_size,
            choice: Choice::SameAsCurrent,
            landscape: false,
            count: 1,
            position: Where::AfterCurrent,
            insert_requested: false,
            close_requested: false,
        }
    }

    /// The sheet this dialog describes, in points, after orientation.
    fn sheet_pt(&self) -> (f64, f64) {
        match self.choice {
            Choice::SameAsCurrent => self.current_size,
            Choice::Standard(size) => {
                let rect = size.rect_with(if self.landscape {
                    Orientation::Landscape
                } else {
                    Orientation::Portrait
                });
                (rect.width(), rect.height())
            }
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "insert-blank", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 480.0),
            egui::vec2(380.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.insert_requested) {
            let (width_pt, height_pt) = self.sheet_pt();
            let position = self.position.position(self.current_page);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "insert-blank-dialog choice={:?} landscape={} n={} w_pt={width_pt:.2} \
                     h_pt={height_pt:.2} position={position:?}",
                    self.choice, self.landscape, self.count,
                )
            });
            actions.push(Action::Page(PageAction::InsertBlankPages {
                width_pt,
                height_pt,
                count: self.count,
                position,
            }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The size, the count, where, and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        self.size_controls(ui);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(t::count_label());
            let count = ui.add(egui::DragValue::new(&mut self.count).range(1..=MAX_COUNT));
            crate::diag::ui_rect(REGION_COUNT, count.rect);
        });

        ui.add_space(8.0);
        self.position.radios(ui, self.current_page, "insert-blank"); // ui-text-exempt: a region prefix, never displayed.

        ui.add_space(8.0);
        let (w, h) = self.sheet_pt();
        ui.label(
            egui::RichText::new(sheets::sheet_summary(w, h))
                .small()
                .weak(),
        );

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(sheets::cancel()).clicked() {
                self.close_requested = true;
            }
            let button = ui.button(t::commit(self.count));
            crate::diag::ui_rect(REGION_INSERT, button.rect);
            if button.clicked() {
                self.insert_requested = true;
            }
        });
    }

    /// The size combo, and the orientation pair beneath it for a standard
    /// size. The pair is absent for "Same as page N", whose shape is fixed.
    fn size_controls(&mut self, ui: &mut Ui) {
        ui.label(t::size_heading());
        let page = self.current_page.saturating_add(1);
        let same = t::same_as_page(page, self.current_size.0, self.current_size.1);
        let selected = match self.choice {
            Choice::SameAsCurrent => same.clone(),
            Choice::Standard(size) => sheets::size_entry(&sheets::size_name(size), size.size_pt()),
        };
        let combo = egui::ComboBox::from_id_salt("insert-blank-size")
            .selected_text(selected)
            .show_ui(ui, |ui| {
                let entry = ui.selectable_value(&mut self.choice, Choice::SameAsCurrent, same);
                crate::diag::ui_rect(&format!("{REGION_SIZE_ITEM_PREFIX}0"), entry.rect);
                for (index, size) in PaperSize::ALL.iter().enumerate() {
                    let entry = ui.selectable_value(
                        &mut self.choice,
                        Choice::Standard(*size),
                        sheets::size_entry(&sheets::size_name(*size), size.size_pt()),
                    );
                    crate::diag::ui_rect(
                        &format!("{REGION_SIZE_ITEM_PREFIX}{}", index + 1),
                        entry.rect,
                    );
                }
            });
        crate::diag::ui_rect(REGION_SIZE, combo.response.rect);

        if matches!(self.choice, Choice::Standard(_)) {
            ui.horizontal(|ui| {
                if ui
                    .radio(!self.landscape, sheets::orientation_portrait())
                    .clicked()
                {
                    self.landscape = false;
                }
                let landscape = ui.radio(self.landscape, sheets::orientation_landscape());
                crate::diag::ui_rect(REGION_LANDSCAPE, landscape.rect);
                if landscape.clicked() {
                    self.landscape = true;
                }
            });
        }
    }
}

/// The size `page` displays at, in points: its crop box, turned a quarter
/// when `/Rotate` is 90 or 270 — the shape the operator sees.
#[must_use]
pub fn displayed_size(page: &pdfcer_core::page_tree::Page) -> (f64, f64) {
    let (w, h) = (page.crop_box.width(), page.crop_box.height());
    if page.rotate % 180 == 90 {
        (h, w)
    } else {
        (w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default is one sheet the size of the current page, after it.
    #[test]
    fn it_opens_on_one_sheet_like_the_current_page_after_it() {
        let d = InsertBlankDialog::open(3, (1683.8, 2383.94));
        assert_eq!(d.sheet_pt(), (1683.8, 2383.94));
        assert_eq!(d.count, 1);
        assert_eq!(
            d.position.position(d.current_page),
            pdfcer_core::pageops::InsertPosition::After(3)
        );
    }

    /// A standard size is the engine's number, turned by Landscape; the
    /// current page's size ignores the orientation pair.
    #[test]
    fn landscape_turns_a_standard_size_and_not_the_current_pages() {
        let mut d = InsertBlankDialog::open(0, (100.0, 200.0));
        d.landscape = true;
        assert_eq!(d.sheet_pt(), (100.0, 200.0));
        d.choice = Choice::Standard(PaperSize::A4);
        let (w, h) = d.sheet_pt();
        let (pw, ph) = PaperSize::A4.size_pt();
        assert!((w - ph).abs() < 0.01 && (h - pw).abs() < 0.01, "{w} x {h}");
    }
}
