//! # `app::status::anomalies` — one reading of `load_anomalies()`, for two
//! surfaces
//!
//! The derivation behind the status bar's *"this file contradicted itself"* line
//! and the Document-properties list underneath it. It counts, orders and words
//! nothing itself — [`crate::text::anomalies`] holds the words — but it decides
//! **which anomalies are reported, in what order, and how they are grouped**,
//! and it decides that **once**.
//!
//! ## ★★★ Why the derivation is split out rather than written at each site
//!
//! This is the same seam, for the same reason, as [`super::notes::findings`],
//! whose header states it plainly:
//!
//! > Split out … so that the status bar's one line and the Render-diagnostics
//! > dialog's list are the *same nine decisions* … Two tables would agree on the
//! > day they were written and disagree the first time a tenth counter was added
//! > to one of them, and the symptom would be two surfaces describing one raster
//! > differently, which is the worst available outcome for a *diagnostic*.
//!
//! Every word of that transfers. `LoadAnomaly` is `#[non_exhaustive]`; a fifth
//! variant is not a hypothetical, it is the thing
//! `tools/gates/check-engine-api-drift` exists to catch. One match statement is
//! one place to teach.
//!
//! ★ It lives under `app::status` rather than beside the document state because
//! [`super::notes`] set that precedent and `crate::dialogs::diagnostics` already
//! reaches across for it. A second convention for the same shape would cost a
//! reader more than the slight misfiling does.
//!
//! ## ★★★ The lifetime question, answered honestly
//!
//! Two of this module's neighbours in [`super::disclosure`] — the fill and edit
//! disclosures — are keyed on [`crate::app::state::OpenDoc::edit_epoch`] and
//! **retire on the next edit**. That is right for them: they describe *something
//! the operator just did*, and a later edit moves the document past it.
//!
//! A load anomaly is not that. It is true of the **file as it was opened**, and
//! it stays true for as long as that document is open — through every edit,
//! every undo, and every save. Keying it on `edit_epoch` would delete a
//! permanent fact the first time the operator nudged a line, which is precisely
//! the failure decision 145 was written against: the operator would be told once
//! and then quietly un-told.
//!
//! So there is **no epoch key here, and no cached state anywhere**. The census
//! is recomputed from `doc.session.document().load_anomalies()` on the frame it
//! is drawn, exactly as [`super::disclosure::recovered_disclosure`] recomputes
//! from `Document::recovery()`. The slice lives on the `Document`, is populated
//! at load, and is replaced wholesale when a different document is opened into
//! the tab — so there is nothing to clear, nothing to invalidate, and no way for
//! one file's anomalies to be shown against another's. State that must be
//! cleared is state that will one day be shown against the wrong document; this
//! has none.
//!
//! ⚠ The list is short by construction — one entry per contradiction in the file
//! — so recomputing per frame is a walk over a handful of items, not a parse.
//! The one file that motivated the feature has two.
//!
//! ## ★★ `recovery()` and `load_anomalies()` are different questions
//!
//! The shell already discloses `Document::recovery()`, and it would be easy to
//! assume this is the same fact twice. It is not, and the two are disjoint in
//! both directions:
//!
//! | | `recovery()` | `load_anomalies()` |
//! |---|---|---|
//! | fires when | the stored cross-reference table could not be parsed and pdfcer rebuilt the index by scanning | any object contradicted itself, **including on a file whose index was perfect** |
//! | the operator's file | index was fine | doubled `/PageMode`, and a `/Metadata` stream with no `/Length` |
//!
//! The file behind engine decision 145 has a **sound xref** and would light
//! neither the status line nor the Properties note that existed before today.
//! That is the gap this module closes.

use pdfcer_core::document::LoadAnomaly;

use crate::text::anomalies as t;

