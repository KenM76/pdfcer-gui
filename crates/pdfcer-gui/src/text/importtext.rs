//! # `text::importtext` — what the import tells the operator afterwards
//!
//! The receipt for `file.import_text`, and its refusals.
//! `dialogs::import_text` holds the words the *window* says before the press;
//! this module holds the words that arrive after it.
//!
//! ## ★★★ THE ENGINE COMPOSES ITS OWN DISCLOSURES AND THIS SHELL DOES NOT PRINT
//! THEM
//!
//! `PlaceTextReport::disclosures` is a `Vec<String>` documented as *"every
//! operator-facing disclosure, verbatim, ready to print"*. It is not printed
//! here, and the reason is not pride — it is that they are written in the
//! implementer's voice. Verbatim, the first one reads:
//!
//! > *"imported text was PAGINATED by pdfcer: 47 line(s) at 13.20pt leading
//! > (derived default 1.2 x size), 44 line(s) per page"*
//!
//! and the tab one:
//!
//! > *"12 tab(s) were collapsed into ordinary word spacing. INDENTATION IS
//! > LOST: showing has no tab stops, so there is nothing to preserve a tab as"*
//!
//! Both are **correct and useful to a developer**, and neither is a sentence
//! this operator would read. *"Showing has no tab stops"* is about the `Tj`
//! operator; leading in points is a typesetting unit he has no control over in
//! this window; and the count of lines per page is arithmetic rather than
//! information.
//!
//! ⇒ So this module writes the same facts from the **structured fields**, in
//! his terms — which is the split this program makes everywhere: the engine
//! owns the measurement, the shell owns the sentence. The one place the
//! engine's own words are printed is the catch-all refusal, where a message we
//! have not anticipated is better than a shrug.
//!
//! ## ★★ Every sentence here is CONDITIONAL except the first
//!
//! The page count always shows, because it is the answer to *"did that work?"*.
//! Everything else appears only when its count is non-zero. A receipt that
//! listed six disclosures reading *"0 tabs collapsed"* would be a form, and by
//! the third import nobody would read the line that mattered.

/// **How many pages arrived, and where.**
///
/// ★★ It names **where** as well as how many, because the pages may have landed
/// anywhere in the document — the window offers four positions — and *"11 pages
/// added"* leaves an operator scrolling to find them. `pages_before` is read in
/// the apply arm before the edit for exactly this sentence.
///
/// ★ *"sheets"* rather than *"pages"* in the operator's half of the sentence
/// would be wrong here: these are PDF pages and he is looking at a page count
/// in the sidebar. `pages` is the word the rest of the program uses.
#[must_use]
pub fn pages_created(pages: usize, pages_before: usize) -> String {
    let noun = if pages == 1 { "page" } else { "pages" };
    format!(
        "Imported {pages} {noun}. The document had {pages_before} before and has \
         {} now.",
        pages_before + pages
    )
}

/// **The undo promise cannot be kept** — `PlaceTextReport::coalesced` is false.
///
/// ★★★ The engine's doc is explicit that the one-undo-entry fold is *checked,
/// not assumed*: past `MAX_UNDO_DEPTH` — more than 255 non-blank pages — every
/// page is still placed and they simply are not grouped.
///
/// ⇒ This is the **only** moment the fact is actionable. After the first
/// `Ctrl+Z` the remaining 200 look like the program undoing things by itself,
/// and an operator who presses it twice and sees two pages vanish will conclude
/// something is broken. Said once, before he touches undo.
#[must_use]
pub fn many_undo_steps(entries: usize) -> String {
    format!(
        "This import was too large to undo in one step — it will take {entries} presses of Undo \
         to reverse it completely."
    )
}

/// **A paragraph he wrote as one block is now on two sheets.**
///
/// Rule 4's surviving half: the result looks exactly like a paragraph he wrote
/// that way, so nothing on the page can tell him this happened.
#[must_use]
pub fn paragraphs_split(count: usize) -> String {
    let noun = if count == 1 {
        "paragraph"
    } else {
        "paragraphs"
    };
    format!("{count} {noun} ran past the bottom of a page and continue on the next one.")
}

/// **Tabs became spaces**, so column alignment is gone.
///
/// ★ It says *"columns will not line up"* rather than *"tabs were collapsed"*,
/// because the operator's word for what he loses is columns. The mechanism —
/// PDF text showing has no tab stops — is true, is in the engine's own
/// sentence, and is not something he can act on.
///
/// ★★ It names the remedy, and the remedy is a control in the window he just
/// used: a monospaced face keeps space-aligned columns lined up, which is the
/// one case where the font choice changes whether the import is readable.
/// `dialogs::import_text`'s `FACES` note is why Courier is in the list at all.
#[must_use]
pub fn tabs_collapsed(count: usize) -> String {
    let noun = if count == 1 { "tab" } else { "tabs" };
    format!(
        "{count} {noun} became ordinary spaces, so anything lined up in columns will not line up \
         any more. Importing again in Courier keeps space-aligned columns straight."
    )
}

/// **Form feeds in the source became page breaks.**
///
/// ★★ Worth a sentence because it is the one disclosure where the operator may
/// not know his own file contains the character. U+000C is invisible in every
/// editor, and it is what `Export text` writes between pages — so a file that
/// left pdfcer, was edited, and came back keeps its original pagination, which
/// is **correct and surprising** in equal measure.
#[must_use]
pub fn page_breaks(count: usize) -> String {
    let noun = if count == 1 {
        "page break"
    } else {
        "page breaks"
    };
    format!(
        "{count} {noun} already in the file were used as they were — text exported from pdfcer \
         carries them, so a file that came from here keeps its original pagination."
    )
}

