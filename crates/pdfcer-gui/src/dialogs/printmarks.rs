//! # `dialogs::printmarks` — **Printer's marks**
//!
//! `file.printer_marks`, File ▸ Print ▸ Printer's marks…. Five kinds of mark,
//! their gap, length and line width, and the page-information label become
//! one `PrinterMarks` over every page or the page rail's pick. The engine
//! grows each sheet past its trim box and draws the marks in the new band as
//! one undo step; the receipt says how far the sheets grew.
//!
//! The scope rule is Bates numbering's (`dialogs::bates`).

use egui::Ui;
use pdfcer_core::printer_marks::PrinterMarks;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::text::printmarks as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "printmarks.body";
/// The commit button.
const REGION_ADD: &str = "printmarks.add";
/// The two scope choices.
const REGION_SCOPE_ALL: &str = "printmarks.scope.all";
const REGION_SCOPE_PICKED: &str = "printmarks.scope.picked";

/// What the operator chose, kept between openings of the window. The label
/// is not kept: it names the document the window opened on.
#[derive(Debug, Clone, PartialEq)]
pub struct MarksForm {
    trim: bool,
    bleed: bool,
    registration: bool,
    color_bars: bool,
    page_info: bool,
    offset: f64,
    length: f64,
    line_width: f64,
}

impl Default for MarksForm {
    /// The engine's defaults: every mark on.
    fn default() -> Self {
        let d = PrinterMarks::all(String::new());
        Self {
            trim: d.trim,
            bleed: d.bleed,
            registration: d.registration,
            color_bars: d.color_bars,
            page_info: d.page_info,
            offset: d.offset,
            length: d.length,
            line_width: d.line_width,
        }
    }
}

impl MarksForm {
    /// The marks over `pages` with `label`, or the reason they cannot be
    /// drawn, checked the way the engine checks them before writing.
    fn marks(&self, label: &str, pages: &[usize]) -> Result<PrinterMarks, String> {
        let mut m = PrinterMarks::none(label);
        m.trim = self.trim;
        m.bleed = self.bleed;
        m.registration = self.registration;
        m.color_bars = self.color_bars;
        m.page_info = self.page_info;
        m.offset = self.offset;
        m.length = self.length;
        m.line_width = self.line_width;
        m.check().map_err(|e| t::problem(&e))?;
        if pages.is_empty() {
            return Err(t::problem(
                &pdfcer_core::printer_marks::PrinterMarksError::NoPages,
            ));
        }
        m.pages = Some(pages.to_vec());
        Ok(m)
    }
}

/// The Printer's marks window's state.
pub struct MarksDialog {
    /// The page rail's pick, 0-based, ascending; possibly empty.
    picked: Vec<usize>,
    /// The document's page count.
    of: usize,
    /// Mark only `picked` rather than every page.
    only_picked: bool,
    form: MarksForm,
    /// The page-information line's first words; seeded with the file name.
    label: String,
    /// Set by the commit button, consumed after the window closure returns.
    apply_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
}

