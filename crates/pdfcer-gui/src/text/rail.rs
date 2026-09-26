//! # `text::rail` — the left rail's own words
//!
//! `OPERATOR_REQUESTS.md` O123 part 7. Three strings, and each exists because
//! the rail says something no other surface has to say.
//!
//! The group captions themselves live in [`crate::text::ribbon`] beside the
//! ribbon's, because they name the *same* groups — `Navigate` on the rail and
//! Navigate on the View tab are one group in two places, and two spellings of
//! one caption is how two surfaces start disagreeing about what a group is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/rail.md`.

/// What a **pinned** row means — shown in its hover at the rail's floor.
#[must_use]
pub fn pinned() -> &'static str {
    "This is the tool you are holding. The rest of the group is behind the chevron below — \
     the strip is short of room."
}

/// The chevron's face: a downward glyph and how many entries are behind it.
#[must_use]
pub fn chevron_glyph(count: usize) -> String {
    format!("⏷{count}")
}

/// The chevron's hover: what the strip folded away, in the order it went.
#[must_use]
pub fn chevron_hint(names: &[&str]) -> String {
    if names.is_empty() {
        // Unreachable through the renderer, which draws no chevron over an
        // empty overflow — but a sentence rather than an empty tooltip, because
        // an empty tooltip reads as a broken one.
        return "Nothing is folded away.".to_owned();
    }
    format!("Folded away, in the order they went: {}", names.join(", "))
}

/// One rail row's hover: the control's name, its sentence, and — when the row
/// is a **pinned** stand-in for a whole group — [`pinned`] under it.
///
#[must_use]
pub fn hover(label: &str, tooltip: Option<&str>, pinned_row: bool) -> String {
    let head = match tooltip {
        Some(tip) => format!("{label} — {tip}"),
        None => label.to_owned(),
    };
    if pinned_row {
        format!("{head}\n\n{}", pinned())
    } else {
        head
    }
}
