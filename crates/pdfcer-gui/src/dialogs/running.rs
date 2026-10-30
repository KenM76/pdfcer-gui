//! # `dialogs::running` — **Header & footer**
//!
//! `pages.header_footer`, Pages ▸ Stamp ▸ Header & footer…. One line of text
//! with `{page}` and `{pages}` tokens, at the top or bottom edge, left,
//! centred or right, becomes one `RunningText` over every page or over the
//! page rail's pick; the engine writes it as one undo step. The form outlives
//! the window, so the next document opens with the same line.
//!
//! The scope rule is Bates numbering's (`dialogs::bates`): the whole
//! document, unless two or more pages are picked in the rail.

use egui::Ui;
use pdfcer_core::running_text::{RunningText, RunningTextAlign, RunningTextKind};
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::text::running as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "running.body";
/// The text box.
const REGION_TEXT: &str = "running.text";
/// The commit button.
const REGION_ADD: &str = "running.add";
/// The header choice; the footer is `running.kind.footer`.
const REGION_HEADER: &str = "running.kind.header";
const REGION_FOOTER: &str = "running.kind.footer";
/// The two scope choices.
const REGION_SCOPE_ALL: &str = "running.scope.all";
const REGION_SCOPE_PICKED: &str = "running.scope.picked";
/// The three alignments, in this order, publish as `running.align.<i>`.
const ALIGNS: [RunningTextAlign; 3] = [
    RunningTextAlign::Left,
    RunningTextAlign::Center,
    RunningTextAlign::Right,
];

/// What the operator chose, kept between openings of the window.
#[derive(Debug, Clone, PartialEq)]
pub struct RunningForm {
    kind: RunningTextKind,
    align: RunningTextAlign,
    text: String,
    margin_mm: f64,
    font_size: f64,
}

impl Default for RunningForm {
    /// The engine's defaults, with a page-number footer as the text.
    fn default() -> Self {
        let d = RunningText::new(RunningTextKind::Footer, t::default_text());
        Self {
            kind: d.kind,
            align: d.align,
            text: d.text,
            margin_mm: (pdfcer_gui_base::units::mm_from_points(d.margin) * 10.0).round() / 10.0,
            font_size: d.font_size,
        }
    }
}

impl RunningForm {
    /// The line over `pages`, or the reason it cannot be written, checked
    /// the way the engine checks it before writing.
    fn running(&self, pages: &[usize]) -> Result<RunningText, String> {
        let mut r = RunningText::new(self.kind, self.text.clone());
        r.align = self.align;
        r.margin = pdfcer_gui_base::units::points_from_mm(self.margin_mm);
        r.font_size = self.font_size;
        r.check().map_err(|e| t::problem(&e))?;
        if pages.is_empty() {
            return Err(t::problem(
                &pdfcer_core::running_text::RunningTextError::NoPages,
            ));
        }
        r.pages = Some(pages.to_vec());
        Ok(r)
    }
}

/// The Header & footer window's state.
pub struct RunningDialog {
    /// The page rail's pick, 0-based, ascending; possibly empty.
    picked: Vec<usize>,
    /// The document's page count.
    of: usize,
    /// Write only on `picked` rather than every page.
    only_picked: bool,
    form: RunningForm,
    /// Set by the commit button, consumed after the window closure returns.
    apply_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
}

