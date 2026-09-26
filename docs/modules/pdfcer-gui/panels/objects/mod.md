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

## Item notes

### `struct RowText`

A named struct rather than the `(row, String, Option<String>)` tuple it
replaced, and the reason is the third string: with three fields of which two
are `String`-ish and one is `Option<String>`, a tuple destructure at the
draw site is one transposition away from hanging the *headline* on the hover
and drawing the *description* — which would look almost right, and would
reintroduce the defect this change fixes. Named fields cannot be
transposed.

### `fn row_text_room`

The pane, less the indentation this row will spend before its first
character and less the expander column it holds whether or not it draws one.

# Why this is a function

Because it is the arithmetic the elision decision is made against, and a
row-depth term that is silently dropped reintroduces clipping **for exactly
the deepest rows** — the ones an operator had to work hardest to reach. That
is not hypothetical: the same term was the subject of
`indentation_counts_toward_the_row_width` under the previous mechanism, and a
term that mattered under one mechanism matters under its replacement.

⚠ It is not the whole call site, and the difference is worth stating: no
unit test can see whether [`body`] still **calls** `elide_to_width` at all.
Deleting that call leaves every test in this module green. That is the gap
`the_inspector_is_one_master_detail_column` exists to close, and it is why
it samples pixels rather than only reading a trace.

### `const ROWS_SLOT`

Its own slot rather than sharing `objects-panel`'s, because the two lines
change on different events: the panel line moves when the page does, and
this one moves when the dock is dragged. Sharing a slot would make each
suppress the other's changes.

### `const EXPANDER_SPACE`

Added to every measured row width so the container is wide enough for the
control as well as the text. Held even for a leaf, so labels stay aligned
down the column (R83: hold the space, draw no dead control).

### `fn expander`

A **separate control** from the row, not a click on the label: expanding
to look inside and pointing the Properties panel at the object are
different intents, and one gesture cannot mean both without one of them
being a surprise.

Drawn as ASCII rather than as chevron art, which is an interim rather than
a preference: `crate::icons` carries `Icon::ChevronDown` and
`Icon::ChevronRight`, and this control does not yet read them. It is a
real, clickable, tooltipped control either way; only the glyph changes.

### `fn row_label`

Object rows go through [`summary::describe_object`] and
[`crate::text::panels::objects::object_row_headline`], the master form of
the **single description path** the Properties panel also reads — so a fill
colour cannot be described one way here and another way there.

Deliberately the headline and not
[`crate::text::panels::objects::object_row`], the *full* description —
`OPERATOR_REQUESTS.md` O123 defect 2: at the width this panel opens at,
**every** row of the full form elides. The full description is not lost;
[`row_description`] returns it and [`body`] hangs it on the row's hover.
See `object_row_headline`'s header for the measurement.

### `fn row_description`

The counterpart to [`row_label`], which draws the headline. Only an
**object** row has a longer form: a part row reads
*"Subpath 3"* and a point row *"Node 12"*, and there is nothing longer to
say about either — attaching a tooltip that repeated the row would be the
noise [`body`] already refuses on a row that fits.

Returned as an `Option` rather than as an empty string so the call site
cannot hang a blank tooltip on a row by forgetting to test it. An empty
popup is a control that opens and says nothing, which is worse than none.

### `fn fixture_provider`

**Necessary for anything about text runs.** The resolver-free
`decompose` used by [`provider`] above reports `runs.len() == 0` for
every text object by construction — the run layout needs the font
resolver — so a content-stream literal cannot exercise the text side
of the part rung at all. That is a property of the seam, not of the
panel, and it is why the geometry cases use one helper and the text
cases use the other.

### `fn a_collapsed_tree_lists_every_object_once_topmost_first`

Two invariants in one, and the second is the panel's whole diagnostic
premise: row 0 is the object painted LAST, because that is the one a
click resolves to first. Getting it backwards buries the answer at
the bottom of a thousand rows.

### `fn expanding_walks_object_then_part_then_point`

The point indices are the ones `node-move --node N` takes, and they
keep counting across a part boundary. A tree that restarted them at 0
per part would print numbers that address a different point.

### `fn a_text_objects_parts_are_runs_and_have_no_points`

Its parts are runs, and a run has no anchors — so the point rung is
unreachable for text *by construction* rather than by a guard. If
this ever produced a point row, something would have started matching
on the object's kind in a second place.

