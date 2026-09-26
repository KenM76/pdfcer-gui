# `pdfcer-gui/canvas/trace`

## Item notes

### `const LAYOUT_SLOT`

One slot for all of them — the `canvas` line and both
`canvas-unavailable` variants — because they answer **one** question
("where is the canvas, and is there one?"), and a consumer reads the
answer as the most recent line about it. Splitting them would let a stale
`canvas` line sit after the page stopped rendering, with nothing in the
trace to say the situation had changed.

### `const REGION_PAGE`

This is the rect every canvas coordinate conversion is relative to, so it
is the one a screenshot oracle needs in order to crop the page out of a
window capture. See [`crate::diag::ui_rect`] on naming.

### `const REGION_CANVAS_VIEWPORT`

Distinct from [`REGION_PAGE`], and the difference is exactly where the
old GUI's selection-offset defect lived (see the centring comment inside
[`super::show`]): at fit-page on a small page the two rects differ by the
centring margin, and a check that measured one while meaning the other
would sample the grey surround.

### `const REGION_STRIP`

# Why a third rect, when [`REGION_PAGE`] and [`REGION_CANVAS_VIEWPORT`]
already exist

`OPERATOR_REQUESTS.md` **O177**:

> *"fit page when in 2 pages side by side views should fit the two side by
> side pages onto the canvas - right now it snaps to fitting one."*

That sentence is a claim about the **spread**, and until this region existed
no consumer outside the process could see one. [`REGION_PAGE`] is the
ACTING page's rect and says so in its own doc; under a facing mode the
acting page is one half of what is on screen, so a check asserting "the two
pages are inside the canvas" from `page` alone would have to reconstruct the
other half from a page size it scanned out of the PDF and a spread gap it
hard-coded. Both of those are the harness guessing at the application's
arithmetic, which is the exact defect class `crop=` and `rot=` were added to
this trace to end.

⇒ The union of the drawn pages' rects is the one rect that answers *"is what
the operator is looking at on the canvas, and is it centred on it?"* for
**every** display mode at once: it degenerates to [`REGION_PAGE`] under
`Single`, it is the spread under `Facing`, and it is the visible run of the
strip under either continuous mode.

⚠ It is the union of the pages **drawn this frame**, not of the whole
document: under a continuous mode the strip extends far past the viewport
and only the laid-out window of it is in `drawn`. A consumer asserting
containment must therefore do so only in a non-continuous mode, where every
page of the row is drawn by construction.

Absent — the region is simply not published — on a frame that drew no pages
at all, which is the same silence [`REGION_PAGE`] keeps and for the same
reason: a zero-sized rect at the origin is a measurement of something that
did not happen.

### `const REGION_PAGE_MESSAGE`

Shares a name across the no-pages and render-failed arms on purpose: it
is the same region of the screen serving the same purpose, and a
legibility check asking "is the canvas's explanatory text readable?"
should not have to enumerate every reason the text might be there.

### `const SELECTION_SLOT`

Separate from [`LAYOUT_SLOT`] because the two answer different questions
and de-duplicate on different timescales: the layout line reports *where
the canvas is*, this one reports *what the operator just did to the
selection*. Sharing a slot would let each silence the other.

### `const TEXT_SELECTION_SLOT`

