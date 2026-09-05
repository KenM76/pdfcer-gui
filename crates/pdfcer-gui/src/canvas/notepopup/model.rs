//! # `canvas::notepopup::model` — what a note says, and where its window goes
//!
//! The **pure** half of the note pop-up: it reads the document and answers
//! three questions, and it draws nothing, stores nothing and decides nothing
//! about the interface.
//!
//! | question | answer |
//! |---|---|
//! | *what notes are on this page, and where?* | [`notes_on`] |
//! | *which one is under the pointer?* | [`under`] |
//! | *what replies hang off this one?* | [`replies_to`] |
//!
//! It is separate from [`super`] for the reason every `model` module in this
//! crate is: a `Ui` cannot be driven from a unit test here, so anything that
//! needs a `Ui` is untestable by construction. Everything in this file takes an
//! [`ObjectGraph`] and a [`Page`] and returns data, which means every rule it
//! states can be asserted.
//!
//! ## ★★★ Why this exists at all — the operator's report, 2026-09-05
//!
//! > *"I could add a yellow sticky note but even in read mode I don't think I
//! > could figure out how to read it."*
//!
//! He was right, and the measurement was worse than the report. Before this
//! module the **only** route to a note's `/Contents` in the whole shell was
//! the Comments panel, which is mounted from the `markup` tab — and
//! `crate::app::modes::defaults`' `"read"` arm gives Read the tab list
//! `["file", "view"]`. So in Read mode there was **no route at all**, which is
//! the posture exactly backwards: Acrobat *Reader* is a read-only product and
//! reading comments is its whole purpose.
//!
//! A pop-up on the canvas is the fix that cannot regress that way, because it
//! is **canvas behaviour rather than a ribbon item** — it is mode-independent
//! by construction, and no future tab-list edit can take it away.
//!
//! ## ★★ What the file already contained, and what it did not
//!
//! `pdfcer-core` writes a `/Popup` companion for every sticky note it authors
//! (`D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:3223`), with its
//! `/Open` state (`:3224`) and a rectangle 150 pt wide placed to the right of
//! the note (`:3217-3222`). **Nothing in this shell had ever drawn one.** The
//! data was in the operator's files the whole time.
//!
//! ### ★★★ `/Open` is READ from the file, never defaulted
//!
//! §12.5.6.4 Table 172 gives `/Open` on a `/Text` annotation as *"a flag
//! specifying whether the annotation shall initially be displayed open"*, and
//! §12.5.6.14 Table 183 gives the same key the same meaning on the `/Popup`.
//! A note authored open must therefore **open on load**, with no click.
//!
//! ⚠ **`pdfcer_core::annot::Annotation` does not model `/Open`.** Confirmed by
//! audit on 2026-09-05: `b"Open"` appears exactly twice in the whole crate,
//! both write sites in `annot_author.rs` (`:3212`, `:3224`), and the read
//! model's parser (`annot.rs:905-1000`) never looks at the key. So this module
//! reads the raw dictionary through [`ObjectGraph::value`], which is public
//! and is the same graph `page_annotations` walks.
//!
//! ⇒ **That is a workaround and it is reported as one** (pdfcer decision 058:
//! *anything the GUI has to work around is a place the crate boundary was
//! drawn wrong*). Filed as `request_popup_open_state_cannot_be_read.md`. It is
//! a **read** of a documented key rather than an inference, so it is honest —
//! but it means this shell now parses a piece of annotation structure the
//! engine's read model owns, which is exactly the seam that drifts.
//!
//! ## ★ Where a pop-up is drawn, in priority order
//!
//! 1. **The `/Popup`'s own `/Rect`**, when it has one. The file said where the
//!    window goes and honouring it is what makes a document look the same here
//!    as it does in the reader that wrote it.
//! 2. **Beside the note**, when there is no `/Popup` or its rect is unusable —
//!    to the right, top-aligned, which is where every reader in the class puts
//!    one and where `pdfcer-core`'s own author places it.
//!
//! [`PopupBox::rect`] is `None` in case 2 and [`super`] does the placing,
//! because case 2 needs the *drawn* size of a window this module cannot see.
//!
//! ## ★★ Rule 15: a ce dimension is a `/Line` and it is NOT excluded here
//!
//! `crate::panels::comments`' header settles this and the argument is not
//! re-derived: **ce dimensions** (the ones pdfcer authors, `/Line` with `/IT
//! /LineDimension` and a `/PieceInfo` sidecar) are annotations on the
//! document, so hiding them by subtype would also hide a genuine `/Line`
//! markup an operator drew.
//!
//! What *is* different here is that a ce dimension's `/Contents` is
//! **regenerated from its measurement** by `author_dimension`, so it is never
//! a note somebody wrote. [`NoteView::contents`] carries whatever the file
//! says and [`super`] captions it; this module does not filter on it.
//!
//! ## Cost, stated rather than discovered
//!
//! [`notes_on`] walks one page's `/Annots` — one array read plus one
//! dictionary per entry, bounded by `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`,
//! decomposing nothing. It is the same walk
//! `crate::canvas::selection::annot::under_pointer` already pays per click.
//!
//! [`replies_to`] walks **every page**, because §12.5.6.2 permits a reply to
//! live on a different page from the comment it replies to and `pdfcer-core`'s
//! own `locate_annotation` scans every page for exactly that reason
//! (`edit.rs:25654-25669`). It is called only while a pop-up is open, never
//! otherwise. `crate::panels::comments` pays the same walk every frame it is
//! visible and states so in its own header; this is the same bound with a
//! stricter gate.