/// **A word wider than the column.**
///
/// ★ The engine does not hyphenate and does not shrink, so such a word runs
/// past the right margin. The remedy is the two controls that set the column:
/// a smaller size or a narrower margin.
#[must_use]
pub fn overlong_words(count: usize) -> String {
    let noun = if count == 1 { "word is" } else { "words are" };
    format!(
        "{count} {noun} wider than the column and will run past the right margin. A smaller \
         font size or a narrower margin would fit them."
    )
}

/// **Non-printing bytes were removed.**
///
/// ★ Reported rather than silent because they were *in his file*, and a
/// character count that does not match is the kind of thing somebody notices a
/// week later on a register they are reconciling.
#[must_use]
pub fn control_chars(count: usize) -> String {
    let noun = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{count} non-printing {noun} were removed — they have no shape in any PDF font, so they \
         could not have been drawn."
    )
}

/// **Characters the font could not write, dropped.**
///
/// ⚠ Unreachable from this shell today: `dialogs::import_text` never sets
/// `Unmappable::Drop`, so the engine refuses instead and [`unmappable_refused`]
/// is what the operator sees. It is written and wired anyway, because the day a
/// *"import anyway and tell me what was lost"* button is added, the sentence
/// that reports the loss must already exist — and a disclosure written after
/// its button is a disclosure somebody has to remember to write.
#[must_use]
pub fn unmappable_dropped(count: usize) -> String {
    let noun = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{count} {noun} could not be written in the chosen font and were left out of the \
         imported pages."
    )
}

/// **The engine's own self-check failed** — this is a defect report.
///
/// ★★★ `box_overflow_lines` is documented as *"a self-check that must be 0"*.
/// A non-zero value is a fault in the placer, not a judgement about the
/// operator's file, and the sentence says so — because an operator who reads it
/// alongside the six ordinary disclosures would otherwise file it under
/// *"things imports do"* and never mention it.
#[must_use]
pub fn overflowed(lines: usize) -> String {
    let noun = if lines == 1 { "line" } else { "lines" };
    format!(
        "{lines} {noun} did not fit the page and were drawn outside it. That is a fault in \
         pdfcer rather than in your file — the import worked, but it is worth reporting."
    )
}

// ---------------------------------------------------------------------------
// Refusals. Nothing was created; the document is exactly as it was.
// ---------------------------------------------------------------------------

/// The file could not be opened.
#[must_use]
pub fn unreadable_file(why: &str) -> String {
    format!("That file could not be opened, so nothing was imported. {why}")
}

/// The file is not UTF-8.
///
/// ★★ Its own sentence rather than folded into [`unreadable_file`], because it
/// is the only one of the two with a remedy the operator can carry out, and the
/// remedy is specific: re-save as UTF-8. A register exported from an older
/// system in Windows-1252 is the realistic case and it is completely ordinary.
#[must_use]
pub fn not_utf8() -> String {
    "That file is not UTF-8 text, so nothing was imported. Re-save it as UTF-8 — most editors \
     offer that under Save As — and try again."
        .to_owned()
}

/// The margins leave no width.
///
/// ★★ Both this and [`page_too_short`] name **the two controls that fix it**
/// rather than the geometry that caused it. They are the only refusals in this
/// module answerable *before* the press, so their sentences are instructions
/// for the window rather than reports about a failure.
#[must_use]
pub fn no_column() -> String {
    "The margins leave no room for text on that sheet, so nothing was imported. Choose a larger \
     sheet size or a smaller margin."
        .to_owned()
}

/// The margins leave no height for even one line.
#[must_use]
pub fn page_too_short() -> String {
    "The margins leave no room for even one line on that sheet, so nothing was imported. Choose \
     a larger sheet size, a smaller margin, or a smaller font size."
        .to_owned()
}

/// The font cannot write some of the text.
///
/// ★★★ **The refusal a real text file is most likely to meet**, and the one
/// this window's `face_note` warns about before the press.
///
/// It carries the engine's own listing verbatim — `U+2014 '—' ×12, …` — because
/// that is the part the operator needs and no rewording improves it: he has to
/// find those characters in his file. What the shell adds is the two remedies,
/// in the order of least work: try a different font, or edit the file.
///
/// ⚠ *"or ask for them to be dropped"* — the engine's third remedy — is
/// deliberately **not** offered, because this window has no such control. A
/// sentence naming a button that does not exist is worse than one remedy fewer.
#[must_use]
pub fn unmappable_refused(base_font: &str, total: usize, listing: &str) -> String {
    let noun = if total == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{base_font} cannot write {total} {noun} in that file, so nothing was imported: \
         {listing}. Try importing in a different font, or replace those characters in the file."
    )
}

/// The document has no page to insert beside.
#[must_use]
pub fn no_page_to_insert_beside() -> String {
    "This document has no pages, so there is nowhere to put the imported ones. Imported pages go \
     before or after an existing page."
        .to_owned()
}

/// Everything else, carrying the engine's own words.
///
/// ★ The engine's message rather than a shrug — `annots::refusal_for`'s
/// posture. What this adds is the guarantee, which is the half an operator
/// needs before he starts looking for an undo: `place_text` plans before it
/// writes, so a refusal means **nothing was created**.
#[must_use]
pub fn refused(why: &str) -> String {
    format!("Nothing was imported and the document is exactly as it was. {why}")
}