### `fn a_capped_point_list_says_how_many_it_hid`

The measured case is 6,681 anchors in one path object. A list quietly
shortened to its first N is indistinguishable from a list that is N
long, which is the same defect `bookmarks_truncated` exists to
prevent one panel over.

### `fn a_long_row_is_shortened_to_the_pane_and_stays_recoverable`

The requirement is **nothing is lost silently**: the drawn row fits the
pane, it ends in the ellipsis so the eye knows it was cut, and it is a
prefix of the full row the hover carries. The alternative mechanism —
`crate::panels::content_width` growing the container past the viewport
so `ScrollArea::both` draws a horizontal bar — satisfies the letter of
that and not the request: *"rows that ellipsise with a tooltip instead
of hard-clipping mid-character."*

⚠ **The pane is narrow on purpose.** The drawn row is the headline,
which is 32 characters for this fixture's object against 51 for the
full description, so a wider pane fits it and the test would assert
nothing. The pane is narrowed rather than the fixture lengthened,
because the subject of this test is the **mechanism**, not the width:
any row wider than its room must be shortened and stay recoverable, and
a narrow dock is the ordinary way that happens.
`every_object_row_of_the_a1_sheet_fits_the_measured_pane` is the test
that owns the width question.

### `fn a_deeper_row_has_less_room_for_its_text`

The term this arithmetic is most likely to lose. Dropping `row_indent`
leaves every object row correct and clips every point row by 28 pt —
which reads as "the deep rows are broken" rather than as an arithmetic
slip, and is the reason the sum is a function.

### `fn indentation_counts_toward_the_row_width`

A point row indented twice needs 28 pt more container than its text
alone. Measuring the text and forgetting the indent reintroduces
clipping for exactly the deepest rows, which are the ones an operator
had to work hardest to reach.

### `fn an_object_row_draws_the_headline_and_hovers_the_description`

One description path: the row and the Properties panel read the same
`ObjectSummary`, so a fill colour cannot be described one way here
and another way there.

It asserts the **pair**, not the drawn text alone: the drawn text and
the hovered text are two different renderings of that one record, and a
test naming only one of them passes on a build that hangs the wrong
string on the hover.

### `fn a_part_or_point_row_has_no_description_to_hover`

A part row reads *"Subpath 3"* and a point row *"Node 12"*: there is no
second, fuller sentence to put behind either, and a tooltip that
repeated the row would be a popup that opens and says nothing.

### `fn every_object_row_of_the_a1_sheet_fits_the_measured_pane`

# The measurement this test is built from, and its provenance

The driven check `the_inspector_is_one_master_detail_column` reported
**8 of 8 object rows do not fit**, and the panel's own `objects-rows`
line said how far: `pane=314.0 overflow=473.6`. Re-measured headlessly
here at the same 314 pt pane (`room` = 296 pt after the expander
column), against `fixtures/a1-titleblock.pdf` page 0:


Every description was over the 296 pt room — which is why *every* row
was elided — and the widest headline uses 70 % of it.

⚠ **314 pt is not "the default width", and the driven check's failure
message said it was.** That run's trace reads `mode-changed …
remembered=true`: the dock restored a saved workspace, so
`EDIT_INSPECTOR_WIDTH`'s 360 was never applied. Which is the second
reason widening a constant could not have fixed this — a remembered
layout does not read it.

# Why this is a unit test as well as a driven one

The driven check owns *"the dock drew it"*; this owns *"the arithmetic
says it fits"*, and it is the half that runs on every commit. Neither
substitutes for the other: this test cannot see whether [`body`] still
calls `elide_to_width` at all (see [`row_text_room`]'s ⚠), and the
driven check cannot run headless.

The fixture is pinned for the reason the driven check pins it: on a
document of short rows this assertion could not fail.

### `fn a_row_quoting_a_full_length_string_still_elides`

The complement of the test above, and the reason it is needed: an
implementation that made every row fit by *deleting the elision call*
would satisfy that one and be a regression of the request that produced
it. Measured at the same 296 pt room: a text row quoting
[`ROW_TEXT_CHARS`]-worth of wide characters wants **457.1 pt**.

### `fn a_row_naming_a_missing_object_does_not_panic`

The rows are materialised from one snapshot of the provider, so this
cannot happen today. It is asserted because the row builder and the
row renderer are separate functions, and the day something rebuilds
one without the other, an index panic in a draw closure is a crash
with no useful stack.