use std::collections::BTreeSet;

use egui::{Pos2, Rect};
use pdfcer_core::annot::{Annotation, ReplyType, page_annotations};
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{ObjId, Object};
use pdfcer_core::page_tree::Page;

use crate::canvas::mapping::annot_canvas_rect;

/// One annotation that can carry a note, with everything a pop-up needs.
///
/// Deliberately **not** `pdfcer_core::annot::Annotation` passed through: that
/// type carries sixteen fields of which four matter here, its `/Rect` is in
/// PDF user space rather than canvas space, and it cannot answer the `/Open`
/// question at all. A projection with a name is what lets [`under`] and
/// [`super`] agree about what they are talking about.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteView {
    /// The annotation's object id — what every `EditSession` verb takes.
    ///
    /// `Option<ObjId>` on the engine's model becomes a plain `ObjId` here,
    /// because [`notes_on`] **drops** an annotation with no id: a dictionary
    /// written directly into `/Annots` is a malformed file (§12.5.2 Table 164
    /// requires an indirect object) and there would be nothing for a pop-up's
    /// Edit or Delete to name. `crate::panels::comments` lists one and says
    /// why it cannot be acted on; the canvas cannot say that about a window it
    /// would have to draw first.
    pub id: ObjId,
    /// `/Subtype`, as the file spells it — `Text`, `Square`, `Line`.
    pub subtype: String,
    /// `/Contents`, verbatim. `None` when the key is absent.
    pub contents: Option<String>,
    /// `/T` — §12.5.6.4 Table 170's *"name of the person who created the
    /// annotation"*. `None` is legitimate and means **anonymous**, never
    /// *unknown*.
    pub author: Option<String>,
    /// `/M`, **raw and unparsed**.
    ///
    /// ★ The same ruling `crate::text::panels::comments::comment_row_byline`
    /// records at length and it is not re-argued: §12.5.2 gives `/M`'s type as
    /// *"date **or** text string"* and requires a conforming reader to accept
    /// any format, so formatting it here would mean writing a parser whose
    /// failure mode is either rejecting a legal value or mangling one. Two
    /// surfaces, one rule — and if this module formatted while the panel did
    /// not, the operator would see two different dates for one comment.
    pub modified: Option<String>,
    /// The annotation's own `/Rect`, in **canvas space** — the same
    /// zoom-independent space `crate::canvas::selection::annot` caches
    /// outlines in, so a zoom or a pan moves where the pop-up is drawn without
    /// changing which note it belongs to.
    pub anchor: Rect,
    /// The `/Popup` companion, when the file gives one.
    pub popup: Option<PopupBox>,
    /// Whether the file says this note starts **open** — §12.5.6.4 Table 172
    /// on the parent, §12.5.6.14 Table 183 on the pop-up.
    ///
    /// See [`read_open`] for which of the two is consulted and why.
    pub authored_open: bool,
    /// §12.5.3 Table 165 bit 8 — the file says the user interface may not
    /// change this annotation's properties.
    ///
    /// Carried so [`super`] can **omit** the editing controls rather than
    /// offer them and let the engine refuse. R83.
    pub locked: bool,
    /// The annotation this one replies to (`/IRT`), when it is a reply.
    ///
    /// Read only — `pdfcer-core` v0.38.0 has no verb that authors an `/IRT`;
    /// see [`super`]'s header and
    /// `request_a_reply_can_be_read_and_never_written.md`.
    pub in_reply_to: Option<ObjId>,
}

/// A `/Popup` annotation, reduced to the two things a window needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupBox {
    /// The pop-up annotation's own object id.
    pub id: ObjId,
    /// Its `/Rect` in **canvas space**, when it has a usable one.
    ///
    /// `None` when `/Rect` is absent or degenerate, in which case [`super`]
    /// places the window beside the note instead. Not defaulted to the note's
    /// own rectangle: a pop-up drawn exactly over the icon it belongs to hides
    /// the thing the operator just clicked.
    pub rect: Option<Rect>,
}