impl RunningDialog {
    /// Open the window on a document of `of` pages, seeded with the form the
    /// last window left.
    #[must_use]
    pub fn open(picked: &[usize], of: usize, form: Option<&RunningForm>) -> Option<Self> {
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
                "running-opened picked={} of={of} only_picked={}",
                picked.len(),
                u8::from(only_picked)
            )
        });
        Some(Self {
            picked,
            of,
            only_picked,
            form: form.cloned().unwrap_or_default(),
            apply_requested: false,
            close_requested: false,
        })
    }

    /// The form to seed the next window with.
    #[must_use]
    pub fn remembered(&self) -> RunningForm {
        self.form.clone()
    }

    /// The pages the line reaches.
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
            "running", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(440.0, 380.0),
            egui::vec2(360.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        if std::mem::take(&mut self.apply_requested)
            && let Ok(stamp) = self.form.running(&self.pages())
        {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "running-commit n={} kind={:?} align={:?} margin_pt={:.2} size={:.2}",
                    self.pages().len(),
                    stamp.kind,
                    stamp.align,
                    stamp.margin,
                    stamp.font_size,
                )
            });
            actions.push(Action::Page(PageAction::StampRunningText { stamp }));
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
        let pages = self.pages();
        let verdict = self.form.running(&pages);
        match &verdict {
            Ok(r) => {
                let first = pages.first().copied().unwrap_or(0) + 1;
                ui.label(t::preview(first, &r.label(first, self.of)));
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
            // Absent, not greyed, when the line cannot be written: the line
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

    /// Edge, alignment, text, margin and size.
    fn controls(&mut self, ui: &mut Ui) {
        let form = &mut self.form;
        egui::Grid::new("running.grid") // ui-text-exempt: widget id
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(t::kind_heading());
                ui.horizontal(|ui| {
                    let r = ui.radio_value(
                        &mut form.kind,
                        RunningTextKind::Header,
                        t::kind(RunningTextKind::Header),
                    );
                    crate::diag::ui_rect(REGION_HEADER, r.rect);
                    let r = ui.radio_value(
                        &mut form.kind,
                        RunningTextKind::Footer,
                        t::kind(RunningTextKind::Footer),
                    );
                    crate::diag::ui_rect(REGION_FOOTER, r.rect);
                });
                ui.end_row();

                ui.label(t::align_heading());
                ui.horizontal(|ui| {
                    for (i, a) in ALIGNS.iter().enumerate() {
                        let r = ui.radio_value(&mut form.align, *a, t::align(*a));
                        crate::diag::ui_rect(&format!("running.align.{i}"), r.rect); // ui-text-exempt: region id
                    }
                });
                ui.end_row();

                ui.label(t::text());
                let r = ui
                    .text_edit_singleline(&mut form.text)
                    .on_hover_text(t::text_tooltip());
                crate::diag::ui_rect(REGION_TEXT, r.rect);
                ui.end_row();

                ui.label(t::margin());
                let (w, refusal) = entry::drag_value(
                    ui,
                    &mut form.margin_mm,
                    entry::Kind::Length(entry::LengthUnit::Of(
                        pdfcer_core::dimension::Unit::Millimeter,
                    )),
                );
                refusal.show(ui.add(w.range(0.0..=500.0)));
                ui.end_row();

                ui.label(t::font_size());
                let (w, refusal) =
                    entry::drag_value(ui, &mut form.font_size, entry::Kind::Number(&["pt"]));
                refusal.show(ui.add(w.range(1.0..=144.0)));
                ui.end_row();
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default form writes "Page 1 of N" at the bottom centre.
    #[test]
    fn the_default_is_a_centred_page_number_footer() {
        let r = RunningForm::default().running(&[0, 1, 2]).unwrap();
        assert_eq!(r.kind, RunningTextKind::Footer);
        assert_eq!(r.align, RunningTextAlign::Center);
        assert_eq!(r.label(1, 3), "Page 1 of 3");
        assert_eq!(r.pages, Some(vec![0, 1, 2]));
    }

    /// Empty text, and a character the font cannot print, are refused here,
    /// before anything is sent.
    #[test]
    fn unwritable_text_is_refused_before_the_engine() {
        let mut f = RunningForm {
            text: "  ".to_owned(),
            ..Default::default()
        };
        assert!(f.running(&[0]).is_err());
        f.text = "\u{4e2d}".to_owned();
        assert!(f.running(&[0]).unwrap_err().contains('\u{4e2d}'));
    }

    /// A pick of two or more pages is the default scope; fewer is the whole
    /// document.
    #[test]
    fn the_scope_follows_the_bates_rule() {
        assert_eq!(
            RunningDialog::open(&[2], 4, None).unwrap().pages(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            RunningDialog::open(&[1, 3], 4, None).unwrap().pages(),
            vec![1, 3]
        );
    }
}