/// How many of each class of contradiction this file contained.
///
/// A named struct rather than a tuple so the two call sites read as prose and so
/// a fifth field cannot be added without every construction site being visited
/// by the compiler.
///
/// ★ `unknown` is not a bug counter. It is the count of anomalies reported by a
/// `LoadAnomaly` variant this build has no arm for, which becomes possible the
/// moment the engine adds one — see [`crate::text::anomalies::clause_unknown`]
/// for why silently dropping them would be the worst available behaviour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Census {
    /// [`LoadAnomaly::ObjectUnreadable`] — content the document does not have.
    pub unreadable: usize,
    /// [`LoadAnomaly::DuplicateDictKey`] — one key, two values, pdfcer chose.
    pub duplicate_keys: usize,
    /// [`LoadAnomaly::StreamLengthRecovered`] — `/Length` unusable, extent
    /// measured.
    pub stream_lengths: usize,
    /// [`LoadAnomaly::MissingEndobjRecovered`] — no `endobj`, ended at the next
    /// header.
    pub missing_endobj: usize,
    /// A variant this build predates.
    pub unknown: usize,
}

impl Census {
    /// True when the file contained no contradiction at all.
    ///
    /// The control the engine's notice calls out by name: *"An empty slice means
    /// the file was clean — that is the control, and it is tested."* Both
    /// surfaces draw nothing in this case; see
    /// [`crate::text::anomalies`]' header for why the all-clear is not stated
    /// out loud.
    pub const fn is_clean(self) -> bool {
        self.unreadable == 0
            && self.duplicate_keys == 0
            && self.stream_lengths == 0
            && self.missing_endobj == 0
            && self.unknown == 0
    }
}

/// One census count paired with the catalog entry that puts it into words.
///
/// A named type for the reason `super::notes`' own `NoteEntry` is one: it makes
/// [`clauses`]' table read as a table, and the `fn(usize) -> String` half is a
/// plain function pointer rather than a closure so the pairing is a *lookup* a
/// reviewer can read straight down the page.
type Clause = (usize, fn(usize) -> String);

/// Count the anomalies by class.
///
/// ⚠ The `_` arm is **not** dead code and must never become an `unreachable!()`.
/// [`LoadAnomaly`] is `#[non_exhaustive]`, which means a newer `pdfcer-core`
/// compiles against this shell without the compiler saying a word about a
/// variant that has no arm here. The gate catches the API growing; this arm
/// catches what the operator sees in the window between the engine growing and
/// this file being taught.
pub(crate) fn census(anomalies: &[LoadAnomaly]) -> Census {
    let mut census = Census::default();
    for anomaly in anomalies {
        match anomaly {
            LoadAnomaly::ObjectUnreadable { .. } => census.unreadable += 1,
            LoadAnomaly::DuplicateDictKey { .. } => census.duplicate_keys += 1,
            LoadAnomaly::StreamLengthRecovered { .. } => census.stream_lengths += 1,
            LoadAnomaly::MissingEndobjRecovered { .. } => census.missing_endobj += 1,
            _ => census.unknown += 1,
        }
    }
    census
}

/// The census as an ordered list of clauses, ready to be joined into one line.
///
/// Empty when the file was clean, which is how the caller knows to draw nothing.
///
/// # ★★ The order is the argument
///
/// Most consequential first, because the bar **truncates** (R128) and truncation
/// drops from the right. Whatever an operator's window width leaves room for, he
/// should be reading the gravest clause:
///
/// 1. **Objects that could not be read** — the only class where content is
///    *absent*. Every other class is a choice between two readings of content
///    that is present.
/// 2. **Duplicate keys** — pdfcer picked one of two values the file offered, and
///    on a drawing a wrong pick is a line in the wrong place on a page that
///    renders perfectly.
/// 3. **Stream lengths** and 4. **missing terminators** — the file is damaged
///    and pdfcer measured what the file failed to state. There was no second
///    reading to choose between, so there is nothing here the operator could
///    have decided differently.
/// 5. **Unknown** — last, because it is the only clause that is about *this
///    build* rather than about the file.
pub(crate) fn clauses(anomalies: &[LoadAnomaly]) -> Vec<String> {
    let c = census(anomalies);
    // ★ The control, taken first and named. Almost every file an operator opens
    // is clean, and this is the branch that says so out loud rather than leaving
    // "no clauses survived the filter" to be inferred from an empty vector two
    // statements later. It costs nothing and it is the one line a reader looking
    // for *"what happens to a healthy document?"* will find.
    if c.is_clean() {
        return Vec::new();
    }
    // A table rather than five `if`s, for the reason `notes::findings` uses one:
    // a reviewer checking that every counted class has a sentence has exactly
    // one place to look, and the order is visible as an order.
    let entries: [Clause; 5] = [
        (c.unreadable, t::clause_unreadable),
        (c.duplicate_keys, t::clause_duplicate_keys),
        (c.stream_lengths, t::clause_stream_lengths),
        (c.missing_endobj, t::clause_missing_endobj),
        (c.unknown, t::clause_unknown),
    ];
    entries
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, render)| render(n))
        .collect()
}

