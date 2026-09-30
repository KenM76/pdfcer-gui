//! # `dialogs::labels` — **Number pages**
//!
//! `pages.labels`, Pages ▸ Stamp ▸ Number pages…. A page range, a style, a
//! prefix and a start number become one `set_page_labels` call; *Remove all
//! labels* is `clear_page_labels`. The window lists the ranges the document
//! stores now and previews the labels the range will get.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/labels.md`.

use std::num::NonZeroU32;

use egui::Ui;
use pdfcer_core::page_labels::{LabelFormat, LabelRange, LabelStyle};

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::text::labels as t;

/// The dialog body's published region, for `ui-verify`.
pub const REGION_BODY: &str = "labels.body"; // ui-text-exempt: trace region name, never displayed
/// The first-page box.
pub const REGION_FROM: &str = "labels.from"; // ui-text-exempt: trace region name, never displayed
/// The last-page box.
pub const REGION_TO: &str = "labels.to"; // ui-text-exempt: trace region name, never displayed
/// One style choice; the suffix is its index in [`STYLES`].
pub const REGION_STYLE_PREFIX: &str = "labels.style."; // ui-text-exempt: trace region name, never displayed
/// The prefix box.
pub const REGION_PREFIX: &str = "labels.prefix"; // ui-text-exempt: trace region name, never displayed
/// The start box.
pub const REGION_START: &str = "labels.start"; // ui-text-exempt: trace region name, never displayed
/// The commit button.
pub const REGION_APPLY: &str = "labels.apply"; // ui-text-exempt: trace region name, never displayed
/// The clear button.
pub const REGION_CLEAR: &str = "labels.clear"; // ui-text-exempt: trace region name, never displayed

/// The styles offered, in the order drawn.
pub const STYLES: [LabelStyle; 6] = [
    LabelStyle::Decimal,
    LabelStyle::LowerRoman,
    LabelStyle::UpperRoman,
    LabelStyle::LowerLetters,
    LabelStyle::UpperLetters,
    LabelStyle::PrefixOnly,
];

/// The most labels the preview computes.
const PREVIEW_MAX: usize = 3;

/// The Number pages window's state.
pub struct LabelsDialog {
    /// The document's page count.
    of: usize,
    /// The ranges the document stored when the window opened or last applied.
    current: Vec<LabelRange>,
    /// First page, 1-based as typed.
    from: u32,
    /// Last page, 1-based as typed.
    to: u32,
    style: LabelStyle,
    prefix: String,
    start: u32,
    apply_requested: bool,
    clear_requested: bool,
    close_requested: bool,
}

