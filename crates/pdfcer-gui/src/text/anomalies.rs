//! # `text::anomalies` — every word for *"the file contradicted itself, and
//! pdfcer decided"*
//!
//! The catalog behind the two surfaces that disclose
//! [`pdfcer_core::document::Document::load_anomalies`]: the status bar's one
//! elided line ([`crate::app::status::disclosure`]) and the Document-properties
//! detail beneath it ([`crate::panels::docprops`]). Both read the **same**
//! derivation in [`crate::app::status::anomalies`], so the words here are
//! written once and cannot drift between the glance and the answer.
//!
//! ## ★★★ What this catalog is actually about, and why the wording is careful
//!
//! Engine `Pass 283.0`, decision 145 — *"fail-clean never meant refuse"* — was
//! written against a real file of the operator's. A 46 KB drawing that opens in
//! Acrobat was **refused whole** by pdfcer because its document catalog names
//! `/PageMode` twice with two different values. His ruling, verbatim:
//!
//! > *"acrobat just picks one — but what if it is the wrong one? Would be great
//! > if the user could choose or also have pdfcer pick one for them. We should
//! > be making pdfcer so that it opens pdfs that have errors, and have a way
//! > that it manages those errors such that they aren't fatal, and if the user
//! > can intervene in a decision that should always be an option along with them
//! > not having to intervene."*
//!
//! Three obligations fall out of that sentence and every string below is shaped
//! by them:
//!
//! 1. **Not fatal.** The document is open and usable before a word of this is
//!    read. So nothing here is phrased as a problem to resolve, a question to
//!    answer, or an action to take — the operator may read none of it and lose
//!    nothing.
//! 2. **The operator can see WHAT WAS CHOSEN BETWEEN.** For a duplicate key the
//!    engine deliberately carries *both* values rather than a count, because
//!    *"a count says pdfcer chose; only the pair lets you show the operator what
//!    it chose between"*. [`duplicate_key_row`] is the reason that pair exists,
//!    and it is why [`value`] is in this file rather than a number being
//!    printed.
//! 3. **The file is the subject, not pdfcer.** Every sentence names what *the
//!    file* said, then what pdfcer did about it. `crate::text::panels::docprops`
//!    settled this register for the recovered-index note next door — *"pdfcer
//!    had to repair this file" would read as pdfcer struggling; the file is the
//!    thing that is damaged, and the operator's next question is about the
//!    file.*
//!
//! ## ★★ Why there is no "this file opened cleanly" string
//!
//! There is a true one available — `load_anomalies()` is empty for a sound file
//! and the engine tests that control — and it is deliberately not written.
//! `DEFECTS.md`'s "Not defects" table records what a permanent all-clear does to
//! a status bar: *"The first thing a user reads is the app talking about
//! itself."* A reassurance shown on every document forever costs bar width on
//! every document forever, and **R9** says an inapplicable capability renders
//! nothing rather than a placeholder saying it is inapplicable. Both surfaces
//! draw nothing for a clean file, exactly as the recovered-index note beside
//! them does.
//!
//! ## Contract with the rest of the crate
//!
//! Pure functions over already-derived values. Nothing here reads a
//! [`pdfcer_core::document::Document`], counts anything, or decides what to
//! show; that is [`crate::app::status::anomalies`]' job. This file only turns
//! decided facts into English, which is what makes **R4** (`check-ui-strings`)
//! satisfiable: the two drawing sites contain no literal an operator can read.

use pdfcer_core::object::{ObjId, Object};

/// Render a PDF name-like byte string the way an operator sees it in a file —
/// `/PageMode`.
///
/// ★ Lossy UTF-8 rather than a refusal. A PDF name is *raw bytes* (§7.3.5:
/// interpretation as UTF-8 applies only where a name is used as text), so a key
/// that is not valid UTF-8 is legal and reachable. Refusing to name it would
/// hide the one thing the row exists to say — *which* key was doubled — behind
/// an encoding technicality the operator did not create and cannot fix.
#[must_use]
pub fn key_name(key: &[u8]) -> String {
    format!("/{}", String::from_utf8_lossy(key))
}

