//! # `text::ocrmodels` — the copy for Recognise text's model list
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/catalog.md`.

use crate::ocr::catalog::{Choice, Unrunnable};

/// The label before the model list.
#[must_use]
pub const fn heading() -> &'static str {
    "Model"
}

/// The list's text when nothing is chosen.
#[must_use]
pub const fn none_chosen() -> &'static str {
    "Choose a model"
}

/// A model's name as the list shows it: its manifest label, else the
/// recogniser's name, else the model's own name.
#[must_use]
pub fn label(choice: &Choice) -> String {
    choice.label.clone().unwrap_or_else(|| {
        choice.engine.map_or_else(
            || choice.name.clone(),
            |e| super::ocr::engine_label(e).to_owned(),
        )
    })
}

/// A model this build cannot run, as the list shows it.
#[must_use]
pub fn unrunnable_label(choice: &Choice) -> String {
    format!("{} (cannot run here)", label(choice))
}

/// Why a model cannot run, as a clause.
#[must_use]
pub fn why(why: &Unrunnable) -> String {
    match why {
        Unrunnable::NotInBuild(engine) => format!(
            "this build does not include the {} recogniser",
            super::ocr::engine_label(*engine)
        ),
        Unrunnable::MissingFiles(files) => {
            format!("its folder is missing {}", files.join(", "))
        }
        Unrunnable::NoVlRunner => {
            "pdfcer cannot run PaddleOCR-VL models yet; support is waiting on the engine".to_owned()
        }
        Unrunnable::Program => {
            "it is a program of its own, and this build cannot run add-on programs".to_owned()
        }
        Unrunnable::UnknownEngine(token) => format!("pdfcer does not know the engine {token}"),
    }
}

/// A model's hover: where it is, and why it cannot run if it cannot.
#[must_use]
pub fn hover(choice: &Choice) -> String {
    let folder = choice.folder.display();
    match &choice.unrunnable {
        None => format!("From {folder}"),
        Some(w) => format!("From {folder}. It cannot run: {}.", why(w)),
    }
}

/// The remembered model is in no searched folder.
#[must_use]
pub fn remembered_missing(name: &str) -> String {
    format!(
        "The model you used last, {name}, was not found. Choose another, or add its folder \
         under Settings, OCR models."
    )
}

/// The remembered model is found but cannot run.
#[must_use]
pub fn remembered_unrunnable(name: &str, reason: &Unrunnable) -> String {
    format!(
        "The model you used last, {name}, cannot run: {}. Choose another.",
        why(reason)
    )
}

/// The Run button's hover while no model is chosen.
#[must_use]
pub const fn choose_first() -> &'static str {
    "Choose a model first."
}
