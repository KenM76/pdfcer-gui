//! # `panels::comments::filter` — narrowing the reviewer's work list
//!
//! One subject: **which rows the Comments panel shows, and in what order.**
//! The state, the pure predicate, and the control strip that sets them.
//!
//! ## ★★★ Why this exists — the gap the operator's report exposed
//!
//! The operator, 2026-09-05: *"the review features should look and act the
//! same as they do in Acrobat Reader."* Acrobat's Comment pane is a **work
//! list**, and a work list you cannot narrow is a list you scroll. Its filter
//! offers reviewer, type, status and checkmark; its sort offers page, type,
//! author, date and colour.
//!
//! This panel had **none of it**: every annotation in the document, in
//! document order, always. On `SW41177.pdf` — thirty-six sheets — a reviewer
//! looking for their own three comments scrolls past everybody else's.
//!
//! ## ★★ What is offered, and what is NOT, and why each absence
//!
//! | Acrobat offers | here | why |
//! |---|---|---|
//! | filter by **reviewer** | ✅ [`Filter::author`] | `/T` is modelled and read |
//! | filter by **type** | ✅ [`Filter::subtype`] | `/Subtype` is modelled and read |
//! | filter to **comments with text** | ✅ [`Filter::with_note_only`] | ★ this shell's own, and it earns its place: pdfcer's own markup authoring cannot write `/Contents` on a geometric shape, so a drawing pdfcer marked up is a column of "no note" rows with the reviewer's actual remarks scattered through it |
//! | filter by **status** (Accepted / Rejected / …) | ❌ | `/State` and `/StateModel` have **zero occurrences** in `pdfcer-core` v0.38.0 — not read, not written, not modelled. R9: nothing is drawn. Filed as `request_review_status_is_not_modelled_at_all.md` |
//! | filter by **checkmark** | ❌ | a per-viewer flag Acrobat keeps outside the PDF. Not a document property, so not this panel's |
//! | sort by **page** | ✅ [`Sort::Document`], the default | |
//! | sort by **type** | ✅ [`Sort::Subtype`] | |
//! | sort by **author** | ✅ [`Sort::Author`] | |
//! | sort by **date** | ❌ | ★★ **`/M` is not reliably a date.** §12.5.2 gives its type as *"date **or** text string"* and requires a reader to accept any format, so `pdfcer-core` stores it raw and its own docs say *"do not assume it parses"*. A sort would have to either parse it — rejecting legal values — or sort the strings, which orders `D:2026…` before `17 January` and calls it chronology. `crate::text::panels::comments::comment_row_byline` makes the same ruling for display and this is it holding for ordering |
//! | sort by **colour** | ❌ | `/C` is **not in the engine's read model** at all — `annot.rs`'s parser reads `/CA` and never `/C`. Filed as `request_an_annotations_colour_cannot_be_read.md` |
//!
//! ⇒ Three of the four absences are **engine gaps with requests filed**, and
//! the fourth is a deliberate exclusion. None of them is drawn greyed: R9.
//!
//! ## ★★★ The disclosure that makes filtering safe
//!
//! A filtered list is a list that is **lying by omission** unless it says so.
//! `crate::panels::comments`' founding discipline is that *"nothing is
//! silently omitted"* — its exclusion line already states the arithmetic for
//! widgets, pop-ups and `/TrapNet` — and a filter is a fourth kind of
//! omission, and the only one the operator caused.
//!
//! So [`Filter::is_narrowing`] exists, and the panel draws
//! `comments_filtered(shown, total)` above the list whenever it is true. A
//! reviewer who set a filter, went to lunch and came back must not conclude
//! from six rows that the drawing carries six comments.
//!
//! ## The sort is STABLE, and that is load-bearing
//!
//! `sort_by_key` on a `Vec` is stable in Rust, so sorting by author leaves
//! each author's comments in **document order** — page order then `/Annots`
//! order, which is `pdfcer list-annotations`' ordering, reused by name rather
//! than re-decided. An unstable sort would reshuffle a reviewer's own comments
//! on every frame, which reads as the panel flickering.

use crate::panels::comments::model::{CommentRow, Note};

/// How the list is ordered.
///
/// An enum rather than a pair of booleans for the reason
/// `crate::canvas::selection::annot::AnnotKind` is one: exactly one ordering
/// is in force, and a type that could say *by author* and *by type* at once is
/// a type whose illegal states are prevented by discipline instead of by
/// construction.
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
///
/// `Default` is **everything**, which is the state the panel has always been
/// in — so a fresh profile behaves exactly as this panel did before the filter
/// existed, and nothing is hidden from an operator who never touches it.
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
    /// ★ `Note::Description` counts as **text**, and that is deliberate:
    /// §12.5.2 makes `/Contents` dual-purpose, and a `/Link`'s accessibility
    /// description is still somebody's words. The row already says which
    /// meaning it is; the filter's job is *"is there anything to read here"*.
    pub with_note_only: bool,
    /// How the surviving rows are ordered.
    pub sort: Sort,
}