/// One PDF value, short enough to sit inside a sentence.
///
/// # ★★★ Why this exists at all
///
/// [`pdfcer_core::object::Object`] has no `Display`, only `Debug`, and `Debug`
/// on a `Dict` is a whole object graph. This is the *sentence-sized* rendering:
/// the scalar kinds are shown exactly, and the three container kinds are named
/// rather than expanded.
///
/// That asymmetry is the design, not a shortcut. The pair this function serves
/// is `kept` versus `discarded` on a duplicate key, and the question the
/// operator is answering is *"did pdfcer pick the right one?"*. For the case
/// that motivated the whole Pass — `/PageMode /UseOC` against
/// `/PageMode /UseOutlines` — the answer is two short names side by side, and it
/// is readable at a glance. For a doubled key whose values are two large
/// dictionaries there is no glance-sized answer, and pretending otherwise by
/// dumping a graph into a status-bar sentence would make the readable case
/// unreadable too. Naming the shape ("a dictionary of 14 entries") is the honest
/// amount to say, and it still tells the operator the two values differ in kind
/// or in size when they do.
///
/// ⚠ [`pdfcer_core::object::Object`] is `#[non_exhaustive]`, so the wildcard arm
/// is genuinely reachable rather than a formality — a value kind a later engine
/// adds must produce *a* description here, not a compile error and not a blank.
#[must_use]
pub fn value(object: &Object) -> String {
    match object {
        Object::Null => "null".to_owned(),
        Object::Boolean(b) => if *b { "true" } else { "false" }.to_owned(),
        Object::Integer(n) => n.to_string(),
        Object::Real(r) => r.to_string(),
        // Quoted, because a PDF string's content can be empty or all spaces and
        // an unquoted empty string in the middle of a sentence reads as a bug in
        // the sentence.
        Object::String(bytes) => format!("\u{201c}{}\u{201d}", String::from_utf8_lossy(bytes)),
        Object::Name(name) => key_name(name.as_bytes()),
        Object::Array(items) => match items.len() {
            1 => "an array of 1 item".to_owned(),
            n => format!("an array of {n} items"),
        },
        Object::Dict(dict) => match dict.len() {
            1 => "a dictionary of 1 entry".to_owned(),
            n => format!("a dictionary of {n} entries"),
        },
        Object::Stream(_) => "a stream".to_owned(),
        // `num gen R` — the form the reference is written in inside the file, so
        // an operator comparing this against the bytes finds it.
        Object::Reference(id) => format!("{id} R"),
        _ => "a value this build does not recognise".to_owned(),
    }
}

/// How a row names the object an anomaly is about.
///
/// ★ `None` is a real answer, not a missing one:
/// [`pdfcer_core::document::LoadAnomaly::DuplicateDictKey`] carries
/// `object: None` when the contradicting dictionary is the **trailer**, which is
/// not an indirect object and therefore has no id. Saying "the file's trailer"
/// is more useful than any placeholder id, and it is the only case where a
/// reader would otherwise wonder which object "object 0 0" meant.
#[must_use]
pub fn subject(object: Option<ObjId>) -> String {
    match object {
        Some(id) => format!("Object {id}"),
        None => "The file's trailer".to_owned(),
    }
}

/// The status bar's one line, built from the clause list
/// [`crate::app::status::anomalies::clauses`] produced.
///
/// # ★★ Why it names the panel
///
/// The bar answers *"is there something I should know?"* and has room for
/// nothing else — **R128** caps it at one elided row. The clause list is a
/// census, so the operator who reads it knows *that* pdfcer chose and *how many
/// times*, and the next question is always *which*. The recovered-index line
/// beside it settled this shape on 2026-08-26 and points at the same panel for
/// the same reason; two disclosure lines that sent the operator to two different
/// places would be worse than either.
#[must_use]
pub fn status_line(clauses: &[String]) -> String {
    format!(
        "This file contradicted itself and pdfcer decided rather than refusing to open it: {}. Document properties says which.",
        clauses.join(", ")
    )
}

/// *"3 duplicate dictionary keys"* — one clause of the census.
///
/// Singular and plural written out rather than an `(s)`, which is the register
/// every other counted sentence in this crate uses.
#[must_use]
pub fn clause_duplicate_keys(n: usize) -> String {
    match n {
        1 => "1 duplicate dictionary key".to_owned(),
        n => format!("{n} duplicate dictionary keys"),
    }
}

/// *"2 streams whose declared length was wrong"* — one clause of the census.
#[must_use]
pub fn clause_stream_lengths(n: usize) -> String {
    match n {
        1 => "1 stream whose declared length was wrong".to_owned(),
        n => format!("{n} streams whose declared length was wrong"),
    }
}