/// **Every annotation on `page_index` that can carry a note**, in paint order.
///
/// # What is excluded, and why each one
///
/// The same list `crate::canvas::selection::annot::selectable_on` uses, with
/// one addition and one deliberate difference, because this surface answers a
/// different question — *what did somebody write here* rather than *what may I
/// restyle*.
///
/// | excluded | why |
/// |---|---|
/// | `/Widget` | a form field is not a comment; the Forms panel owns it, and its `/Contents` is a tooltip rather than a remark |
/// | `/Popup` | §12.5.6.14 is a `shall`: a pop-up *"shall not appear alone but is associated with a markup annotation, its parent annotation."* It is the window, not the note |
/// | `/Link`, `/Movie`, `/PrinterMark`, `/TrapNet` | nobody wrote them. `/TrapNet` in particular is prepress output state a RIP applied |
/// | **not drawn on screen** (§12.5.3 bit 2 `Hidden` **or** bit 6 `NoView`) | ★ nothing is painted there, so a pop-up would hang off a point on blank paper with no visible anchor. The Comments panel is where such an annotation is reached, and it marks it as hidden — the same split the canvas already makes for an undrawn form field |
///
/// ### ★★★ This is STRICTER than the selection's exclusion, and the difference
/// is a finding
///
/// `crate::canvas::selection::annot::selectable_on` excludes `flags.hidden()`
/// alone. `pdfcer_core::annot::AnnotFlags::suppressed_on_screen` — used here —
/// is `hidden() || no_view()`, and §12.5.3 Table 165 bit 6 (`NoView`) means
/// *"do not display on screen, but do print"*. So a `/NoView` annotation is
/// **selectable on the canvas today with nothing drawn under the pointer**:
/// the operator gets a selection outline round blank paper.
///
/// Found by building this door, which is the second time in this project a new
/// route onto an existing capability has exposed a divergence the old one was
/// hiding. **Not fixed here** — `canvas/selection/**` belongs to a concurrent
/// track — and reported instead. This module takes the stricter reading
/// because a pop-up window is a much louder wrong answer than an outline.
/// | **no object id** | there would be nothing for Edit or Delete to name — see [`NoteView::id`] |
/// | **no usable `/Rect`** | §12.5.5's placement target is missing, so there is no anchor and the renderer drew nothing either |
///
/// ★ **A `/FreeText` is NOT excluded**, and that is worth stating because it
/// is the one case where a pop-up duplicates what is already on the page: a
/// free-text annotation paints its own words. Acrobat still gives it a
/// pop-up — the words on the page are the *appearance*, which a producer may
/// have styled, clipped or rotated, while `/Contents` is what was typed — and
/// a reviewer correcting a typo needs the second one.
///
/// # Ordering
///
/// `/Annots` order, which is paint order: later entries draw on top. [`under`]
/// takes the **last** match so the topmost note wins a click, which is the
/// rule page content and annotation selection both already follow.
#[must_use]
pub fn notes_on<G: ObjectGraph + ?Sized>(graph: &G, page: &Page) -> Vec<NoteView> {
    let mut popups: Vec<(ObjId, Option<Rect>)> = Vec::new();
    let mut parents: Vec<Annotation> = Vec::new();

    // One walk, two collections. The pop-ups have to be gathered in the same
    // pass because a parent names its companion by id and the id alone carries
    // no rectangle — §12.5.6.14's `/Parent` back-reference is deliberately not
    // modelled by `pdfcer-core` (`annot.rs:81-87`: *"the authoritative
    // direction is the parent's `/Popup`"*), so the pairing is ours to make
    // and it is made from the parent's side.
    for annot in page_annotations(graph, page.id) {
        if annot.is_popup {
            if let Some(id) = annot.id {
                popups.push((id, annot.rect.and_then(|r| canvas_rect(r, page))));
            }
            continue;
        }
        if annot.is_widget() || annot.flags.suppressed_on_screen() {
            continue;
        }
        let subtype = annot.subtype_label();
        if matches!(
            subtype.as_str(),
            "Link" | "Movie" | "PrinterMark" | "TrapNet"
        ) {
            continue;
        }
        parents.push(annot);
    }

    parents
        .into_iter()
        .filter_map(|annot| {
            let id = annot.id?;
            let anchor = canvas_rect(annot.rect?, page)?;
            let popup = annot.popup.map(|pid| PopupBox {
                id: pid,
                rect: popups
                    .iter()
                    .find(|(candidate, _)| *candidate == pid)
                    .and_then(|(_, rect)| *rect),
            });
            Some(NoteView {
                authored_open: read_open(graph, id, annot.popup),
                id,
                subtype: annot.subtype_label(),
                contents: annot.contents.clone(),
                author: annot.title.clone(),
                modified: annot.mod_date.clone(),
                anchor,
                popup,
                locked: annot.flags.locked(),
                in_reply_to: annot.in_reply_to,
            })
        })
        .collect()
}

