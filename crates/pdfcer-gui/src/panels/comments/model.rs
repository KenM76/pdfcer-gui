//! # `panels::comments::model` — turning a document into a comment list
//!
//! The whole of the Comments panel that is not drawing. [`collect`] walks the
//! session's pages, applies the exclusion rule, classifies what survives, and
//! hands back a [`Listing`] the body renders row for row. Nothing here touches
//! `egui`.
//!
//! ## Why the panel is split this way
//!
//! Because every interesting decision this panel makes is a *classification*,
//! and a classification is only testable if it is separable from the widget
//! that shows it. The list of things that have to be right —
//!
//! - which annotations are excluded, and how many of each,
//! - whether a `/Line` is a **ce dimension** or a genuine `/Line` markup,
//! - whether `/Contents` is a note or an accessibility description,
//! - whether the row is a reply, a group subordinate, neither, or something
//!   `/RT` named that pdfcer has never heard of,
//! - whether the annotation is suppressed on screen,
//! - whether its appearance state could be resolved,
//! - and the ordering of the whole thing
//!
//! — is exactly the list `tests` below sweeps against real engine fixtures.
//! `crate::panels::objects` is split on the same seam and for the same reason
//! (`provider.rs` and `summary.rs` beside its `mod.rs`), and this module's
//! `Vec<CommentRow>` is that pattern at a much smaller scale.
//!
//! ## The ordering is `pdfcer list-annotations`', reused by name
//!
//! **Page order, then `/Annots`-array order.** Reused rather than reinvented:
//! a second, GUI-only ordering rule could disagree with the CLI's, and an
//! operator comparing a panel against a command's output on the same file
//! would have no way to tell which of the two had drifted. `/Annots` order is
//! whatever [`pdfcer_core::annot::page_annotations`] returns, which is the
//! array order the file itself carries — not a sort pdfcer imposes.
//!
//! There is deliberately **no sort by date**. `/M` is stored raw because
//! §12.5.2 makes it *"date or text string"* and requires a reader to accept
//! any format, so ordering by it would mean parsing a value the standard says
//! may not parse — and any such feature owns that decision itself rather than
//! inheriting it from a list nobody asked to be sorted.
//!
//! ## Read the SESSION, not the file on disk
//!
//! [`collect`] takes an [`ObjectGraph`], and the body hands it
//! `doc.session.view()` — the base revision with **every unsaved edit
//! applied**, which is the same thing the canvas rasterizes. An operator who
//! has just drawn three shapes must see three rows without saving first.
//! `crate::panels::forms`' body carries the same rule and the same sentence.

use std::collections::BTreeSet;

use pdfcer_core::annot::{Annotation, Appearance, ReplyType, page_annotations};
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

/// The whole panel's content, computed once per frame.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Listing {
    /// Every listable annotation, in page order then `/Annots` order.
    pub rows: Vec<CommentRow>,
    /// What the filter removed, counted by kind.
    pub excluded: Excluded,
}

impl Listing {
    /// How many rows carry note text a person could have written.
    ///
    /// Counts [`Note::Text`] only. A [`Note::Description`] is the document's
    /// accessibility alternate for a control that displays no text of its own
    /// (§12.5.2, §14.9.3) — counting it here would make
    /// [`Self::every_row_lacks_note_text`] false on a document whose only
    /// "note" is a screen-reader label for a link, and the panel would then
    /// withhold the disclosure that stops the list reading as broken.
    #[must_use]
    pub fn with_note_text(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| matches!(r.note, Note::Text(_)))
            .count()
    }

    /// Whether **every** row lacks note text — the condition for the
    /// document-wide "shapes pdfcer drew carry no note" disclosure.
    ///
    /// `false` on an empty listing, deliberately: with no rows there is
    /// nothing for the sentence to explain, and the panel says
    /// `comments_none()` instead. A vacuous truth here would print a paragraph
    /// about note text under a heading that just said there is nothing at all.
    #[must_use]
    pub fn every_row_lacks_note_text(&self) -> bool {
        !self.rows.is_empty() && self.with_note_text() == 0
    }
}

