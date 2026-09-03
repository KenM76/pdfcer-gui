//! # `text::find` — every string the Find bar shows
//!
//! One area of the catalog described in [`crate::text`]'s header. Two
//! consumers, and the split between them is the same one
//! [`crate::text::status`] draws:
//!
//! | Consumer | What it reads from here |
//! |---|---|
//! | [`crate::find::bar`] | the floating box — the field, the two step buttons, the position readout, the options menu and everything in it, the close button |
//! | [`crate::app::status`] | the **Find toggle**, which `RIBBON_IA.md` §6 puts on the status bar |
//!
//! The command's own label and tooltip are **not** here: `edit.find` is a
//! registered command, so its copy lives in [`crate::text::commands`] with
//! every other command's, keyed by command id and consumed by
//! `crate::shell::commands`. This file holds the copy the *controls* own.
//!
//! ## ★ Why the wildcard control's label says what `#` and `?` do
//!
//! Because the alternative shipped once and was a defect. `pdfcer-core`'s
//! [`pdfcer_core::edit::TextSearchOptions::wildcards`] records it in full:
//! the old shell's Find bar ran through `EditSession::find_text`, which
//! passes `with_wildcards(true)`, so typing a literal `?` matched **every
//! character on the page** and nothing on screen said why. Core's own
//! conclusion is that the fix belongs in the front end — the verb keeps its
//! documented pattern behaviour and the *default* moved to off — so this
//! build searches for what was typed, and the operator who wants patterns
//! ticks a box whose label names the two characters and what each does.
//!
//! A control called "Wildcards" with no further explanation would be the
//! same defect with a checkbox in front of it: the operator still would not
//! know that `#` is a digit class, and would still be surprised by `?`.
//!
//! ## ★ Why the whole-word rule is worded as three plain descriptions
//!
//! ISO 32000-1 §14.8.2.5 NOTE 1 declines to define "word", and
//! [`pdfcer_core::edit::WordBoundary`] carries the whole argument. What
//! reaches an operator must not be the enum's names — `Alphanumeric`,
//! `NonSpace`, `NonSpaceOrDash` are the *implementation's* vocabulary — but
//! a description of the consequence they will actually notice: whether
//! `well-known` contains the word `known`, and whether `A-12/B` is one token
//! or three. So each label states the rule and each tooltip states the
//! consequence, with a worked example.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the chord only when the chord works.** [`toggle_tooltip`] names
//!   `Ctrl+F` because `crate::shell::manifest::built_in`'s keymap binds it
//!   and `crate::app::keyboard::parse_chord` can spell it. Both halves are
//!   required — the keymap alone was not enough for `Ctrl+O`, which was
//!   named in a tooltip and did nothing for the whole of the ribbon's first
//!   life.
//! - **Never state a capability the build does not have.** There is no
//!   "Find all", no "Search open documents", no results list, and no string
//!   here for any of them.
//!
//! ## The glyphs
//!
//! `⏴` and `⏵` are the step buttons' entire visible text, and `×` is the
//! close button's. A codepoint the bundled font set cannot draw renders as a
//! tofu box, which is defect D2's shape — an invisible label — on a control
//! an operator has to hit. `crate::text::status`'s header records that the
//! obvious choices (`◀ ▶ ▸ ▾`) are **absent** from egui's bundled fonts and
//! that `⏴ ⏵ ⏶ ⏷` are present; the same three glyphs are re-asserted by
//! [`crate::find::bar::tests::every_glyph_the_find_bar_draws_has_a_glyph`],
//! because a test that lives beside the status bar cannot see this file.

// ---------------------------------------------------------------------------
// The bar's own identity
// ---------------------------------------------------------------------------

/// The word in front of the search field.
///
/// A label rather than placeholder text inside the field. Placeholder text
/// disappears the moment the operator types, so a box identified only by its
/// placeholder has no name for as long as it is being used — and this one
/// floats over the page rather than sitting in a bar the operator already
/// recognises, so "what is this?" is worth answering permanently.
#[must_use]
pub fn field_label() -> &'static str {
    "Find"
}

/// Hover text for the search field.
///
/// Names the three keys the field itself owns, because none of them is
/// discoverable by looking: Enter searches (and then steps), Shift+Enter
/// steps backwards, Escape closes. The last one is qualified — it closes the
/// bar *while you are typing in it* — because once focus has left the field
/// Escape belongs to the canvas's selection ladder, and a tooltip that
/// promised otherwise would be describing a key fight this build
/// deliberately does not have.
#[must_use]
pub fn field_tooltip() -> &'static str {
    "Type what to look for, then press Enter. Enter again goes to the next hit and \
     Shift+Enter to the previous one. Escape closes this bar while the box has focus. \
     pdfcer searches the text drawn on the pages — not form fields, comments, bookmarks \
     or attachments."
}

/// The close button's label.
///
/// `×` (U+00D7 MULTIPLICATION SIGN), not `✕`/`✖`, and not the word "Close".
/// The word is three times as wide as the control needs on a box that floats
/// over the page and is kept deliberately narrow; and U+00D7 is Latin-1, so it
/// is present in every font a desktop toolkit ships, where the dingbat crosses
/// are not.
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
///
/// `RIBBON_IA.md` §6 lists this first among the status bar's controls. It is
/// a *toggle* rather than a button because the box it opens is a persistent
/// surface the operator leaves up while working through hits, and a control
/// that showed no state would give them no way to tell "it is closed" from "it
/// is open at the other end of the window".
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
///
/// **One-based**, because it is read next to the status bar's page counter,
/// which is one-based for the same reason: an operator counts hits from one.
/// The zero-based index is an internal fact and never reaches this function.
///
/// Deliberately not `3/47`: a slash reads as a fraction or a date, and this
/// row already carries `n/N` shapes in the page box a few points to its
/// right.
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
///
/// A sentence rather than `0 of 0`, because zero of zero is arithmetic and
/// the operator's question is whether the search ran at all. This says it
/// ran.
#[must_use]
pub fn no_matches() -> &'static str {
    "No matches"
}

