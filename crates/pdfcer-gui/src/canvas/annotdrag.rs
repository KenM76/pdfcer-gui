//! # `canvas::annotdrag` — dragging a markup annotation to where it belongs
//!
//! The other half of the annotation-drag fork. [`crate::canvas::dimdrag`]
//! answers for a **ce dimension**; this answers for everything else pdfcer puts
//! on a page — a stamp, an ink stroke, a callout box, a highlight, a note.
//!
//! ## ★★★ What this closes, and how long it was open
//!
//! `FEATURES.md` recorded it under the Format contextual tab:
//!
//! > *"In `pdfcer-gui` a placed markup can be selected and deleted but not moved
//! > or resized yet — that is the Format-tab slice, still building."*
//!
//! Selecting worked. Restyling worked. Deleting worked. **Dragging did
//! nothing**, and it did nothing in the most confusing way available: the
//! gesture was *consumed*. `canvas::interact` forks on `selection.annot()
//! .is_some()`, so an annotation selection took the dimension branch, and that
//! branch answers `None` for anything that is not a ce dimension. The content
//! branch — which does move things — was unreachable behind an annotation
//! selection by construction.
//!
//! So the operator pressed inside a stamp, dragged it across the sheet, let go,
//! and the stamp was where it started with no message anywhere. That reads as a
//! broken program rather than as a missing feature.
//!
//! ## ★★★ The half a canvas cannot see, and it is why this needed an engine Pass
//!
//! `pdfcer-core` shipped `move_annotation` on 2026-08-28 (`Pass 149.0`) and its
//! note is the reason this module does not simply rewrite a `/Rect`:
//!
//! > A move has two halves and **only one of them shows up in a render.**
//! >
//! > 1. **`/Rect`** moves the painted result for free — §12.5.5 recomputes the
//! >    placement matrix from the appearance `BBox` and the new `/Rect`.
//! > 2. **The geometry keys** — `/L`, `/Vertices`, `/InkList`, `/QuadPoints`,
//! >    `/CL` — hold *absolute page coordinates*, and they are what **any other
//! >    tool** regenerates an appearance from.
//! >
//! > Move only (1) and the annotation looks right in your canvas, right in a
//! > screenshot, right in pdfcer — and is reconstructed **in the old place** by
//! > the next viewer that rebuilds it.
//!
//! ⇒ **That is a defect this shell could have shipped and never seen.** Every
//! instrument this project owns — the rendered canvas, a screenshot, a driven
//! pixel check — reads the appearance stream, and all four would have agreed
//! the stamp moved. The operator would have found out a week later, in Acrobat,
//! and reported it as *"it moved back"*. Recorded here because the class
//! generalises: **when a document format stores one fact twice, a renderer is
//! not an oracle for whether both copies were written.**
//!
//! ## ★★ Why there is no shell-side geometry arithmetic here at all
//!
//! This module computes a `(dx, dy)` in page points and sends it. It does not
//! touch `/Rect`, does not enumerate geometry keys, and does not know which
//! subtypes have them. That is not laziness about coverage — it is the same
//! rule `dimdrag` states for placement: a second implementation of the engine's
//! own arithmetic is a second thing to keep in step, and the one that drifts is
//! the one whose tests are thinner.
//!
//! `AnnotationMove::geometry_keys_moved` reports which keys were found, **and
//! an empty list is a correct answer** — a Text note, a Stamp or a Link has no
//! geometry key because its `/Rect` *is* its geometry. The engine says so
//! explicitly, and reading empty as failure is the mistake it warned about.
//!
//! ## ★ Two refusals, and the engine names the verb for each
//!
//! `EditError::AnnotationMoveWrongVerb` fires for a **widget** (use
//! `move_widget`) and for a **ce dimension** (use `move_dimension`, which
//! re-measures). Neither can reach here:
//!
//! | | why it cannot arrive |
//! |---|---|
//! | widget | `selection::annot::selectable` excludes `/Widget` outright — the form surface owns those presses |
//! | ce dimension | [`crate::canvas::dimdrag`] claims it first, and `AnnotKind` makes the fork a `match` the compiler checks |
//!
//! ⇒ The engine's refusals are the **backstop**, not the mechanism. That is the
//! arrangement `set_markup_style` established — the shell routes by `AnnotKind`
//! and the engine refuses by name — and it has now paid three times.
//!
//! ## Rule 4
//!
//! The ghost drawn while a drag is in flight is **the cursor**, which the rule
//! permits by name: the same class as a snap indicator, a rubber band or a
//! resize grip. Nothing about the annotation itself is tinted, badged or
//! flagged, and the one-line test passes — a screenshot of the canvas mid-drag
//! differs from the saved file by a marching outline, which is where the
//! pointer is and not what the document says.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. D5 (Shift constrains to one axis) is
//! applied **above** this module, in `canvas::interact`, so both branches of the
//! annotation fork and the content branch all receive one already-constrained
//! delta from one filter. A second copy of that rule here is how two drags come
//! to disagree about what Shift means.

