# `canvas::pasteboard` — which surface this frame's gesture belongs to

## The report this module exists for, verbatim


> *"also objects should still be reachable even if they are off the page."*


> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

There was nothing to enable. O23's **part A** shipped — a viewport of
scrollable slack on every side of the strip, so the operator can *scroll* to
where an off-page object is ([`super::geometry::pasteboard`]). **Part B did
not**: the space that slack creates senses hover and refuses clicks, so a
press out there never became a gesture. An object dragged past the sheet
edge was invisible, unclickable, and still in the file.

## The one-way door, and the exact line it was

[`super::present::show`] allocates two kinds of interactive rectangle:

| Allocated | Covers | Sense (before) |
|---|---|---|
| the scroll **content**, once, before any page | every page, every gap, and all of the pasteboard | `hover` |
| each **page**, in strip order | that page's sheet only | `click_and_drag` |

`interact` is handed **one** `Response`, and it was always the acting page's.
So the machinery below it was never the problem — and this is the part worth
knowing, because it is why part B is small:

- [`super::mapping::PageMapping::to_page`] does **not** clamp. A screen point
  two sheets to the left of the paper maps to a truthful negative page point.
- `pdfcer-core`'s decomposer applies no page-box culling, so an object
  painted at `(-5000, -5000)` is already in `PageObjects::objects` with a
  truthful negative bounding box.
- `hit_test_point_all`'s only predicate on the query point is `is_finite`.
- The selection outline is clipped to the **viewport**, not to the page — so
  an outline drawn out in the pasteboard is already visible.

Every layer below the input surface already accepted off-page points. The
whole of part B's "reach" half is therefore: **sense the clicks, and hand
`interact` the right one of the two responses.** That is this module.

## Why a two-response choice rather than one bigger rectangle

The obvious shape — widen the page's own interaction rect until it covers
the pasteboard — is wrong, and the comment at its call site has said so for
weeks: in a continuous strip a widened page rect **overlaps its neighbours**,
and then the page that happens to be allocated last steals clicks aimed at
the page above and below it. That is a navigation defect traded for an
editing one.

The content rectangle is already allocated **before** any page, and egui
resolves an overlap in favour of the widget registered later. So the pages
keep winning on their own sheets no matter what the content senses, and the
content only ever sees a pointer that no page wanted. Nothing arbitrates;
the allocation order already did.

## The rule, and the two clauses that are not obvious

[`surface`] is a pure function of four booleans so that it can be tested at
all — `egui::Response` cannot be built in a unit test without a live
context, and a rule with this many cases that is only ever exercised by
driving is a rule that silently loses a case.


## The fifth clause, and the two-day-old defect that earned it


egui derives a popup's identity from the id of the `Response` it was
attached to — `Popup::default_response_id(r) == r.id.with("popup")`. The
canvas attaches its menu to whichever response THIS function names, so a
frame that changes the answer re-attaches the menu under an id nobody
opened; `keep_popup_open` no-ops on the stale id and `Memory::end_pass`
drops the popup as abandoned. **No close call, no event, no trace line.**

And the answer changed for the most ordinary reason there is.
`pointer_on_page` is read from `Response::contains_pointer`, which is
layer-aware — egui's own words, `egui-0.35.0/src/response.rs:323-324`: *"also checks that no
other widget is covering this response rectangle."* **The open menu is
that other widget.** Cursor enters menu ⇒ menu covers page ⇒ page does
not contain the pointer ⇒ [`Surface::Pasteboard`] ⇒ menu destroyed,
before any button went down.

⇒ The rule earned, and it is not about this canvas: **a popup's identity
is its anchor `Response`'s id, so the code that chooses between two
responses per frame must not be the code that attaches a popup** — or,
failing that, the choice must treat an open popup as belonging to its
anchor. This function takes the second route because the choice is
already expressed here, as booleans, where it can be unit-tested.

It is a clause of the SAME rule the first two rows state, not a special
case: an open menu owned by the page is an interaction in flight owned by
the page, in precisely the sense row three means by *"a band started on
the sheet and dragged off it is one gesture"*.


The third row is the one that would be got wrong by a naive "is the pointer
over the page" test, and it is not a corner case — it is the gesture the
operator already uses to catch an object hanging over the edge, and
`interact`'s own step 1 has a note explaining that `interact_pointer_pos`
keeps reporting after the pointer leaves the widget *precisely so that it
works*. Handing that frame the pasteboard's response instead would end the
drag the moment the pointer crossed the sheet edge.

## What this module does NOT do

It does not make off-page content **visible**. The raster is still sized to
the crop box, so an off-page object is reachable, selectable, movable by its
properties and draggable by its outline — and still not painted. That is the
other half of part B, it lives in the render path, and conflating the two
would have made a change that could not be driven one assertion at a time.
[`crate::render::offpage`] holds the engine properties it will stand on.

## Rule 15

Nothing here is a **ce dimension**. The coordinates this module's decision
leads to are **pdf dimensions** — points in the CAD-exported page's own
space, which is exactly why they are allowed to be negative.

## Item notes

### `fn the_page_keeps_the_frame_while_its_own_menu_covers_the_pointer`

The operator right-clicks an object, the menu opens over the sheet, and
the operator moves the cursor down onto it. `contains_pointer` is
layer-aware, so from that frame on the page reports that it does NOT
contain the pointer — the menu is covering it. Every other input is
false: no drag is in flight, and nothing has been clicked yet.

Without the popup clause this frame answers [`Surface::Pasteboard`],
`present` hands `interact` the other response, the menu is re-attached
under a different id, and egui drops it as abandoned. The operator sees
the menu vanish as they reach for it.

### `fn a_page_menu_outranks_a_pasteboard_drag`

This combination should not arise — the secondary click that opens a
menu ends any drag — but the clause order decides it, so the decision
is written down rather than left to whoever next reorders the function:
**the popup wins.** A drag whose owner is the pasteboard while a page
menu is open is a contradiction, and resolving a contradiction toward
the surface that owns the visible pop-up is what keeps the pop-up
alive, which is the failure mode that cost two days.
