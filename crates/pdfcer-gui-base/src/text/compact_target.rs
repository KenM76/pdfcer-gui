//! # `text::compact_target` — the compact window's starting point and size target
//!
//! Every word of the *Start from* list and the *Fit under* switch in Save a
//! compacted copy, and of the lines reporting the size search.

use pdfcer_core::optimize::Preset;
use pdfcer_core::optimize::budget::BudgetAttempt;

use super::compact::bytes;

/// The starting-point row's label.
#[must_use]
pub const fn preset_label() -> &'static str {
    "Start from"
}

/// A starting point's name.
#[must_use]
pub const fn preset(preset: Preset) -> &'static str {
    match preset {
        Preset::Standard => "Standard",
        Preset::Screen => "Screen",
        Preset::Print => "Print",
        Preset::Archive => "Archive",
        _ => "Lossless",
    }
}

/// Hover on a starting point: what it does to the copy.
#[must_use]
pub const fn preset_tooltip(preset: Preset) -> &'static str {
    match preset {
        Preset::Standard => {
            "The everyday choice. Pictures shown above 225 ppi are lowered to 150 ppi \
             (black-and-white ones above 450 ppi to 300), and page thumbnails are removed. \
             Pictures lose detail."
        }
        Preset::Screen => {
            "The smallest file, for reading on a screen. Pictures shown above 144 ppi are \
             lowered to 96 ppi and stored as JPEG; page thumbnails, private application data \
             and print-only pictures are removed. Pictures lose detail."
        }
        Preset::Print => {
            "Keeps what a printer needs. Only pictures shown above 450 ppi are lowered, to \
             300 ppi (black-and-white ones above 900 ppi to 600); page thumbnails are removed. \
             Pictures lose detail."
        }
        Preset::Archive => {
            "Changes nothing that an archival (PDF/A) file needs. Nothing on the page changes."
        }
        _ => "Only changes that leave every page exactly as it is. Nothing on the page changes.",
    }
}

/// The size-target switch.
#[must_use]
pub const fn target_toggle() -> &'static str {
    "Fit under"
}

/// Hover on the size-target switch.
#[must_use]
pub const fn target_tooltip() -> &'static str {
    "Makes the copy no larger than this. If your other choices are not enough, colour and \
     grey pictures are stored as JPEG at lower and lower quality, then at lower resolution, \
     until the copy fits; black-and-white pictures are never changed for this. The least \
     change that fits is kept. Each try writes the whole file, so this can take a while."
}

/// The unit beside the size box.
#[must_use]
pub const fn target_unit() -> &'static str {
    "MB"
}

/// What one try was, in words.
fn step(attempt: &BudgetAttempt) -> String {
    match (attempt.quality, attempt.ppi) {
        (None, _) => "your choices as they are".to_owned(),
        (Some(q), None) => format!("pictures as JPEG at quality {q}"),
        (Some(q), Some(ppi)) => format!("pictures as JPEG at quality {q} and {ppi} ppi"),
    }
}

/// The search's result.
#[must_use]
pub fn target_result(max: u64, met: bool, chosen: Option<&BudgetAttempt>) -> String {
    match (met, chosen) {
        (true, Some(a)) if a.quality.is_none() => {
            format!("Fits under {} with your choices alone.", bytes(max))
        }
        (true, Some(a)) => format!("Fits under {} with {}.", bytes(max), step(a)),
        (false, Some(a)) => format!(
            "Could not fit under {}. The smallest reached is {}, with {}.",
            bytes(max),
            bytes(a.bytes),
            step(a)
        ),
        (_, None) => format!("Could not fit under {}.", bytes(max)),
    }
}

/// One try of the search.
#[must_use]
pub fn target_attempt(attempt: &BudgetAttempt) -> String {
    format!("Tried {}: {}.", step(attempt), bytes(attempt.bytes))
}

/// The search's failure.
#[must_use]
pub fn target_failed(detail: &str) -> String {
    format!("The size search stopped: {detail}")
}
