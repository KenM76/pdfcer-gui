//! # `text::find` — every string the Find bar shows
//!
//! One area of the catalog described in [`crate::text`]'s header. Two
//! consumers, and the split between them is the same one
//! [`crate::text::status`] draws:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/find.md`.

// ---------------------------------------------------------------------------
// The bar's own identity
// ---------------------------------------------------------------------------

/// The word in front of the search field.
#[must_use]
pub fn field_label() -> &'static str {
    "Find"
}

/// Hover text for the search field.
#[must_use]
pub fn field_tooltip() -> &'static str {
    "Type what to look for, then press Enter. Enter again goes to the next hit and \
     Shift+Enter to the previous one. Escape closes this bar while the box has focus. \
     pdfcer searches the text drawn on the pages — not form fields, comments, bookmarks \
     or attachments."
}

/// The close button's label.
#[must_use]
pub fn close() -> &'static str {
    "×"
}

/// Hover text for the close button.
#[must_use]
pub fn close_tooltip() -> &'static str {
    "Close the Find bar and clear its highlights. What you typed is kept for the next time \
     you open it."
}

/// The status bar's Find toggle.
#[must_use]
pub fn toggle() -> &'static str {
    "Find"
}

/// Hover text for the status bar's Find toggle.
#[must_use]
pub fn toggle_tooltip() -> &'static str {
    "Show or hide the Find bar, which searches the text drawn on this document's pages \
     (Ctrl+F)."
}

// ---------------------------------------------------------------------------
// Stepping, and the position readout
// ---------------------------------------------------------------------------

/// The previous-hit button's label.
///
/// `⏴` U+23F4. See this module's header on why not `◀`.
#[must_use]
pub fn previous() -> &'static str {
    "⏴"
}

/// Hover text for the previous-hit button.
#[must_use]
pub fn previous_tooltip() -> &'static str {
    "Go to the previous hit, wrapping to the last one from the first (Shift+Enter)."
}

/// The next-hit button's label.
///
/// `⏵` U+23F5. See this module's header on why not `▶`.
#[must_use]
pub fn next() -> &'static str {
    "⏵"
}

/// Hover text for the next-hit button.
#[must_use]
pub fn next_tooltip() -> &'static str {
    "Go to the next hit, wrapping to the first one from the last (Enter)."
}

/// `3 of 47` — which hit the view is on, out of how many.
#[must_use]
pub fn position(current_one_based: usize, total: usize) -> String {
    format!("{current_one_based} of {total}")
}

/// Hover text for the position readout.
#[must_use]
pub fn position_tooltip() -> &'static str {
    "Which hit the page is showing, and how many there are in the whole document."
}

/// Shown after a search that found nothing.
#[must_use]
pub fn no_matches() -> &'static str {
    "No matches"
}

/// Hover text for the two step buttons while they are **greyed**.
#[must_use]
pub fn step_unavailable_tooltip() -> &'static str {
    "There are no hits to step through yet. Type what to look for and press Enter."
}

/// Hover text for the no-matches readout.
#[must_use]
pub fn no_matches_tooltip() -> &'static str {
    "Nothing on any page matched. pdfcer searches only text that is drawn with real glyphs, \
     and it matches within one text run at a time — so a word a producer split across two \
     runs, or text that is really a scanned image, is not found."
}

/// Shown when the document has been edited since the search ran.
#[must_use]
pub fn stale() -> &'static str {
    "Document changed"
}

/// Hover text for the stale-results readout.
#[must_use]
pub fn stale_tooltip() -> &'static str {
    "You have edited this document since the last search, so the highlights would no longer \
     be reliable and have been cleared. Press Enter to search again."
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// The options menu button's label.
#[must_use]
pub fn options() -> &'static str {
    "Options ⏷"
}

/// Hover text for the options menu button.
#[must_use]
pub fn options_tooltip() -> &'static str {
    "Case sensitivity, whole-word matching, wildcards, and whether going to a hit may change the zoom. They are in a menu so this bar stays narrow enough to sit over the page without hiding it."
}

/// The case-sensitivity control's label.
#[must_use]
pub fn match_case() -> &'static str {
    "Match case"
}

/// Hover text for the case-sensitivity control.
///
/// States the ASCII limit, because it is real and it is the kind of thing
/// that looks like a bug on a document in a language with accents.
#[must_use]
pub fn match_case_tooltip() -> &'static str {
    "Off by default: \"total\" also finds \"Total\" and \"TOTAL\". Case folding is applied to \
     ASCII letters only, so accented letters are always matched exactly as typed."
}

/// The whole-word control's label.
#[must_use]
pub fn whole_word() -> &'static str {
    "Whole word"
}

/// Hover text for the whole-word control.
#[must_use]
pub fn whole_word_tooltip() -> &'static str {
    "Only match where the hit is a complete word: \"total\" stops finding the \"total\" inside \
     `subtotal` and `totals`. What counts as a word is a choice — the standard declines to \
     define one — so a rule chooser appears in this menu when it is switched on."
}

