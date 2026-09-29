//! # `dialogs::bates` — **Bates-number a document**
//!
//! `pages.bates`, Pages ▸ Stamp ▸ Bates numbering…. Prefix, digits, suffix,
//! start number, one of six positions, margin and text size become one
//! `BatesStamp` over every page, or over the page rail's pick. The form
//! outlives the window: on a stamp its start number advances past the last
//! label, so the next document of a batch opens the window continuing the run.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/bates.md`.

use egui::Ui;
use pdfcer_core::bates::{BatesNumbering, BatesPosition, BatesStamp};
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::text::bates as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "bates.body";
/// The prefix box.
const REGION_PREFIX: &str = "bates.prefix";
/// The start-number box.
const REGION_START: &str = "bates.start";
/// One position choice; the suffix is its index in `BatesPosition::ALL`.
const REGION_POSITION_PREFIX: &str = "bates.position.";
/// The commit button.
const REGION_STAMP: &str = "bates.stamp";
/// The two scope choices.
const REGION_SCOPE_ALL: &str = "bates.scope.all";
const REGION_SCOPE_PICKED: &str = "bates.scope.picked";

/// What the operator typed, kept between openings of the window.
#[derive(Debug, Clone, PartialEq)]
pub struct BatesForm {
    prefix: String,
    suffix: String,
    digits: u8,
    start: u64,
    position: BatesPosition,
    margin_mm: f64,
    font_size: f64,
}

impl Default for BatesForm {
    /// The engine's defaults, numbering from 1.
    fn default() -> Self {
        let d = BatesStamp::default();
        Self {
            prefix: d.numbering.prefix.clone(),
            suffix: d.numbering.suffix.clone(),
            digits: d.numbering.digits,
            start: 1,
            position: d.position,
            margin_mm: (pdfcer_gui_base::units::mm_from_points(d.margin) * 10.0).round() / 10.0,
            font_size: d.font_size,
        }
    }
}

impl BatesForm {
    /// The numbering the prefix, digits and suffix describe.
    fn numbering(&self) -> BatesNumbering {
        BatesNumbering::new(self.prefix.clone(), self.digits, self.suffix.clone())
    }

    /// The stamp over `pages`, or the reason it cannot be made, checked the
    /// way the engine checks it before writing.
    fn stamp(&self, pages: &[usize]) -> Result<(BatesStamp, String, String), String> {
        let numbering = self.numbering();
        numbering.check().map_err(|e| t::problem(&e))?;
        let n = u64::try_from(pages.len()).unwrap_or(u64::MAX);
        if n == 0 {
            return Err(t::problem(&pdfcer_core::bates::BatesError::NoPages));
        }
        let last = self.start.saturating_add(n - 1);
        let first_label = numbering.label(self.start).map_err(|e| t::problem(&e))?;
        let last_label = numbering.label(last).map_err(|e| t::problem(&e))?;
        let margin = pdfcer_gui_base::units::points_from_mm(self.margin_mm);
        if !(margin >= 0.0 && margin.is_finite() && self.font_size > 0.0) {
            return Err(t::problem(&pdfcer_core::bates::BatesError::Geometry {
                margin: String::new(),
                size: String::new(),
            }));
        }
        let mut stamp = BatesStamp::new(numbering);
        stamp.position = self.position;
        stamp.margin = margin;
        stamp.font_size = self.font_size;
        stamp.pages = Some(pages.to_vec());
        Ok((stamp, first_label, last_label))
    }
}

/// The Bates window's state.
pub struct BatesDialog {
    /// The page rail's pick, 0-based, ascending; possibly empty.
    picked: Vec<usize>,
    /// The document's page count.
    of: usize,
    /// Stamp only `picked` rather than every page.
    only_picked: bool,
    form: BatesForm,
    /// Set by the commit button, consumed after the window closure returns.
    apply_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
    /// Set once a stamp was sent, so [`Self::remembered`] advances the run.
    stamped: bool,
}

