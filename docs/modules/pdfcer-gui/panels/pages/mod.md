# `panels::pages` — the document's pages, as pictures

The thumbnail grid — `FEATURES.md`'s Phase 3 row, and one of the surfaces
`MODES_AND_PANELS.md` Part 1's table gives **all three** modes.

| | |
|---|---|
| Ribbon command | `view.panel_pages` |
| Acts on the document | [`Action::GoToPage`], and **nothing else** |
| Owns | [`select::PageSelection`] — the operand list the ribbon's Pages tab already promises |

## Why a page panel sits in **Review**, and not only in Edit

It is in all three default arrangements (`crate::app::modes::defaults::spec`), and
Review is the placement that needed an argument. `README.md` records the
operator's, and it is the reason this panel offers page verbs rather than
only navigation:

> Reviewing a set means rotating a sheet to read it and extracting the
> pages you were asked about. The stance that matters is the content is
> not yours to alter, and page operations do not alter content.

That is what separates rotate/extract/delete from the Edit tab's verbs. A
rotation changes `/Rotate`; an extraction writes a *different* file. Neither
touches a single content-stream operator, so neither breaches the stance
Review takes.

## What this panel draws, and what that costs

The rendering and caching policy is [`thumbnails`]', and its header is the
one to read before changing anything here — it carries the measurements.
The one sentence that decides the shape of this file:

> **A two-pixel render of the benchmark CAD drawing costs 691 ms.** ~99 %
> of a page's cost is resolution-independent, so a thumbnail is *not*
> cheap because it is small.

Consequently this body renders **at most one page per frame**, only for
tiles that are actually on screen, and stops on its own the first time a
page proves expensive. An undrawn tile says so **in words**: a blank
rectangle the colour of paper is a picture of an *empty page*, which is a
thing a real PDF contains, so drawing one would assert something false
about the document rather than merely look unfinished.

## Two ways this panel can go silently missing, and what holds each shut

Neither failure is this module's to make: both live in `shell/`, and both
are **invisible rather than broken**, which is the expensive kind.

1. **An unregistered command hides the whole panel.**
   `crate::app::mod`'s panel registry registers a panel **only if its
   command is registered**, which is `SHELL_FRAMEWORK.md` §5b's capability
   rule — so a `view.panel_pages` left out of the manifest filters this
   panel out of every default arrangement with no error anywhere.
   `every_panel_is_reachable_from_the_ribbon` is what makes that visible.
2. **An undefined menu context detaches every right-click.** [`PAGES_ROW`]
   is attached to every tile below, on every frame, through the same
   [`MenuHost`] the canvas and the Objects panel use — and
   `egui_shell::menu::Menu::attach` treats an unknown context as *"this
   surface has no menu yet"*, so an id `crate::shell::menus::built_in`
   does not define opens nothing at all, silently.

That second seam is also the payoff of routing a right-click through a
context id rather than through a list of items at the call site: the attach
site needs no edit when the menu's contents change.

## What a click does, and what it does not

[`select`] owns the rule; the summary is that a plain click navigates and
picks one page, Ctrl+click toggles without navigating, and Shift+click
extends a range without navigating. **Only a plain click navigates**,
because building a five-page set that dragged the canvas through five
renders would cost ~4 s on a drawing set to perform a gesture that changes
nothing about what the operator is looking at.

## The selection is not a decoration

Every one of the ribbon's Pages-tab tooltips already says *"the selected
pages"* — `pages.delete` is *"Remove **the selected pages** from this
document"* — and `crate::shell::commands`' own comment on that band says
those commands *"respect the thumbnail rail's selection when there is
one"*. This panel is where that selection comes from, and
[`crate::panels::PanelsState::selected_pages`] is how a dispatch arm reads
it.

All six of the verbs this panel's own context menu offers work — rotate
left and right, delete, extract, move up and move down — through five
dispatch arms, since the two rotations share one and so do the two moves.
The reading path is through that accessor and through [`ops::operands`],
which is the single place the *"with nothing picked, act on the current
page"* rule is written down.

## What an edit does to this panel's own state

Nothing here has to remember anything, and that is by construction rather
than by discipline:

