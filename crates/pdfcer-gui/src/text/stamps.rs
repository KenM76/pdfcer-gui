//! # `text::stamps` — the words for custom stamp collections
//!
//! Two surfaces, one vocabulary: the **Save as stamp collection** window that
//! authors one, and the **Document Properties** section that discloses one
//! when the open file already is one. They share this catalog because they
//! share the concept, and a feature that calls the same thing a *category*
//! in one place and a *set* in the other has already lost the operator.
//!
//! ## ★ The hardest sentence in here, and what it must not say
//!
//! pdfcer can **make** a stamp collection and cannot **use** one — placing a
//! custom stamp needs an engine verb that does not exist
//! (`ENGINE_BACKLOG.md`, `Pass 288.0`). So the copy has to explain a feature
//! whose payoff happens in *another application*, without either apologising
//! or pretending.
//!
//! What it says: your stamps appear in Acrobat's stamp menu. What it must
//! never say: some variant of *"pdfcer cannot do this yet"* dressed as a
//! limitation notice. **R9** — an unavailable capability renders nothing.
//! There is no greyed *Place stamp* control anywhere in this surface, so there
//! is nothing to explain, and a sentence explaining an absent control would be
//! the nagging rule 4 exists to prevent.
//!
//! ## The vocabulary, fixed here
//!
//! | word | means | never used for |
//! |---|---|---|
//! | **stamp** | one page of artwork with a name | the `/Stamp` annotation the Markup tab places — those are *standard stamps* |
//! | **collection** | the file: one category, its stamps | "set", "library", "pack" |
//! | **category** | the collection's name, shown as Acrobat's submenu heading | "title", even though it is stored as `/Info` `/Title` |
//! | **display name** | what a picker shows for one stamp | "label" |
//!
//! ⚠ The **standard stamps** distinction matters. `crate::text::markup` and
//! `crate::text::textannot` already own the words for the `/Stamp` annotation
//! pdfcer *does* place — `Approved`, `Draft`, and the rest of §12.5.6.12's
//! closed vocabulary. Those are a different feature that happens to share a
//! noun, and no string in this file may imply that this window produces one.
//!
//! ## Rule 15
//!
//! No string here says "dimension" in either sense, and none should acquire
//! one: a stamp collection has nothing to do with **ce dimensions** or with
//! **pdf dimensions**. The word to reach for when describing a page's extent
//! here is *size*.

use crate::stamps::Adjustment;

// ─────────────────────────────────────────────────────────────────────────
// The model's own defaults
// ─────────────────────────────────────────────────────────────────────────

/// The display name a page gets before the operator types one.
///
/// **One-based**, because it sits in a row beside a page number that is also
/// one-based. A set of stamps called `Stamp 0 … Stamp 11` is a set somebody
/// has to renumber by hand before showing it to anyone.
#[must_use]
pub fn default_stamp_name(page_number: usize) -> String {
    format!("Stamp {page_number}")
}

/// The filename stem used when the category cannot make one.
///
/// Reachable when a category is entirely punctuation — `***` — which
/// sanitises to nothing. The collection still writes; only its filename falls
/// back, and Acrobat reads the category from inside the file regardless.
#[must_use]
pub const fn default_category_file_stem() -> &'static str {
    "Stamps"
}

// ─────────────────────────────────────────────────────────────────────────
// The command
// ─────────────────────────────────────────────────────────────────────────

/// The ribbon command's label.
#[must_use]
pub const fn command_label() -> &'static str {
    "Save as stamp collection"
}

/// The ribbon command's tooltip.
///
/// ★ Names **Acrobat** explicitly. The whole value of this feature is that the
/// file lands somewhere another application reads, and a tooltip that said
/// only *"save this document as a stamp collection"* would describe the act
/// and hide the point.
#[must_use]
pub const fn command_tooltip() -> &'static str {
    "Turn this document's pages into named stamps that appear in Acrobat's \
     stamp menu — one page per stamp, one file per category."
}

// ─────────────────────────────────────────────────────────────────────────
// The window
// ─────────────────────────────────────────────────────────────────────────

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Save as stamp collection"
}

/// The paragraph under the title.
///
/// Three facts, in the order an operator needs them: what becomes a stamp,
/// where the file goes, and what happens next. The last one is the one that
/// stops the question *"and now what?"*.
#[must_use]
pub const fn intro() -> &'static str {
    "Each page becomes one stamp. Name them below, give the set a category, \
     and save it where Acrobat keeps stamps — they appear in its stamp menu \
     the next time it starts."
}

/// The line under [`intro`] when the open document already **is** a stamp
/// collection.
///
/// ★ It exists to answer a question the window would otherwise raise and
/// leave: *"where did these names come from?"*. With names already in the
/// rows, an operator has no way to tell pdfcer's defaults from his own
/// previous work, and the difference decides whether he reads every row or
/// none of them.
///
/// Shown only in that case. A sentence saying "these are defaults" on every
/// ordinary document would be one more line nobody reads.
#[must_use]
pub const fn reopened_note() -> &'static str {
    "This document is already a stamp collection — the names below are the \
     ones it carries."
}

