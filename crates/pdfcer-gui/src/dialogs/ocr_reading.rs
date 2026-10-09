//! # `dialogs::ocr_reading` — how the recogniser reads: word lists and layout
//!
//! The Recognise-text dialog's second group. It holds the operator's choice of
//! word lists ([`pdfcer_ocr_host::Dictionaries`]) and whether a vision model
//! reads the page by layout, and hands both to `ocr::Request`.
//!
//! R9: a choice the chosen model cannot take renders nothing, and is not sent.
//! "No word lists" is drawn when [`crate::ocr::can_drop_word_lists`], the word
//! files when [`crate::ocr::takes_word_files`], "Read by layout" when
//! [`crate::ocr::reads_by_layout`]; a model with none of them draws no group.
//! A choice made for one model and hidden by switching to another stays held
//! for switching back. Should the engine still refuse, its sentence
//! (`RunnerError::DictionariesUnsupported`) is shown verbatim at Run.

use std::path::PathBuf;

use crate::ocr::Dictionaries;
use pdfcer_core::ocr::addons::OcrModel;

use crate::app::files::{Picked, from_env};
use crate::text::ocr as t;

/// The word-list group, for a driven check.
const REGION_WORDS: &str = "ocr-word-lists"; // ui-text-exempt: trace region name
/// The "no built-in word lists" choice.
const REGION_NO_LISTS: &str = "ocr-no-word-lists"; // ui-text-exempt: trace region name
/// The add-a-word-file control.
const REGION_ADD_WORDS: &str = "ocr-add-words"; // ui-text-exempt: trace region name
/// The read-by-layout checkbox.
const REGION_LAYOUT: &str = "ocr-by-layout"; // ui-text-exempt: trace region name
/// Answers the word-file picker in driven runs.
const DIAG_WORDS_PATH: &str = "PDFCER_DIAG_WORDS_PATH"; // ui-text-exempt: an environment variable name

/// The operator's reading choices for one Recognise-text transaction.
#[derive(Debug, Clone)]
pub(super) struct Reading {
    /// Use the engine's built-in word lists. On by default, which is what
    /// every engine does when not told otherwise.
    builtin: bool,
    /// UTF-8 word files, one word per line, in the order added.
    user_words: Vec<PathBuf>,
    /// Read the page by layout regions (vision model only).
    by_layout: bool,
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            builtin: true,
            user_words: Vec::new(),
            by_layout: false,
        }
    }
}

impl Reading {
    /// The word lists the run asks of `model`: only the choices it was shown.
    #[must_use]
    pub(super) fn dictionaries(&self, model: &OcrModel) -> Dictionaries {
        let base = if self.builtin || !crate::ocr::can_drop_word_lists(model) {
            Dictionaries::builtin()
        } else {
            Dictionaries::none()
        };
        if !crate::ocr::takes_word_files(model) {
            return base;
        }
        self.user_words
            .iter()
            .fold(base, |d, p| d.with_user_words(p.clone()))
    }

    /// Whether the run reads by layout: only when asked AND the model can.
    #[must_use]
    pub(super) fn by_layout(&self, model: &OcrModel) -> bool {
        self.by_layout && crate::ocr::reads_by_layout(model)
    }

    /// Draw the group for the chosen model.
    pub(super) fn show(&mut self, ui: &mut egui::Ui, model: Option<&OcrModel>) {
        let Some(model) = model else { return };
        let drop = crate::ocr::can_drop_word_lists(model);
        let files = crate::ocr::takes_word_files(model);
        if drop || files {
            let group = ui.group(|ui| {
                ui.label(t::word_lists_heading());
                if drop {
                    ui.radio_value(&mut self.builtin, true, t::word_lists_builtin())
                        .on_hover_text(t::word_lists_builtin_tooltip());
                    let none = ui
                        .radio_value(&mut self.builtin, false, t::word_lists_none())
                        .on_hover_text(t::word_lists_none_tooltip());
                    crate::diag::ui_rect(REGION_NO_LISTS, none.rect);
                }
                if files {
                    self.word_files(ui);
                }
            });
            crate::diag::ui_rect(REGION_WORDS, group.response.rect);
        }
        if crate::ocr::reads_by_layout(model) {
            let layout = ui
                .checkbox(&mut self.by_layout, t::by_layout())
                .on_hover_text(t::by_layout_tooltip());
            crate::diag::ui_rect(REGION_LAYOUT, layout.rect);
        }
    }

    /// The added word files, each removable, and the control that adds one.
    fn word_files(&mut self, ui: &mut egui::Ui) {
        let mut remove = None;
        for (i, path) in self.user_words.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(path.display().to_string());
                if ui.small_button(t::remove_word_file()).clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.user_words.remove(i);
        }
        let add = ui
            .button(t::add_word_file())
            .on_hover_text(t::add_word_file_tooltip());
        crate::diag::ui_rect(REGION_ADD_WORDS, add.rect);
        if add.clicked()
            && let Picked::Path(path) = pick_word_list()
        {
            self.user_words.push(path);
        }
    }
}

/// Ask for a word file: the diagnostic seam when set, else the native dialog.
fn pick_word_list() -> Picked {
    let (answer, source) = match from_env(std::env::var_os(DIAG_WORDS_PATH)) {
        Some(answer) => (answer, "env"), // ui-text-exempt: trace token
        None => (
            rfd::FileDialog::new()
                .set_title(t::add_word_file())
                .add_filter(crate::text::files::filter_text(), &["txt"])
                .add_filter(crate::text::files::filter_all(), &["*"])
                .pick_file()
                .map_or(Picked::Cancelled, Picked::Path),
            "native", // ui-text-exempt: trace token
        ),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("ocr-words-picked source={source} answer={answer:?}")
    });
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(engine: &str) -> OcrModel {
        OcrModel {
            name: engine.to_owned(),
            engine: engine.to_owned(),
            folder: PathBuf::new(),
            root: PathBuf::new(),
            manifest: None,
        }
    }

    fn everything_off() -> Reading {
        Reading {
            builtin: false,
            user_words: vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")],
            by_layout: true,
        }
    }

    #[test]
    fn the_default_reads_with_the_built_in_lists_only() {
        let d = Reading::default().dictionaries(&model("ocrs"));
        assert!(d.uses_builtin());
        assert!(d.user_words().is_empty());
    }

    #[test]
    fn an_in_process_engine_is_sent_lists_off_but_no_word_files() {
        let d = everything_off().dictionaries(&model("ocrs"));
        assert!(!d.uses_builtin());
        assert!(d.user_words().is_empty());
    }

    #[test]
    fn the_vision_model_keeps_its_lists_whatever_was_chosen() {
        let d = everything_off().dictionaries(&model("paddle-vl"));
        assert!(d.uses_builtin());
        assert!(d.user_words().is_empty());
    }

    #[test]
    fn layout_is_not_asked_of_a_model_without_the_layout_file() {
        assert!(!everything_off().by_layout(&model("paddle-vl")));
    }
}