/// **Does the file say this note starts open?**
///
/// # ★★★ Why this reads a raw dictionary, when nothing else in this crate does
///
/// Because `pdfcer_core::annot::Annotation` has no `/Open` field and the
/// parser never reads the key — audited 2026-09-05 against v0.38.0, where
/// `b"Open"` occurs exactly twice in the crate and both are write sites in
/// `annot_author.rs`. The alternative to reading it here is **defaulting it**,
/// and the assignment that commissioned this module forbids that in as many
/// words: *"A note authored open should open. Read it; do not default it."*
///
/// It is a **read of a documented key through the crate's own public graph**,
/// not an inference and not a re-parse of anything structural: one dictionary
/// lookup for one boolean, at exactly the two places the standard puts it.
/// Rule 4 is satisfied — nothing is guessed and nothing is written.
///
/// Filed as `request_popup_open_state_cannot_be_read.md`, and the day the
/// engine models `/Open` this function becomes two field reads.
///
/// # The order: the note first, then its pop-up
///
/// Both carry the key and the standard gives both the same meaning — Table 172
/// for a `/Text` annotation, Table 183 for the `/Popup`. `pdfcer-core`'s
/// author writes the **same** value to both (`annot_author.rs:3212` and
/// `:3224`), so on a file pdfcer wrote the order cannot matter. It matters on
/// a file somebody else wrote, and the note wins because it is the annotation
/// the operator interacts with and the one whose `/Open` a `/Square` or an
/// `/Ink` — which have no `/Open` of their own in Table 170 — cannot supply.
///
/// Absent on both is `false`: Table 172's default value is `false`, stated.
fn read_open<G: ObjectGraph + ?Sized>(graph: &G, note: ObjId, popup: Option<ObjId>) -> bool {
    open_flag(graph, note)
        .or_else(|| popup.and_then(|id| open_flag(graph, id)))
        .unwrap_or(false)
}

/// `/Open` on one object, or `None` when the key is absent or is not a boolean.
///
/// `None` rather than `false` for a non-boolean, so [`read_open`] can fall
/// through to the pop-up rather than concluding "closed" from a malformed
/// parent. A file that writes `/Open 1` is malformed; a file that writes it on
/// the pop-up alone is ordinary.
fn open_flag<G: ObjectGraph + ?Sized>(graph: &G, id: ObjId) -> Option<bool> {
    let Object::Dict(dict) = graph.value(id)? else {
        return None;
    };
    match graph.resolve(dict.get(b"Open")?) {
        Object::Boolean(open) => Some(*open),
        _ => None,
    }
}

/// An annotation `/Rect` in canvas space, or `None` when it is unusable.
///
/// A thin adapter so the two call sites above spell the conversion once.
/// `annot_canvas_rect` already rejects a degenerate or non-finite rectangle,
/// which is the whole of what "unusable" means here.
fn canvas_rect(rect: pdfcer_core::page_tree::Rect, page: &Page) -> Option<Rect> {
    annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
}

/// **Which note is under `point`** — topmost wins.
///
/// The **last** match in paint order, exactly as
/// `crate::canvas::selection::annot::hit` takes the last: a sticky dropped on
/// top of a cloud is the thing the operator sees and therefore the thing they
/// mean.
///
/// # ★ The tolerance is the frame's, not a number of this module's own
///
/// Handed in from `crate::canvas::mapping::PageMapping::tolerance`, which is
/// the same click tolerance content and annotation selection both use. A note
/// icon must be exactly as easy to hit as the shape beside it, and a
/// separately chosen constant here would drift from that the first time either
/// was tuned.
///
/// # ★★ Rectangle containment, not ink
///
/// Unlike the selection hit test, which narrows a ce dimension to its drawn
/// segments, this claims the whole `/Rect`. That is deliberate and it is the
/// convention: in every reader in the class, clicking anywhere on a
/// highlight's span or inside a cloud's box opens its note. Narrowing to ink
/// would make the note on a hollow rectangle reachable only by clicking its
/// hairline border.
#[must_use]
/// **Whether this annotation has a pop-up worth opening at all.**
///
/// # The defect this closes
///
/// Until 2026-09-05 a click opened a pop-up for *every* annotation that **could**
/// carry a note, not for those that **do**. So clicking a revision cloud you
/// only meant to select produced an empty window over the drawing — and, until
/// the placement fix landed the same day, one that sat on top of the shape and
/// swallowed the drag as well. It was recorded as a known limit in [`super`]'s
/// header and on `OPERATOR_REQUESTS.md` O133 as *"a question about WHEN a pop-up
/// opens rather than where it goes"*. This is that question, answered.
///
/// # The rule, and why it is not simply "has words"
///
/// ★★ **A sticky note is a note whether or not anybody typed in it.** Opening it
/// empty is not noise — it is the annotation's entire purpose, it is what
/// Acrobat does, and an operator who placed one and has not written in it yet
/// needs the window in order to write. The same is true of a free-text box,
/// whose words *are* its appearance.
///
/// ⇒ So the subtype decides for the two whose **purpose** is the note, and the
/// **content** decides for everything else. A square, a circle, a line, a
/// cloud, a polygon, freehand ink and a text-markup highlight are all *marks
/// on a drawing* that may additionally carry a comment; where they carry none,
/// there is nothing to show and the click belongs to selection.
///
/// ★ A byline with no words is deliberately **not** enough. Knowing that
/// B. Reviewer drew this cloud is a fact about the drawing, and the Comments
/// panel lists it — putting a window over the page to say only that would be
/// the noise this function exists to remove. The panel is where facts live;
/// the pop-up is where a *message* lives.
///
/// ⚠ Whitespace does not count. A `/Contents` of `"   "` renders as an empty
/// window just as surely as an absent one, and a file that carries it was not
/// trying to say anything.
#[must_use]
pub fn has_something_to_read(note: &NoteView) -> bool {
    // The two whose purpose IS the note. `subtype_label` is the engine's own
    // spelling, which is why these are compared as strings rather than against
    // an enum this crate would have to keep in step.
    if matches!(note.subtype.as_str(), "Text" | "FreeText") {
        return true;
    }
    note.contents
        .as_deref()
        .is_some_and(|c| !c.trim().is_empty())
}

