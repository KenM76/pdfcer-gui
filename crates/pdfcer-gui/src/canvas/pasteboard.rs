//! # `canvas::pasteboard` — which surface this frame's gesture belongs to
//!
//! ## The report this module exists for, verbatim
//!
//! `OPERATOR_REQUESTS.md` **O23**, second half, 2026-08-21:
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! and, on 2026-09-10, after the first half had shipped and the second had not:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! There was nothing to enable. O23's **part A** shipped — a viewport of
//! scrollable slack on every side of the strip, so the operator can *scroll* to
//! where an off-page object is ([`super::geometry::pasteboard`]). **Part B did
//! not**: the space that slack creates senses hover and refuses clicks, so a
//! press out there never became a gesture. An object dragged past the sheet
//! edge was invisible, unclickable, and still in the file.
//!
//! ## ★★★ The one-way door, and the exact line it was
//!
//! [`super::present::show`] allocates two kinds of interactive rectangle:
//!
//! | Allocated | Covers | Sense (before) |
//! |---|---|---|
//! | the scroll **content**, once, before any page | every page, every gap, and all of the pasteboard | `hover` |
//! | each **page**, in strip order | that page's sheet only | `click_and_drag` |
//!
//! `interact` is handed **one** `Response`, and it was always the acting page's.
//! So the machinery below it was never the problem — and this is the part worth
//! knowing, because it is why part B is small:
//!
//! - [`super::mapping::PageMapping::to_page`] does **not** clamp. A screen point
//!   two sheets to the left of the paper maps to a truthful negative page point.
//! - `pdfcer-core`'s decomposer applies no page-box culling, so an object
//!   painted at `(-5000, -5000)` is already in `PageObjects::objects` with a
//!   truthful negative bounding box.
//! - `hit_test_point_all`'s only predicate on the query point is `is_finite`.
//! - The selection outline is clipped to the **viewport**, not to the page — so
//!   an outline drawn out in the pasteboard is already visible.
//!
//! Every layer below the input surface already accepted off-page points. The
//! whole of part B's "reach" half is therefore: **sense the clicks, and hand
//! `interact` the right one of the two responses.** That is this module.
//!
//! ## ★★ Why a two-response choice rather than one bigger rectangle
//!
//! The obvious shape — widen the page's own interaction rect until it covers
//! the pasteboard — is wrong, and the comment at its call site has said so for
//! weeks: in a continuous strip a widened page rect **overlaps its neighbours**,
//! and then the page that happens to be allocated last steals clicks aimed at
//! the page above and below it. That is a navigation defect traded for an
//! editing one.
//!
//! The content rectangle is already allocated **before** any page, and egui
//! resolves an overlap in favour of the widget registered later. So the pages
//! keep winning on their own sheets no matter what the content senses, and the
//! content only ever sees a pointer that no page wanted. Nothing arbitrates;
//! the allocation order already did.
//!
//! ## ★★ The rule, and the two clauses that are not obvious
//!
//! [`surface`] is a pure function of four booleans so that it can be tested at
//! all — `egui::Response` cannot be built in a unit test without a live
//! context, and a rule with this many cases that is only ever exercised by
//! driving is a rule that silently loses a case.
//!
//! | Pointer | Button | Answer | Why |
//! |---|---|---|---|
//! | over the sheet | — | [`Surface::Page`] | the ordinary case, unchanged |
//! | in the pasteboard | up | [`Surface::Pasteboard`] | a hover out there is a hover over off-page content |
//! | in the pasteboard | **down, pressed on the sheet** | [`Surface::Page`] | ★ a rubber-band started on the paper and dragged off it is one gesture, and it belongs to the page it began on |
//! | in the pasteboard | **down, pressed out there** | [`Surface::Pasteboard`] | ★ the new half: a band may now START off the sheet |
//!
//! The third row is the one that would be got wrong by a naive "is the pointer
//! over the page" test, and it is not a corner case — it is the gesture the
//! operator already uses to catch an object hanging over the edge, and
//! `interact`'s own step 1 has a note explaining that `interact_pointer_pos`
//! keeps reporting after the pointer leaves the widget *precisely so that it
//! works*. Handing that frame the pasteboard's response instead would end the
//! drag the moment the pointer crossed the sheet edge.
//!
//! ## What this module does NOT do
//!
//! It does not make off-page content **visible**. The raster is still sized to
//! the crop box, so an off-page object is reachable, selectable, movable by its
//! properties and draggable by its outline — and still not painted. That is the
//! other half of part B, it lives in the render path, and conflating the two
//! would have made a change that could not be driven one assertion at a time.
//! [`crate::render::offpage`] holds the engine properties it will stand on.
//!
//! ## Rule 15
//!
//! Nothing here is a **ce dimension**. The coordinates this module's decision
//! leads to are **pdf dimensions** — points in the CAD-exported page's own
//! space, which is exactly why they are allowed to be negative.