| | how it is kept honest |
|---|---|
| the **thumbnails** | keyed on `(edit_epoch, pixels_per_point)`; [`thumbnails::ThumbnailCache::sync`] empties itself the moment the epoch moves, and the epoch moves on every edit |
| the **page count** and the tiles | read from `doc.pages` every frame, which `crate::app::actions::pages::resync` refreshes from the session on every edit |
| the **picks**, after a delete | cleared by the apply arm — every picked sheet is gone, so there is nothing to point at |
| the **picks**, after a reorder | **remapped**, through [`select::PageSelection::remap`]: the permutation states where each sheet went, so the arrows stay usable twice in a row |
| the **picks**, after anything else | [`select::PageSelection::retain_below`] on the next frame, which is belt to all of the above |

## Item notes

### `const MIN_TILE_WIDTH_PTS`

Below roughly this width a drawing sheet's title block is no longer
legible and one thumbnail stops being distinguishable from the next, which
is the only job a thumbnail has. `crate::app::modes::defaults::NAVIGATOR_WIDTH` is
280 pt *because* it fits two of these, and the two numbers are meant to
stay in step.

### `const SELECTION_MAT_PTS`

A *shape* difference as well as a colour one — the tile visibly gains a
border where an unselected one has none. The standing rule is **a shape AND
a fill, never colour alone**, because a colour-only state is invisible to a
substantial fraction of operators. A mat is the version of that rule which
needs no glyph, and therefore cannot land on a font that has no glyph to
draw and render as an empty box.

The header's *"N pages selected"* line is the third, wholly textual,
statement of the same fact.

### `const DEFER_SLOT`

Its own slot rather than `PANEL_SLOT`'s, because the two answer different
questions and would overwrite each other every frame: one says what the
panel is showing, this says why it is not rendering.

### `const SETTLE_AFTER_EDIT`

The operator: *"The last thing that should matter is updating the
preview."*

# Why 250 ms, and what the number is answerable to

It has to be longer than the gap between two deliberate acts in one
sequence — ticking two check boxes, tabbing between two fields — because
rendering between them is work thrown away before anybody looks at it. And
it has to be short enough that a single edit followed by a pause feels like
the rail simply kept up.

250 ms sits above the ~100-150 ms of a comfortable double act and well
below the ~500 ms at which a delay stops reading as "just happened". It is
deliberately **not** derived from a render cost: a slow document should
defer for the same period as a fast one, because the quantity being waited
for is the OPERATOR settling, not the renderer finishing.

It is a constant rather than a literal so the next person to tune it does
so once, with a paper trail — the same argument `render::settle`'s
`ZOOM_SETTLE` makes, and this is its sibling on the other end of the frame.

### `struct DropTarget`

## Why this exists at all, rather than the drag storing a gap

Because a gap has no position until the grid has been laid out. The panel
is a hand-rolled row layout over sheets of mixed sizes at a column count
derived from the dock's current width, so *"the boundary between page 6 and
page 7"* is a rectangle that only exists inside [`grid_rows`]. Resolving it
there and carrying it out is the same shape `visible`, `go` and `tokens`
already have: an answer only the layout pass is in a position to give.

## Why the caret is a `Rect` and not a stroke

It is a **line**, and the two endpoints are all the layout pass knows. A
`Rect` carries both in one value the grid can build and [`paint_caret`] can
consume without either naming a colour or a width — which keeps the
*geometry* decision beside the tile rectangles and the *appearance*
decision beside the theme.

### `const CARET_PTS`

The same weight as [`CURRENT_RING_PTS`], deliberately: both are "the panel
pointing at something", and a caret thinner than the ring would read as a
hairline artefact on a dense grid rather than as a deliberate mark.

### `const CARET_DIMMED`

**Dimmed, not hidden.** Drawing no caret over a boundary that would not
land cannot be told apart from the panel having stopped tracking the
pointer — and the no-op boundary is where *every* drag begins, because a
block starts out hovering over itself. This is the same full-strength /
dimmed pair `canvas::guides::preview` uses to say *"release here and this
does not happen"*, at a comparable ratio (it uses 170 and 60 out of 255).

### `fn paint_caret`

# Rule 4: this is the cursor, not a mark on content

