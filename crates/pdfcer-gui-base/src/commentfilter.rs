//! # `commentfilter` — narrowing the reviewer's work list
//!
//! One subject: **which rows the Comments panel shows, and in what order.**
//! The state, the pure predicate, and the control strip that sets them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/commentfilter.md`.

use crate::commentmodel::{CommentRow, Note};

/// How the list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Page order, then `/Annots` order — the document's own.
    ///
    /// The default, and it is the right one: it is where the comments *are*,
    /// which is how a reviewer working through a drawing set moves. Every
    /// other order is a question ("what did Ken say?") rather than a walk.
    #[default]
    Document,
    /// By `/T`, then document order within each author.
    Author,
    /// By `/Subtype`, then document order within each kind.
    Subtype,
}

impl Sort {
    /// Every ordering, for the chooser and for the sweep that asserts each one
    /// has a label.
    pub const ALL: &'static [Self] = &[Self::Document, Self::Author, Self::Subtype];
}

/// What the reviewer has narrowed the list to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Show only comments whose `/T` is this. `None` shows every author.
    ///
    /// The whole string, matched exactly rather than by substring: two
    /// reviewers called *Ken* and *Ken Mantle* are two people, and a substring
    /// match would fold one into the other silently.
    pub author: Option<String>,
    /// Show only comments of this `/Subtype`. `None` shows every kind.
    pub subtype: Option<String>,
    /// Show only comments that carry note text.
    ///
    /// `Note::Description` counts as **text**, and that is deliberate:
    /// §12.5.2 makes `/Contents` dual-purpose, and a `/Link`'s accessibility
    /// description is still somebody's words. The row already says which
    /// meaning it is; the filter's job is *"is there anything to read here"*.
    pub with_note_only: bool,
    /// **Show only comments at this review status** — `/State`, added
    /// 2026-09-06 when `pdfcer-core` `Pass 253.1` closed the engine gap this
    /// module's header recorded as absent.
    ///
    /// # It lives here but is NOT evaluated by [`Self::keeps`], and that is
    /// deliberate rather than an oversight
    ///
    /// **A review status is not on the row, because it is not on the
    /// annotation.** §12.5.6.3 is explicit — *"the state is not specified in
    /// the annotation itself but in a separate text annotation that refers to
    /// the original annotation by means of its `IRT` entry"* — so answering
    /// *"is this comment accepted?"* needs the **whole document**, not this
    /// row. [`Self::keeps`] takes one [`CommentRow`] and could only answer it
    /// by guessing.
    ///
    /// So the predicate lives where the reading does:
    /// [`crate::commentreviewstate::Statuses::keeps`], applied by
    /// [`crate::commentreviewstate::narrow`] after [`apply`] has run. Two functions,
    /// because there are genuinely two questions — one about a row, one about a
    /// document.
    ///
    /// # Why the STATE is here anyway, when the predicate is not
    ///
    /// Because two properties of this panel hang off this struct and both would
    /// silently break if the field lived elsewhere:
    ///
    /// - [`Self::is_narrowing`] is what draws
    ///   `crate::text::panels::comments::comments_filtered`. A status filter
    ///   that did not count would hide rows with **no disclosure that anything
    ///   was hidden**, which is the one thing this panel's founding rule
    ///   forbids.
    /// - *Show all* clears this struct. A narrowing the operator could set and
    ///   not lift is the trap version of a filter.
    ///
    /// ⇒ One place for *what the operator asked for*; two places for *how it is
    /// answered*. The alternative — a second filter state beside this one —
    /// would have made both of those an ongoing act of memory.
    pub status: Option<crate::commentreviewstate::StatusChoice>,
    /// How the surviving rows are ordered.
    pub sort: Sort,
}

impl Filter {
    /// **Is this filter hiding anything?**
    #[must_use]
    pub fn is_narrowing(&self) -> bool {
        self.author.is_some()
            || self.subtype.is_some()
            || self.with_note_only
            || self.status.is_some()
    }

    /// Does one row survive?
    #[must_use]
    pub fn keeps(&self, row: &CommentRow) -> bool {
        if let Some(author) = &self.author
            && row.author.as_deref().map(str::trim) != Some(author.as_str())
        {
            return false;
        }
        if let Some(subtype) = &self.subtype
            && &row.subtype != subtype
        {
            return false;
        }
        if self.with_note_only && matches!(row.note, Note::Absent) {
            return false;
        }
        true
    }
}

/// **Narrow and order a listing's rows.**
#[must_use]
pub fn apply(rows: Vec<CommentRow>, filter: &Filter) -> Vec<CommentRow> {
    let mut kept: Vec<CommentRow> = rows.into_iter().filter(|r| filter.keeps(r)).collect();
    match filter.sort {
        // Already in document order — `model::collect` walks pages in order
        // and `/Annots` in order — so this arm does nothing at all. Written as
        // an explicit no-op rather than an early return so that adding a
        // fourth ordering cannot forget to handle it.
        Sort::Document => {}
        // An author-less comment sorts to the END rather than the start.
        // `/T` is legitimately absent — it means anonymous — and a reviewer
        // ordering by author is looking for a *person*; putting the unsigned
        // rows first would bury the thing they asked for under the thing they
        // did not. `None` > `Some` is not the derived order, so the key is
        // built to say it.
        Sort::Author => kept.sort_by_key(|r| {
            let author = r.author.as_deref().map(str::trim).unwrap_or_default();
            (author.is_empty(), author.to_lowercase())
        }),
        Sort::Subtype => kept.sort_by_key(|r| r.subtype.to_lowercase()),
    }
    kept
}