pub fn under(notes: &[NoteView], point: Pos2, tolerance: f32) -> Option<&NoteView> {
    notes
        .iter()
        .rev()
        .find(|note| note.anchor.expand(tolerance).contains(point))
}

/// One reply in a thread.
///
/// Flattened deliberately: §12.5.6.2 permits a reply to a reply, but every
/// reader in the class draws a comment thread as a **flat chronological list**
/// under its root rather than as a nested tree, and a tree drawn in a 260 pt
/// window would be four indents of two words each.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// The reply annotation's object id.
    pub id: ObjId,
    /// Its `/Contents`.
    pub contents: Option<String>,
    /// Its `/T`.
    pub author: Option<String>,
    /// Its `/M`, raw — see [`NoteView::modified`].
    pub modified: Option<String>,
    /// Whether this is a §12.5.6.2 **group member** rather than an ordinary
    /// reply — `/RT /Group`.
    ///
    /// ★ It matters to what the operator is being shown. For a group member
    /// the standard says the subordinate's own `/Contents`, `/M`, `/T` and the
    /// rest *"shall be ignored"* in favour of the primary's, so the words
    /// displayed beside it are words a conforming reader is instructed **not**
    /// to use. `pdfcer-core` deliberately does not apply that rule
    /// (`annot.rs:347-348`), so pdfcer shows the raw value — and rule 4 makes
    /// saying so mandatory, which is what this flag is for.
    pub group_member: bool,
}

/// **The thread hanging off `root`**, gathered from the whole document.
///
/// # ★★ Every page, because a reply need not be on the parent's page
///
/// §12.5.6.2 puts no page constraint on `/IRT`, and `pdfcer-core`'s own
/// `locate_annotation` walks every page for exactly this reason —
/// `edit.rs:24818-24821`: *"`all` is every annotation on every page, because a
/// reply may live on a DIFFERENT page from the comment it replies to."*
/// Scanning the current page alone would silently drop replies on a
/// forty-sheet drawing set, which is the shape of document this program is
/// for.
///
/// # ★ Replies to replies are flattened onto the root
///
/// One transitive pass: anything whose `/IRT` chain reaches `root` is in the
/// thread. `MAX_THREAD_DEPTH` bounds it, because a file may legally contain a
/// cycle (`a` replies to `b`, `b` replies to `a`) — §7.3.10 says a dangling
/// reference is not an error and says nothing at all about a circular one, and
/// `pdfcer-core` surfaces `/IRT` *"unresolved, same as `popup`: a dangling
/// `/IRT` is modelled, not repaired"* (`annot.rs:431`). A depth bound is the
/// only thing standing between that and a hang.
///
/// # Order
///
/// Document order — page order, then `/Annots` order. The same ordering
/// `crate::panels::comments` uses, **reused rather than re-decided**, and for
/// its reason: a second GUI-only rule is a second thing that can disagree with
/// `pdfcer list-annotations`. There is deliberately no sort by date, because
/// `/M` is not reliably a date (see [`NoteView::modified`]).
#[must_use]
pub fn replies_to<G: ObjectGraph + ?Sized>(graph: &G, pages: &[Page], root: ObjId) -> Vec<Reply> {
    // Every annotation in the document, once. Gathering first and resolving
    // afterwards is what makes the transitive pass affordable: the alternative
    // is re-walking every page per depth level.
    let mut all: Vec<Annotation> = Vec::new();
    for page in pages {
        all.extend(page_annotations(graph, page.id));
    }

    let mut thread: BTreeSet<ObjId> = BTreeSet::new();
    thread.insert(root);
    for _ in 0..MAX_THREAD_DEPTH {
        let before = thread.len();
        for annot in &all {
            let (Some(id), Some(parent)) = (annot.id, annot.in_reply_to) else {
                continue;
            };
            if thread.contains(&parent) {
                thread.insert(id);
            }
        }
        if thread.len() == before {
            break;
        }
    }

    all.into_iter()
        .filter(|annot| {
            annot
                .id
                .is_some_and(|id| id != root && thread.contains(&id))
        })
        .map(|annot| {
            // Asked BEFORE the fields are moved out — `effective_reply_type`
            // borrows the whole annotation, and a struct literal that
            // interleaved the two would not compile. Taken first rather than
            // last so the reason is visible instead of being an ordering
            // accident a later edit could undo.
            let group_member = matches!(annot.effective_reply_type(), Some(ReplyType::Group));
            Reply {
                // Safe by the filter above, which required `Some`. Expressed
                // as a default rather than an unwrap because a panic in a
                // frame that is trying to draw is the worst available outcome
                // and this is a display surface.
                id: annot.id.unwrap_or(root),
                contents: annot.contents,
                author: annot.title,
                modified: annot.mod_date,
                group_member,
            }
        })
        .collect()
}

