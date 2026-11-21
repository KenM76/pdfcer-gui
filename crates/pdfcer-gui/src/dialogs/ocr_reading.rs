//! # `dialogs::ocr_reading` — how the recogniser reads: word lists and layout
//!
//! The Recognise-text dialog's second group. It holds the operator's choice of
//! word lists ([`pdfcer_ocr_host::Dictionaries`]) and whether a vision model
//! reads the page by layout, and hands both to `ocr::Request`.
//!
//! R9: a choice the chosen model cannot take renders nothing, and is not sent.
//! Which choices it takes is asked of the engine, one choice at a time
//! ([`crate::ocr::reading_offer`], traced as `ocr-reading-offer`); a model that
//! takes none of them draws no group. A choice made for one model and hidden by
//! switching to another stays held for switching back. The whole choice is
//! asked again ([`crate::ocr::check_reading`]) and a refusal is shown in the
//! engine's sentence under the group, before Run.

use std::path::PathBuf;

use crate::ocr::{Dictionaries, ReadingOffer};
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
/// The engine's refusal of the whole choice.
const REGION_REFUSED: &str = "ocr-reading-refused"; // ui-text-exempt: trace region name
/// Answers the word-file picker in driven runs.
const DIAG_WORDS_PATH: &str = "PDFCER_DIAG_WORDS_PATH"; // ui-text-exempt: an environment variable name

/// One question put to the engine: the model's folder and name, the word
/// lists and whether to read by layout.
type Asked = (PathBuf, String, Dictionaries, bool);

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
    /// The engine's answer for the last model drawn, keyed by its folder and
    /// name, so the probe runs once per model rather than every frame.
    offer: Option<(PathBuf, String, ReadingOffer)>,
    /// The engine's answer to the whole choice last asked, with what was asked.
    verdict: Option<(Asked, Result<(), String>)>,
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            builtin: true,
            user_words: Vec::new(),
            by_layout: false,
            offer: None,
            verdict: None,
        }
    }
}

impl Reading {
    /// The word lists the run asks of `model`: only the choices it was shown.
    #[must_use]
    pub(super) fn dictionaries(&self, model: &OcrModel) -> Dictionaries {
        let offer = self.offer_of(model);
        let base = if self.builtin || !offer.drop_lists {
            Dictionaries::builtin()
        } else {
            Dictionaries::none()
        };
        if !offer.word_files {
            return base;
        }
        self.user_words
            .iter()
            .fold(base, |d, p| d.with_user_words(p.clone()))
    }

    /// Whether the run reads by layout: only when asked AND the model can.
    #[must_use]
    pub(super) fn by_layout(&self, model: &OcrModel) -> bool {
        self.by_layout && self.offer_of(model).layout
    }

    /// The engine's answer for `model`: the held one when it is for `model`.
    fn offer_of(&self, model: &OcrModel) -> ReadingOffer {
        match &self.offer {
            Some((folder, name, offer)) if *folder == model.folder && *name == model.name => *offer,
            _ => crate::ocr::reading_offer(model),
        }
    }

    /// Ask the engine afresh when the model changed, and trace its answer.
    fn refresh_offer(&mut self, model: &OcrModel) -> ReadingOffer {
        let held = self
            .offer
            .as_ref()
            .is_some_and(|(f, n, _)| *f == model.folder && *n == model.name);
        if !held {
            let offer = crate::ocr::reading_offer(model);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "ocr-reading-offer engine={} model={} drop-lists={} word-files={} layout={} source=check_options",
                    model.engine, model.name, offer.drop_lists, offer.word_files, offer.layout
                )
            });
            self.offer = Some((model.folder.clone(), model.name.clone(), offer));
        }
        self.offer_of(model)
    }

    /// Draw the group for the chosen model.
    pub(super) fn show(&mut self, ui: &mut egui::Ui, model: Option<&OcrModel>) {
        let Some(model) = model else { return };
        let offer = self.refresh_offer(model);
        let (drop, files) = (offer.drop_lists, offer.word_files);
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
        if offer.layout {
            let layout = ui
                .checkbox(&mut self.by_layout, t::by_layout())
                .on_hover_text(t::by_layout_tooltip());
            crate::diag::ui_rect(REGION_LAYOUT, layout.rect);
        }
        if let Err(why) = self.verdict(model) {
            let refusal = ui.colored_label(ui.visuals().warn_fg_color, why);
            crate::diag::ui_rect(REGION_REFUSED, refusal.rect);
        }
    }

    /// The engine's answer to the whole choice, asked again only when the
    /// model or the choice changed.
    fn verdict(&mut self, model: &OcrModel) -> Result<(), String> {
        let asked = (
            model.folder.clone(),
            model.name.clone(),
            self.dictionaries(model),
            self.by_layout(model),
        );
        match &self.verdict {
            Some((held, answer)) if *held == asked => answer.clone(),
            _ => {
                let answer = crate::ocr::check_reading(model, &asked.2, asked.3);
                self.verdict = Some((asked, answer.clone()));
                answer
            }
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
            crate::filedialog::FileDialog::new()
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
            offer: None,
            verdict: None,
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
