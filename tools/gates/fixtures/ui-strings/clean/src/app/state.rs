//! FIXTURE — a NESTED module of the CLEAN ui-strings fixture.
//!
//! Two levels down (`src/app/state.rs`), so it is invisible to the flat
//! `src/*.rs` glob the ported gate replaced. Everything here is legal under
//! rule R1, and each block exercises one of the gate's five exclusions — so
//! assertion A ("the clean fixture passes") is a real test of the exclusion
//! machinery and not just a test that an empty file has no literals.
//!
//! Not compiled by anything.

use crate::ui_text;

// EXCLUSION 2a — a module declared `#[cfg(test)] mod X;` lives in its OWN
// file, which the compiler never emits into a release build. That file is
// dropped from the scan whole, so the operator-shaped literals it holds are
// legal. Break the exclusion and assertion A turns red on
// `state/gated_fixture.rs`.
#[cfg(test)]
mod gated_fixture;

/// Errors this fixture module can report.
#[derive(Debug)]
pub enum StateError {
    /// The document could not be opened.
    Unopenable,
}

// EXCLUSION 3 — `impl Display for` carries diagnostic prose, not UI copy.
// Whitespace-bearing literals here must not be flagged.
impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unopenable => write!(f, "the document could not be opened"),
        }
    }
}

/// Returns the operator-visible label for the delete command.
///
/// EXCLUSION 1 in action: the prose lives in the catalog, and this function
/// only names it.
pub fn delete_label() -> &'static str {
    ui_text::DELETE_SELECTED
}

/// Emits a diagnostic line describing a click.
pub fn trace_click(x: f32, y: f32) {
    // EXCLUSION 4 — a `diag::trace(...)` body is stderr diagnostics. The
    // whitespace-bearing format string below is not operator copy, and the
    // gate must skip it by paren depth rather than needing a marker.
    diag::trace(|| {
        format!(
            "canvas-click screen=[{x} {y}] note=pointer landed on the page",
        )
    });
}

/// A widget id, not prose.
pub fn panel_id() -> &'static str {
    // ui-text-exempt: an egui id_salt, never rendered. EXCLUSION 5, trailing
    // form.
    "objects panel"
}

/// A second widget id, exempted by the block form.
///
// ui-text-exempt: this is the reason, written above the line rather than
// smeared past column 100 — the block form of EXCLUSION 5, which exists
// because a real justification rarely fits on one line.
pub const TREE_ID: &str = "objects tree";

/// Stand-in for the real `diag` module, so the fixture reads like real code.
mod diag {
    /// Mirrors `pdfcer_gui::diag::trace`.
    pub fn trace(f: impl FnOnce() -> String) {
        let _ = f;
    }
}

// EXCLUSION 2b — the BODY of a braced `#[cfg(test)]` item, and nothing else.
// The skip starts at the attribute and ends at the item's closing brace at
// column 0. Test assertion messages inside it are not operator copy; a shipped
// item AFTER it is scanned normally, which is what the dirty fixture proves by
// planting a violation there.
#[cfg(test)]
mod tests {
    #[test]
    fn labels_come_from_the_catalog() {
        assert_eq!(
            super::delete_label(),
            crate::ui_text::DELETE_SELECTED,
            "the delete label must come from the catalog, not a bare literal"
        );
    }
}

/// A shipped item BELOW the braced test module, legal because it names the
/// catalog.
///
/// Present so the clean fixture has the same shape as the dirty one. If the
/// scan ever failed to resume after a test module, only the dirty fixture
/// would notice; this line is the clean half of that pair.
pub fn save_label() -> &'static str {
    ui_text::SETTINGS_APPEARANCE
}