/// How many levels of reply-to-a-reply are followed.
///
/// Eight, which is far past any thread a human writes and far short of
/// anything that costs a frame. The bound exists for the malformed case rather
/// than the deep one: a `/IRT` cycle is legal syntax and would otherwise loop
/// for ever. See [`replies_to`].
const MAX_THREAD_DEPTH: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::{Dict, Name};

    /// A graph of loose objects — enough for the two dictionary questions this
    /// module asks and nothing more.
    ///
    /// Hand-built rather than loaded from a fixture because the subject of
    /// these tests is **one key**, and a fixture would make the assertion
    /// depend on a file, a page tree and an `/Annots` walk — three things that
    /// can fail for reasons the assertion is not about.
    struct Objects(Vec<(ObjId, Object)>);

    impl ObjectGraph for Objects {
        fn value(&self, id: ObjId) -> Option<&Object> {
            self.0.iter().find(|(o, _)| *o == id).map(|(_, v)| v)
        }
        fn trailer_entry(&self, _key: &[u8]) -> Option<&Object> {
            None
        }
    }

    fn id(num: u32) -> ObjId {
        ObjId::new(num, 0)
    }

    fn with_open(num: u32, open: Option<bool>) -> (ObjId, Object) {
        let mut dict = Dict::new();
        if let Some(open) = open {
            dict.insert(Name::from(b"Open"), Object::Boolean(open));
        }
        (id(num), Object::Dict(dict))
    }

    /// ★★★ **A note the file says is open reads as open.**
    ///
    /// The assertion the whole `/Open` path exists for, and the one the
    /// assignment names: *"A note authored open should open. Read it; do not
    /// default it."* An implementation that returned `false` unconditionally
    /// would pass every other test in this module.
    #[test]
    fn a_note_authored_open_reads_as_open() {
        let graph = Objects(vec![with_open(7, Some(true))]);
        assert!(read_open(&graph, id(7), None));
    }

    /// The other half, and it is what makes the first one mean something: a
    /// note authored **closed** must not open. Asserting only the open case
    /// would pass on an implementation that returned `true` for everything —
    /// every pop-up in the document open on load, which is the same defect
    /// wearing the other value.
    #[test]
    fn a_note_authored_closed_stays_closed() {
        let graph = Objects(vec![with_open(7, Some(false))]);
        assert!(!read_open(&graph, id(7), None));
    }

    /// **Absent means closed**, which is Table 172's stated default value —
    /// not an assumption this module is making.
    #[test]
    fn a_note_with_no_open_key_is_closed() {
        let graph = Objects(vec![with_open(7, None)]);
        assert!(!read_open(&graph, id(7), None));
    }

    /// ★★ **The pop-up's own `/Open` is consulted when the note has none.**
    ///
    /// The case that matters on a file another product wrote: `/Square`,
    /// `/Ink` and every other geometric markup have **no `/Open` in Table
    /// 170** — the key belongs to `/Text` — so their open state lives only on
    /// the `/Popup`. Reading the parent alone would report every shape's note
    /// as closed however the producer saved it.
    #[test]
    fn a_shapes_open_state_comes_from_its_popup() {
        let graph = Objects(vec![with_open(7, None), with_open(8, Some(true))]);
        assert!(read_open(&graph, id(7), Some(id(8))));
    }

    /// The note wins when both carry the key. Stated as a test rather than
    /// left to the `or_else`, because the precedence is a decision with a
    /// reason (see [`read_open`]) and a reordering would be silent.
    #[test]
    fn the_note_outranks_its_popup() {
        let graph = Objects(vec![with_open(7, Some(false)), with_open(8, Some(true))]);
        assert!(!read_open(&graph, id(7), Some(id(8))));
    }

    /// A `/Open` that is not a boolean falls **through** to the pop-up rather
    /// than being read as `false`.
    ///
    /// `/Open 1` is a malformed file, and concluding "closed" from it would
    /// throw away a perfectly good answer sitting on the companion.
    #[test]
    fn a_malformed_open_falls_through() {
        let mut dict = Dict::new();
        dict.insert(Name::from(b"Open"), Object::Integer(1));
        let graph = Objects(vec![(id(7), Object::Dict(dict)), with_open(8, Some(true))]);
        assert!(read_open(&graph, id(7), Some(id(8))));
    }

    fn note_at(num: u32, rect: Rect) -> NoteView {
        NoteView {
            id: id(num),
            subtype: "Text".to_owned(),
            contents: Some("words".to_owned()),
            author: None,
            modified: None,
            anchor: rect,
            popup: None,
            authored_open: false,
            locked: false,
            in_reply_to: None,
        }
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// ★ **The topmost note wins, not the first one found.**
    ///
    /// `/Annots` is paint order, so a sticky dropped over a cloud is drawn
    /// last and is what the operator sees. A hit test taking the first match
    /// would open the note underneath — which reads as the click having
    /// missed, because a window appears about something the operator was not
    /// pointing at.
    #[test]
    fn the_topmost_note_takes_the_click() {
        let notes = vec![
            note_at(1, rect(0.0, 0.0, 100.0, 100.0)),
            note_at(2, rect(10.0, 10.0, 20.0, 20.0)),
        ];
        let hit = under(&notes, Pos2::new(15.0, 15.0), 0.0).expect("a hit");
        assert_eq!(hit.id, id(2));
    }

    /// A click on nothing is a miss, and a miss must be `None` rather than the
    /// nearest note — otherwise clicking blank paper anywhere on the sheet
    /// would open whatever comment happened to be closest.
    #[test]
    fn a_click_on_blank_paper_hits_nothing() {
        let notes = vec![note_at(1, rect(0.0, 0.0, 10.0, 10.0))];
        assert!(under(&notes, Pos2::new(400.0, 400.0), 2.0).is_none());
    }

    /// ★ The tolerance widens the target, which is what makes a 20 pt sticky
    /// icon hittable at a low zoom where it is a few pixels across.
    #[test]
    fn the_tolerance_widens_the_target() {
        let notes = vec![note_at(1, rect(0.0, 0.0, 10.0, 10.0))];
        assert!(under(&notes, Pos2::new(12.0, 5.0), 0.0).is_none());
        assert!(under(&notes, Pos2::new(12.0, 5.0), 4.0).is_some());
    }

    // -----------------------------------------------------------------------
    // The fixture, end to end
    // -----------------------------------------------------------------------

    /// `fixtures/comment-note.pdf` — the sheet the driven pop-up checks are
    /// aimed at.
    ///
    /// Regenerate with `python tools/gen-comment-note-fixture.py`; that
    /// script's docstring is the argument for every key it writes.
    fn fixture() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/comment-note.pdf")
    }

    /// ★★★ **THE FIXTURE CARRIES WHAT THE DRIVEN CHECKS ASSERT ABOUT.**
    ///
    /// # Why this test is here and not in `tools/ui-verify`
    ///
    /// `RESUME.md`'s falsification discipline: *"a fixture note with empty
    /// `/Contents` makes 'the pop-up shows the words' pass on a build that
    /// shows nothing."* A driven check aimed at a document that cannot
    /// exercise its case reports **SKIP or a green pass**, and neither is
    /// distinguishable from the feature working. This project has been bitten
    /// by that seven times in one month, three of them on one afternoon.
    ///
    /// So the fixture's own properties are asserted **here**, in a cheap unit
    /// test that runs on every `cargo test`, rather than trusted. If the
    /// generator is edited, or the file is regenerated by a different hand,
    /// this goes red long before a driven sweep would notice anything.
    ///
    /// It doubles as the only end-to-end exercise of [`notes_on`] and
    /// [`replies_to`] against a real document, which is why the assertions
    /// below are about the model's output rather than about the bytes.
    #[test]
    fn the_fixture_carries_a_real_thread_with_words_an_author_and_a_date() {
        let path = fixture();
        let doc = pdfcer_core::document::Document::load(&path)
            .expect("fixtures/comment-note.pdf loads — regenerate it with tools/gen-comment-note-fixture.py");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(doc);
        let view = session.view();
        let notes = notes_on(&view, &pages[0]);

        // Three notes, not four: the `/Popup` is the window, not a comment.
        assert_eq!(
            notes.len(),
            3,
            "expected the open note, the reply and the closed note; got {:?}",
            notes.iter().map(|n| &n.subtype).collect::<Vec<_>>()
        );

        let open = notes
            .iter()
            .find(|n| n.contents.as_deref() == Some("Check this weld before rev C"))
            .expect(
                "the open note's /Contents must be non-empty, or every \
                     'the pop-up shows the words' assertion is vacuous",
            );
        assert_eq!(
            open.author.as_deref(),
            Some("Ken Mantle"),
            "the note needs a real /T, or 'the pop-up names the author' passes \
             on a build that draws no byline at all"
        );
        assert_eq!(
            open.modified.as_deref(),
            Some("D:20260905143000Z"),
            "the note needs a real /M, for the same reason"
        );

        // ★★★ The two halves of the `/Open` assertion, on ONE document. A
        // build that ignored the key and defaulted to closed passes the second
        // and fails the first; one that opened everything passes the first and
        // fails the second. Neither can pass both.
        assert!(
            open.authored_open,
            "the fixture's open note must be authored /Open true, or \
             `a_note_authored_open_reads_as_open` has nothing to prove against \
             a real file"
        );
        let closed = notes
            .iter()
            .find(|n| n.contents.as_deref() == Some("Dimension missing on this view"))
            .expect("the closed note is missing");
        assert!(
            !closed.authored_open,
            "the fixture needs a note the file says is CLOSED, or 'a build that \
             opens every pop-up' passes every check here"
        );

        // ★ The pop-up's own rectangle, placed away from the note on purpose —
        // see the generator's docstring. Without this, a build that always
        // draws beside the note is indistinguishable from one that honours the
        // file's placement.
        let popup = open.popup.expect("the open note must carry a /Popup");
        let rect = popup.rect.expect(
            "the /Popup must carry a usable /Rect, or the placement \
                     assertion is untestable",
        );
        assert!(
            rect.width() > 100.0,
            "the /Popup rect collapsed to {rect:?} — the canvas projection is \
             wrong, or the fixture's rectangle is degenerate"
        );

        // The thread: one reply, by somebody else.
        let replies = replies_to(&view, &pages, open.id);
        assert_eq!(
            replies.len(),
            1,
            "expected exactly one reply, got {replies:?}"
        );
        assert_eq!(replies[0].author.as_deref(), Some("Jo Smith"));
        assert_eq!(
            replies[0].contents.as_deref(),
            Some("Done - rev C issued 5 Sep")
        );
        assert!(
            !replies[0].group_member,
            "/RT /R is an ordinary reply, not a group subordinate"
        );

        // …and the closed note is not in anybody's thread.
        assert!(
            replies_to(&view, &pages, closed.id).is_empty(),
            "a note with no replies must produce an empty thread, or the \
             transitive walk is claiming unrelated annotations"
        );
    }
    /// ★★★ **A mark with nothing to say opens no window; a note does even when
    /// it is empty.**
    ///
    /// # What this is really asserting
    ///
    /// Not "has words". The rule has two halves and a test that only checked
    /// the content half would pass on a build that had dropped the subtype
    /// exemption — and that build would refuse to open a sticky note the
    /// operator had just placed and was trying to type into. **Both halves are
    /// asserted here, in one test, because a build that gets either wrong is
    /// broken in a way the operator meets immediately.**
    ///
    /// ★ The byline case is the one worth having by name. An annotation signed
    /// by a reviewer but carrying no message is a *fact about the drawing*, and
    /// the Comments panel is where facts live. Putting a window over the page to
    /// say only "B. Reviewer drew this" is the noise this predicate exists to
    /// remove, and it is easy to talk yourself into showing it.
    #[test]
    fn only_a_mark_with_something_to_say_opens_a_window() {
        let note = |subtype: &str, contents: Option<&str>, author: Option<&str>| NoteView {
            authored_open: false,
            id: ObjId {
                num: 1,
                generation: 0,
            },
            subtype: subtype.to_owned(),
            contents: contents.map(str::to_owned),
            author: author.map(str::to_owned),
            modified: None,
            anchor: Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
            popup: None,
            locked: false,
            in_reply_to: None,
        };

        // The two whose PURPOSE is the note, empty or not.
        assert!(
            has_something_to_read(&note("Text", None, None)),
            "an empty sticky note is still a note, and opening it is how you \
             write in one"
        );
        assert!(
            has_something_to_read(&note("FreeText", None, None)),
            "a free-text box's words are its appearance"
        );

        // A mark that merely MAY carry a comment.
        assert!(
            !has_something_to_read(&note("Square", None, None)),
            "a shape with no comment must fall through to selection"
        );
        assert!(
            has_something_to_read(&note("Square", Some("check this dimension"), None)),
            "a shape that DOES carry a comment must open"
        );
        assert!(
            !has_something_to_read(&note("Polygon", Some("   "), None)),
            "whitespace renders as an empty window just as surely as nothing does"
        );
        assert!(
            !has_something_to_read(&note("Circle", None, Some("B. Reviewer"))),
            "a byline is a fact for the Comments panel, not a message worth a \
             window over the drawing"
        );
    }
}