/// Every distinct author in a listing, in the order a chooser should offer
/// them.
#[must_use]
pub fn authors(rows: &[CommentRow]) -> Vec<String> {
    let mut seen: Vec<String> = rows
        .iter()
        .filter_map(|r| r.author.as_deref())
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .map(str::to_owned)
        .collect();
    seen.sort_by_key(|a| a.to_lowercase());
    seen.dedup();
    seen
}

/// Every distinct `/Subtype` in a listing, alphabetically.
#[must_use]
pub fn subtypes(rows: &[CommentRow]) -> Vec<String> {
    let mut seen: Vec<String> = rows.iter().map(|r| r.subtype.clone()).collect();
    seen.sort_by_key(|s| s.to_lowercase());
    seen.dedup();
    seen
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::ObjId;

    fn row(page: usize, num: u32, subtype: &str, author: Option<&str>, note: Note) -> CommentRow {
        CommentRow {
            page_index: page,
            id: Some(ObjId::new(num, 0)),
            subtype: subtype.to_owned(),
            is_ce_dimension: false,
            note,
            author: author.map(str::to_owned),
            modified: None,
            suppressed: false,
            appearance_unresolved: false,
            relation: None,
            in_reply_to: None,
        }
    }

    fn sheet() -> Vec<CommentRow> {
        vec![
            row(0, 1, "Text", Some("Ken Mantle"), Note::Text("weld".into())),
            row(0, 2, "Square", Some("Jo Smith"), Note::Absent),
            row(1, 3, "Text", Some("Ken Mantle"), Note::Absent),
            row(2, 4, "Line", None, Note::Text("check".into())),
        ]
    }

    /// **The default filter hides nothing.**
    #[test]
    fn the_default_shows_everything() {
        let filter = Filter::default();
        assert!(!filter.is_narrowing());
        assert_eq!(apply(sheet(), &filter).len(), 4);
    }

    /// Filtering by author keeps that author's comments and only those.
    #[test]
    fn filtering_by_author_keeps_only_that_author() {
        let filter = Filter {
            author: Some("Ken Mantle".to_owned()),
            ..Filter::default()
        };
        let kept = apply(sheet(), &filter);
        assert_eq!(kept.len(), 2);
        assert!(
            kept.iter()
                .all(|r| r.author.as_deref() == Some("Ken Mantle"))
        );
        assert!(filter.is_narrowing());
    }

    /// **An exact match, not a substring.**
    #[test]
    fn an_author_filter_does_not_match_a_prefix() {
        let filter = Filter {
            author: Some("Ken".to_owned()),
            ..Filter::default()
        };
        assert!(apply(sheet(), &filter).is_empty());
    }

    /// Filtering by type keeps that type and only that type.
    #[test]
    fn filtering_by_type_keeps_only_that_type() {
        let filter = Filter {
            subtype: Some("Text".to_owned()),
            ..Filter::default()
        };
        let kept = apply(sheet(), &filter);
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|r| r.subtype == "Text"));
    }

    /// **"With text only" drops the rows pdfcer's own markup produces.**
    #[test]
    fn with_text_only_drops_the_noteless_rows() {
        let filter = Filter {
            with_note_only: true,
            ..Filter::default()
        };
        let kept = apply(sheet(), &filter);
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|r| !matches!(r.note, Note::Absent)));
    }

    /// A `/Link`'s accessibility **description** counts as text.
    #[test]
    fn a_description_counts_as_text() {
        let rows = vec![row(0, 1, "Link", None, Note::Description("a URL".into()))];
        let filter = Filter {
            with_note_only: true,
            ..Filter::default()
        };
        assert_eq!(apply(rows, &filter).len(), 1);
    }

    /// **Sorting by author is stable, and document order survives inside
    /// each name.**
    #[test]
    fn sorting_by_author_is_stable_within_a_name() {
        let filter = Filter {
            sort: Sort::Author,
            ..Filter::default()
        };
        let kept = apply(sheet(), &filter);
        let ken: Vec<usize> = kept
            .iter()
            .filter(|r| r.author.as_deref() == Some("Ken Mantle"))
            .map(|r| r.page_index)
            .collect();
        assert_eq!(ken, vec![0, 1], "Ken's own comments left document order");
    }

    /// **An unsigned comment sorts to the END, not the start.**
    #[test]
    fn an_unsigned_comment_sorts_last() {
        let filter = Filter {
            sort: Sort::Author,
            ..Filter::default()
        };
        let kept = apply(sheet(), &filter);
        assert!(
            kept.last().is_some_and(|r| r.author.is_none()),
            "the anonymous row is at {:?}",
            kept.iter().position(|r| r.author.is_none())
        );
    }

    /// The author chooser lists each person once, alphabetically, and never a
    /// blank.
    #[test]
    fn the_author_chooser_is_distinct_sorted_and_never_blank() {
        let mut rows = sheet();
        rows.push(row(3, 5, "Text", Some("  "), Note::Absent));
        rows.push(row(3, 6, "Text", Some("Ken Mantle"), Note::Absent));
        assert_eq!(authors(&rows), vec!["Jo Smith", "Ken Mantle"]);
    }

    /// The type chooser lists each subtype once, alphabetically, in the file's
    /// own spelling.
    #[test]
    fn the_type_chooser_uses_the_files_own_spelling() {
        assert_eq!(subtypes(&sheet()), vec!["Line", "Square", "Text"]);
    }

    /// **Sorting is not narrowing**, so it raises no disclosure.
    #[test]
    fn sorting_alone_raises_no_disclosure() {
        let filter = Filter {
            sort: Sort::Author,
            ..Filter::default()
        };
        assert!(!filter.is_narrowing());
        assert_eq!(apply(sheet(), &filter).len(), 4);
    }
}