/// What [`collect`] filtered out, by kind.
///
/// Counted rather than discarded, because the panel discloses it. A reviewer
/// looking at six rows on a drawing they know carries forty annotations needs
/// the arithmetic; see `crate::text::panels::comments::comments_excluded`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Excluded {
    /// `/Widget` — form fields. The Forms panel owns them.
    pub widgets: usize,
    /// `/Popup` — a reader-UI window belonging to another annotation.
    pub popups: usize,
    /// `/TrapNet` — prepress output state written by a RIP.
    pub trap_nets: usize,
}

impl Excluded {
    /// How many annotations were removed altogether.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.widgets + self.popups + self.trap_nets
    }
}

/// One annotation, as a row.
///
/// Owned strings rather than borrows of the annotation. The annotations are
/// modelled fresh from the graph inside [`collect`] and dropped when it
/// returns, so borrowing would tie the listing's lifetime to a temporary; and
/// the whole listing is a few hundred short strings at most, bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` per page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentRow {
    /// **0-based** page index — what [`crate::app::actions::Action::GoToPage`]
    /// takes. The `+ 1` happens only where a human reads it.
    pub page_index: usize,
    /// The annotation object's identity, when it has one.
    ///
    /// `None` for an annotation written as a **direct dictionary** inside
    /// `/Annots`, which Table 164 forbids (its dictionaries are indirect
    /// objects). Such a row is still listed — it really is on the page — and
    /// nothing that needs a handle may be offered for it. Nothing in this
    /// build needs one; a Delete would, which is why the field is carried
    /// rather than dropped.
    pub id: Option<ObjId>,
    /// `/Subtype`, decoded — or `(no Subtype)` when the key is absent, which
    /// is a malformed annotation surfaced rather than repaired.
    pub subtype: String,
    /// Whether this is a **ce dimension** — see [`ce_dimension_annots`].
    pub is_ce_dimension: bool,
    /// What `/Contents` is, if anything.
    pub note: Note,
    /// `/T`, conventionally the author. See [`Note`] on why an absent one
    /// prints nothing rather than "anonymous".
    pub author: Option<String>,
    /// `/M`, **raw and unparsed**, exactly as the file wrote it.
    pub modified: Option<String>,
    /// `AnnotFlags::suppressed_on_screen` — `/F` Hidden or NoView.
    pub suppressed: bool,
    /// `Appearance::StateUnresolved` — pdfcer could not select an appearance
    /// state and draws nothing, by choice rather than by failure.
    pub appearance_unresolved: bool,
    /// How this annotation relates to another one, if it does.
    pub relation: Option<Relation>,
}

/// What an annotation's `/Contents` actually is.
///
/// # §12.5.2 gives the key two jobs, and they are not interchangeable
///
/// *"Text displayed for the annotation, **or** (if the type does not display
/// text) an alternate human-readable description"* for accessibility
/// (§14.9.3). Which one it is depends on the subtype, and `pdfcer-core`
/// deliberately models the raw value **without** that interpretation, because
/// *"a UI labelling this 'comment' is right for markup and wrong for a Link"*
/// — the interpretation *"belongs to whoever displays it"*
/// (`annot.rs:315-324`). This enum is this panel accepting that job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// A note somebody wrote, on a subtype that displays text.
    Text(String),
    /// The document's accessibility description of a control that displays no
    /// text of its own — a `/Link`, a `/Movie`, a `/PrinterMark`.
    Description(String),
    /// `/Contents` is absent. **Not an error**, and the ordinary case on every
    /// shape pdfcer itself drew: `MarkupSpec` has no contents field on any
    /// variant, deliberately, so note text on geometric markup is an engine
    /// capability that does not exist yet (a filed request, `HANDOFF.md` §1).
    Absent,
}

