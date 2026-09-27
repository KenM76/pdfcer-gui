//! # `notepopupmodel` — what a note says, and where its window goes
//!
//! The **pure** half of the note pop-up: it reads the document and answers
//! three questions, and it draws nothing, stores nothing and decides nothing
//! about the interface.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/notepopupmodel.md`.

use std::collections::BTreeSet;

use egui::{Pos2, Rect};
use pdfcer_core::annot::{Annotation, ReplyType, page_annotations};
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::canvasmapping::annot_canvas_rect;

/// One annotation that can carry a note, with everything a pop-up needs.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteView {
    /// The annotation's object id — what every `EditSession` verb takes.
    ///
    /// `Option<ObjId>` on the engine's model becomes a plain `ObjId` here,
    /// because [`notes_on`] **drops** an annotation with no id: a dictionary
    /// written directly into `/Annots` is a malformed file (§12.5.2 Table 164
    /// requires an indirect object) and there would be nothing for a pop-up's
    /// Edit or Delete to name. `pdfcer_gui::panels::comments` lists one and says
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
    pub modified: Option<String>,
    /// The annotation's own `/Rect`, in **canvas space** — the same
    /// zoom-independent space `pdfcer_gui::canvas::selection::annot` caches
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
    ///
    /// This field therefore survives on a [`NoteView`] only as a **guard**: it
    /// is `None` for every note this module returns, and a test asserts so. It
    /// is kept rather than dropped because dropping it would make that
    /// assertion inexpressible, and because `Reply` carries the same fact for
    /// the rows in a thread.
    ///
    /// ⚠ This used to say *"read only — `pdfcer-core` v0.38.0 has no verb that
    /// authors an `/IRT`"*. `Pass 253.0` closed that; the Comments panel
    /// authors replies through `pdfcer_gui::app::actions::annot::AnnotAction::Reply`.
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
#[must_use]
pub fn notes_on<G: ObjectGraph + ?Sized>(graph: &G, page: &Page) -> Vec<NoteView> {
    let mut popups: Vec<(ObjId, Option<Rect>, Option<bool>)> = Vec::new();
    let mut parents: Vec<Annotation> = Vec::new();

    // One walk, two collections. The pop-ups have to be gathered in the same
    // pass because a parent names its companion by id and the id alone carries
    // no rectangle — §12.5.6.14's `/Parent` back-reference is deliberately not
    // modelled by `pdfcer-core`, whose `annot` module doc lists it under *"what
    // this module deliberately does NOT model yet"* and says *"the authoritative
    // direction is the parent's `/Popup`"*. The modelled direction is the field
    // `Annotation::popup`, so the pairing is ours to make and it is made from
    // the parent's side.
    //
    for annot in page_annotations(graph, page.id) {
        if annot.is_popup {
            if let Some(id) = annot.id {
                popups.push((
                    id,
                    annot.rect.and_then(|r| canvas_rect(r, page)),
                    annot.open,
                ));
            }
            continue;
        }
        if annot.is_widget() || annot.flags.suppressed_on_screen() {
            continue;
        }
        //
        // `EditSession::add_reply` places a reply at **its parent's own
        // `/Rect`** — deliberately, and the engine says why: *"a reader draws a
        // reply inside its parent's window rather than at its own
        // coordinates"*. So a reply is a second annotation sitting exactly on
        // top of the comment it answers, and it is appended to `/Annots`, which
        // makes it the LAST match — and [`under`] takes the last match so the
        // topmost note wins a click.
        //
        //
        // And the reply is not hidden by this — it is shown where §12.5.6.2
        // says it belongs. [`super::thread`] lists the whole transitive thread
        // inside the root's window, so excluding replies here **removes a
        // duplicate**, not a route: before this line a reply's words appeared
        // twice, once in its parent's thread and once in a bubble of its own.
        //
        // ⚠ The cost, named rather than left to be found: the reply's own
        // `/Popup` — which `add_reply` authors and `ReplyAdded::reply_has_popup`
        // reports — is a window nothing in pdfcer draws. Another reader will
        // draw it. That is disclosed at the moment of authoring rather than
        // silently absorbed; see
        // `crate::text::panels::comments::reply_posted`.
        if annot.in_reply_to.is_some() {
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
            let companion = annot
                .popup
                .and_then(|pid| popups.iter().find(|(candidate, _, _)| *candidate == pid));
            let popup = annot.popup.map(|pid| PopupBox {
                id: pid,
                rect: companion.and_then(|(_, rect, _)| *rect),
            });
            Some(NoteView {
                authored_open: read_open(annot.open, companion.and_then(|(_, _, open)| *open)),
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
#[must_use]
fn read_open(annotation: Option<bool>, popup: Option<bool>) -> bool {
    annotation.or(popup).unwrap_or(false)
}

/// An annotation `/Rect` in canvas space, or `None` when it is unusable.
fn canvas_rect(rect: pdfcer_core::page_tree::Rect, page: &Page) -> Option<Rect> {
    annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
}

/// **Whether this annotation has a pop-up worth opening at all.**
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

/// **Is there anywhere in this document to record a window state?** — the
/// R83 gate on the *Open by default* control.
#[must_use]
pub fn can_record_open_state(note: &NoteView) -> bool {
    note.subtype == "Text" || note.popup.is_some()
}

/// **Which note is under `point`** — topmost wins.
#[must_use]
pub fn under(notes: &[NoteView], point: Pos2, tolerance: f32) -> Option<&NoteView> {
    notes
        .iter()
        .rev()
        .find(|note| note.anchor.expand(tolerance).contains(point))
}

/// One reply in a thread.
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
    /// It matters to what the operator is being shown. For a group member
    /// the standard says the subordinate's own `/Contents`, `/M`, `/T` and the
    /// rest *"shall be ignored"* in favour of the primary's, so the words
    /// displayed beside it are words a conforming reader is instructed **not**
    /// to use. `pdfcer-core` deliberately does not apply that rule —
    /// `Annotation::contents` stays *"the raw value the dictionary carries"* —
    /// so pdfcer shows the raw value, and rule 4 makes
    /// saying so mandatory, which is what this flag is for.
    pub group_member: bool,
}

/// **The thread hanging off `root`**, gathered from the whole document.
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
const MAX_THREAD_DEPTH: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;

    fn id(num: u32) -> ObjId {
        ObjId::new(num, 0)
    }

    /// **A note the file says is open reads as open.**
    #[test]
    fn a_note_authored_open_reads_as_open() {
        assert!(read_open(Some(true), None));
    }

    /// The other half, and it is what makes the first one mean something: a
    /// note authored **closed** must not open. Asserting only the open case
    /// would pass on an implementation that returned `true` for everything —
    /// every pop-up in the document open on load, which is the same defect
    /// wearing the other value.
    #[test]
    fn a_note_authored_closed_stays_closed() {
        assert!(!read_open(Some(false), None));
    }

    /// **Absent means closed**, which is Table 172's stated default value —
    /// not an assumption this module is making.
    #[test]
    fn a_note_with_no_open_key_is_closed() {
        assert!(!read_open(None, None));
    }

    /// **The pop-up's own `/Open` is consulted when the note has none.**
    #[test]
    fn a_shapes_open_state_comes_from_its_popup() {
        assert!(read_open(None, Some(true)));
        assert!(!read_open(None, Some(false)));
    }

    /// The note wins when both carry the key. Stated as a test rather than
    /// left to the `or`, because the precedence is a decision with a reason
    /// (see [`read_open`]) and a reordering would be silent.
    #[test]
    fn the_note_outranks_its_popup() {
        assert!(!read_open(Some(false), Some(true)));
        assert!(read_open(Some(true), Some(false)));
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

    /// **The topmost note wins, not the first one found.**
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

    /// The tolerance widens the target, which is what makes a 20 pt sticky
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

    /// **THE FIXTURE CARRIES WHAT THE DRIVEN CHECKS ASSERT ABOUT.**
    #[test]
    fn the_fixture_carries_a_real_thread_with_words_an_author_and_a_date() {
        let path = fixture();
        let doc = pdfcer_core::document::Document::load(&path)
            .expect("fixtures/comment-note.pdf loads — regenerate it with tools/gen-comment-note-fixture.py");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(doc);
        let view = session.view();
        let notes = notes_on(&view, &pages[0]);

        // **TWO notes**, not four and not three — and the two exclusions
        // are different rules that this one number is the control for.
        //
        // The `/Popup` is the window rather than a comment: §12.5.6.14 is a
        // `shall`, and it has never been listed.
        //
        assert_eq!(
            notes.len(),
            2,
            "expected the open note and the closed note; a reply must not be \
             drawn as an independent note, or answering a comment makes that \
             comment unclickable on the canvas. Got {:?}",
            notes.iter().map(|n| &n.subtype).collect::<Vec<_>>()
        );
        // …and the positive control for that exclusion, without which
        // `len() == 2` would also pass on a build that had dropped the reply
        // from the file, from the walk, or from the fixture. The reply must
        // still be **in the document** and still reachable as part of the
        // thread — which is asserted below, on `open.id`, against the same
        // annotation this loop declines to list.
        assert!(
            notes.iter().all(|n| n.in_reply_to.is_none()),
            "a listed note carries an /IRT, so the reply exclusion did not fire"
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

        // The two halves of the `/Open` assertion, on ONE document. A
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

        // The pop-up's own rectangle, placed away from the note on purpose —
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
    /// **A mark with nothing to say opens no window; a note does even when
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
    /// The byline case is the one worth having by name. An annotation signed
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

    /// **Only an annotation with somewhere to write it is offered the
    /// *Open by default* control** — R83, and the two halves are different
    /// rules.
    #[test]
    fn only_a_note_with_somewhere_to_record_it_may_record_it() {
        let note = |subtype: &str, popup: bool| NoteView {
            id: ObjId::new(1, 0),
            subtype: subtype.to_owned(),
            contents: Some("words".to_owned()),
            author: None,
            modified: None,
            anchor: Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
            popup: popup.then(|| PopupBox {
                id: ObjId::new(2, 0),
                rect: None,
            }),
            authored_open: false,
            locked: false,
            in_reply_to: None,
        };

        // A `/Text` carries `/Open` itself, companion or not.
        assert!(can_record_open_state(&note("Text", true)));
        assert!(can_record_open_state(&note("Text", false)));
        // A shape's window state lives only on the companion.
        assert!(can_record_open_state(&note("Square", true)));
        // …and with no companion there is nowhere to put it. This is the row
        // that makes the control R83-correct rather than merely cautious.
        assert!(!can_record_open_state(&note("Square", false)));
        assert!(!can_record_open_state(&note("Ink", false)));
    }
}
