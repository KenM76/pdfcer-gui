//! # `anomalycensus` — one reading of `load_anomalies()`, for two
//! surfaces
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/anomalycensus.md`.

use pdfcer_core::document::LoadAnomaly;

use crate::text::anomalies as t;

/// How many of each class of contradiction this file contained.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Census {
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
    pub const fn is_clean(self) -> bool {
        self.unreadable == 0
            && self.duplicate_keys == 0
            && self.stream_lengths == 0
            && self.missing_endobj == 0
            && self.unknown == 0
    }
}

/// One census count paired with the catalog entry that puts it into words.
type Clause = (usize, fn(usize) -> String);

/// Count the anomalies by class.
pub fn census(anomalies: &[LoadAnomaly]) -> Census {
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
pub fn clauses(anomalies: &[LoadAnomaly]) -> Vec<String> {
    let c = census(anomalies);
    // The control, taken first and named. Almost every file an operator opens
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
pub fn status_line(anomalies: &[LoadAnomaly]) -> Option<String> {
    let clauses = clauses(anomalies);
    if clauses.is_empty() {
        return None;
    }
    Some(t::status_line(&clauses))
}

/// One sentence per anomaly, in the order the engine reported them.
pub fn rows(anomalies: &[LoadAnomaly]) -> Vec<String> {
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
            // The wildcard arm can destructure nothing, and that is exactly
            // what `LoadAnomaly::object()` and `LoadAnomaly::kind()` are for —
            // the engine documents `kind()` as "a short stable token for
            // machine-readable output". Between them the row still says which
            // object and what class, which is the whole of what can honestly be
            // said about a variant this build has never seen.
            other => t::unknown_row(other.object(), other.kind()),
        })
        .collect()
}