/// How one annotation relates to another (`/IRT` + `/RT`, Table 170).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Relation {
    /// `/RT /R`, or `/IRT` with `/RT` absent — Table 170's default. A
    /// threaded reply that keeps its own author and text.
    Reply,
    /// `/RT /Group` — a **subordinate** whose own `Contents`, `M`, `T`,
    /// `Popup` and friends §12.5.6.2 says *"shall be ignored"* in favour of
    /// the group primary's. This panel shows what the annotation says and
    /// discloses that another reader will show something else.
    GroupMember,
    /// An `/RT` name that is neither, carried verbatim by core rather than
    /// coerced to the default.
    ///
    /// Modelled here rather than folded into [`Self::Reply`] for the reason
    /// core gives for keeping it: *"a name pdfcer does not recognise is a
    /// document fact and flattening it to the default would make the model
    /// claim the file said something it did not."* The panel treats it as an
    /// unremarkable relation and says nothing about it — an operator has no
    /// use for a `/RT` name, and inventing a sentence for a value nobody has
    /// seen would be a placeholder.
    Other,
}

/// Whether a subtype's `/Contents` is an accessibility description rather
/// than a note.
///
/// The list is §12.5.6.2's, quoted in `pdfcer-core`'s own docs: *"`Link` /
/// `Movie` / `Widget` / `PrinterMark` / `TrapNet` use it purely as an
/// accessibility alternate"* (`annot.rs:321-322`).
///
/// `Widget` and `TrapNet` are in the list even though [`collect`] excludes
/// both, and that is deliberate: this predicate answers *"what does the
/// standard say about this subtype"*, and a copy of the spec's list that had
/// been trimmed to match today's filter would silently become wrong the day
/// the filter changed. Two rules, kept separate.
///
/// Everything not named is treated as a subtype that **displays** its
/// `/Contents`, which is the conservative direction: mislabelling a markup
/// note as an accessibility description would tell an operator that somebody's
/// comment was written for a screen reader.
#[must_use]
fn contents_is_description(subtype: &str) -> bool {
    matches!(
        subtype,
        "Link" | "Movie" | "Widget" | "PrinterMark" | "TrapNet"
    )
}

/// The object ids of every annotation that is a **ce dimension**.
///
/// # Why this cannot be answered from the annotation alone
///
/// A **ce dimension** is a `/Line` annotation carrying `/IT /LineDimension`, a
/// baked `/AP` and a record in the document's `/PieceInfo` sidecar — and
/// `pdfcer_core::annot::Annotation` models **none** of those three: `/IT` is
/// among the per-subtype keys it deliberately does not carry
/// (`annot.rs:284-288`), and the sidecar is a different structure entirely.
/// The authoritative answer is the sidecar's own model, whose
/// `DimensionRecord::annot` is the annotation each record was written for.
///
/// # Why the panel bothers
///
/// Project rule 15. A **ce dimension** and a **pdf dimension** have opposite
/// properties — one pdfcer authors and can restyle, regroup and delete as a
/// unit; the other is CAD-exported page content pdfcer reads and must not
/// silently alter — and a row that showed the first as plain "Line" would be
/// true about the file and useless to the operator.
///
/// This is also the constructive half of the old shell's exclusion argument.
/// ce dimensions are **not** filtered out, because filtering by subtype would
/// also hide a genuine `/Line` markup somebody drew; the sidecar is what lets
/// the panel tell the two apart *without* filtering either.
///
/// # Cost
///
/// One catalog → `/PieceInfo` → `/pdfcer` → `/Private` walk and a
/// deserialization of the sidecar, bounded by the number of ce dimensions in
/// the document rather than by its size. Called **once** per frame by
/// [`crate::panels::comments::body`], never per row. A document that has never
/// been dimensioned has no sidecar and gets an empty set, which is the
/// ordinary case and the cheapest one.
#[must_use]
pub fn ce_dimension_annots(session: &pdfcer_core::edit::EditSession) -> BTreeSet<ObjId> {
    session
        .dimension_model()
        .dimensions()
        .iter()
        .filter_map(|d| d.annot)
        .collect()
}

