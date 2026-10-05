//! # `text::ocrmodels` — the copy for Recognise text's model list
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/catalog.md`.

use pdfcer_core::ocr::addon_manifest::AddonKind;

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
/// recogniser's name, else the model's own name; a program add-on also names
/// the program it starts.
#[must_use]
pub fn label(choice: &Choice) -> String {
    let base = choice.label.clone().unwrap_or_else(|| {
        choice
            .engine
            .map(super::ocr::engine_label)
            .or_else(|| super::ocr::token_label(&choice.engine_token))
            .map_or_else(|| choice.name.clone(), str::to_owned)
    });
    match choice.model.program() {
        Some(program) if choice.model.kind() == AddonKind::Program => {
            format!("{base} (runs {program})")
        }
        _ => base,
    }
}

/// A model this build cannot run, as the list shows it.
#[must_use]
pub fn unrunnable_label(choice: &Choice) -> String {
    format!("{} (cannot run here)", label(choice))
}

/// Why a model cannot run: the engine's own sentence, without its stop.
#[must_use]
pub fn why(why: &Unrunnable) -> String {
    why.reason().trim_end_matches('.').to_owned()
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