/// The category field's label.
#[must_use]
pub const fn category_label() -> &'static str {
    "Category"
}

/// The category field's hint.
#[must_use]
pub const fn category_hint() -> &'static str {
    "The heading this set appears under in Acrobat's stamp menu."
}

/// The heading over the per-page rows.
#[must_use]
pub fn stamps_heading(count: usize) -> String {
    if count == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{count} stamps")
    }
}

/// The label on a row's page number.
#[must_use]
pub fn page_label(page_number: usize) -> String {
    format!("Page {page_number}")
}

/// The tooltip on a row's include checkbox.
#[must_use]
pub const fn include_hint() -> &'static str {
    "Uncheck to leave this page out of the collection. It stays in your \
     document either way — nothing here changes the file you have open."
}

/// The commit button.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save collection…"
}

/// The cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The native save dialog's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save stamp collection"
}

/// Why the Save button is greyed, when it is.
///
/// **R9**: greying is for *temporarily* unavailable, and both of these are
/// fixed by typing. Both are explained on hover, which is what this is for.
#[must_use]
pub const fn blocked_no_stamps() -> &'static str {
    "Every page is unchecked. A collection needs at least one stamp."
}

/// The second refusal — see [`blocked_no_stamps`].
#[must_use]
pub const fn blocked_no_category() -> &'static str {
    "Type a category name. Acrobat uses it as the heading for this set."
}

// ─────────────────────────────────────────────────────────────────────────
// The disclosures — R8b rule 4, off-canvas, before the write
// ─────────────────────────────────────────────────────────────────────────

/// The heading over the adjustment list.
///
/// Shown **only when there is something to say**. A permanently-present box
/// reading *"no adjustments"* trains an operator to stop reading the place
/// adjustments appear.
#[must_use]
pub const fn adjustments_heading() -> &'static str {
    "What pdfcer changed"
}

/// The explanation under [`adjustments_heading`].
///
/// ★ Says *why there are two names* — the single most confusing thing about
/// this file format, and the reason every adjustment below exists. Acrobat's
/// own files carry it (`SBApproved=Approved`) and an operator who does not
/// know about the hidden half will read every sentence below as pdfcer having
/// mangled what he typed.
#[must_use]
pub const fn adjustments_intro() -> &'static str {
    "Each stamp is stored under a hidden identifier as well as the name you \
     typed. Your names are written exactly as you typed them; these are the \
     identifiers pdfcer had to adjust."
}

/// One adjustment, as a sentence.
///
/// Takes the page number rather than the index — this is read by a person
/// beside a row that says `Page 3`.
#[must_use]
pub fn adjustment(page_number: usize, adjustment: &Adjustment) -> String {
    match adjustment {
        Adjustment::CharactersRemoved(1) => {
            format!("Page {page_number}: one character left out of the identifier.")
        }
        Adjustment::CharactersRemoved(n) => {
            format!("Page {page_number}: {n} characters left out of the identifier.")
        }
        // ★ The one adjustment with a consequence rather than a tidiness
        // reason, so it is the one that says WHY at length. A `#` marks a
        // stamp Acrobat expects to rewrite its own text; pdfcer writing one
        // would promise a recalculation that never happens.
        Adjustment::DynamicMarkerRemoved => format!(
            "Page {page_number}: the leading # was removed. Acrobat reads it as \
             a stamp that rewrites its own text when placed, which pdfcer does \
             not make."
        ),
        Adjustment::MadeUnique(name) => format!(
            "Page {page_number}: another stamp already had that name, so the \
             identifier became {name}. The name you typed is unchanged."
        ),
        Adjustment::Truncated => {
            format!("Page {page_number}: the identifier was shortened.")
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The outcome
// ─────────────────────────────────────────────────────────────────────────

/// The status line after a collection is written.
///
/// ★ Says **restart Acrobat**, because that is true and finding it out by
/// experiment costs an operator ten minutes of believing the feature is
/// broken. Acrobat scans its stamps folder at startup.
#[must_use]
pub fn saved(count: usize, path: &str) -> String {
    let stamps = if count == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{count} stamps")
    };
    format!("Saved {stamps} to {path}. Restart Acrobat to see them in its stamp menu.")
}

/// The disclosure when the engine named fewer stamps than were planned.
///
/// ⚠ **Should be unreachable.** The plan's name list and its page list are the
/// same filter over the same rows, so a shortfall means this shell built an
/// inconsistent pair — which is why the sentence blames pdfcer rather than the
/// document, and says the collection is short rather than implying it is fine.
#[must_use]
pub fn short_by(missing: usize) -> String {
    format!(
        "pdfcer wrote {missing} fewer stamps than planned. This is a fault in \
         pdfcer, not in your document — the collection is incomplete."
    )
}

/// A failed write, framed by stage, with the engine's own detail.
///
/// The frame is ours and the detail is the engine's, on `app::save`'s rule: a
/// sentence this shell invents about a failure it did not diagnose is a
/// sentence that will eventually be wrong.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The stamp collection could not be written. {detail}")
}

// ─────────────────────────────────────────────────────────────────────────
// Document Properties — reading a collection someone else wrote
// ─────────────────────────────────────────────────────────────────────────

/// The Document Properties section heading.
///
/// Appears **only** when the open document actually is a collection, which is
/// the name-tree test rather than the title test. An ordinary PDF shows
/// nothing here at all.
#[must_use]
pub const fn properties_heading() -> &'static str {
    "Stamp collection"
}