impl MarksDialog {
    /// Open the window on a document of `of` pages named `label`, seeded
    /// with the form the last window left.
    #[must_use]
    pub fn open(
        picked: &[usize],
        of: usize,
        label: &str,
        form: Option<&MarksForm>,
    ) -> Option<Self> {
        if of == 0 {
            return None;
        }
        let picked: Vec<usize> = if picked.len() < of {
            picked.to_vec()
        } else {
            Vec::new()
        };
        let only_picked = picked.len() >= 2;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "printmarks-opened picked={} of={of} only_picked={}",
                picked.len(),
                u8::from(only_picked)
            )
        });
        Some(Self {
            picked,
            of,
            only_picked,
            form: form.cloned().unwrap_or_default(),
            label: label.to_owned(),
            apply_requested: false,
            close_requested: false,
        })
    }

    /// The form to seed the next window with.
    #[must_use]
    pub fn remembered(&self) -> MarksForm {
        self.form.clone()
    }

    /// The pages the marks reach.
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
            "printmarks", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(420.0, 440.0),
            egui::vec2(340.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        if std::mem::take(&mut self.apply_requested)
            && let Ok(marks) = self.form.marks(&self.label, &self.pages())
        {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "printmarks-commit n={} trim={} bleed={} registration={} bars={} info={} \
                     offset={:.2} length={:.2} width={:.2}",
                    self.pages().len(),
                    u8::from(marks.trim),
                    u8::from(marks.bleed),
                    u8::from(marks.registration),
                    u8::from(marks.color_bars),
                    u8::from(marks.page_info),
                    marks.offset,
                    marks.length,
                    marks.line_width,
                )
            });
            actions.push(Action::Page(PageAction::AddPrinterMarks { marks }));
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
        self.controls(ui);
        ui.add_space(6.0);
        let verdict = self.form.marks(&self.label, &self.pages());
        if let Err(why) = &verdict {
            ui.label(egui::RichText::new(why).color(ui.visuals().error_fg_color));
        }
        ui.label(egui::RichText::new(t::permanence()).small().weak());
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // Absent, not greyed, when the marks cannot be drawn: the line
            // above already says why (R9).
            if verdict.is_ok() {
                let add = ui.button(t::add());
                crate::diag::ui_rect(REGION_ADD, add.rect);
                if add.clicked() {
                    self.apply_requested = true;
                }
            }
        });
    }

    /// The five toggles, the label and the three sizes.
    fn controls(&mut self, ui: &mut Ui) {
        let form = &mut self.form;
        for (i, (on, name)) in [
            (&mut form.trim, t::trim()),
            (&mut form.bleed, t::bleed()),
            (&mut form.registration, t::registration()),
            (&mut form.color_bars, t::color_bars()),
        ]
        .into_iter()
        .enumerate()
        {
            let r = ui.checkbox(on, name);
            crate::diag::ui_rect(&format!("printmarks.mark.{i}"), r.rect); // ui-text-exempt: region id
        }
        let r = ui
            .checkbox(&mut form.page_info, t::page_info())
            .on_hover_text(t::page_info_tooltip());
        crate::diag::ui_rect("printmarks.mark.4", r.rect); // ui-text-exempt: region id
        ui.add_space(6.0);
        egui::Grid::new("printmarks.grid") // ui-text-exempt: widget id
            .num_columns(2)
            .show(ui, |ui| {
                if form.page_info {
                    ui.label(t::label());
                    ui.text_edit_singleline(&mut self.label);
                    ui.end_row();
                }
                for (value, name) in [
                    (&mut form.offset, t::offset()),
                    (&mut form.length, t::length()),
                    (&mut form.line_width, t::line_width()),
                ] {
                    ui.label(name);
                    let (w, refusal) = entry::drag_value(ui, value, entry::Kind::Number(&["pt"]));
                    refusal.show(ui.add(w.range(0.0..=144.0)));
                    ui.end_row();
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default form asks for every mark at the engine's sizes.
    #[test]
    fn the_default_is_every_mark() {
        let m = MarksForm::default().marks("job", &[0]).unwrap();
        assert!(m.trim && m.bleed && m.registration && m.color_bars && m.page_info);
        assert_eq!(m.pages, Some(vec![0]));
    }

    /// No marks, or a zero line width, is refused here, before anything is
    /// sent.
    #[test]
    fn undrawable_marks_are_refused_before_the_engine() {
        let none = MarksForm {
            trim: false,
            bleed: false,
            registration: false,
            color_bars: false,
            page_info: false,
            ..MarksForm::default()
        };
        assert!(none.marks("", &[0]).is_err());
        let thin = MarksForm {
            line_width: 0.0,
            ..MarksForm::default()
        };
        assert!(thin.marks("", &[0]).unwrap_err().contains("line width"));
    }
}