/// *"2 objects with no end marker"* — one clause of the census.
#[must_use]
pub fn clause_missing_endobj(n: usize) -> String {
    match n {
        1 => "1 object with no end marker".to_owned(),
        n => format!("{n} objects with no end marker"),
    }
}

/// *"2 objects it could not read"* — one clause of the census.
///
/// ★ The gravest clause, and the wording says so without alarm: this is the one
/// class where content is **absent from the document**, not merely chosen
/// between. [`crate::app::status::anomalies::clauses`] orders it first so a
/// truncating bar drops it last — see that function's ordering note.
#[must_use]
pub fn clause_unreadable(n: usize) -> String {
    match n {
        1 => "1 object it could not read".to_owned(),
        n => format!("{n} objects it could not read"),
    }
}

/// *"2 contradictions this build has no words for"* — the census clause for a
/// [`pdfcer_core::document::LoadAnomaly`] variant added after this shell was
/// built.
///
/// # ★★★ Why an unknown class is counted out loud instead of being skipped
///
/// `LoadAnomaly` is `#[non_exhaustive]`, so a newer engine can report a class
/// this build cannot name, and the `_` arm that catches it is not decoration —
/// `tools/gates/check-engine-api-drift` exists precisely because the compiler
/// will *not* raise its hand. Dropping such an anomaly on the floor would make
/// two very different states identical from a chair: *the file is clean* and
/// *the file contradicted itself in a way this build cannot describe*. The first
/// is a claim; the second is an admission, and only one of them is true.
#[must_use]
pub fn clause_unknown(n: usize) -> String {
    match n {
        1 => "1 contradiction this build has no words for".to_owned(),
        n => format!("{n} contradictions this build has no words for"),
    }
}

/// The Document-properties heading above the detail rows.
///
/// Same register as the recovered-index heading directly above it: the file is
/// the subject, and the sentence states a fact rather than raising an alarm.
#[must_use]
pub const fn heading() -> &'static str {
    "This file contradicted itself, and pdfcer chose how to read it"
}

/// The one line of context under the heading.
///
/// ★★ It says the document is fine *as opened* and that the choices are pdfcer's
/// documented defaults, because without that the list underneath reads as a list
/// of damage the operator is expected to repair. There is nothing to repair
/// here; saying so is what keeps the disclosure from behaving like a prompt,
/// which decision 059 and the engine's own notice both forbid by name.
///
/// ★★★ **Corrected 2026-09-10.** This doc used to end *"and no control that
/// would repair it"*, which was true when it was written and stopped being true
/// the day [`reread_first_button`] landed under these rows. The sentence itself
/// did not need changing and has not changed — *"the document is complete and
/// usable as it is"* is exactly what a control **underneath** must not
/// contradict, and it is what makes pressing the button a choice rather than a
/// repair. What needed changing was the doc's claim about the shell, and a
/// stale limitation sentence is a defect in whoever believes it.
#[must_use]
pub const fn note() -> &'static str {
    "Other programs open this file too, and make their own choices in the same \
     places. Everything below is what pdfcer chose; the document is complete \
     and usable as it is."
}

/// The hover explanation on the detail block.
#[must_use]
pub const fn tooltip() -> &'static str {
    "A PDF can say two different things in the same place — the same dictionary key twice with different values, a stream whose stated length does not match its contents, an object with no end marker. pdfcer used to refuse a file like that outright. It now reads it the way other PDF programs do and lists every place it had to decide, so that if something looks wrong on the page you know where the file was ambiguous."
}

/// **The button that reads the file again, taking the FIRST of each pair.**
///
/// # ★★★ The third of the operator's three obligations
///
/// > *"...and if the user can intervene in a decision that should always be an
/// > option along with them not having to intervene."*
///
/// The document opens (not fatal) and nothing has to be answered first
/// (intervention not required); this is the sentence that makes intervention
/// *possible*. Until 2026-09-10 the panel said what pdfcer had chosen and
/// offered no way to choose otherwise, which is the difference between being
/// told and being asked.
///
/// # ★★ Why it names the VALUE and not the policy
///
/// The engine's term is `DuplicateKeyPolicy::KeepFirst`, and *"keep first"* is
/// meaningless to an operator looking at a titleblock. The rows above have just
/// said *"pdfcer kept /UseOutlines and left /UseOC"*, so the button says the
/// thing that follows from those rows: take the one it left. The word *"first"*
/// appears because the rows are ordered and the operator can see which is which.
///
/// ⚠ **Whole file, not this row.** `LoadOptions::with_duplicate_keys` sets one
/// policy for the entire load — there is no per-key form — so the label must not
/// promise one. *"every"* is doing that work and is not decoration.
#[must_use]
pub const fn reread_first_button() -> &'static str {
    "Read this file again, using the first value at every one of these"
}

