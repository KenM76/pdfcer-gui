# `printgeometry`

## Item notes

### `fn frames`

Its own function because two callers need the same answer and neither may
compute it: [`super::paint`] draws these rectangles, and [`super::column`] hit-tests the
page inside them to decide whether a primary drag moves the page or pans the
view (operator request O208). A second copy of this arithmetic would let the
gesture be classified against a rectangle other than the one on screen,
which is a defect with no symptom except the occasional drag doing the wrong
thing near an edge.

`pan` is passed rather than read from the dialog so the caller decides which
frame's pan it wants. [`super::column`] hit-tests against the pan the page was
last DRAWN at, which is the position the operator pressed on.

### `fn placed_rect`

The offsets are *within the printable area*, not within the sheet, which is
why this takes `printable` rather than the sheet origin — see
[`crate::dialogs::print::spooler::Placement`].

The `Placement` here is the spooler's, spelled out in full because
this module has a private enum of its own by the same name (the dialog-or-
popout one). Importing it would make the two indistinguishable at a glance
in a file that uses both.

### `fn hatch_lost_content`

# Why ink and not geometry — operator request O113

> *"can you make it so the red pattern you put over the page if it is going
> to print beyond the printable borders is only over the areas that extend
> beyond the printable page? Our drawing get drawn 1:1 and the area that
> isn't printed is just empty border."*

`Placement::clipped` is a *geometric* verdict — the page box exceeds the
printable rectangle — and on a CAD sheet printed 1:1 the part that exceeds
it is empty paper. Hatching on that flag alone shouts about losing something
on every drawing while nothing is being lost, which is a disclosure that is
technically true and practically false. An operator who sees the same red
band on every 1:1 drawing learns to ignore it, and then does not see it on
the one sheet where the border really does have a title block in it.

So this asks [`ink::InkMask`] what is actually in the band, and hatches the
**ink extent within it**. No ink in the band ⇒ **no hatch at all**.

# All four edges, because the operator can now move the page

Operator request O208, his second clause: *"the hash lines we use to show
what won't be printed should have a line for each edge of the page."*

Before the page could be dragged, only the far edges could overhang: a
placement offsets the page *into* the printable area from its top-left
corner and `place_page` clamps that offset at zero, so the near edges were
unreachable and a near band would have hatched paper that prints. Both
halves of that changed together. The operator's displacement is applied
after the clamp and is allowed to go negative, so a page dragged left or up
loses content off the near edges, and a hatch that marked only the far ones
would be silent about exactly the crop the drag was performed to choose.

The four bands are computed unconditionally rather than under a test for
which edges overhang, because an empty band is dropped by `is_positive()`
below — so a page nobody has moved produces the same two bands it
always did, and the widening costs nothing to a job with no displacement.

# The four bands are DISJOINT, and `Rect::union` cannot make them so

`Rect::union` is a **bounding box**, not a set union: the union of a tall
strip on the right and a wide strip along the bottom is a rectangle that
also covers the region which is neither right of nor below the printable
area — paper that prints perfectly. That is the same over-hatch O113 reports,
one size smaller and hiding inside it.

So the bottom band is cut at `printable.max.x` and the two meet without
overlapping. Disjoint also means the shared bottom-right corner is hatched
once rather than twice, so its lines are the same weight as everywhere else
instead of reading as a darker patch.

# What happens when there is no mask

`mask` is `None` when the page did not render — the same degraded state
[`super::texture_for`] documents, in which the preview shows a flat fill instead of
the page. In that state the honest answer to *"is anything in the band?"* is
**"unknown"**, so the disclosure falls back to hatching the whole band.
Silence is the wrong failure direction here: a missing render must not be
able to turn a warning off.
# Returns

What the band turned out to hold, so the caption can be written from the
same computation the hatch was — see [`Overhang`].

### `fn edge_bands`

Left and right take the page's full height; top and bottom are then cut to
the printable area's own horizontal span. That asymmetry is what makes the
four disjoint: each corner of the overhang belongs to the vertical band that
covers it and to no horizontal one, so their union is exactly `placed` minus
`printable` with nothing counted twice.

A band that does not exist comes back non-positive rather than absent, so
the array is always four long and the order is a contract shared by the
hatch and by [`overhang_edges`]. An unmoved page can only overhang right and
bottom, which is why widening this from two bands to four is a no-op until
somebody moves one.

### `fn overhang_edges`

The headless evidence for the four-edge hatch, and it has to say *which*
rather than *how many*: a capture cannot tell three hatched bands from two,
and a count cannot tell a page hanging off the left from one hanging off the
right, which is the whole of what changed. The hatch is then filtered by the
ink test, so the geometry has to be reported separately from what was
actually drawn.

Derived from [`edge_bands`] rather than from the offsets, so the word in the
trace and the rectangles on screen cannot disagree.

### `fn lost_regions`

# Pure on purpose, because this is the pair that must not disagree

It returns the rectangles to hatch *and* the [`Overhang`] the caption is
written from, from **one** computation. That is the only structural
guarantee that the picture and the sentence agree: they are not two readings
of the same data, they are two halves of one answer. Splitting it out from
[`hatch_lost_content`] also makes that agreement **testable with no GUI at
all** — see [`super::tests::a_blank_overhang_hatches_nothing_and_says_so`], which
asserts the empty list and the `BlankBand` verdict together, and
[`super::tests::an_inked_overhang_hatches_only_the_ink_and_says_so`], which asserts
the hatched rectangle really is a small part of the band.

The returned rectangles are in screen points and are ready to draw; the
caller does no further arithmetic on them.

### `fn normalised_in`

A degenerate `whole` — a page placed at zero scale, which a nonsense
`/MediaBox` can produce — yields a rectangle the mask rejects rather than a
`NaN` that would propagate into the hatch geometry.

### `fn hatch`

Separate from [`hatch_lost_content`] so the geometry question (*what is
lost?*) and the drawing question (*what does a hatch look like?*) do not
share a body. The lines run at 45° and are clamped to the rectangle at both
ends, which is what lets a caller hatch several small regions without any of
them bleeding into the paper between.
