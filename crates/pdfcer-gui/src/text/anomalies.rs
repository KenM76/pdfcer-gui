//! # `text::anomalies` — every word for *"the file contradicted itself, and
//! pdfcer decided"*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/anomalies.md`.

use pdfcer_core::object::{ObjId, Object};

/// Render a PDF name-like byte string the way an operator sees it in a file —
/// `/PageMode`.
#[must_use]
pub fn key_name(key: &[u8]) -> String {
    format!("/{}", String::from_utf8_lossy(key))
}

/// One PDF value, short enough to sit inside a sentence.
#[must_use]
pub fn value(object: &Object) -> String {
    object.to_string()
}

/// How a row names the object an anomaly is about.
#[must_use]
pub fn subject(object: Option<ObjId>) -> String {
    match object {
        Some(id) => format!("Object {id}"),
        None => "The file's trailer".to_owned(),
    }
}

/// The status bar's one line, built from the clause list
/// [`crate::app::status::anomalies::clauses`] produced.
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
#[must_use]
pub const fn reread_first_button() -> &'static str {
    "Read this file again, using the first value at every one of these"
}

/// **The same button when the file is already open under the first values** —
/// it offers pdfcer's ordinary reading back.
#[must_use]
pub const fn reread_last_button() -> &'static str {
    "Read this file again, using pdfcer's usual choice (the last value)"
}

/// The hover sentence on either re-read button.
#[must_use]
pub const fn reread_tooltip() -> &'static str {
    "pdfcer reads the file from disk again, so anything you have edited in this \
     document since opening it is not carried over — you are asked about that \
     first. Nothing is written to the file either way; this changes only how \
     pdfcer reads it."
}

/// *"Object 57 0 named /PageMode twice. pdfcer kept /UseOutlines and left
/// /UseOC."*
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