impl Filter {
    /// **Is this filter hiding anything?**
    ///
    /// The predicate the disclosure hangs off — see the module header. Note
    /// that [`Self::sort`] is **not** part of it: reordering a list omits
    /// nothing, and a "you have sorted this" notice would be noise attached to
    /// a change the operator can see.
    #[must_use]
    pub fn is_narrowing(&self) -> bool {
        self.author.is_some() || self.subtype.is_some() || self.with_note_only
    }

    /// Does one row survive?
    ///
    /// Split out from [`apply`] so the rule can be asserted against a single
    /// row without building a list — and so the three clauses are visible
    /// together rather than spread through an iterator chain.
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
///
/// Takes and returns owned rows rather than borrowing, because the caller
/// draws from the result and a borrow would keep the whole `Listing` alive
/// across the draw for no benefit — the rows are a handful of `String`s each
/// and a document with enough comments for that to matter has a layout cost
/// two orders of magnitude larger (see `crate::panels::comments`' cost note).
///
/// ★ **Stable**, so an ordering by author or by kind preserves document order
/// within each group. See the module header.
#[must_use]
pub fn apply(rows: Vec<CommentRow>, filter: &Filter) -> Vec<CommentRow> {
    let mut kept: Vec<CommentRow> = rows.into_iter().filter(|r| filter.keeps(r)).collect();
    match filter.sort {
        // Already in document order — `model::collect` walks pages in order
        // and `/Annots` in order — so this arm does nothing at all. Written as
        // an explicit no-op rather than an early return so that adding a
        // fourth ordering cannot forget to handle it.
        Sort::Document => {}
        // ★ An author-less comment sorts to the END rather than the start.
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
///
/// # ★ Sorted and de-duplicated, and blank names dropped
///
/// A chooser is a list of *people*, so it is alphabetical rather than in
/// document order — the operator is looking up a name, not walking the sheet.
/// A `/T` of `"  "` is a byline nobody wrote (the commonest way for one to
/// exist is a producer writing an empty string) and offering it would put a
/// blank row in the menu that filters to comments credited to a space. The
/// same trimming rule `crate::panels::comments::keeps_author_name` applies,
/// which is what keeps *"has an author"* meaning one thing across the surface.
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
///
/// The **file's own spelling**, never a friendly relabelling: `Square` is the
/// word every other surface in this shell uses for that annotation, and a
/// chooser offering "Rectangle" would be a fourth name for one thing.
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
    ///
    /// The assertion that protects every operator who never opens the filter
    /// strip: this panel's contract has always been *every annotation in the
    /// document*, and a filter whose `Default` narrowed would silently change
    /// what a surface means for everybody.
    #[test]
    fn the_default_shows_everything() {
        let filter = Filter::default();
        assert!(!filter.is_narrowing());
        assert_eq!(apply(sheet(), &filter).len(), 4);
    }

    /// Filtering by author keeps that author's comments and only those.
    ///
    /// Both halves asserted — the count *and* that no other author survives —
    /// because a predicate that returned `true` for everything passes a bare
    /// "Ken's comments are still here" check.
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

    /// ★ **An exact match, not a substring.**
    ///
    /// *Ken* and *Ken Mantle* are two reviewers. A substring match would fold
    /// one into the other, and the operator would read one person's comments
    /// under another's name — the worst available failure on a surface whose
    /// whole subject is attribution.
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

    /// ★★ **"With text only" drops the rows pdfcer's own markup produces.**
    ///
    /// The filter this shell added that Acrobat does not have, and the reason
    /// it earns its place: `MarkupSpec` has no contents field on any variant,
    /// so every shape pdfcer draws arrives with no `/Contents`. On a drawing
    /// marked up here, this is the switch that turns forty rows into the three
    /// somebody actually wrote on.
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

    /// ★ A `/Link`'s accessibility **description** counts as text.
    ///
    /// §12.5.2 makes `/Contents` dual-purpose and the row already says which
    /// meaning it carries. The filter asks *"is there anything to read"*, and
    /// dropping a description would hide a string somebody wrote on the
    /// grounds that it is not a *comment* — a distinction the operator did not
    /// ask this switch to make.
    #[test]
    fn a_description_counts_as_text() {
        let rows = vec![row(0, 1, "Link", None, Note::Description("a URL".into()))];
        let filter = Filter {
            with_note_only: true,
            ..Filter::default()
        };
        assert_eq!(apply(rows, &filter).len(), 1);
    }

    /// ★★★ **Sorting by author is stable, and document order survives inside
    /// each name.**
    ///
    /// The property that stops the list flickering. An unstable sort would
    /// reshuffle one reviewer's own comments between frames — the panel
    /// redraws sixty times a second — which reads as the surface being broken
    /// rather than as an ordering choice.
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

    /// ★★ **An unsigned comment sorts to the END, not the start.**
    ///
    /// A reviewer ordering by author is looking for a person. `None` sorting
    /// first — which is the derived order on `Option` and therefore what an
    /// implementation gets for free — buries the name they asked for under
    /// every anonymous row in the document.
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
    ///
    /// Stated as a test because the obvious implementation of `is_narrowing`
    /// is *"the filter is not `Default`"*, which would put a "some comments
    /// are hidden" notice above a list that is hiding nothing.
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