/// **The same button when the file is already open under the first values** —
/// it offers pdfcer's ordinary reading back.
///
/// ★★ **R9, and the reason there are two strings rather than one greyed
/// control.** A re-read is not a toggle whose off state is unavailable: after
/// re-reading, the *other* choice is exactly as available as this one was, so
/// the honest control is one button whose label names whichever reading the
/// operator does not currently have. A disabled *"use the first value"* button
/// on a document already read that way would be a placeholder describing a
/// state the operator is standing in.
///
/// ★ It says *usual* rather than *default*, and *last* rather than *KeepLast*,
/// for [`reread_first_button`]'s reason: the operator is choosing between two
/// values they can see, not between two settings.
#[must_use]
pub const fn reread_last_button() -> &'static str {
    "Read this file again, using pdfcer's usual choice (the last value)"
}

/// The hover sentence on either re-read button.
///
/// # ★★★ It states the cost, because the button cannot show it
///
/// Pressing this closes the document and parses the bytes on disk again. The
/// tab does not go away, the path does not change and the pages look identical
/// — and every edit made since the file was opened is gone, because the
/// intervention is a re-load rather than a patch. That is the single most
/// surprising fact about this control and it is the first clause here.
///
/// ⚠ It is not the whole guard. `crate::app::actions::document`'s
/// `apply_reread_with_duplicate_keys` asks about unsaved edits before anything
/// is discarded, exactly as a close does. A tooltip is a warning; the prompt is
/// the protection, and an operator who never hovers still keeps their work.
#[must_use]
pub const fn reread_tooltip() -> &'static str {
    "pdfcer reads the file from disk again, so anything you have edited in this      document since opening it is not carried over — you are asked about that      first. Nothing is written to the file either way; this changes only how      pdfcer reads it."
}

/// *"Object 57 0 named /PageMode twice. pdfcer kept /UseOutlines and left
/// /UseOC."*
///
/// # ★★★ The row the whole feature is for
///
/// `kept` and `discarded` are both carried out of the engine — see
/// [`pdfcer_core::document::LoadAnomaly::DuplicateDictKey`], whose own comment
/// says a count *"makes the intervention theoretical"*. This sentence is where
/// that pair becomes readable, and it answers the operator's exact question —
/// *"what if it is the wrong one?"* — as far as this build can: he can see both
/// values and judge.
///
/// ★★★ **And, since 2026-09-10, take the other one.** Choosing it is a re-load
/// rather than an edit — the engine is explicit that *"a decision made during
/// parsing is not a value that can be edited afterwards"* — and the button that
/// performs that re-load is drawn directly under these rows by
/// `crate::panels::docprops`. This row is what makes it meaningful: without both
/// values on screen, *use the first value instead* is a button an operator has
/// no basis to press. (This paragraph used to say the ask *"is filed in
/// `ENGINE_BACKLOG.md`"*. It was, as rows 280 and 281; both are wired.)
///
/// ★ "left" rather than "discarded" or "threw away". The discarded value is
/// still in the file and still visible to any other reader; pdfcer did not
/// destroy anything, it declined to use one of two things the file offered.
#[must_use]
pub fn duplicate_key_row(
    object: Option<ObjId>,
    key: &[u8],
    kept: &Object,
    discarded: &Object,
) -> String {
    format!(
        "{} named {} twice. pdfcer kept {} and left {}.",
        subject(object),
        key_name(key),
        value(kept),
        value(discarded),
    )
}

/// *"Object 12 0: the stream's stated length was unusable, so pdfcer measured
/// the data to its end marker instead."*
///
/// ★ No alternative is offered because there is none — the engine's own comment
/// on this variant says *"the alternative to the scanned extent is no object at
/// all. The record exists so the operator learns the file is damaged, not so a
/// decision can be re-taken."* The sentence therefore states what happened and
/// stops, and it does **not** invite an action.
#[must_use]
pub fn stream_length_row(object: ObjId) -> String {
    format!(
        "Object {object}: the stream's stated length was unusable, so pdfcer \
         measured the data to its end marker instead."
    )
}

