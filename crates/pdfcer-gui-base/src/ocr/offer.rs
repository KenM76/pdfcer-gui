//! # `ocr::offer` — which reading choices a model takes, asked of the engine
//!
//! The Recognise-text dialog draws a reading choice only when the model can
//! honour it (R9). Each choice is probed alone through
//! [`pdfcer_ocr_host::check_options`], so the shell restates no engine rule:
//!
//! - *No word lists*: built-in lists off, refused only by
//!   `RunnerError::DictionariesUnsupported`.
//! - word files: one placeholder word file, refused only by
//!   `DictionariesUnsupported`. A program add-on answers with a file error for
//!   the placeholder, which means it takes word files.
//! - *Read by layout*: accepted outright.
//!
//! [`check_reading`] asks the same question of the operator's whole choice,
//! and its refusal is the engine's sentence.

use pdfcer_core::ocr::addons::OcrModel;
use pdfcer_ocr_host::{Dictionaries, RunOptions, RunnerError, check_options};

/// A word-file path the probe names; never read for an in-process engine.
const PROBE_WORDS: &str = "probe-words.txt"; // ui-text-exempt: a probe path, never displayed

/// Which reading choices `check_options` accepts for one model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReadingOffer {
    /// The built-in word lists can be turned off.
    pub drop_lists: bool,
    /// The model takes the operator's word files.
    pub word_files: bool,
    /// The model can read the page by layout regions.
    pub layout: bool,
}

/// Probe each reading choice for `model`, one at a time.
#[must_use]
pub fn reading_offer(model: &OcrModel) -> ReadingOffer {
    let takes = |dictionaries: Dictionaries| {
        !matches!(
            check_options(model, &options(model, dictionaries, false)),
            Err(RunnerError::DictionariesUnsupported { .. })
        )
    };
    ReadingOffer {
        drop_lists: takes(Dictionaries::none()),
        word_files: takes(Dictionaries::builtin().with_user_words(PROBE_WORDS)),
        layout: check_options(model, &options(model, Dictionaries::builtin(), true)).is_ok(),
    }
}

/// Whether `model` accepts the whole choice, or the engine's sentence.
///
/// # Errors
///
/// The engine's refusal, as its `Display` text.
pub fn check_reading(
    model: &OcrModel,
    dictionaries: &Dictionaries,
    by_layout: bool,
) -> Result<(), String> {
    check_options(model, &options(model, dictionaries.clone(), by_layout))
        .map_err(|e| e.to_string())
}

/// The run options the worker would build, minus resolution and upscale.
fn options(model: &OcrModel, dictionaries: Dictionaries, layout: bool) -> RunOptions {
    RunOptions::new(super::languages(model), super::MAX_DPI)
        .with_dictionaries(dictionaries)
        .with_layout(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn model(engine: &str) -> OcrModel {
        OcrModel {
            name: engine.to_owned(),
            engine: engine.to_owned(),
            folder: PathBuf::new(),
            root: PathBuf::new(),
            manifest: None,
        }
    }

    #[test]
    fn an_in_process_engine_drops_its_lists_and_takes_no_word_file() {
        let offer = reading_offer(&model("ocrs"));
        assert!(offer.drop_lists);
        assert!(!offer.word_files);
        assert!(!offer.layout);
    }

    #[test]
    fn the_vision_model_keeps_its_lists_and_needs_the_layout_file() {
        let offer = reading_offer(&model("paddle-vl"));
        assert!(!offer.drop_lists);
        assert!(!offer.word_files);
        #[cfg(feature = "ocr-vl")]
        assert!(!offer.layout);
    }

    #[test]
    fn a_refused_whole_choice_carries_the_engine_sentence() {
        let words = Dictionaries::builtin().with_user_words("a.txt");
        let why = check_reading(&model("ocrs"), &words, false).unwrap_err();
        assert!(!why.is_empty());
    }
}