/// Build the listing for a whole document.
///
/// `graph` must be the **session view**, not the loaded file — see this
/// module's header. `pages` is `OpenDoc::pages`, the flattened page vector
/// resolved once at open, and its index is the page index every row carries.
/// `ce_dimensions` comes from [`ce_dimension_annots`].
///
/// # The exclusion rule, which is settled law
///
/// Carried across whole from the old shell (`main.rs:7031-7051`), with its
/// argument rather than as a code snippet. See
/// [`crate::panels::comments`]' header, which states all four clauses and the
/// one place this build departs.
#[must_use]
pub fn collect<G: ObjectGraph + ?Sized>(
    graph: &G,
    pages: &[Page],
    ce_dimensions: &BTreeSet<ObjId>,
) -> Listing {
    let mut listing = Listing::default();
    for (page_index, page) in pages.iter().enumerate() {
        for annot in page_annotations(graph, page.id) {
            // ★ THE EXCLUSION, and the order of the three tests does not
            // matter because nothing can be two of them: `/Subtype` has one
            // value. It is written as three separate arms rather than one
            // `||` so each kind can be counted, which is what lets the panel
            // disclose the filter in numbers instead of in doctrine.
            if annot.is_widget() {
                listing.excluded.widgets += 1;
                continue;
            }
            if annot.is_popup {
                listing.excluded.popups += 1;
                continue;
            }
            if annot.subtype == b"TrapNet" {
                listing.excluded.trap_nets += 1;
                continue;
            }
            listing.rows.push(row(page_index, &annot, ce_dimensions));
        }
    }
    listing
}