impl LabelsDialog {
    /// Open on a document of `of` pages whose stored ranges are `current`.
    /// The range defaults to the rail's pick when it spans two or more pages,
    /// else to every page.
    #[must_use]
    pub fn open(picked: &[usize], of: usize, current: Vec<LabelRange>) -> Option<Self> {
        if of == 0 {
            return None;
        }
        let (first, last) = match (picked.iter().min(), picked.iter().max()) {
            (Some(&a), Some(&b)) if a < b && b < of => (a, b),
            _ => (0, of - 1),
        };
        // Seed the form with the range that already covers the first page.
        let seed = current
            .iter()
            .rev()
            .find(|r| r.first_page <= first)
            .map(|r| r.format.clone())
            .unwrap_or_else(|| LabelFormat::new(LabelStyle::Decimal));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "labels-opened of={of} ranges={} first={first} last={last}",
                current.len()
            )
        });
        Some(Self {
            of,
            current,
            from: to_u32(first + 1),
            to: to_u32(last + 1),
            style: seed.style,
            prefix: seed.prefix,
            start: seed.start.get(),
            apply_requested: false,
            clear_requested: false,
            close_requested: false,
        })
    }

    /// The 0-based range typed, when it is a valid one.
    fn range(&self) -> Option<(usize, usize)> {
        let (from, to) = (self.from as usize, self.to as usize);
        (1 <= from && from <= to && to <= self.of).then(|| (from - 1, to - 1))
    }

    /// The format the form describes.
    fn format(&self) -> LabelFormat {
        LabelFormat::new(self.style)
            .with_prefix(self.prefix.clone())
            .with_start(NonZeroU32::new(self.start).unwrap_or(NonZeroU32::MIN))
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "page-labels", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 440.0),
            egui::vec2(360.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.apply_requested)
            && let Some((first, last)) = self.range()
        {
            let format = self.format();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "labels-commit first={first} last={last} style={:?} prefix={:?} start={}",
                    format.style, format.prefix, format.start
                )
            });
            actions.push(Action::Page(PageAction::SetLabels {
                first,
                last,
                format,
            }));
            return false;
        }
        if std::mem::take(&mut self.clear_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "labels-clear-requested".to_owned()
            });
            actions.push(Action::Page(PageAction::ClearLabels));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        if self.current.is_empty() {
            ui.small(t::none_yet());
        } else {
            ui.small(t::current_heading());
            for (i, range) in self.current.iter().enumerate() {
                let next = self.current.get(i + 1).map(|r| r.first_page);
                ui.small(t::range_line(range, next, self.of));
            }
        }
        ui.add_space(8.0);

        let max = to_u32(self.of);
        ui.horizontal(|ui| {
            ui.label(t::from_page());
            let from = ui.add(egui::DragValue::new(&mut self.from).range(1..=max));
            crate::diag::ui_rect_visible(REGION_FROM, from.rect, ui.clip_rect());
            ui.label(t::to_page());
            let to = ui.add(egui::DragValue::new(&mut self.to).range(1..=max));
            crate::diag::ui_rect_visible(REGION_TO, to.rect, ui.clip_rect());
            ui.label(t::of_pages(self.of));
        });

        ui.add_space(4.0);
        ui.label(t::style_label());
        for (i, style) in STYLES.into_iter().enumerate() {
            let choice = ui.radio_value(&mut self.style, style, t::style_name(style));
            crate::diag::ui_rect_visible(
                // ui-text-exempt: trace region name, never displayed
                &format!("{REGION_STYLE_PREFIX}{i}"),
                choice.rect,
                ui.clip_rect(),
            );
        }

        ui.add_space(4.0);
        egui::Grid::new("labels-form") // ui-text-exempt: widget id, never displayed
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(t::prefix_label());
                let prefix = ui
                    .add(
                        // escape-disposition: dialog-cancels — `dialogs::host` owns the key.
                        egui::TextEdit::singleline(&mut self.prefix).desired_width(120.0),
                    )
                    .on_hover_text(t::prefix_tooltip());
                crate::diag::ui_rect_visible(REGION_PREFIX, prefix.rect, ui.clip_rect());
                ui.end_row();
                ui.label(t::start_label());
                let start = ui
                    .add_enabled(
                        self.style != LabelStyle::PrefixOnly,
                        egui::DragValue::new(&mut self.start).range(1..=u32::MAX),
                    )
                    .on_hover_text(t::start_tooltip());
                crate::diag::ui_rect_visible(REGION_START, start.rect, ui.clip_rect());
                ui.end_row();
            });

        ui.add_space(8.0);
        let range = self.range();
        match range {
            Some((first, last)) => {
                let format = self.format();
                let n = last - first + 1;
                let mut shown: Vec<String> = (0..n.min(PREVIEW_MAX - 1))
                    .map(|offset| format.label(offset))
                    .collect();
                if n >= PREVIEW_MAX {
                    shown.push(format.label(n - 1));
                }
                ui.label(t::preview(first, last, &shown));
            }
            None => {
                ui.label(t::bad_range(self.of));
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| {
            let apply = ui.add_enabled(range.is_some(), egui::Button::new(t::apply_button()));
            crate::diag::ui_rect_visible(REGION_APPLY, apply.rect, ui.clip_rect());
            if apply.clicked() {
                self.apply_requested = true;
            }
            if !self.current.is_empty() {
                let clear = ui
                    .button(t::clear_button())
                    .on_hover_text(t::clear_tooltip());
                crate::diag::ui_rect_visible(REGION_CLEAR, clear.rect, ui.clip_rect());
                if clear.clicked() {
                    self.clear_requested = true;
                }
            }
            if ui.button(t::close_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pick_of_two_or_more_pages_seeds_the_range_else_every_page() {
        let d = LabelsDialog::open(&[3, 1], 10, Vec::new()).expect("opens");
        assert_eq!(d.range(), Some((1, 3)));
        let d = LabelsDialog::open(&[4], 10, Vec::new()).expect("opens");
        assert_eq!(d.range(), Some((0, 9)));
        assert!(LabelsDialog::open(&[], 0, Vec::new()).is_none());
    }

    #[test]
    fn a_back_to_front_range_is_not_applied() {
        let mut d = LabelsDialog::open(&[], 10, Vec::new()).expect("opens");
        d.from = 6;
        d.to = 2;
        assert_eq!(d.range(), None);
        d.to = 11;
        d.from = 1;
        assert_eq!(d.range(), None);
    }

    #[test]
    fn the_form_describes_prefix_style_and_start() {
        let mut d = LabelsDialog::open(&[], 4, Vec::new()).expect("opens");
        d.style = LabelStyle::LowerRoman;
        d.prefix = "A-".to_owned();
        d.start = 2;
        assert_eq!(d.format().label(0), "A-ii");
        assert_eq!(d.format().label(2), "A-iv");
    }
}