/// The category row's label in Document Properties.
#[must_use]
pub const fn properties_category_label() -> &'static str {
    "Category"
}

/// What a collection with no `/Info` `/Title` shows for its category.
///
/// ★ Not "Unknown". The category is genuinely **absent** from the file, which
/// is a fact about the file, and Acrobat would list this set without a
/// heading. Saying so is more useful than saying pdfcer does not know.
#[must_use]
pub const fn properties_no_category() -> &'static str {
    "None — Acrobat would list these without a heading"
}

/// The stamp-count row's label.
#[must_use]
pub const fn properties_count_label() -> &'static str {
    "Stamps"
}

/// The stamp-count row's value.
#[must_use]
pub fn properties_count(total: usize, dynamic: usize) -> String {
    let stamps = if total == 1 {
        "1 stamp".to_owned()
    } else {
        format!("{total} stamps")
    };
    match dynamic {
        0 => stamps,
        d if d == total => format!("{stamps}, all of them dynamic"),
        d => format!("{stamps}, {d} of them dynamic"),
    }
}

/// The explanation of what a dynamic stamp is, shown when there is one.
///
/// ★ Written so it reads as a **fact about the file**, not as a pdfcer
/// limitation notice. He is looking at somebody else's collection; what he
/// needs to know is what those entries do, not what pdfcer declines to author.
#[must_use]
pub const fn properties_dynamic_note() -> &'static str {
    "A dynamic stamp rewrites its own text when Acrobat places it — a date, a \
     name, a time. Its page here shows the text it was designed with."
}

/// One stamp's row in Document Properties.
///
/// Shows the display name, falling back to the identifier when the stored
/// string had no `=` at all. That fallback is the engine's reading of a
/// malformed entry reported rather than repaired, and showing the identifier
/// is the only true thing available.
#[must_use]
pub fn properties_stamp(display: &str, internal: &str) -> String {
    if display.is_empty() {
        internal.to_owned()
    } else {
        display.to_owned()
    }
}

/// The page a stamp names, for its row's second column.
#[must_use]
pub fn properties_stamp_page(page_number: usize) -> String {
    format!("page {page_number}")
}

/// The second column when a stamp names no page of **this** document.
///
/// # ★★ It is now a claim, and it was not before
///
/// This sentence used to be written defensively, because `page_index: None`
/// carried two opposite truths at once: a name pointing outside the document,
/// and a page tree that could not be read at all. Engine `Pass 290.1`
/// (2026-09-10) split them — see [`crate::stamps::page_tree_unreadable`] — so
/// this row is only ever drawn when the first is true, and the other case has
/// its own words in [`properties_stamp_page_unreadable`].
///
/// ⚠ **Still not *"this stamp is broken"*, and that is not leftover caution.**
/// A collection legitimately outlives the document it was cut from; Acrobat
/// shows such an entry too. *"Names no page in this document"* is a fact about
/// the pairing. *"Broken"* is a verdict on the file, and pdfcer does not have
/// the standing to make it.
#[must_use]
pub const fn properties_stamp_no_page() -> &'static str {
    "names no page in this document"
}

/// The second column when the **page tree** could not be read.
///
/// ★ Deliberately not *"names no page in this document"*. That sentence is a
/// statement about the stamp; this situation is a statement about the
/// document, and pdfcer knows nothing at all about which page this stamp
/// names. On the operator's own signature file the old, merged wording told
/// him both of his signatures pointed at nothing when both were fine.
///
/// The cause is not repeated on every row — it is the same cause for all of
/// them, and it is stated once by [`properties_page_tree_unreadable`].
#[must_use]
pub const fn properties_stamp_page_unreadable() -> &'static str {
    "page not known"
}

/// The one sentence that names why no stamp in this collection has a page.
///
/// # Rule 4 — disclosure, off-canvas, and not a refusal notice
///
/// The names, display titles and dynamic flags above are read from the file's
/// `/Names` → `/Pages` **name** tree and are entirely unaffected by whatever
/// is wrong with the **page** tree. So this says what pdfcer could not work
/// out and why, and explicitly says the list itself still stands — an operator
/// who reads *"the page tree could not be read"* and nothing else will
/// reasonably assume the twelve names above are suspect too.
///
/// `why` is the engine's own `PageTreeError` text, passed through rather than
/// paraphrased: a cycle in `/Kids` and a missing `/MediaBox` are different
/// repairs, and a shell that flattened both to *"damaged"* would cost him the
/// one word that says which.
#[must_use]
pub fn properties_page_tree_unreadable(why: &str) -> String {
    format!(
        "Which page each stamp names could not be worked out: this document's page tree could \
         not be read ({why}). The names above come from the file's name tree and are unaffected."
    )
}