/// The status bar's sentence, or `None` for a file that contained no
/// contradiction.
///
/// `Option` rather than an empty `String` so the caller cannot accidentally draw
/// an empty row: an allocated, empty, hoverable rect in the status bar is a
/// disclosure that reads as a rendering bug, and R9 says an inapplicable
/// capability renders **nothing**.
pub(crate) fn status_line(anomalies: &[LoadAnomaly]) -> Option<String> {
    let clauses = clauses(anomalies);
    if clauses.is_empty() {
        return None;
    }
    Some(t::status_line(&clauses))
}

/// One sentence per anomaly, in the order the engine reported them.
///
/// # ★ Why these are NOT re-ordered the way [`clauses`] is
///
/// The bar's census is a summary and is sorted by consequence. This is the list
/// the operator reads when he has gone looking for *which*, and the engine's
/// order is the file's own order — roughly the order the objects appear in the
/// bytes. Sorting it by class would scatter the two defects of one broken object
/// (the operator's file has a doubled key *and*, one object later, a stream with
/// no `/Length`) into two distant groups, and locating a fault in a file is a
/// positional job.
///
/// ⚠ Unbounded in length: a badly damaged file can report hundreds. The caller
/// draws this inside the Properties panel's existing scroll area — never in the
/// status bar, which gets the census and nothing else.
pub(crate) fn rows(anomalies: &[LoadAnomaly]) -> Vec<String> {
    anomalies
        .iter()
        .map(|anomaly| match anomaly {
            LoadAnomaly::DuplicateDictKey {
                object,
                key,
                kept,
                discarded,
            } => t::duplicate_key_row(*object, key, kept, discarded),
            LoadAnomaly::StreamLengthRecovered { object } => t::stream_length_row(*object),
            LoadAnomaly::MissingEndobjRecovered { object } => t::missing_endobj_row(*object),
            LoadAnomaly::ObjectUnreadable { object, reason } => t::unreadable_row(*object, reason),
            // ★★ The wildcard arm can destructure nothing, and that is exactly
            // what `LoadAnomaly::object()` and `LoadAnomaly::kind()` are for —
            // the engine documents `kind()` as "a short stable token for
            // machine-readable output". Between them the row still says which
            // object and what class, which is the whole of what can honestly be
            // said about a variant this build has never seen.
            other => t::unknown_row(other.object(), other.kind()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::{Name, ObjId, Object};

    /// The file behind engine decision 145, as far as this shell can build it:
    /// a doubled `/PageMode` in object 57, and a stream one object later whose
    /// `/Length` was unusable.
    fn the_operators_file() -> Vec<LoadAnomaly> {
        vec![
            LoadAnomaly::DuplicateDictKey {
                object: Some(ObjId::new(57, 0)),
                key: b"PageMode".to_vec(),
                kept: Object::Name(Name(b"UseOutlines".to_vec())),
                discarded: Object::Name(Name(b"UseOC".to_vec())),
            },
            LoadAnomaly::StreamLengthRecovered {
                object: ObjId::new(58, 0),
            },
        ]
    }

    /// A file with no contradiction produces no line at all.
    ///
    /// ★ The control the engine's notice names. The assertion is `None`, not an
    /// empty string: an empty status line would still allocate a row.
    #[test]
    fn a_clean_file_discloses_nothing() {
        assert!(census(&[]).is_clean());
        assert_eq!(status_line(&[]), None);
        assert!(rows(&[]).is_empty());
    }

    /// The operator's own file reaches the bar as one line naming both defects.
    #[test]
    fn the_operators_file_reaches_the_status_line() {
        let line = status_line(&the_operators_file()).expect("two anomalies is not clean");
        assert!(line.contains("1 duplicate dictionary key"), "{line}");
        assert!(
            line.contains("1 stream whose declared length was wrong"),
            "{line}"
        );
        assert!(!line.contains('\n'), "the bar gets one line: {line}");
    }

    /// The census is ordered by consequence, and the order is pinned whole.
    ///
    /// ★ Every class at once, deliberately. A weaker fixture — two classes, one
    /// of them absent — is satisfied by orderings that differ from the intended
    /// one, so it would pass against a table somebody had shuffled. The property
    /// being asserted is *"most consequential first, because the bar truncates
    /// from the right"*, and only the full sequence states it.
    #[test]
    fn the_census_is_ordered_by_consequence() {
        let anomalies = vec![
            LoadAnomaly::MissingEndobjRecovered {
                object: ObjId::new(1, 0),
            },
            LoadAnomaly::StreamLengthRecovered {
                object: ObjId::new(2, 0),
            },
            LoadAnomaly::DuplicateDictKey {
                object: None,
                key: b"Size".to_vec(),
                kept: Object::Integer(1),
                discarded: Object::Integer(2),
            },
            LoadAnomaly::ObjectUnreadable {
                object: ObjId::new(3, 0),
                reason: "parse error".to_owned(),
            },
        ];
        assert_eq!(
            clauses(&anomalies),
            vec![
                "1 object it could not read".to_owned(),
                "1 duplicate dictionary key".to_owned(),
                "1 stream whose declared length was wrong".to_owned(),
                "1 object with no end marker".to_owned(),
            ],
            "most consequential first; see `clauses`' ordering note"
        );
    }

    /// Every anomaly gets exactly one row, and the rows keep the engine's order.
    ///
    /// The order half is the point: the two defects of one damaged region of the
    /// file must stay adjacent.
    #[test]
    fn every_anomaly_gets_one_row_in_file_order() {
        let anomalies = the_operators_file();
        let rows = rows(&anomalies);
        assert_eq!(rows.len(), anomalies.len());
        assert!(rows[0].contains("/PageMode"), "{rows:?}");
        assert!(rows[1].contains("Object 58 0"), "{rows:?}");
    }

    /// Both values of a duplicate key survive all the way to the panel row.
    ///
    /// ★ This is the assertion that distinguishes a built feature from a counted
    /// one. The engine carries `kept` and `discarded` specifically so a shell can
    /// show what pdfcer chose between; if only the count arrives, the operator's
    /// question — *"what if it is the wrong one?"* — has no answer on screen.
    #[test]
    fn the_panel_row_shows_what_pdfcer_chose_between() {
        let rows = rows(&the_operators_file());
        let row = rows.first().expect("the duplicate key is first");
        assert!(row.contains("/UseOutlines"), "kept value missing: {row}");
        assert!(row.contains("/UseOC"), "discarded value missing: {row}");
    }

    /// An unreadable object's row carries the object id and the engine's reason.
    #[test]
    fn an_unreadable_object_names_itself_and_why() {
        let rows = rows(&[LoadAnomaly::ObjectUnreadable {
            object: ObjId::new(9, 0),
            reason: "parse error at byte 43992".to_owned(),
        }]);
        let row = rows.first().expect("one anomaly, one row");
        assert!(row.contains("Object 9 0"), "{row}");
        assert!(row.contains("parse error at byte 43992"), "{row}");
    }
}