use egui::Rect;

use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{AnnotKind, SelectionState};
use pdfcer_core::page_tree::Page;

/// The trace line a committed move writes.
// ui-text-exempt: diagnostic trace name, never displayed
const TRACE: &str = "annot-drag";

/// One frame of an annotation drag.
pub struct Frame {
    /// The pointer's travel since the press, in canvas space, already
    /// constrained by Shift if it is held.
    pub delta: egui::Vec2,
    /// Where the gesture is.
    pub phase: Phase,
}

/// Whether this selection is one this module moves, and its outline if so.
///
/// ★ Three conditions, and each one is a different kind of "no":
///
/// | condition | what it means |
/// |---|---|
/// | an annotation is selected | otherwise the content branch owns the press |
/// | it is [`AnnotKind::Markup`] | a ce dimension is `dimdrag`'s, and it does more |
/// | it is not **locked** | §12.5.3 Table 165 bit 8 — *the file* says the user interface may not change this |
///
/// The locked case is the one worth stating. It is a flag the **document**
/// carries, not a state this shell invented, and honouring it here rather than
/// letting the engine refuse is what stops a drag drawing a ghost for a move
/// that will not happen. `canvas::moving`'s obligation 3, applied one surface
/// along: *a ghost is drawn if and only if the release would commit.*
#[must_use]
fn eligible(selection: &SelectionState) -> Option<(pdfcer_core::object::ObjId, Rect)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::Markup || annot.target.locked {
        return None;
    }
    Some((annot.target.id, annot.outline))
}

/// The screen-space box a press must land in to mean *move this markup*.
///
/// The annotation's own `/Rect`, projected — the same rectangle
/// `overlay::draw_selection` strokes when a markup is selected, and the same
/// one [`drag`] translates into a ghost. @@ Three uses of one rectangle, on
/// purpose: **what the operator can see, what they can grab, and what moves
/// must be one number.** `dimdrag::grab_box` states the identical rule one
/// module along, and it exists because a grab box larger than the drawn outline
/// is a press that works where nothing is shown, and one smaller is an operator
/// missing something they can see.
///
/// `None` for anything [`eligible`] refuses, so no gesture is ever started that
/// could not commit.
#[must_use]
pub fn grab_box(
    map: &crate::canvas::mapping::PageMapping,
    selection: &SelectionState,
) -> Option<Rect> {
    let (_, outline) = eligible(selection)?;
    Some(map.rect_to_screen(outline))
}

