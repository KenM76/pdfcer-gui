//! FIXTURE — the NESTED module carrying the planted violation.
//!
//! `src/app/state.rs` is two levels down. The gate's predecessor scanned
//! `"$SRC_DIR"/*.rs`, which does not match this path, so it read `lib.rs` and
//! `ui_text.rs`, found nothing, and printed `ui-strings: clean` — the exact
//! fail-open documented in PROJECT_PLAN.md §4.1.
//!
//! The ported gate uses `find`, sees this file, and fails. That difference is
//! the whole content of `--self-test` assertion B.
//!
//! DO NOT "FIX" THE VIOLATION BELOW. It is the fixture.
//!
//! Not compiled by anything.

/// Returns the label drawn on the object context menu's destructive item.
pub fn delete_label() -> &'static str {
    // THE PLANTED VIOLATION: a bare, whitespace-bearing, operator-visible
    // literal, outside the catalog, with no exemption marker. Everything the
    // gate looks for, in a file a flat glob cannot see.
    "Delete selected object"
}

/// A widget id, exempted — present so the fixture proves the gate reports the
/// violation and NOT this line.
pub fn panel_id() -> &'static str {
    // ui-text-exempt: an egui id_salt, never rendered.
    "objects panel"
}

// ───────────────────────────────────────────────────────────────────────────
// SHAPE 1 — the one-line `#[cfg(test)] mod X;` declaration.
//
// This is the DOMINANT shape in the real tree: 508 files carry a column-0
// `#[cfg(test)]`, and the commonest form is a declaration whose body lives in
// a sibling FILE, sitting with the other `mod` lines near the top. The
// scanner's predecessor `exit`ed at the first column-0 `#[cfg(test)]`, so
// every shipped item below one of these was invisible — measured at 66 files.
// The convention that rule was documented under, "keep the test module last",
// does not describe this shape at all: there is no test module here to be last.
//
// PLANTED VIOLATION 2 is the item below. It must be reported.
// ───────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests;

/// Returns the label drawn on the Save command.
pub fn save_label() -> &'static str {
    "Save a copy of this drawing"
}

// ───────────────────────────────────────────────────────────────────────────
// SHAPE 2 — the braced `#[cfg(test)] mod X { ... }` item.
//
// Here the skip is right, but it must END at the item's closing brace at
// column 0 rather than running to end of file. The assertion message inside
// must NOT be reported; PLANTED VIOLATION 3 below it must be.
// ───────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod inline_tests {
    #[test]
    fn an_assertion_message_is_not_operator_copy() {
        assert!(
            super::save_label().is_empty() || true,
            "this assertion message must never be reported as UI copy"
        );
    }
}

/// Returns the label drawn on the discard-changes command.
pub fn close_label() -> &'static str {
    "Close without saving"
}

// ───────────────────────────────────────────────────────────────────────────
// SHAPE 3 — the attribute and the braced item on ONE line.
//
// The scanner has a separate branch for this, because the two-line form has to
// arm and wait for the item while this one does not. Without a fixture that
// reaches it, that branch can be broken and every assertion below still
// passes — which happened once while these assertions were being falsified.
//
// PLANTED VIOLATION 4 is the item below. It must be reported; the message
// inside the module must not.
// ───────────────────────────────────────────────────────────────────────────
#[cfg(test)] mod oneline_tests {
    #[test]
    fn a_one_line_module_is_skipped_too() {
        assert!(true, "this one-line module message must never be reported");
    }
}

/// Returns the label drawn on the print command.
pub fn print_label() -> &'static str {
    "Print the current sheet"
}