/// *"Object 12 0: the object had no end marker, so pdfcer ended it where the
/// next object begins."*
///
/// Same shape and same absence of a choice as [`stream_length_row`]: a required
/// keyword the file omits and an unambiguous end.
#[must_use]
pub fn missing_endobj_row(object: ObjId) -> String {
    format!(
        "Object {object}: the object had no end marker, so pdfcer ended it \
         where the next object begins."
    )
}

/// *"Object 12 0 could not be read at all, so the document opened without it."*
///
/// # ★★ The one class where something is genuinely missing
///
/// The others are choices between two readings; this one is content the document
/// does not contain. It is still not an error: §7.3.10 says a reference to an
/// undefined object *"shall not be considered an error by a conforming reader;
/// it shall be treated as a reference to the null object"*, so the document
/// pdfcer produced is one the standard describes rather than a repair pdfcer
/// invented. The sentence says the consequence plainly — something a page
/// referred to is not there — because this is the only class here where the
/// operator might see a difference on screen and needs to know why.
///
/// `reason` is the loader's own words, passed through rather than rewritten: it
/// names *which* failure, and a paraphrase would lose the only detail that
/// distinguishes one unreadable object from another.
#[must_use]
pub fn unreadable_row(object: ObjId, reason: &str) -> String {
    format!(
        "Object {object} could not be read at all, so the document opened \
         without it — anything that referred to it sees nothing there. pdfcer's \
         reason: {reason}"
    )
}

/// The detail row for a [`pdfcer_core::document::LoadAnomaly`] variant this
/// build does not know.
///
/// ★★ It prints the engine's own stable token from
/// [`pdfcer_core::document::LoadAnomaly::kind`], which is exactly what that
/// method is documented for — *"a short stable token for machine-readable
/// output"*. That token plus [`pdfcer_core::document::LoadAnomaly::object`] are
/// the **only** two things reachable through a wildcard arm, and between them
/// they still answer *which object* and *what class*, which is enough for the
/// operator to report it and enough for a maintainer to recognise which variant
/// went unwired.
#[must_use]
pub fn unknown_row(object: Option<ObjId>, kind: &str) -> String {
    format!(
        "{}: this file contained a contradiction reported as \u{201c}{kind}\u{201d}, \
         which this build of pdfcer has no description for. It was handled, not \
         ignored.",
        subject(object),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::{Dict, Name};

    /// The operator's own file, in one sentence.
    ///
    /// The literal case from engine decision 145: a catalog naming `/PageMode`
    /// twice, `/UseOC` against `/UseOutlines`. If this row cannot say both
    /// values the feature has not been built, only counted.
    #[test]
    fn the_duplicate_key_row_shows_both_values() {
        let row = duplicate_key_row(
            Some(ObjId::new(57, 0)),
            b"PageMode",
            &Object::Name(Name(b"UseOutlines".to_vec())),
            &Object::Name(Name(b"UseOC".to_vec())),
        );
        assert!(row.contains("Object 57 0"), "{row}");
        assert!(row.contains("/PageMode"), "{row}");
        assert!(row.contains("/UseOutlines"), "{row}");
        assert!(row.contains("/UseOC"), "{row}");
    }

    /// A trailer has no object id, and the row says so in words.
    #[test]
    fn a_trailer_duplicate_names_the_trailer() {
        let row = duplicate_key_row(None, b"Size", &Object::Integer(12), &Object::Integer(9));
        assert!(row.contains("trailer"), "{row}");
        assert!(row.contains("12") && row.contains('9'), "{row}");
    }

    /// Every value kind renders to something a sentence can hold.
    ///
    /// The container arms name a shape instead of expanding it; the assertion
    /// that matters is that none of them is empty and none of them contains a
    /// newline, because either would break the one-line bar (**R128**).
    #[test]
    fn every_value_kind_renders_to_one_short_line() {
        let values = [
            Object::Null,
            Object::Boolean(true),
            Object::Integer(-3),
            Object::Real(1.5),
            Object::String(b"hi".to_vec()),
            Object::Name(Name(b"UseOC".to_vec())),
            Object::Array(vec![Object::Null, Object::Null]),
            Object::Dict(Dict::new()),
            Object::Reference(ObjId::new(58, 0)),
        ];
        for object in &values {
            let text = value(object);
            assert!(!text.is_empty(), "{object:?} rendered to nothing");
            assert!(!text.contains('\n'), "{object:?} rendered to two lines");
        }
    }
}