impl BatesDialog {
    /// Open the window on a document of `of` pages, seeded with the form the
    /// last window left. A pick of two or more pages is the default scope; a
    /// single picked page is usually only the page being looked at.
    #[must_use]
    pub fn open(picked: &[usize], of: usize, form: Option<&BatesForm>) -> Option<Self> {
        if of == 0 {
            return None;
        }
        let picked: Vec<usize> = if picked.len() < of {
            picked.to_vec()
        } else {
            Vec::new()
        };
        let only_picked = picked.len() >= 2;
        let form = form.cloned().unwrap_or_default();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "bates-opened picked={} of={of} only_picked={} start={} digits={}",
                picked.len(),
                u8::from(only_picked),
                form.start,
                form.digits,
            )
        });
        Some(Self {
            picked,
            of,
            only_picked,
            form,
            apply_requested: false,
            close_requested: false,
            stamped: false,
        })
    }

    /// The form to seed the next window with: after a stamp, numbering
    /// continues from the label after the last one written.
    #[must_use]
    pub fn remembered(&self) -> BatesForm {
        let mut form = self.form.clone();
        if self.stamped {
            let n = u64::try_from(self.pages().len()).unwrap_or(u64::MAX);
            form.start = form.start.saturating_add(n);
        }
        form
    }

    /// The pages the stamp reaches.
    fn pages(&self) -> Vec<usize> {
        if self.only_picked && !self.picked.is_empty() {
            self.picked.clone()
        } else {
            (0..self.of).collect()
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "bates", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(440.0, 420.0),
            egui::vec2(360.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.apply_requested)
            && let Ok((stamp, first_label, last_label)) = self.form.stamp(&self.pages())
        {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "bates-commit n={} first={} labels={first_label}..{last_label} \
                     position={:?} margin_pt={:.2} size={:.2}",
                    self.pages().len(),
                    self.form.start,
                    stamp.position,
                    stamp.margin,
                    stamp.font_size,
                )
            });
            actions.push(Action::Page(PageAction::StampBates {
                stamp,
                first: self.form.start,
            }));
            self.stamped = true;
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The controls and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(4.0);
        if self.picked.is_empty() {
            ui.label(egui::RichText::new(t::scope_all(self.of)).weak());
        } else {
            let r = ui.radio_value(&mut self.only_picked, false, t::scope_all(self.of));
            crate::diag::ui_rect(REGION_SCOPE_ALL, r.rect);
            let r = ui.radio_value(
                &mut self.only_picked,
                true,
                t::scope_picked(self.picked.len()),
            );
            crate::diag::ui_rect(REGION_SCOPE_PICKED, r.rect);
        }
        ui.add_space(8.0);

        egui::Grid::new("bates.grid") // ui-text-exempt: widget id
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(t::prefix());
                let r = ui.text_edit_singleline(&mut self.form.prefix);
                crate::diag::ui_rect(REGION_PREFIX, r.rect);
                ui.end_row();

                ui.label(t::digits());
                let (w, refusal) = entry::drag_value(ui, &mut self.form.digits, entry::Kind::Count);
                refusal
                    .show(ui.add(w.range(1..=pdfcer_core::bates::MAX_DIGITS)))
                    .on_hover_text(t::digits_tooltip());
                ui.end_row();

                ui.label(t::suffix());
                ui.text_edit_singleline(&mut self.form.suffix);
                ui.end_row();

                ui.label(t::start());
                let (w, refusal) = entry::drag_value(ui, &mut self.form.start, entry::Kind::Count);
                let r = refusal
                    .show(ui.add(w.range(0..=u64::MAX)))
                    .on_hover_text(t::start_tooltip());
                crate::diag::ui_rect(REGION_START, r.rect);
                ui.end_row();

                ui.label(t::margin());
                let (w, refusal) = entry::drag_value(
                    ui,
                    &mut self.form.margin_mm,
                    entry::Kind::Length(entry::LengthUnit::Of(
                        pdfcer_core::dimension::Unit::Millimeter,
                    )),
                );
                refusal.show(ui.add(w.range(0.0..=500.0)));
                ui.end_row();

                ui.label(t::font_size());
                let (w, refusal) =
                    entry::drag_value(ui, &mut self.form.font_size, entry::Kind::Number(&["pt"]));
                refusal.show(ui.add(w.range(1.0..=144.0)));
                ui.end_row();
            });

        ui.add_space(6.0);
        ui.label(t::position_heading());
        egui::Grid::new("bates.positions") // ui-text-exempt: widget id
            .num_columns(3)
            .show(ui, |ui| {
                for (i, p) in BatesPosition::ALL.iter().enumerate() {
                    let r = ui.radio_value(&mut self.form.position, *p, t::position(*p));
                    crate::diag::ui_rect(&format!("{REGION_POSITION_PREFIX}{i}"), r.rect);
                    if i % 3 == 2 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(6.0);
        let verdict = self.form.stamp(&self.pages());
        match &verdict {
            Ok((_, first, last)) => {
                ui.label(t::preview(first, last));
            }
            Err(why) => {
                ui.label(egui::RichText::new(why).color(ui.visuals().error_fg_color));
            }
        }
        ui.label(egui::RichText::new(t::permanence()).small().weak());

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // Absent, not greyed, when the numbering cannot be stamped: the
            // line above already says why (R9).
            if verdict.is_ok() {
                let stamp = ui.button(t::stamp());
                crate::diag::ui_rect(REGION_STAMP, stamp.rect);
                if stamp.clicked() {
                    self.apply_requested = true;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(prefix: &str, digits: u8, start: u64) -> BatesForm {
        BatesForm {
            prefix: prefix.to_owned(),
            digits,
            start,
            ..BatesForm::default()
        }
    }

    /// The preview labels are the first and last page's, zero-padded.
    #[test]
    fn labels_span_the_operand_pages() {
        let (stamp, first, last) = form("ABC", 4, 7).stamp(&[0, 2, 5]).unwrap();
        assert_eq!((first.as_str(), last.as_str()), ("ABC0007", "ABC0009"));
        assert_eq!(stamp.pages, Some(vec![0, 2, 5]));
    }

    /// A run that outgrows its digits is refused before anything is sent.
    #[test]
    fn a_run_past_the_digit_count_is_refused() {
        let why = form("", 2, 98).stamp(&[0, 1, 2]).unwrap_err();
        assert!(why.contains("100"), "{why}");
    }

    /// After a stamp the next window continues the run.
    #[test]
    fn a_stamp_advances_the_remembered_start() {
        let mut d = BatesDialog::open(&[], 3, Some(&form("X", 3, 10))).unwrap();
        assert_eq!(d.remembered().start, 10);
        d.stamped = true;
        assert_eq!(d.remembered().start, 13);
    }

    /// Nothing picked, or one page picked, stamps the whole document; a pick
    /// of two or more is the default scope and can be widened to all.
    #[test]
    fn the_scope_defaults_to_every_page_unless_several_are_picked() {
        assert_eq!(
            BatesDialog::open(&[], 4, None).unwrap().pages(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            BatesDialog::open(&[2], 4, None).unwrap().pages(),
            vec![0, 1, 2, 3]
        );
        let mut d = BatesDialog::open(&[1, 3], 4, None).unwrap();
        assert_eq!(d.pages(), vec![1, 3]);
        d.only_picked = false;
        assert_eq!(d.pages(), vec![0, 1, 2, 3]);
    }
}