Separate from [`SELECTION_SLOT`] for the reason every slot here is separate:
they answer different questions. It is also the only honest arrangement,
because the two are mutually exclusive by construction
(`canvas::textsel`'s header §3) — sharing a slot would make a mode change
look like a selection event, since the *other* selection's line would arrive
next in the same slot and silence nothing.

### `fn pointer`

# Why this is here rather than in a later stage

`PROJECT_PLAN.md` §4.2 lists three prerequisites that *"belong in S1, not
later"*, and the first is: **`ui-verify` scripts document-space
coordinates, never absolute screen coordinates.** User-rearrangeable
panels make widths arbitrary at runtime, and the project's own RAG
records this exact class producing a filed-then-retracted false
coordinate-space defect.

A harness cannot script in document space unless the application will
*tell* it where a screen point lands in document space. This is that
channel, and it exists from S0 so the harness written at S1 has
something to read on its first run rather than needing the canvas
reopened to add it.

Two spaces are reported because the harness needs both and the
distinction is exactly where coordinate bugs live:
`page=` is **canvas space** (Y-down, origin top-left, `/Rotate` applied),
`pdf=` is genuine **PDF user space** (Y-up, un-rotated lower-left origin)
— the frame an annotation `/Rect` is written in.

Costs nothing when tracing is off: [`crate::diag::trace_changed`] takes a
closure and never calls it.

# Why this is gated on movement

`pointer_latest_pos` returns the **last known** position, not "the position
it moved to this frame", so an ungated line re-reports the same three
coordinate pairs on every frame a stationary pointer sits over the canvas.
Measured: **50 identical lines in 9 seconds.** A driven run is minutes long,
so the events that actually matter — an open, a click, a deletion — end up
separated by thousands of lines saying nothing, and `ui-verify` re-parses
the whole capture after every settle.

The gate is [`crate::diag::trace_changed`] rather than a hand-rolled
comparison against a stored `Pos2` for a specific reason: the printed line
is the thing the consumer reads, so the printed line is the right unit of
"changed". A movement too small to alter `{:.2}` is a movement no parser
could have seen.

The line's *shape* is the contract — `screen=`, `page=`, `pdf=` and `zoom=`
— and the gate changes only how often it is written, never what it says.

### `const SURFACE_SLOT`

Separate from [`LAYOUT_SLOT`] for the ordinary reason — a surface decision
is taken every frame and the layout line is not — but also for a sharper
one: the whole VALUE of this trace is that it changes. A run that never
emits `surface=pasteboard` is a run in which the operator never reached off
the page, and sharing a slot with a line that churns would hide that.

### `fn surface`

`canvas-surface surface=page|pasteboard onpage=<bool> pagegesture=<bool>
pastegesture=<bool>`

Emitted from `present::show` immediately before `interact`, on the value
[`super::pasteboard::surface`] returned and the four booleans it was
handed. The booleans ride along deliberately: when a driven check finds
`surface=page` where it expected `pasteboard`, the next question is always
*"which input was wrong?"*, and without them the answer needs a second run
with a debugger. They are the difference between a check that reports a
failure and one that reports a cause.

⚠ De-duplicated, so a stationary pointer emits once and not once per
frame. A check must therefore read this with `events()` and not assume one
line per gesture — three presses in the pasteboard with no page visit in
between produce ONE line.

### `const HALO_SLOT`

Separate from [`LAYOUT_SLOT`] for the same reason [`SURFACE_SLOT`] is: the
tier is decided every frame, and a run that never emits `tier=halo` is the
evidence that O23's "see" half never engaged.

### `fn halo`

`canvas-halo tier=whole|halo|region known=<bool> offpage=on|off box=<llx,lly,urx,ury|none>`

* `tier=whole` — an ordinary page: one raster of the crop box, unchanged
  since before this feature existed.
* `tier=halo` — the raster has been widened to cover content placed off the
  sheet. **This line is the feature.**
* `tier=region` — above the pixmap (or ink) ceiling, so the request is the
  visible rectangle instead; off-page content is still covered, by
  [`crate::render::halo::reach`] rather than by a widened box, and `box=`
  is the visible region in the page's own space.

`known=` is the field that distinguishes *"this page has no off-page
content"* from *"nobody has decomposed this page yet"*, which look
identical from outside and need opposite responses. See
`crate::app::state::OpenDoc::content_bounds_if_known` for why the second
state exists at all and why it resolves itself a frame later.

`offpage=` is the field that keeps `known=` HONEST under the View ▸ Display
toggle.

With off-page display switched off, [`crate::canvas::tier`] substitutes
`None` for the content bounds — which is the same value the *not yet
decomposed* state produces, and would therefore be reported as
`known=false`. Those are opposite facts: one resolves itself on the next
frame and one never will, because the operator asked for it. A reader of
the trace — and `ui-verify`, which reads exactly this distinction — must
be able to tell *"the halo is suppressed by a setting"* from *"the halo
is missing"*, so the suppression is stated on its own field rather than
left to be inferred from a value that means something else.

⚠ De-duplicated, so a still canvas emits once and not once per frame.

### `const PASTEBOARD_SLOT`

Its own slot rather than sharing [`HALO_SLOT`], because the two lines
report the two halves of one switch and they do not change together: the
tier is a property of the page's content, the overhang is a property of the
content AND the zoom. Sharing a slot would let a change in one suppress the
other and leave a check reading a fossil.

### `fn pasteboard`

`canvas-pasteboard offpage=on|off overhang=<x>,<y>`

# Why this exists as its own line

The toggle has **two** consequences, produced by two functions:
[`crate::canvas::tier::decide`] widens the *raster*, and
[`crate::canvas::tier::overhang`] widens the *layout*. [`halo`] above
reports the first. Nothing reported the second, which left the operator's
own words — *"when not showing the stuff that is off page there shouldn't
be a gap between pages where the stuff is"* — with no oracle a driven check
could read.

That asymmetry is the exact shape of defect this suite keeps finding: a
check watches the off-sheet ink disappear, reports green, and a band of
grey is still standing where the operator said it must not be. The ink and
the gap are different observations and they need different evidence.

# What the numbers mean

**Screen points, not page points.** The value has already been multiplied
by the zoom, because that is the form its nine consumers want. A check
comparing across two launches must therefore either hold the zoom — which
`view.zoom_actual` does — or compare against zero. Comparing against zero
is the assertion that matters anyway: with the switch off this is
**exactly** `0.000,0.000`, produced by an early return rather than by
arithmetic on a measured box, so not one rounding away from a one-pixel
band nobody could explain. See [`crate::canvas::tier::overhang`] on why
that distinction was made deliberately.

# Why `offpage=` is repeated here

So the line stands alone. A reader who has only this line must be able to
tell *the switch is off, hence no overhang* from *the switch is on and this
page simply has nothing outside its sheet*. Both print zeros and they are
not the same fact — the first is a setting working, the second is a page
with nothing to show, and a third state that prints the same zeros is *the
page has not been decomposed yet*, which resolves itself one frame later.
`offpage=` separates the first from the other two; [`halo`]'s `known=`
separates those two from each other.

⚠ De-duplicated, so a still canvas emits once and not once per frame.

### `const PLACE_SLOT`

Its own slot, not shared with [`LAYOUT_SLOT`]: the placement decision is a
*request*, the layout line is the *outcome*, and the whole value of this
trace is being able to see the two disagree. Sharing a slot would let one
overwrite the other and hide exactly the case it exists to expose.

### `fn placed`

# Why this line exists

`canvas::offset`'s header says it plainly: the decision returns an offset
rather than applying one so that *"which branch won?"* is answerable. It is
answerable only if something publishes the answer, and the `canvas` line's
`off=` field is not that — it reports what the area **settled on**, which is
a different number whenever egui clamps the request against a content size
it has not laid out yet.

That gap is the whole diagnosis. *"The decision asked for the wrong offset"*
and *"the decision was right and egui overrode it"* need opposite fixes, and
the settled offset alone cannot tell them apart: a document that opens with
its page parked off the bottom-right corner reports `off=[0.0 0.0]` under
either. This line is what says whether the **request** was `0.0,0.0` too.

⚠ **`(0.0, 0.0)` is the most over-subscribed value in this subsystem.** The
deep-tier arm returns it as a literal; `geometry::strip_offset`'s lower
clamp manufactures it from any sufficiently negative page-local solve; a
strip-space page scroll to the top of the content produces it honestly; and
*no arm firing at all* leaves the area sitting on it. That is why `src=`
exists beside `want=`: the number alone cannot tell those four apart.

# Fields

* `src=` — [`crate::canvas::offset::Decision::source`], the name of the arm
  that claimed the frame, or `none`. **Read this field first.** Every other
  field on the line is only interpretable once it is known which arm's
  arithmetic produced the number.
* `want=` — the offset the decision returned, or `none` when no source
  claimed the frame and egui's own scrolling is left alone. **A `none` is
  the normal case**; a canvas that forced an offset on every frame would be
  a canvas the wheel could not move. `src=` and `want=` are always `none`
  together; they are separate fields because a reader greps one and
  arithmetic uses the other.
* `frames=` — `OpenDoc::canvas_frames` as the decision saw it, i.e. before
  this frame's increment. The open-seed arm fires on exactly `1`, so this
  field is what distinguishes "the seed did not fire" from "the seed fired
  and was ignored".
* `strip=` — the whole strip's drawn size, which is what every `geometry`
  margin term above is computed against.
* `row=` — the row's rect in strip space, the origin the request is
  measured from.
* `vp=` — the viewport measured inside the scroll bars.
* `over=` — `OpenDoc::pasteboard_overhang` as this frame published it. On
  the line because it is a hidden input to **every** `geometry` term here:
  `pasteboard`, `strip_margin` and `content_extent` all take it, so a
  request that does not reconcile with the other fields is most often a
  frame where the overhang was not what the reader assumed. It is zero
  except when the sheet is far enough off-page to need the extra room.

⚠ De-duplicated, so a still canvas emits once rather than once per frame —
which means a repeated identical request prints once. That is the right
trade for a line whose readers are looking for a transition, but a check
counting requests must not count these.

### `const CONFINED_SLOT`

Its own slot rather than sharing [`LAYOUT_SLOT`], on this module's standing
rule: the layout line changes on every scroll and every zoom, and a
confinement is a **transition that happens once** at the far end of a climb.
Sharing a slot would bury the one line a check is looking for under two
hundred it is not.

### `fn confined`

`canvas-confined axes=none|x|y|xy`

# What it is evidence of

[`crate::canvas::geometry::visible_origin_range`] carries the defect: above
the deep-position threshold nothing bounded the anchor, an unbounded
`panned`/`zoomed_about` walk carried it a hair past the end of the strip,
and the whole page left the screen. The fix clamps it. **A clamp that fires
is invisible by construction** — the picture simply stays where it should —
so without this line a check could only assert the *absence* of
`canvas-unavailable`, which is satisfied equally by the clamp working and by
the climb never having got deep enough to need it.

⇒ That distinction is this project's most expensive recurring defect: an
assertion both outcomes satisfy measures neither. `axes=` is what makes the
check able to say *the mechanism ran*, separately from *the symptom is
gone*.

# Four values and no numbers, deliberately

The magnitudes are already on `canvas-pos`, published by the code that
decided them, and this slot is de-duplicated through
[`crate::diag::trace_changed`]. A slot carrying a float **changes on every
frame of a zoom gesture** and so defeats its own de-duplication — one line
per wheel notch, a hundred lines per climb, and the transition a check wants
is no longer findable. The same reasoning is written on `strip-beyond-raster`,
which was built with its scale deliberately left out.

# Why `none` is emitted at all

So the transition is readable in **both** directions. A check that only ever
saw `axes=xy` could not tell a canvas that stopped needing the clamp from a
canvas that stopped emitting. ⚠ `none` also covers the two frames that have
no anchor to confine — see [`crate::canvas::deep::confine`]'s early returns.
At this tier the anchor is seeded on the first frame, so those are
unreachable in a real climb; a reader who needs to distinguish them has
`canvas-pos … deep` on the same frame.

### `fn move_ghost`

`canvas-move-ghost boxes=N rung=object|part suppressed=no|o63`

# What it is evidence of

A ghost that is withheld looks exactly like a ghost that is drawn at zero
displacement, and both look exactly like a drag the shell declined to
preview at all. The operator's report for all three is the same sentence:
*nothing follows the pointer*. `boxes=` separates them — it is the number
of rectangles this frame actually stroked, so `0` is a claim the painter
makes about itself rather than an absence a check has to infer.

`suppressed=o63` names the one case where withholding is correct: an inner
rung whose real geometry is already travelling, where a perimeter box on
top of it is O63's complaint word for word. Any other zero is a defect.

# No delta, deliberately

The slot is de-duplicated through [`crate::diag::trace_changed`], and a
displacement changes on every frame of a drag — carrying it would emit one
line per mouse-move and bury the transition a check is looking for. The
magnitude a check needs is on the commit's own `move-text-line(s)` line,
published by the code that applied it.

### `enum RasterGhostReason`

An enum rather than a `&str` at the call site because the value is what a
driven check matches on, and a misspelling in a trace field is a check that
silently stops asserting anything.

### `fn raster_ghost`

`canvas-raster-ghost drawn=N clipped=M reason=none|geometry|no-raster|off-raster|no-selection`

# What it is evidence of

`drawn=` is the number of sub-rectangles of the page texture this frame
actually blitted, so a zero is the painter's own statement rather than an
absence a check has to infer — the same property [`move_ghost`]'s `boxes=`
has, for the same reason, and the two together separate *"the outline moved
but the lettering did not"* from *"nothing moved"*.

`clipped=` is how many of those had their source cropped to the part of the
page that has actually been rastered. Separate from `drawn=` because a
cropped copy is still a correct copy — the alternative, stretching the
source to fill the destination, is a subtler wrong than a missing one —
and because a check that starts seeing crops where it saw none is reading a
change in the region tier rather than in this feature.
