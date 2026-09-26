# `panels::objects` — everything drawn on the current page

Three parts: the panel body here, the page decomposition in [`provider`],
and the per-object description in [`summary`].

## What it is FOR — the operator's own words

> *"I'd like to have a layer tree there for the document that I can also
> click on to select objects. at least that way we can troubleshoot
> better what I am clicking on in the GUI area."*

So its first job is answering **"what am I looking at"**, and every
design choice below is subordinate to that. It is a diagnostic instrument
first and a navigation aid second. The clause about *selecting* is the
half that arrives at S4 — see "What is not here yet".

## Front-most FIRST — justified, not merely conventional

The list is drawn in **reverse paint order**: the last-painted (topmost)
object is the first row. Two reasons, in priority order:

1. **It matches what a click does.** Hit-testing resolves overlapping
   candidates topmost-first, so the object an operator is most likely
   confused about is at the top of the list, not scrolled to the bottom
   of a thousand rows. For a panel whose whole purpose is "what did I
   just click", any other order buries the answer.
2. It is the prevailing convention for layer/object panels (top of list =
   top of z-order), cited strictly as a metaphor-level convention.

**The row's visible `#n` is the PAINT-ORDER index, not the display
position.** That is deliberate: `#n` is the number `pdfcer
object-list` prints as `index=`, and the number
`object-move` / `object-delete` / `node-move` take as an operand. A
display-position number would look equally authoritative and address a
different object.

## The nesting is the LEVEL LADDER, and nothing else

**object → part → point**, which is exactly the ladder the canvas walks
when it lands. `PathObject` owns `subpaths` and each subpath owns its
segments; a text object owns its runs. Every level here is structure
`pdfcer-core` already models and addresses with the same indices, so the
tree and the canvas will agree **by construction** rather than by care.

[`provider::PartKind`] is the one place that dispatch is decided —
a path's part is a subpath, a text object's is a run, an image has none —
so the row builder never matches on `VectorObject` itself. A text object
caps at two rungs by construction: a run has no anchors, so
`subpath_node_points` cannot produce a point for it and no guard is
needed anywhere.

### What is deliberately NOT nested here

**Marked-content / optional-content grouping.**
`pdfcer_core::vector::PageObjects` has **no optional-content-group
membership for page content at all** — `VectorObject::{Path,Text,Image}`
carry no `/OC` — so there is no such grouping to render, and inventing
one would be a lie about the document's structure. That level becomes
available only once `decompose_page` tracks `BDC`/`EMC` membership.
Deferred, not overlooked.

(Do not conflate this with the ce-dimension group OCGs, which are an
annotation-layer visibility mechanism and have their own surface.)

## Two properties of a row, and why each is the way it is

### 1. Row text ellipsises rather than clipping

A row here can easily be wider than its pane: `#1382  Path · filled
(even-odd) and stroked #1A73E8, 0.50 pt wide · 6681 node(s) · zero height`
is not a contrived example. Row text must not clip, and the requirement is
`OPERATOR_REQUESTS.md` O123 — *"rows that ellipsise with a tooltip instead
of hard-clipping mid-character."* Three pieces answer it, and each is
needed:

- **[`crate::panels::elide_to_width`] per row, against that row's own
  room.** The pane's width less this row's indent and its expander column,
  because a point row indented twice has 28 pt less text room than the
  object row above it. The row is shortened to fit and ends in a single
  ellipsis, so the eye is told it was cut.
- **The full text on hover, on every shortened row**, including the point
  rows and the capped-rows disclosure — which never carried one before,
  because they could not overflow while the pane scrolled sideways and they
  can now.
- **`ScrollArea::vertical`, not `both`.** A horizontal axis would exist to
  reach the part of a row past the pane's edge; there is no such part, and
  a scroll bar with nothing beyond its viewport is a control that cannot do
  anything — R9.

The rule the elision honours is that loss must not be **silent**. An
operator seeing `#27 Text · "A1" · AAAAAA+SpaceGrotesk-Bold 1` with a `2`
invisibly cut off is the defect; `…` plus a hover is not it. The trade is
stated rather than glossed: a very long row is read in a tooltip rather
than by dragging a bar across the panel.

### 2. No dock width fixes an over-long row, so the ROW is what is short