/// The wildcard control's label.
#[must_use]
pub fn wildcards() -> &'static str {
    "Wildcards: # digit, ? any"
}

/// Hover text for the wildcard control.
#[must_use]
pub fn wildcards_tooltip() -> &'static str {
    "Off by default, so pdfcer searches for exactly what you typed. Switch it on and \"#\" \
     matches any digit and `?` matches any single character, so `A#` finds `A1` and `A7`. \
     Note that redaction always matches literally: a pattern search can highlight hits that \
     redaction would decline to mark."
}

/// The zoom control's label.
#[must_use]
pub fn find_zoom() -> &'static str {
    "Zoom"
}

/// Hover text for the zoom control.
#[must_use]
pub fn find_zoom_tooltip() -> &'static str {
    "On by default. Going to a hit lands on whatever page it is on, centres the hit, and \
     if Fit page or Fit width is switched on the view re-scales to that page — so on a set \
     whose sheets are different sizes, the zoom changes too. Switch this off and going to a \
     hit leaves the page where it is on the canvas at the size you are reading at: the page \
     still changes and the hit is still highlighted, but nothing is scrolled to the middle, \
     the view stops following the fit, and the zoom readout shows the percentage instead."
}

/// The caption in front of the whole-word rule chooser.
#[must_use]
pub fn word_rule() -> &'static str {
    "Word rule"
}

/// Hover text for the whole-word rule chooser.
#[must_use]
pub fn word_rule_tooltip() -> &'static str {
    "What counts as one word. The PDF standard says outright that this has no single \
     correct answer, so it is your choice; the default matches what search boxes usually do."
}

/// The `Alphanumeric` rule's label.
#[must_use]
pub fn word_rule_alphanumeric() -> &'static str {
    "Letters and digits"
}

/// Hover text for the `Alphanumeric` rule.
#[must_use]
pub fn word_rule_alphanumeric_tooltip() -> &'static str {
    "A word is a run of letters, digits and underscores; every space, dash and punctuation \
     mark ends it. So `well-known` contains the words `well` and `known`, and `don't` \
     contains `don`. This is what search boxes and regular expressions normally do."
}

/// The `NonSpace` rule's label.
#[must_use]
pub fn word_rule_non_space() -> &'static str {
    "Split at spaces only"
}

/// Hover text for the `NonSpace` rule.
#[must_use]
pub fn word_rule_non_space_tooltip() -> &'static str {
    "A word is a run of anything that is not a space, so punctuation stays part of the word \
     it touches: `A-12/B` is one word and `(total)` does not contain the word `total`. The \
     right choice when the text is part numbers, file paths or code."
}

/// The `NonSpaceOrDash` rule's label.
#[must_use]
pub fn word_rule_non_space_or_dash() -> &'static str {
    "Split at spaces and dashes"
}

/// Hover text for the `NonSpaceOrDash` rule.
#[must_use]
pub fn word_rule_non_space_or_dash_tooltip() -> &'static str {
    "As \"Split at spaces only\", and hyphens and em dashes end a word too: \"A-12/B\" splits at \
     the hyphen but not at the slash, and `well-known` contains `well` and `known`."
}

/// What a zero-result search says when part of the document could never have
/// matched.
#[must_use]
pub fn unsearchable_one() -> &'static str {
    "No matches. One font in this document stores text that cannot be searched, so there may be more."
}

/// See [`unsearchable_one`].
#[must_use]
pub fn unsearchable_many(n: u64) -> String {
    format!(
        "No matches. {n} fonts in this document store text that cannot be searched, so there may be more."
    )
}

/// The hover explanation behind the sentence above.
#[must_use]
pub fn unsearchable_tooltip() -> &'static str {
    "Some PDFs store text as drawings with no record of which letters they are. It renders correctly and can be printed, but nothing can search or copy it. Recognising the page adds a searchable layer."
}

/// **The sentence when the blank was IGNORED** — `OPERATOR_REQUESTS.md`
/// **O180**, 2026-09-12.
#[must_use]
pub fn blanks_trimmed() -> &'static str {
    "Blanks at the ends of your search were ignored."
}

/// **The sentence when the blank was KEPT** — the same fact, the other
/// setting.
#[must_use]
pub fn blanks_kept() -> &'static str {
    "Your search starts or ends with a blank, and pdfcer is looking for it."
}

/// The hover explanation behind either sentence, naming where the switch is.
#[must_use]
pub fn blanks_tooltip() -> &'static str {
    "Spaces, tabs and similar characters are invisible in this box, and copying out of a spreadsheet or a table very often brings one along. Whether they are ignored is set in Settings, under Copying and extracting text. Blanks in the middle of what you type are always kept."
}