/// Classify one annotation.
///
/// Split out of [`collect`]'s loop so the classification can be read — and
/// tested — without the page walk around it.
fn row(page_index: usize, annot: &Annotation, ce_dimensions: &BTreeSet<ObjId>) -> CommentRow {
    let subtype = annot.subtype_label();
    // An annotation with no object identity cannot be in the sidecar, because
    // the sidecar records the id it wrote. `is_some_and` rather than a default
    // of `true`: a ce dimension is a *positive* finding, and failing closed
    // here means the row reads as an ordinary `/Line`, which is what the file
    // literally says.
    let is_ce_dimension = annot.id.is_some_and(|id| ce_dimensions.contains(&id));
    let note = match &annot.contents {
        Some(text) if contents_is_description(&subtype) => Note::Description(text.clone()),
        Some(text) => Note::Text(text.clone()),
        None => Note::Absent,
    };
    // `effective_reply_type`, never `reply_type` directly. Table 170's default
    // for an absent `/RT` is `R`, so a call site that read the raw field would
    // report "not a reply" for the ordinary threaded comment — core's own docs
    // name that as the trap this method exists to close.
    let relation = annot.effective_reply_type().map(|rt| match rt {
        ReplyType::Reply => Relation::Reply,
        ReplyType::Group => Relation::GroupMember,
        ReplyType::Other(_) => Relation::Other,
    });
    CommentRow {
        page_index,
        id: annot.id,
        subtype,
        is_ce_dimension,
        note,
        author: annot.title.clone(),
        modified: annot.mod_date.clone(),
        suppressed: annot.flags.suppressed_on_screen(),
        // `Appearance` is a plain enum today; matched by NAME so that a
        // variant core adds later defaults to "not the unresolved case"
        // rather than being swept into it by a catch-all on the wrong side.
        appearance_unresolved: matches!(annot.appearance, Appearance::StateUnresolved),
        relation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;

    /// Load a fixture as an `EditSession` plus its page vector — the same two
    /// things `crate::app::state::OpenDoc` carries, so a test exercises the
    /// panel's real inputs rather than a convenient stand-in.
    fn open(rel: &str) -> (EditSession, Vec<Page>) {
        let path = engine_fixture(rel);
        let doc = Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        (EditSession::new(doc), pages)
    }

    /// Collect a fixture through the same path the panel body uses.
    fn listing(rel: &str) -> Listing {
        let (session, pages) = open(rel);
        let ce = ce_dimension_annots(&session);
        collect(&session.view(), &pages, &ce)
    }

    /// **★ A pop-up is excluded, and it is counted rather than dropped.**
    ///
    /// `popup-not-painted.pdf` carries exactly one annotation and it is a
    /// `/Popup`. The listing is therefore empty — which is the *correct*
    /// answer and the one most likely to be mistaken for a broken panel — and
    /// the exclusion count is what lets the panel say so.
    ///
    /// §12.5.6.14: a pop-up *"shall not appear alone but is associated with a
    /// markup annotation, its parent annotation"*. It is a reader-UI window,
    /// never independent content, so one row per real annotation is the whole
    /// rule.
    #[test]
    fn a_popup_is_excluded_and_counted() {
        let l = listing("annot/popup-not-painted.pdf");
        assert!(
            l.rows.is_empty(),
            "the only annotation in this fixture is a pop-up: {:?}",
            l.rows
        );
        assert_eq!(l.excluded.popups, 1);
        assert_eq!(l.excluded.total(), 1);
        assert!(
            !l.every_row_lacks_note_text(),
            "an EMPTY listing must not claim its rows lack note text — there \
             are no rows, and the panel says so with a different sentence"
        );
    }

    /// **★ A form field is excluded — the Forms panel owns those.**
    ///
    /// `Annotation::is_widget` is the exact predicate, reused rather than
    /// re-derived: *"a second one would be a divergence waiting to happen."*
    #[test]
    fn a_widget_is_excluded_and_counted() {
        let l = listing("forms/demo-form.pdf");
        assert!(
            l.excluded.widgets > 0,
            "a form fixture must carry widgets, or this test proves nothing"
        );
        for row in &l.rows {
            assert_ne!(
                row.subtype, "Widget",
                "a widget reached the comment list: {row:?}"
            );
        }
    }

    /// **★ A `/TrapNet` is excluded.**
    ///
    /// Prepress output state — it records the trapping a RIP applied to the
    /// page. Neither a comment nor anything a person wrote. See
    /// `crate::panels::comments`' header for why this build keeps the old
    /// shell's exclusion even though the reason the old shell gave for it (a
    /// Delete whose every press would be refused) does not apply here.
    #[test]
    fn a_trapnet_is_excluded_and_counted() {
        let l = listing("annot/undeletable.pdf");
        assert_eq!(l.excluded.trap_nets, 1);
        for row in &l.rows {
            assert_ne!(row.subtype, "TrapNet", "prepress state reached the list");
        }
        // …and the squares beside it are listed, so the filter is a filter and
        // not a wall.
        assert!(
            l.rows.iter().any(|r| r.subtype == "Square"),
            "the ordinary markup on this page must survive the filter: {:?}",
            l.rows
        );
    }

    /// **★ ce dimensions are NOT excluded, and are named as what they are.**
    ///
    /// The heart of the old shell's exclusion argument, from both sides at
    /// once. They are `/Line` annotations, so they appear here; excluding them
    /// by subtype would also hide a genuine `/Line` markup an operator drew.
    /// And because the sidecar can tell the two apart, the row says
    /// "ce dimension" instead of "Line" without the filter ever being involved.
    #[test]
    fn a_ce_dimension_is_listed_and_recognised() {
        let (session, pages) = open("dimension/linear-dim.pdf");
        let ce = ce_dimension_annots(&session);
        assert!(
            !ce.is_empty(),
            "this fixture exists to carry a ce dimension; if the sidecar is \
             unreadable the rest of this test proves nothing"
        );
        let l = collect(&session.view(), &pages, &ce);
        let dims: Vec<&CommentRow> = l.rows.iter().filter(|r| r.is_ce_dimension).collect();
        assert_eq!(
            dims.len(),
            ce.len(),
            "every sidecar record's annotation must appear as a row: {:?}",
            l.rows
        );
        for d in dims {
            assert_eq!(
                d.subtype, "Line",
                "a ce dimension IS a /Line annotation — that is the whole \
                 reason it cannot be filtered out by subtype"
            );
        }
    }

    /// **…and a document with no sidecar calls nothing a ce dimension.**
    ///
    /// The other direction, which is the one that would go wrong silently. If
    /// [`ce_dimension_annots`] ever returned something for an undimensioned
    /// document, every `/Line` markup in the corpus would be relabelled and
    /// the mislabelling would look like a document fact.
    #[test]
    fn a_document_with_no_sidecar_has_no_ce_dimensions() {
        let (session, _pages) = open("annot/demo-annotated.pdf");
        assert!(ce_dimension_annots(&session).is_empty());
        let l = listing("annot/demo-annotated.pdf");
        assert!(l.rows.iter().all(|r| !r.is_ce_dimension));
    }

    /// **The order is page order, then `/Annots` order.**
    ///
    /// Asserted as a monotonic page index rather than against a hard-coded
    /// list, because the second half — `/Annots` order — is the file's own
    /// array order and pinning it would be pinning
    /// [`pdfcer_core::annot::page_annotations`]' contract rather than this
    /// module's. What this module owns is the outer loop, and the failure it
    /// would produce is a list that jumps between sheets.
    #[test]
    fn rows_are_in_page_order() {
        let l = listing("annot/thread.pdf");
        assert!(!l.rows.is_empty());
        let mut last = 0usize;
        for row in &l.rows {
            assert!(
                row.page_index >= last,
                "the list left page {last} and came back to {}: {row:?}",
                row.page_index
            );
            last = row.page_index;
        }
    }

    /// **★ A reply is recognised through `effective_reply_type`.**
    ///
    /// `thread.pdf` carries `/IRT` links. Table 170 makes `/RT` default to
    /// `R`, so an annotation with `/IRT` and no `/RT` **is** a reply — and a
    /// call site that read `reply_type` directly would report `None` and get
    /// the ordinary threaded comment wrong. Core names that as the trap; this
    /// pins that the panel does not walk into it.
    #[test]
    fn a_threaded_annotation_is_recognised_as_a_relation() {
        let l = listing("annot/thread.pdf");
        let related: Vec<&CommentRow> = l.rows.iter().filter(|r| r.relation.is_some()).collect();
        assert!(
            !related.is_empty(),
            "this fixture exists to carry /IRT links: {:?}",
            l.rows
        );
        // Every relation is one of the three modelled kinds; `Other` is
        // reachable only from an `/RT` name pdfcer has never seen.
        for r in related {
            assert!(matches!(
                r.relation,
                Some(Relation::Reply | Relation::GroupMember | Relation::Other)
            ));
        }
        // An annotation with no `/IRT` has no relation at all — `None` is a
        // fourth state, not a synonym for "not a reply".
        assert!(
            l.rows.iter().any(|r| r.relation.is_none()),
            "the thread's own root must have no relation: {:?}",
            l.rows
        );
    }

    /// **★ A suppressed annotation is listed and flagged, never dropped.**
    ///
    /// `03-capabilities.md:1100`: *"A Comments panel that silently omits it is
    /// hiding document content; list it and mark it hidden."* Hidden
    /// annotations are a recognised document-forensics vector, which is why
    /// core counts them rather than dropping them and why this panel is the
    /// off-canvas surface that reports them.
    #[test]
    fn a_hidden_annotation_is_listed_and_flagged() {
        for fixture in ["annot/flags-hidden.pdf", "annot/flags-noview.pdf"] {
            let l = listing(fixture);
            assert!(
                l.rows.iter().any(|r| r.suppressed),
                "{fixture} exists to carry a suppressed annotation, and the \
                 listing shows none: {:?}",
                l.rows
            );
        }
        // …and the flag DISCRIMINATES, which is the half that makes the
        // marker mean something. Asserted within one document rather than
        // across two, and the reason is a small empirical surprise worth
        // recording: `demo-annotated.pdf` — the ordinary-looking fixture, and
        // the obvious choice for "a document that flags nothing" — carries a
        // suppressed `/Stamp` of its own. A test written the obvious way
        // failed, and it was right to: a document with one hidden annotation
        // among three is exactly the shape this panel exists to disclose, and
        // "an ordinary document" was an assumption about a fixture rather
        // than a property of the code.
        //
        // So the assertion is the one that actually holds and actually
        // proves something: on a document with a mix, the flag is set on some
        // rows and not others. A predicate that returned `true` for
        // everything would satisfy the sweep above and fail here.
        let mixed = listing("annot/demo-annotated.pdf");
        assert!(
            mixed.rows.iter().any(|r| r.suppressed),
            "this fixture carries a suppressed stamp: {:?}",
            mixed.rows
        );
        assert!(
            mixed.rows.iter().any(|r| !r.suppressed),
            "…and ordinary annotations beside it, or the flag is not \
             discriminating: {:?}",
            mixed.rows
        );
    }

    /// **`/Contents` on a subtype that displays no text is a description.**
    ///
    /// The §12.5.2 dual purpose, decided here because core deliberately
    /// declines to: *"a UI labelling this 'comment' is right for markup and
    /// wrong for a Link"*, and the interpretation *"belongs to whoever
    /// displays it."*
    ///
    /// Asserted against the predicate rather than a fixture because the corpus
    /// has no `/Link` carrying `/Contents` — and a test that silently proved
    /// nothing would be worse than one that pins the rule it implements. The
    /// list is checked in both directions, which is what stops a markup note
    /// being relabelled as a screen-reader string.
    #[test]
    fn the_five_non_text_subtypes_are_descriptions_and_nothing_else_is() {
        for s in ["Link", "Movie", "Widget", "PrinterMark", "TrapNet"] {
            assert!(
                contents_is_description(s),
                "{s} displays no text of its own"
            );
        }
        for s in [
            "Text",
            "FreeText",
            "Square",
            "Circle",
            "Line",
            "Polygon",
            "PolyLine",
            "Ink",
            "Highlight",
            "Underline",
            "StrikeOut",
            "Squiggly",
            "Stamp",
            "Caret",
            "FileAttachment",
            "(no Subtype)",
        ] {
            assert!(
                !contents_is_description(s),
                "{s} displays its /Contents; calling it an accessibility \
                 description would tell an operator somebody's comment was \
                 written for a screen reader"
            );
        }
    }

    /// ★★★ **The note editor's painted-words warning fires on exactly one of
    /// the subtypes this panel lets an operator edit — and it is asked in the
    /// SAME vocabulary the panel stores.**
    ///
    /// The wiring guard for `panels::comments::note_editor`, and it is about a
    /// coupling rather than about prose. [`CommentRow::subtype`] is filled from
    /// `pdfcer-core`'s own `Annotation::subtype_label` (`annot.rs:640`) — the
    /// raw `/Subtype` name — and `crate::text::textannot::paints_its_note` is a
    /// **string match on that same vocabulary**. Nothing but this test holds
    /// the two together: a well-meant change that title-cased the panel's
    /// subtype for display, or that swapped it for an enum, would leave the
    /// warning silently never firing, and the symptom is a *missing* sentence,
    /// which no screenshot shows and no other test asks about.
    ///
    /// Reusing the row above's list is the point — it is this panel's own
    /// enumeration of what displays its `/Contents`, i.e. exactly the rows that
    /// get an editor — so the two cannot be brought into disagreement by adding
    /// a subtype to one list and not the other.
    ///
    /// The both-directions shape is deliberate. The costly failure is the
    /// **false positive**: a sticky note that started warning that the page did
    /// not change would be describing a page that never showed those words, and
    /// an operator who learns to dismiss this sentence loses the one case where
    /// it is true.
    #[test]
    fn only_the_text_box_row_warns_that_the_page_will_not_change() {
        use crate::text::textannot::note_edit_hint;

        assert!(
            note_edit_hint("FreeText").is_some(),
            "a /FreeText paints its /Contents (annot_author.rs:3131), so editing a \
             text box's note leaves the page stale and the editor must say so"
        );
        for s in [
            "Text",
            "Square",
            "Circle",
            "Line",
            "Polygon",
            "PolyLine",
            "Ink",
            "Highlight",
            "Underline",
            "StrikeOut",
            "Squiggly",
            "Stamp",
            "Caret",
            "FileAttachment",
            "(no Subtype)",
        ] {
            assert!(
                !contents_is_description(s),
                "the list this test shares with its neighbour drifted: {s}"
            );
            assert!(
                note_edit_hint(s).is_none(),
                "/{s} does not paint its /Contents, so a note edit on one is \
                 complete -- warning about it teaches the operator to ignore \
                 the warning that matters"
            );
        }
    }

    /// **An absent `/Contents` is [`Note::Absent`], and that drives the
    /// document-wide disclosure.**
    ///
    /// The condition the panel keys its "shapes pdfcer drew carry no note"
    /// sentence on. It must be true when every row lacks note text, false when
    /// any row has some, and false on an empty listing — the third being the
    /// one a naive `.all()` gets wrong, because `.all()` on an empty iterator
    /// is `true` and would print a paragraph about note text under a heading
    /// that just said there is nothing at all.
    #[test]
    fn the_all_without_notes_condition_is_not_vacuously_true() {
        assert!(!Listing::default().every_row_lacks_note_text());

        let mut l = Listing::default();
        l.rows.push(CommentRow {
            page_index: 0,
            id: None,
            subtype: "Square".to_owned(),
            is_ce_dimension: false,
            note: Note::Absent,
            author: None,
            modified: None,
            suppressed: false,
            appearance_unresolved: false,
            relation: None,
        });
        assert!(l.every_row_lacks_note_text());
        assert_eq!(l.with_note_text(), 0);

        // An accessibility description is NOT note text — it is the document
        // describing a control to a screen reader, and counting it would
        // withhold the disclosure from a document that genuinely has no notes.
        l.rows[0].note = Note::Description("Opens the drawing index".to_owned());
        assert!(l.every_row_lacks_note_text());
        assert_eq!(l.with_note_text(), 0);

        l.rows[0].note = Note::Text("Check this weld".to_owned());
        assert!(!l.every_row_lacks_note_text());
        assert_eq!(l.with_note_text(), 1);
    }

    /// **Every listed row carries a page index inside the document.**
    ///
    /// The index is fed straight to
    /// [`crate::app::actions::Action::GoToPage`], so an out-of-range value
    /// would be a navigation to nowhere. It cannot happen — the index is the
    /// enumeration of `pages` — and it is pinned anyway, because this is the
    /// one number in the row that leaves the panel.
    #[test]
    fn every_row_can_be_navigated_to() {
        let (session, pages) = open("annot/thread.pdf");
        let ce = ce_dimension_annots(&session);
        let l = collect(&session.view(), &pages, &ce);
        assert!(!l.rows.is_empty());
        for row in &l.rows {
            assert!(row.page_index < pages.len(), "{row:?} is off the end");
        }
    }
}