A drop caret is in exactly the class the rule permits by name — *"snap
indicators, hover highlights, rubber-bands and selection handles are the
cursor and are welcome"*. It draws nothing into a page, tints no thumbnail,
and disappears the instant the pointer is released. A screenshot of this
grid with a drag in flight differs from one of the same document saved and
reopened only by where the pointer is, which is the one-line test.

# The colour is the theme's, never a literal

[`egui_shell::theme::Theme::canvas_selection_ink`] — the same source the
current-page ring and the guide preview take, so a preset that changes the
accent changes all three together. **Not `visuals().selection.stroke`**:
that is `egui`'s selected-*widget* channel, and a thumbnail rail is showing
document content, not widgets — and this theme points the widget channel at
the accent plate, which would flood a tile.

`gamma_multiply` rather than a second, paler constant, for the same reason:
one colour with a stated relationship beats two colours that have to be
kept in step.

### `fn settle_drag`

# Why the release is read from raw input

`canvas::guides::release`'s discipline, and its reason applies here
unchanged: a drag that began on a tile may end anywhere — over the header,
outside the panel, past the end of the last row, or after the pointer left
the window entirely. A `Response` only reports releases inside the widget
that produced it, so a release elsewhere would leave the drag in flight
with a caret nobody could get rid of.

# Why it runs unconditionally

Because a drag that has started has to be able to end. Gating this on a
drop target being resolved would strand every drag released over empty
space below the last row — which is exactly where an operator lets go when
they mean *"put it at the end"* and miss.

# A drag that lands nowhere raises NO action

Not an identity permutation. `reorder_pages` would accept one, record an
undo entry, and bump the edit epoch — so a document would be marked dirty
and a `Ctrl+Z` would appear to do nothing. `ops::drop_order` refuses it by
name and this function drops the refusal, which is the same choice
`app::dispatch::pages`' move arm makes for `MoveRefusal::AtTheEdge`.

### `fn grid_rows`

# Why rows are laid out by hand rather than with `horizontal_wrapped`

Two reasons, and the second is the load-bearing one.

A wrapped layout decides where the break falls from the widths it is
handed, so a grid of pages with **different sheet sizes** — which is what
a real drawing set is — wraps at a different column count on different
rows. The eye reads that as a fault in the panel.

And a wrapped layout gives no seam at which to ask *"is this row on
screen?"*. Culling is the whole reason a 900-page document is affordable
here: every row is *allocated* (so the scroll bar is honest and nothing
jumps as pictures arrive) but only a visible row is painted, interacted
with, or considered for rendering.

### `fn the_default_navigator_width_fits_two_columns`

Asserted here rather than trusted, because the two constants live in
different modules and the claim is only true for a particular tile
width: *"a thumbnail rail one column wide wastes the dock, and three
columns makes each too small to recognise a drawing by."*

### `fn a_degenerate_width_still_produces_a_usable_grid`

Zero columns divides by zero in [`tile_width_for`] and lays out
nothing, which reads as a panel that crashed rather than one that ran
out of room.

### `fn the_columns_and_their_gaps_exactly_fill_the_width`

Exceeding it is the defect `crate::panels::content_width`'s docs
describe from the other side: a row wider than the viewport is
silently squeezed and the overflow is clipped with nothing to say so.

### `fn a_tile_takes_its_shape_from_the_page_tree`

The property that keeps the scroll bar honest while the grid fills:
the aspect ratio comes from the page tree, which is free, so each row
occupies its final height from the first frame and nothing jumps as
pictures arrive.

### `fn the_page_tile_menu_context_is_named_and_defined`

`crate::shell::menus`' own rule: *"a context id is used in exactly two
places that must agree… a typo in either produces silence rather than
an error."* This pins the spelling on both sides.

Both halves matter: the spellings must agree, and something must
actually define the context — a menu attached to a context nobody
defines opens nothing at all, silently.

### `fn every_page_verb_the_menu_would_offer_is_registered`

The rule `crate::shell::menus`' header states — *only real commands* —
checked from the panel's side before the menu exists, so the menu can
be written from this list rather than from memory. A verb that failed
here would be one to leave out, not one to add and grey.