Widening is not the remedy, and the numbers are why. On an A1 sheet the
widest full description wants **473.6 pt** and even the narrowest wants
**306.3 pt**, against a 314 pt pane holding 296 pt of text room — eight of
eight rows overflowing. (`objects-rows overflow=` traces the worst row's
overshoot, which is the field that answers this question from outside.) A
dock wide enough for 473.6 pt is about 526 pt, half of an 1,100 pt window.
`EDIT_INSPECTOR_WIDTH` does not reach the operator in any case: a restored
workspace (`mode-changed … remembered=true`) never reads it.

So what a row **says** is what is short. [`row_label`] draws
`crate::text::panels::objects::object_row_headline` — index, kind, the one
or two facts that tell this object from its neighbour, and a single
disclosure mark — while [`row_description`] hands the **full** description
to the row's hover, on every object row, elided or not. The widest headline
on that sheet is **207.6 pt**, 70 % of the room.

That is what *master–detail* means: the detail pane is on screen, in the
same column, an inch below. A master row that restated it would spend the
width twice on one fact and elide the identity only the master carries.

**R128 is not engaged.** Nothing here feeds a measurement back into a
size: the widths are constants, the elision reads the pane it is given, and
the row is what changed. A build that made the dock follow its content
would be the feedback loop that rule forbids.

And a third guard on top, because neither helps a 6,681-row part:
[`POINT_ROWS_PER_PART`] caps how many point rows one part contributes, and
the cap is **disclosed with both numbers** rather than the list being
quietly shortened. A silently truncated list is indistinguishable from a
short one.

### 3. Scrollbars are visible

egui's default `ScrollStyle` is `floating()` — 2 pt, zero allocated
width, fully transparent when the pointer is elsewhere — so a working
scroll area is indistinguishable in a screenshot from content clipped at
the container edge. `super::scroll_style` fixes it for every panel; the
full measurement is in [`super`]'s header.

## Virtualized, never silently truncated

A complex drawing decomposes to tens of thousands of objects.
`ScrollArea::show_rows` lays out only the rows actually on screen, so the
list stays cheap at any size and **no cap is applied to objects** — there
is nothing to disclose because nothing is hidden. The one cap that does
exist, on points within a part, prints both numbers.

The visible rows are materialised into a flat `Vec` once per frame rather
than walked recursively, because `show_rows` needs a row **count** and
the ability to draw an arbitrary slice — and a recursive tree walk cannot
answer "what is row 4,000" without walking to it. With everything
collapsed the list is exactly the object count, so the nesting costs
nothing until it is used.

## Selection is shared with the canvas, in both directions

Clicking an **object** row raises
`SelectionAction::SelectObject`, so the row click *is* a canvas selection
gesture: the object highlights on the page and the Properties panel
describes it. The reverse holds too — the highlighted row is derived from
`doc.selection.object_indices_on(page)` on every frame, so a canvas click
highlights its row. There is one opinion about what is being worked on and
this panel renders it rather than keeping a second.

Clicking the already-selected row **deselects**, which is what clicking a
selected item does in every list in every application.

Object-scoped, deliberately: `object_indices_on` answers about page
**content**, which is what this tree lists. An annotation or a ce dimension
selected on the canvas leaves every row here unhighlighted, correctly.

**Not yet:** multi-select, Shift+click, scroll-to-reveal.

Part and point rows are **not clickable**. A row that responded to a click
by selecting its parent object instead would be a control answering a
different question from the one it was asked.

## The row's right-click, and the one command it deliberately does not
offer

An **object** row carries the `objects.row` context menu
([`crate::shell::menus::OBJECTS_ROW`]). Right-clicking a row **focuses
it** first — the panel's equivalent of the canvas's select-first rule, so
the menu is about the row the pointer is on rather than about whichever
row was last clicked — and then offers `file.properties`, which is the
command that puts the focused object's description on screen.

Part and point rows carry **no** menu, for the same reason they are not
clickable: a menu on a part row would have to be about its parent object,
which is answering a question nobody asked.

**`format.delete` is not on this menu.** It was kept off while a row click
wrote a panel-local focus rather than a selection, because a Delete gated on
`selection.any` would then have removed whatever was selected on the canvas
rather than the row the pointer was on. That condition no longer holds — the
row click is a selection gesture — so the omission is now unexamined rather
than argued. `DEFECTS.md` D48. Deletion is still reachable: the row click
selects, and Delete on the canvas then acts on that selection.