/// **Which of the canvas's two interactive rectangles owns this frame's
/// gesture.**
///
/// Deliberately a two-variant enum rather than a `bool`, because the two
/// answers are not "yes/no" about one thing — they are two different surfaces
/// with two different meanings, and a `bool` at the call site would read
/// `if on_page { … }` with no hint that the other branch is the pasteboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// The acting page's own rectangle — the sheet.
    Page,
    /// The scroll content outside every sheet: O23's pasteboard.
    Pasteboard,
}

/// **Decide which response `interact` should be handed this frame.**
///
/// See the module header for the table this implements and the argument for
/// each row. The parameters are taken as plain booleans, read off the two
/// `egui::Response`s at the one call site, so that every row of that table is
/// reachable from a unit test.
///
/// * `pointer_on_page` — the pointer is inside the acting page's rect
///   (`Response::contains_pointer`).
/// * `page_has_gesture` — egui considers the page's widget to be the one being
///   dragged or clicked right now, including after the pointer has left it
///   (`dragged() || drag_stopped() || clicked()`).
/// * `pasteboard_has_gesture` — the same, for the content rectangle.
/// * `pointer_in_content` — the pointer is inside the scroll content at all.
///   ⚠ Read but not currently able to change the answer: it is here because the
///   only case it would change is a pointer outside the scroll area entirely,
///   and in that case **neither** response reports a gesture and the answer is
///   arbitrary. Kept as a parameter, with this note, rather than dropped —
///   a later reader deciding whether a third `Surface` is needed will want to
///   know the question was asked.
#[must_use]
pub const fn surface(
    pointer_on_page: bool,
    page_has_gesture: bool,
    pasteboard_has_gesture: bool,
    pointer_in_content: bool,
) -> Surface {
    let _ = pointer_in_content;
    // ★ The in-flight gesture wins over where the pointer happens to be NOW.
    // Row 3 of the table: a band started on the sheet and dragged off it is one
    // gesture and stays with the page. This clause is first for that reason.
    if page_has_gesture {
        return Surface::Page;
    }
    // ★ …and symmetrically, a band started in the pasteboard and dragged ONTO
    // the sheet stays with the pasteboard. Without this the drag would change
    // owner mid-gesture, which the gesture machine reads as the first one being
    // abandoned — the rubber-band would vanish the instant it touched paper.
    if pasteboard_has_gesture {
        return Surface::Pasteboard;
    }
    if pointer_on_page {
        Surface::Page
    } else {
        Surface::Pasteboard
    }
}

#[cfg(test)]
mod tests {
    use super::{Surface, surface};

    /// The ordinary frame: pointer on the sheet, nothing in flight.
    #[test]
    fn a_pointer_on_the_sheet_is_the_pages() {
        assert_eq!(surface(true, false, false, true), Surface::Page);
    }

    /// **The new half.** Pointer out in the pasteboard, nothing in flight — the
    /// frame that used to be thrown away.
    #[test]
    fn a_pointer_off_the_sheet_is_the_pasteboards() {
        assert_eq!(surface(false, false, false, true), Surface::Pasteboard);
    }

    /// ★★★ Row 3, and the one a naive "is the pointer over the page" test gets
    /// wrong: a rubber-band that STARTED on the paper and has been dragged off
    /// it. The pointer is no longer on the sheet and the gesture still belongs
    /// to the page — handing this frame the pasteboard's response would end the
    /// drag at the sheet edge, which is the gesture the operator already uses
    /// to catch an object hanging over it.
    #[test]
    fn a_band_dragged_off_the_sheet_stays_with_the_page() {
        assert_eq!(surface(false, true, false, true), Surface::Page);
    }

    /// …and its mirror. A band started in the pasteboard that has been dragged
    /// back over the paper stays with the pasteboard, so the gesture does not
    /// change owner half way through and get read as abandoned.
    #[test]
    fn a_band_dragged_onto_the_sheet_stays_with_the_pasteboard() {
        assert_eq!(surface(true, false, true, true), Surface::Pasteboard);
    }

    /// ⚠ Both reporting a gesture should not happen — egui resolves an overlap
    /// to exactly one widget — but if it ever does, the page wins. A frame that
    /// silently preferred the pasteboard here would take a click away from a
    /// sheet the operator was plainly pointing at.
    #[test]
    fn the_page_wins_a_contradiction() {
        assert_eq!(surface(true, true, true, true), Surface::Page);
        assert_eq!(surface(false, true, true, true), Surface::Page);
    }

    /// Outside the scroll area entirely: neither reports a gesture, the answer
    /// is arbitrary, and it must at least be *stable* — a rule that flapped
    /// here would hand `interact` a different response on alternating frames.
    #[test]
    fn outside_the_content_is_stable() {
        assert_eq!(surface(false, false, false, false), Surface::Pasteboard);
        assert_eq!(surface(false, false, false, false), Surface::Pasteboard);
    }
}