/// Hover text for the two step buttons while they are **greyed**.
///
/// `RIBBON_IA.md` P3 allows greying only for *temporarily* unavailable and
/// only when it *"is always explained on hover"*. Both hold: there is nothing
/// to step through until a search has found something, that state ends on the
/// next Enter, and this is the sentence that says so. A greyed control with
/// no explanation is the shape defect D1 took — an affordance that looks
/// available and is inert, with nothing on screen to say why.
#[must_use]
pub fn step_unavailable_tooltip() -> &'static str {
    "There are no hits to step through yet. Type what to look for and press Enter."
}

/// Hover text for the no-matches readout.
///
/// Names the two limits that produce a surprising empty result on real
/// files, because both are properties of the *document* rather than of the
/// query and an operator has no way to guess either. Both are
/// `pdfcer-core`'s documented limits on `find_text`, not this shell's.
#[must_use]
pub fn no_matches_tooltip() -> &'static str {
    "Nothing on any page matched. pdfcer searches only text that is drawn with real glyphs, \
     and it matches within one text run at a time — so a word a producer split across two \
     runs, or text that is really a scanned image, is not found."
}

/// Shown when the document has been edited since the search ran.
///
/// ★ **The hits are not merely out of date; their geometry may name text
/// that is no longer there.** A `delete_*` renumbers and re-splices the
/// content stream, so a quad recorded before an edit can cover different
/// glyphs after it — and rule 4 forbids painting a mark over content that
/// does not say what the mark claims. So the highlights stop being drawn the
/// instant `edit_epoch` moves, the readout says so, and the query is kept:
/// re-running it is one keypress, and it is the operator's keypress rather
/// than a search this shell decided to repeat on a 5.6 MB drawing.
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
///
/// A word plus the disclosure triangle the status bar already uses, rather
/// than a bare chevron or a gear: `crate::find::bar` puts the four search
/// options behind this button, and an operator who cannot find "case
/// sensitive" will look for a word before they will click an unlabelled
/// glyph. `⏷` is U+23F7, measured present in egui's bundled font set —
/// `crate::text::status`'s header records that the obvious `▾` is not.
#[must_use]
pub fn options() -> &'static str {
    "Options ⏷"
}

/// Hover text for the options menu button.
///
/// Names what is inside it, because the whole cost of putting the options
/// behind a menu is that they stop being visible — and a button labelled
/// "Options" with no further word is a button an operator has to open to find
/// out whether it is worth opening.
#[must_use]
pub fn options_tooltip() -> &'static str {
    "Case sensitivity, whole-word matching and wildcards. They are in a menu so this bar stays narrow enough to sit over the page without hiding it."
}

/// The case-sensitivity control's label.
///
/// Phrased as the thing the operator switches **on**, matching Acrobat's own
/// "Case-Sensitive" toggle: off means an ordinary, forgiving search, which
/// is what a find bar does everywhere.
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
///
/// ★ The label **names the two characters and what each does**. See this
/// module's header: a bare "Wildcards" would leave the operator exactly
/// where the old shell's silent pattern search left them, one checkbox
/// later.
#[must_use]
pub fn wildcards() -> &'static str {
    "Wildcards: # digit, ? any"
}

/// Hover text for the wildcard control.
///
/// The last sentence is the hazard note the brief asks be left where the
/// next person will see it: a future "redact every hit" control cannot be
/// built on a wildcard search, because `mark_redactions_by_search` matches
/// **literally** and would decline to mark hits this bar highlighted.
#[must_use]
pub fn wildcards_tooltip() -> &'static str {
    "Off by default, so pdfcer searches for exactly what you typed. Switch it on and \"#\" \
     matches any digit and `?` matches any single character, so `A#` finds `A1` and `A7`. \
     Note that redaction always matches literally: a pattern search can highlight hits that \
     redaction would decline to mark."
}

/// The caption in front of the whole-word rule chooser.
#[must_use]
pub fn word_rule() -> &'static str {
    "Word rule"
}

/// Hover text for the whole-word rule chooser.
///
/// Says outright that there is no correct answer to import. That is not
/// hedging: `pdfcer-core`'s `WordBoundary` quotes §14.8.2.5 NOTE 1 saying the
/// notion of a word *"is not precisely defined"*, and NOTE 4 offers a menu
/// of reader strategies rather than a rule. Telling the operator that the
/// choice is theirs is the honest version of a setting that exists because
/// the standard refused to decide.
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
///
/// ★ Worded as a fact about the DOCUMENT, not about the search and not about
/// pdfcer. "No matches" is still the answer to what they asked; the second
/// clause tells them why the answer may be incomplete. It deliberately does not
/// say "pdfcer cannot read" — Acrobat cannot read it either, the file simply
/// does not carry the mapping, and phrasing a file's own gap as a tool
/// limitation invites an operator to go looking for a better tool.
///
/// Singular and plural are separate strings rather than "font(s)", because a
/// parenthesised plural in a sentence an operator reads under pressure is how
/// software sounds when nobody cared.
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