/// Drive one frame of the drag.
///
/// Returns the ghost outline to draw, in **canvas space**, or `None` when there
/// is nothing to draw — which covers both *"this selection is not draggable"*
/// and *"this is the frame that commits"*.
///
/// ★ Nothing is previewed on the committing frame, for `dimdrag`'s stated
/// reason: the annotation is about to be redrawn where it landed, and a ghost
/// left over it would be a second copy of the same artwork, one frame stale.
pub fn drag(
    frame: &Frame,
    page: Option<&Page>,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> Option<Rect> {
    let (id, outline) = eligible(selection)?;

    if frame.phase != Phase::Complete {
        // The ghost is the selection's own outline, translated. It is the same
        // rectangle `overlay::draw_selection` strokes, which is the property
        // that matters: what the operator grabbed and what they see moving are
        // one rectangle, so a drag cannot appear to pick up something else.
        return Some(outline.translate(frame.delta));
    }

    // --- the commit ---------------------------------------------------------
    // ★★ The page arrives as a parameter rather than being read off the
    // document, and it is what lets every rule in this module be tested without
    // a window or a file. `dimdrag` takes `&OpenDoc` because it must scan the
    // dimension model; this needs nothing but a coordinate transform.
    let d = super::moving::page_delta(frame.delta, page?)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{TRACE} id={} dx={:.3} dy={:.3}", id.num, d.dx, d.dy)
    });
    // ★ A zero delta is sent, not filtered out here.
    //
    // The engine accepts one by name — *"a drag that returns to its start
    // should not make you special-case your own arithmetic"* — and filtering it
    // here would mean this shell deciding, from a float comparison, that an
    // operator's gesture was not a gesture. It costs one undo entry for a move
    // of nothing, which is what every drawing program in this class does.
    actions.push(Action::Annot(
        crate::app::actions::annot::AnnotAction::Move {
            id,
            dx: d.dx,
            dy: d.dy,
        },
    ));
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::{AnnotSelection, AnnotTarget};
    use pdfcer_core::object::ObjId;

    fn selection(kind: AnnotKind, locked: bool) -> SelectionState {
        let mut state = SelectionState::default();
        state.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id: ObjId::new(7, 0),
                kind,
                // ui-text-exempt: a PDF /Subtype name in a test fixture.
                subtype: "Square".to_owned(),
                locked,
            },
            outline: Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 30.0)),
        });
        state
    }

    /// **A markup drags and a ce dimension does not.**
    ///
    /// ★ The second half is the load-bearing one. `dimdrag` claims a ce
    /// dimension and does strictly more with it — `place_dimension` moves where
    /// the dimension is *drawn* and cannot alter the number it prints — so a
    /// ce dimension reaching this module would be a translation that leaves the
    /// dimension measuring something it is no longer next to.
    #[test]
    fn only_ordinary_markup_is_this_modules_business() {
        assert!(eligible(&selection(AnnotKind::Markup, false)).is_some());
        assert!(eligible(&selection(AnnotKind::CeDimension, false)).is_none());
    }

    /// ★★ **A locked annotation does not drag, and draws no ghost either.**
    ///
    /// §12.5.3 Table 165 bit 8 is the *document* saying the user interface may
    /// not change this. The failure this guards is the one that looks like it
    /// works: a ghost that tracks the pointer for a move the engine will refuse
    /// is a promise the release cannot keep.
    #[test]
    fn a_locked_annotation_offers_no_ghost() {
        assert!(eligible(&selection(AnnotKind::Markup, true)).is_none());
    }

    /// **A ghost tracks the pointer, and the committing frame draws none.**
    #[test]
    fn the_ghost_is_the_outline_translated_and_stops_on_commit() {
        let state = selection(AnnotKind::Markup, false);
        let mut actions = Vec::new();
        let moving = drag(
            &Frame {
                delta: egui::vec2(5.0, -7.0),
                phase: Phase::InFlight,
            },
            // No page is consulted on a non-committing frame, which is what
            // lets this be tested without one.
            None,
            &state,
            &mut actions,
        )
        .expect("a ghost");
        assert_eq!(moving.min, egui::pos2(15.0, 13.0));
        assert!(
            actions.is_empty(),
            "a frame that is not the release must raise nothing"
        );
    }

    /// **Nothing is raised for a selection this module does not own.**
    #[test]
    fn a_dimension_release_raises_nothing() {
        let mut actions = Vec::new();
        let ghost = drag(
            &Frame {
                delta: egui::vec2(5.0, -7.0),
                phase: Phase::Complete,
            },
            None,
            &selection(AnnotKind::CeDimension, false),
            &mut actions,
        );
        assert!(ghost.is_none());
        assert!(actions.is_empty());
    }
}
