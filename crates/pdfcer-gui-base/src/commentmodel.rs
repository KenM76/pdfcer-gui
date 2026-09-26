//! # `commentmodel` — turning a document into a comment list
//!
//! The whole of the Comments panel that is not drawing. [`collect`] walks the
//! session's pages, applies the exclusion rule, classifies what survives, and
//! hands back a [`Listing`] the body renders row for row. Nothing here touches
//! `egui`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/commentmodel.md`.

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
    #[must_use]
    pub fn with_note_text(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| matches!(r.note, Note::Text(_)))
            .count()
    }

    /// Whether **every** row lacks note text — the condition for the
    /// document-wide "shapes pdfcer drew carry no note" disclosure.
    #[must_use]
    pub fn every_row_lacks_note_text(&self) -> bool {
        !self.rows.is_empty() && self.with_note_text() == 0
    }
}

/// What [`collect`] filtered out, by kind.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentRow {
    /// **0-based** page index — what `pdfcer_gui::app::actions::Action::GoToPage`
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
    /// **The annotation this one answers** — `/IRT` (Table 170), as an
    /// object id rather than as a classification.
    ///
    /// [`Self::relation`] answers *what kind of relationship is this*;
    /// this answers *to what*. Two fields for one key, because the panel needs
    /// both and neither can be derived from the other: `relation` decides the
    /// caption on the row, and this decides **where a Go to press has to land**
    /// — see [`thread_root`].
    ///
    /// `add_reply` places a reply on its parent's own `/Rect`, so
    /// `pdfcer_gui::canvas::notepopup::model::notes_on` draws no window for a reply:
    /// a bubble at those coordinates would sit on top of the comment it
    /// answers, and the newest answer would make the comment itself
    /// unreachable. That exclusion is what leaves a reply row with no window of
    /// its own to open, and this field is how [`thread_root`] finds the one
    /// that will be drawn.
    ///
    /// `None` for an ordinary comment, and also for a reply whose `/IRT` is a
    /// direct dictionary — `pdfcer-core` models a dangling `/IRT` rather than
    /// repairing it, and so does this.
    pub in_reply_to: Option<ObjId>,
}

/// What an annotation's `/Contents` actually is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// A note somebody wrote, on a subtype that displays text.
    Text(String),
    /// The document's accessibility description of a control that displays no
    /// text of its own — a `/Link`, a `/Movie`, a `/PrinterMark`.
    Description(String),
    /// `/Contents` is absent. **Not an error**, and the ordinary case on every
    /// shape pdfcer itself drew: `pdfcer_core::annot_author::MarkupSpec` has no
    /// contents field on any variant, so geometric markup is authored without a
    /// note and acquires one only through a later `set_markup_note`, which
    /// [`super::editor`] offers on exactly the rows that display their
    /// `/Contents`.
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
#[must_use]
#[doc(hidden)]
pub fn contents_is_description(subtype: &str) -> bool {
    matches!(
        subtype,
        "Link" | "Movie" | "Widget" | "PrinterMark" | "TrapNet"
    )
}

/// The object ids of every annotation that is a **ce dimension**.
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
#[must_use]
pub fn collect<G: ObjectGraph + ?Sized>(
    graph: &G,
    pages: &[Page],
    ce_dimensions: &BTreeSet<ObjId>,
) -> Listing {
    let mut listing = Listing::default();
    for (page_index, page) in pages.iter().enumerate() {
        for annot in page_annotations(graph, page.id) {
            // THE EXCLUSION, and the order of the three tests does not
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
        in_reply_to: annot.in_reply_to,
    }
}

/// **Which comment's window shows this row** — walk `/IRT` up to the
/// annotation at the head of the thread.
#[must_use]
pub fn thread_root(rows: &[CommentRow], id: ObjId) -> ObjId {
    let mut current = id;
    for _ in 0..MAX_THREAD_DEPTH {
        let Some(parent) = rows
            .iter()
            .find(|row| row.id == Some(current))
            .and_then(|row| row.in_reply_to)
        else {
            break;
        };
        // **The parent must be a row in this list before the walk moves
        // to it**, and putting that check here rather than at the end is the
        // difference between returning a real destination and returning an
        // object number.
        //
        // A `/IRT` may name a `/Widget`, a `/Popup`, an annotation on a page
        // this listing excluded, or nothing at all — §7.3.10 makes a dangling
        // reference legal and `pdfcer-core` models it rather than repairing
        // it. Advancing first and discovering the absence on the next
        // iteration would hand the caller that dangling id, and the caller
        // opens a pop-up with it: `notepopup::open::set` would record an
        // override for an object that is not on the page, and the operator
        // would press Go to and watch nothing happen.
        if !rows.iter().any(|row| row.id == Some(parent)) {
            break;
        }
        // A row that replies to itself is not a thread — `add_reply` refuses
        // to author one (`edit.rs`'s scope note 2) and a file that carries one
        // would otherwise spin here until the depth bound saved it. Stopping
        // on the first step is cheaper and says why.
        if parent == current {
            break;
        }
        current = parent;
    }
    current
}

/// How many `/IRT` links [`thread_root`] follows before giving up.
const MAX_THREAD_DEPTH: usize = 8;
