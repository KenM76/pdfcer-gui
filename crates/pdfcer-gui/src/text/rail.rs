//! # `text::rail` — the left rail's own words
//!
//! `OPERATOR_REQUESTS.md` O123 part 7. Three strings, and each exists because
//! the rail says something no other surface has to say.
//!
//! The group captions themselves live in [`crate::text::ribbon`] beside the
//! ribbon's, because they name the *same* groups — `Navigate` on the rail and
//! Navigate on the View tab are one group in two places, and two spellings of
//! one caption is how two surfaces start disagreeing about what a group is.

/// What a **pinned** row means — shown in its hover at the rail's floor.
///
/// ★★★ The one sentence the rail owes and nothing else does. At
/// `Rung::Cramped` the navigate group is drawn as a single row showing
/// whatever is armed, and without this sentence the operator sees one tool
/// where a moment ago there were four, with no way to learn that the rest are
/// behind the chevron a few rows down. R9's second half — *greying is always
/// explained* — generalised: a control that changed what it stands for owes
/// the same explanation.
#[must_use]
pub fn pinned() -> &'static str {
    "This is the tool you are holding. The rest of the group is behind the chevron below — \
     the strip is short of room."
}

/// The chevron's face: a downward glyph and how many entries are behind it.
///
/// The count is on the button rather than only in the hover because a chevron
/// that does not say how much it holds is a control an operator has to press
/// to evaluate — and this one is permanent chrome, so they would press it
/// once per session for ever.
///
/// ⚠ **`⏷` is U+23F7**, not the obvious `⌄` (U+2304) and not `▼` (U+25BC).
/// `crate::icons::glyphs` measures the shipped font stack, and both of those
/// render as a **substitution box** in front of the operator — its gate caught
/// exactly that here, on the first draft of this function, which had written
/// U+2304. The `⏴⏵⏶⏷` block is the one this project has verified;
/// `crate::text::find` and `crate::text::status` already stand on the same
/// finding, and `crate::text::panels::bookmarks` records the measurement.
#[must_use]
pub fn chevron_glyph(count: usize) -> String {
    format!("⏷{count}")
}

/// The chevron's hover: what the strip folded away, in the order it went.
///
/// ★ Named rather than counted. *"3 more"* tells an operator that something
/// is missing; naming them tells them whether the thing they want is in there,
/// which is the only question they are actually asking. `RIBBON_SCALING.md`
/// makes the same call for the ribbon's `⏷ N more` menu.
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
/// ★ Composed here rather than at the draw site because R1
/// (`tools/gates/check-ui-strings.sh`) counts the joining as part of the
/// string: the em dash between a name and its sentence, and the blank line
/// before the pinned note, are typography, and typography belongs beside the
/// words it punctuates. The gate caught the first draft doing it in
/// `app::rail`.
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
