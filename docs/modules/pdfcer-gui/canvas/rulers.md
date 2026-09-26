# `canvas::rulers` — the ruler gutters, the tick ladder, and the drawing grid

`RIBBON_IA.md` §5.2's View ▸ Display row *"Rulers · Grid · Guides"*, and
`FEATURES.md`'s last unbuilt Phase 3 line. This module owns two of the
three; [`super::guides`] owns the third and is built on the arithmetic
here.

---

## 1. What unit the ruler reads in — the question, and the answer

`RIBBON_IA.md` says *"rulers along the canvas edges, **in the document's
units**"*, and that phrase hides a real decision rather than a formatting
detail. pdfcer is a CAD-drawing tool: `pdfcer-core` carries a whole
dimensioning subsystem in which a **group** owns a scale, a display unit
and a number format, and in which the displayed value of a length is
*derived* — `value = measured_points × scale`.

The rule this module implements, in one sentence:

> **The ruler reads in exactly the same units, at exactly the same scale,
> and through exactly the same formatter as a dimension placed on the same
> sheet would.**

Concretely, three states:

- `ScaleState::NeverSet`, no scale ever set → **PDF points**: `100.00 pt`,
  `200.00 pt` …
- `ScaleState::OneToOne`, explicitly 1:1 → the group's unit at true size,
  so a 72 pt span reads `25.40 mm`.
- `ScaleState::Calibrated` → the group's unit at the operator's scale, so
  on a sheet drawn 1:50 in metres a 72 pt span reads `1.270 m`.

### Why points is the *default*, and why that is not an arbitrary pick

Because it is the only unit the document itself states. Every other number
this application shows about geometry — the `canvas-pointer` trace's
`page=` reading, the Properties panel's extents, an annotation `/Rect` — is
in PDF user-space units, and a ruler that invented millimetres while every
neighbouring readout said points would be a second measuring system with no
way to tell which one a number came from. Points is what the file says;
anything else is an interpretation, and an interpretation needs the
operator to have supplied it.

### Why a set scale changes the ruler, and why the numbers come from *core*

Because an operator who has calibrated the sheet has said what the drawing
*means*, and a ruler that ignored that would be measuring the paper while
the drawing is about a building. The two readings must also **agree to the
digit**: someone who runs a linear dimension along a wall and then checks
it against the ruler is entitled to the same number, in the same unit, with
the same decimal marker and the same number of places.

The only way to guarantee that is to call the same function, so [`Scale`]
calls [`pdfcer_core::dimension::format_measurement`] — the function that
renders every dimension label and every live measurement readout — and this
module never formats a length itself. That has a second, quieter payoff:
**the ruler contributes no operator-visible string to `crate::text` at
all.** Its labels are core's spelling of a measurement, unit abbreviation
and ISO comma included. The dimensioning subsystem's own rule is
*"disclosures rendered by core, never invented at the GUI layer"*; a ruler
tick is a measurement rendered by core, for the same reason.

### Where the scale is read from, and the honest limit of that today

[`Scale::of`] reads [`pdfcer_core::edit::EditSession::dimension_model`] and
takes the **default group** (`DEFAULT_GROUP_ID`), which `pdfcer-core`
guarantees always exists. That is the document's own
`/PieceInfo /pdfcer /Private` sidecar, so a sheet calibrated in a previous
session — by this application once the scale surface lands, or by
`pdfcer` today — is honoured on open with nothing else wired.

**The limit, stated rather than left to be discovered:** this build has no
way to *set* a scale. `measure.set_scale` is registered and has no dispatch
arm, and `EditSession`'s scale verbs have no caller in the GUI. So today
the ruler reads points on every document that has not been through the CLI,
which is every test document in `D:\Dev\temp\pdfcer`. That is why this is
written as *reading a document property* rather than as *reading a pdfcer
setting*: when the scale surface lands it needs no change here, and until
it does the ruler is not pretending to a calibration nobody supplied.

**There is no cache, and the number is measured rather than assumed.**
`dimension_model()` walks catalog → `/PieceInfo` → `/pdfcer` → `/Private`
and deserialises; on a document with no sidecar — every ordinary PDF — it
stops at the first `None` after four dictionary lookups and one catalog
clone. **Measured on `ncored-benchmark-cad-drawing.pdf` (A3, 129,758
objects): 3.65 µs per call, called twice a frame** — once by the rulers and
once by the grid. The same sheet's *raster* is 1,244 ms
(`render-async-done ms=1244`), and a 60 Hz frame is 16,667 µs.

Caching it would need a key on `edit_epoch`, a `RefCell` on `OpenDoc` and a
staleness question, bought for 7 µs against a frame that has just uploaded
a page texture; the same argument [`crate::viewer::strip`]'s header makes
about not caching the strip.

## Measured cost of the whole overlay

On the same sheet, at its fit zoom of 1.3634 in a 2,200 × 1,300 window:

| | |
|---|---|
| ruler ticks | 223, of which 22 are labelled |
| grid lines | 205 |
| tick walk + label formatting | **8.9 µs per frame** |
| `Scale::of`, twice | **7.3 µs per frame** |
| **total** | **≈16 µs, or 0.1 % of a 60 Hz frame** |

Most of the 8.9 µs is the 22 `format_measurement` calls, each of which
allocates a `String`. That is the thing to attack if this ever needs to be
faster, and it does not: the page it is drawn over costs four orders of
magnitude more.

---

## 2. Which space the grid is drawn in — page space, per page

Under a continuous mode several pages are on screen at once, so "where is
the grid" has two candidate answers and only one survives contact with a
drafter.

**A viewport-space grid** is drawn once, over the whole scrolling area, and
is anchored to the window. Scroll, and it slides across the paper: a line
that sat on an intersection comes off it, and the same feature on two
sheets falls at two different places in the grid. It is wallpaper, not a
reference. It is also cheaper and easier to write, which is why it is the
one to be careful about.

**A page-space grid** is drawn per page, anchored to that page's own
top-left corner, and clipped to that page's rectangle. It scrolls *with*
the sheet, so an intersection is a fixed place on the drawing; every sheet
in a set gets the same grid in the same place relative to its own border;
and the gaps between pages carry no grid, which is truthful — there is no
paper there to be ruled.

pdfcer draws the second, and the reason is what a grid is *for*. A drafter
uses one to judge alignment and spacing **on the drawing**, and every
answer read off it is a statement about the sheet. A grid not attached to
the sheet cannot make such a statement. The same argument settles the
guides, which is why [`super::guides`] stores a guide against a **page**.

**Every numbered ruler tick has a grid line under it**, because both come
from the same 1-2-5 [`Ladder`] — which is the whole reason to ship a ruler
and a grid rather than two independent ornaments: a feature sitting on a
grid line can be read off the ruler without counting. ⚠ Not the stronger
"every *heavy* grid line is numbered", which is **false**: both steps are
1-2-5 numbers and a 1-2-5 number is not always divisible by a smaller one
(500 over 200 is 2.5). Asserted by `canvas::grid`'s
`every_ruler_label_has_a_grid_line_under_it`.

The two ladders are resolved by **different** constructors, and which one
is which is load-bearing: [`Ladder::for_labels`] bounds the *labelled*
step, because that is what must not overlap; [`Ladder::for_lines`] bounds
the *drawn* step, because every grid line is drawn. ⚠ Using the first for
the grid puts a line every 1.4 screen pixels on the benchmark sheet — a
tint rather than a grid — and neither a screenshot nor the suite catches
it.

---

## 3. Rulers reserve layout space, and the reservation is a CONSTANT

The gutters sit along the canvas edges, so switching them on shrinks the
viewport the strip is laid out into — and the viewport is what
[`crate::viewer::ViewState::apply_fit`] divides by, **on every frame a fit
mode is active**. That is rule **R128** (`app::status`'s header carries the
measured case): *a panel whose size feeds a fit-to-viewport computation has
a fixed size.* A content-driven gutter — one that grew to fit its widest
label — would be a measured feedback loop: a wider label on frame N is a
smaller fit scale on frame N+1 is a different label on frame N+2, which
presents as a page visibly shrinking frame after frame.

So [`THICKNESS_PTS`] is a constant, it is the *only* thing [`reserve`]
subtracts, and nothing here measures a string before deciding how much room
to take. Labels are laid out **inside** a gutter whose size was already
settled, and one that does not fit is clipped — the same posture the dock
takes with a panel body, for the same reason.

Switching rulers on therefore costs exactly one re-fit, once, which is what
the operator asked for.
[`tests::the_gutters_are_a_constant_bite_out_of_the_viewport`] asserts the
constancy rather than trusting this paragraph.

---

## 4. Rule 4 — why chrome the operator switched on is allowed

`panels`' one-line test: *would a screenshot of the editing canvas differ
from a screenshot of the same document saved and reopened?* For a ruler and
a grid the answer is **yes, and the operator is the reason**. Rule 4
forbids pdfcer drawing *its own inferences* onto the page — a badge saying
"this text was OCRed", a dashed outline meaning "this bound is
approximate". Nothing here is keyed on any property of the content: the
grid's spacing comes from the zoom and the document's stated scale, never
from what is on the sheet; the ruler's numbers come from the page geometry,
never from an analysis of it; and both vanish the instant the operator
switches them off, which is the second half of what makes them chrome
rather than marking.

The version that would fail the test is a grid that **snapped to something
pdfcer found** — a detected drawing frame, an inferred module size. There is
no such code here and there must not be: that is an inference, and an
inference owes an off-canvas report.

---

## What is in this file

- [`THICKNESS_PTS`], [`reserve`], [`Gutters`] — the constant bite out of
  the viewport, and the child `Ui` the canvas is drawn into.
- [`CanvasGeometry`] — what the frame learned about where its pages are,
  handed back so the gutters can be drawn against it.
- [`Scale`] — what unit the ruler reads in, read from the document.
- [`Ladder`], [`nice_step`] — the 1-2-5 tick ladder, chosen in display
  units and returned in points.
- [`draw`] — the gutters: ticks, labels, the page's own edges, the pointer.

The grid is [`super::grid`], drawn one per visible page in that page's own
space and off the [`Ladder`] here.

## Item notes

### `const MINOR_TICK_PTS`

Deliberately well under half [`MAJOR_TICK_PTS`] rather than a little under:
the two lengths are the *only* thing distinguishing a labelled tick from an
unlabelled one wherever the label has been clipped by the gutter's end, so
the difference has to survive a glance.

### `const PAGE_SPAN_ALPHA`

Low, because it is a *band the ticks and labels are drawn on top of* — see
[`draw`] on why it is a tint rather than the line it started as. High enough
that the paper's edge is findable at a glance on both shipped themes, which
is the whole job: at a fit zoom that edge is a one-pixel difference in fill
and the ruler is the only place it can be stated plainly.

### `const REGION_RULER_TOP`

Published so a screenshot oracle can crop the ruler out of a window capture
and ask the question a trace cannot answer — *is this legible, and is it
aligned with the page it is measuring?* See [`crate::diag::ui_rect`] on
naming: names are matched literally by checks, so renaming one silently
un-aims whatever was measuring it.

### `fn default`

The unit carried alongside `NeverSet` is millimetres, not "points":
`pdfcer-core` has no `Unit::Point`, because points are what a
measurement is *before* a unit is chosen. `format_measurement` ignores
the unit entirely in the never-set branch and renders ` pt`, so the
value here is unobservable — it is `Millimeter` only because
`DimensionModel::new` seeds its default group that way and agreeing
with core costs nothing.

### `fn positive`

The one guard both [`Ladder`] constructors take against a degenerate zoom.
Written once because the two answer it identically and a version that
forgot would produce a ladder of NaN — which paints nothing at all, and is
therefore indistinguishable from the feature being switched off.

### `fn minor_divisions`

Chosen so a minor tick always lands on a number the operator can name: 100
divides into ten tens, which keeps the halfway point on a tick; 200 divides
into four fifties, because 200/10 = 20 would put ticks at 20, 40, 60 …
under a label reading 200, and counting those is harder than not having
them at all.

### `fn from_major`

The one place the display-unit → point conversion happens, so the two
constructors cannot disagree about which side of the division the scale
goes on — a mistake that is invisible at 1:1, where `upp` is 1.

### `fn first_index`

An index rather than the value, because [`Self::steps`] multiplies an
integer index for the exactness reason that function documents. It is
still split out because it is the one place a `ceil` could be a `floor`
and nobody would notice: the ticks would simply start one step outside
the view, which is invisible on a ruler and is a missing first line on a
grid.

### `fn ticks`

Walks the **minor** ladder once and promotes a tick to major when
[`Ladder::is_major`] says so, rather than walking two ladders separately.
One walk means a major tick and the minor tick that would have coincided
with it can never land at two different positions through a rounding
difference — which is exactly the sort of half-point disagreement that
reads as a blurry ruler.

### `fn the_gutters_are_a_constant_bite_out_of_the_viewport`

The property the whole of §3 is about, asserted rather than argued: the
content rect's size depends only on the outer rect and
[`THICKNESS_PTS`], never on anything that could vary with what is
drawn. The test drives it across a range of outer sizes because the
failure mode is a reservation that is *nearly* constant — one that
scales with the canvas, say — which a single-size check would pass.

### `fn the_two_rulers_do_not_overlap_each_other_or_the_canvas`

The corner exists precisely so the two rulers do not both draw into it
— see [`Gutters::corner`] — and an overlap would be two ticks at two
alphas at the place the eye starts reading.

### `fn gutters_for`

The rects are a pure function of the outer rect and the toggle, and
`reserve`'s only other effect is advancing the parent's cursor — so a
headless twin can assert the whole of the geometry. Written as a
re-derivation rather than by calling `reserve` because constructing an
`egui::Ui` in a unit test needs a `Context` and a full frame, and a
geometry test that needs a window is a geometry test nobody runs.

### `fn the_tick_ladder_is_one_two_or_five_times_a_power_of_ten`

The property: the answer is always of the form 1, 2 or 5 times a power
of ten, and it is always at least the minimum asked for. Both halves
matter — a ladder that rounded *down* would put labels closer together
than [`MIN_MAJOR_PITCH_PTS`] allows, which is the overlapping-labels
failure that constant exists to prevent.

### `fn a_degenerate_ladder_still_produces_finite_ticks`

A ruler with the wrong spacing is a visible mistake; a ruler whose
ticks are all at NaN paints nothing, which is indistinguishable from
the feature being switched off.

### `fn labelled_ticks_keep_a_readable_pitch_at_every_zoom`

The law the whole feature rests on, and the one a screenshot cannot
check across fourteen zoom levels: labels never come closer than
[`MIN_MAJOR_PITCH_PTS`], and — because the 1-2-5 sequence never jumps
by more than 2.5× — never spread further than 2.5 times that either. A
ruler whose labels drift apart until only two are visible is as useless
as one whose labels overlap.

### `fn the_ruler_reads_points_until_the_document_says_otherwise`

Asserted through [`Scale::label`] rather than through
`units_per_point`, because the operator's question is what the *label
says*, and the label is core's. A change in core's spelling should fail
here rather than surprise someone reading a drawing.

### `fn a_calibrated_sheet_gets_round_numbers_in_its_own_unit`

This is the whole point of §1 and it is the part a reader is most
likely to doubt: at a 1:50 metric scale the ruler must label metres,
not the awkward point values that happen to correspond to them.

### `fn the_origin_is_never_labelled_minus_zero`

The ruler's zero is the page's top-left corner, and a view scrolled so
that the paper starts a little way into the gutter walks the ticks up
from a negative value. Accumulating `value += minor` from there lands on
`-1.8e-15` rather than on zero; core's formatter is fixed-place and
prints the sign, so the operator reads *minus zero* at the origin of
their drawing.

Both halves are asserted, because the second is the one that surprises:
an exact `0.0` is not enough — `(-0.15f64).ceil()` is **negative zero**,
which multiplies to negative zero and formats with the sign — so
[`Ladder::steps`] normalises it, and this catches a future refactor that
drops the normalisation.

### `fn a_long_tick_walk_stays_exact`

`index × step` rather than repeated addition. Asserted over a thousand
steps because the error accumulates linearly and a dozen would not show
it: at 1,000 minor ticks a naive walk is already off by enough for
[`Ladder::is_major`]'s tolerance to start missing labels.

### `const THICKNESS_PTS`

**A constant, and R128 is why** — see this module's header, §3. It is never
derived from a label's width, from the zoom, or from anything else that
varies per frame, because the viewport it is subtracted from feeds
[`crate::viewer::ViewState::apply_fit`].

22 points holds the small text style with a point of padding either side,
plus the 6-point major tick that runs up from the inner edge. Chosen by
measuring the drawn result rather than by arithmetic: at 18 the labels
touched the ticks, and a number sitting on a line is a number the eye has
to disentangle from it.

### `const MIN_MAJOR_PITCH_PTS`

The one number that decides the ladder: [`Ladder::for_labels`] walks the
1-2-5 sequence upward until a step is at least this far apart on screen.

76 rather than something tighter because core's formatter is *fixed-place*
— `format_measurement` renders `100.00 pt`, not `100 pt` — so a label is
routinely ten characters, which is ≈46 logical points at the small text
style. 76 leaves a 30-point gap between one label's end and the next
label's start. Labels that run together are worse than a coarser ladder:
the operator can always read an exact value off the pointer indicator, and
can read nothing at all off two overlapping numbers.

### `const MAX_LINES`

[`MIN_MAJOR_PITCH_PTS`] and [`super::grid`]'s own minimum pitch already
bound the count by the viewport, so this is unreachable in ordinary use. It exists because
the ladder divides by a zoom and by a scale, both of which arrive from
outside this module, and a degenerate value there would otherwise be a
frame that never finishes rather than a frame that draws slightly wrong. A
visible mistake beats a hang.

### `struct Gutters`

`Copy` for the same reason [`PageMapping`] is: it is a fact about one
frame's layout, and one that outlived the frame would describe a canvas
that has since been resized.

With rulers off, `top`, `left` and `corner` are all `None` and `content`
**is** `outer` — so every downstream expression is exactly the one it was
before this feature. That is the same "the default path is unchanged, and
it is asserted rather than intended" discipline [`crate::viewer::strip`]
applies to single-page display.

### `fn content_ui`

A child rather than a `scope_builder` closure so that
[`super::show_in`] keeps its early `return`s meaning what they say.
Inside a closure a `return` leaves the closure rather than the
function — the kind of silent semantic change a re-indentation hides
perfectly.

The clip is **intersected** with the parent's rather than replacing it,
so a canvas inside an already-clipped dock compartment stays clipped by
both.

### `fn reserve`

# Why the space is claimed here rather than at the end

[`super::show_in`] has four exits and only one reaches the bottom of the
function. Advancing the parent's cursor at each would be four places to
forget; advancing it once, up front, from a rect that is already known,
cannot be forgotten. Nothing else uses the parent `Ui` for layout
afterwards — only for painting, which does not consult the cursor.

# Why a degenerate canvas turns the rulers off rather than clamping them

A dock compartment dragged down to nothing, or a window mid-resize, can
leave less than two gutters' worth of room. Clamping would produce two
rulers and no canvas, a picture that says the application is broken.
Returning the no-ruler shape is the honest answer to *"there is not enough
room to draw this"*, it recovers by itself on the next frame that has room,
and — because the toggle is untouched — it does not silently countermand
the operator's choice.

### `struct CanvasGeometry`

# Why this is handed back rather than read again

Because it is only knowable *inside* [`super::show_in`], after the scroll
area has laid out — the same reason `last_scroll_offset` is stored and the
same reason `strip_visible` is published during layout. Re-deriving it
outside would be a second answer to "where is the page", which is the
failure `canvas::mapping`'s whole header exists to prevent.

`None` from a frame that drew no page at all (no pages, a whole-canvas
render refusal, a strip whose visible window fell outside every page). The
gutters are still drawn in that state, empty: the chrome the operator asked
for is there and has nothing to measure. Hiding it instead would make the
canvas jump by 22 points on a page that failed to draw.

### `fn anchor`

The fallback is reachable for one frame after a mode change, when the
scroll area has moved the strip but `view.page_index` has not caught up
yet. A ruler measuring the wrong page for one frame is invisible; a
ruler that vanishes for one frame is a flicker.

### `fn page_at`

The truthful answer is `None` in the gaps between rows and in the
centring margin either side of a narrow page, and [`super::guides`]
relies on it: a guide dropped into the grey belongs to no page and is
therefore not created.

### `struct Scale`

See this module's header, §1. `Copy`, and both fields are `pdfcer-core`
types — this struct adds no model of its own, it *selects* one the engine
already owns, which is what keeps the ruler and a dimension in agreement by
construction rather than by care.

### `fn of`

The **default group**, which `pdfcer-core` guarantees always exists:
`DimensionModel::new` seeds exactly one group, `DEFAULT_GROUP_ID`, and
nothing can delete it.

**Not the *active* group, and that is an open behaviour question
rather than a gap.** A picker exists —
`crate::panels::dimension_groups`' *Draw into* column, written through
[`crate::canvas::measure::set_active_group`] — and both readings are
defensible:

- **The default group**, which is what this does: the ruler is page
  furniture, read while panning and reading, and a tool state left over
  from ten minutes ago is an arbitrary thing for it to depend on. A
  scale that changes because a radio moved in another window, with
  nothing on screen saying why, is a bug report.
- **The active group**: the ruler and the ce dimension the operator is
  about to draw would **agree**, which is the ruler's stated purpose.
  And on a sheet carrying a 1:50 plan and a 1:5 detail, one fixed scale
  is wrong for half the sheet whatever it is.

It is a question for the operator, so it is recorded here rather than
decided. Answered *active*, this is the one line that changes: the
function would take an `&egui::Context` and ask
[`crate::canvas::measure::active_group`], and everything downstream is
already scale-agnostic.

A document whose sidecar is missing, unreadable or written by a newer
build answers [`Scale::default`] — raw points. Every one of those means
the same thing to a ruler (*nobody has told me what this drawing is
scaled to*), and a preference is not worth an error path; the same
posture `viewer::remembered::recall` takes.

### `fn units_per_point`

`1.0` when no scale is set, which is what makes the raw-points path the
*same* arithmetic as every other path rather than a special case: the
ladder is chosen in display units and converted back to points by
dividing by this, and dividing by one is the identity.

A non-finite or non-positive factor — reachable from a sidecar carrying
a nonsense calibration — degrades to `1.0` rather than producing NaN
tick positions. A ruler in the wrong unit is a legible mistake; a ruler
whose ticks are all at NaN paints nothing and says nothing.

### `struct Ladder`

Both in **PDF points**, because that is the space every position downstream
is computed in. The 1-2-5 choice is made in *display* units — the numbers
the operator reads have to be round, and 100 mm at 1:50 is not a round
number of points — and converted back here, once.

### `fn nice_step`

The universal ruler and axis ladder. 1-2-5 rather than 1-2-2.5-5 (Excel's)
or anything containing a 3, because those are the multiples an operator can
subdivide mentally: halves, fifths and tenths of the labelled step.

Returns `1.0` for a non-finite or non-positive input rather than
propagating it. Every caller then produces a ruler with the wrong spacing
instead of one with no ticks at all, and a wrong ruler is visible where an
empty one looks exactly like the feature being switched off.

### `fn for_labels`

The derivation, in one line: a tick every `s` display units is
`s / units_per_point` points is `s × zoom / units_per_point` logical
points on screen, so the smallest acceptable display-unit step is
`min_pitch × units_per_point / zoom`, and [`nice_step`] rounds that up
to something an operator can read.

The **minor** ticks then fall where [`minor_divisions`] puts them, and
they may be as close as a point or two on screen — which is right for a
ruler, where a fine comb between the numbers is exactly what you want
to count against, and wrong for a grid. See [`Self::for_lines`].

A degenerate zoom yields a one-point ladder, which the caller's own
line-count bound then refuses to draw — rather than an infinity, which
it would happily try to.

### `fn for_lines`

# Why this is a second constructor

⚠ [`Self::for_labels`] bounds the **labelled** step, so handing it the
grid's minimum pitch bounds the wrong one. On the benchmark A3 sheet at
its fit zoom of 1.3634 that picks a 10-point major and therefore a
**1-point minor** — a grid line every 1.4 screen pixels, about 2,450
lines a frame instead of about 250. That is not a grid, it is a tint,
which is exactly what `grid`'s minimum pitch exists to prevent.

**Neither a screenshot nor the suite can see that.** A 1.4-pixel mesh
over a drawing reads as a plausible fine grid, and a check asserting
the grid is *finer* than the ruler passes emphatically. Only printing
the ladder the running application actually chose separates the two.

# How it climbs

The 1-2-5 rung whose minor step clears the pitch is not `nice_step` of
anything simple, because the divisor changes with the mantissa: 100
divides into tens, 200 into fifties, 500 into hundreds. So it climbs the
sequence one rung at a time and stops at the first that clears — at most
three iterations, because three consecutive rungs span a factor of ten
and the divisor never exceeds ten.

### `fn steps`

The one walk the rulers and both grid axes share, and it multiplies an
**integer index** rather than accumulating `value += minor`. Two things
turn on that:

1. **The label at the page's top edge would read `-0.00 pt`.** Repeated
   addition from a negative start lands on `-1.8e-15` instead of zero,
   which `format_measurement` renders with two decimals *and its sign*.
   A ruler whose origin is labelled "minus zero" is a ruler the operator
   has to stop and think about — and every position check stays green,
   because the tick is in the right place to well under a pixel and only
   the number is wrong.
2. **[`Self::is_major`] would drift.** Accumulated error grows without
   bound; from an exact multiple the comparison is exact for any tick
   count a screen can hold.

The residual `-0.0` — `(-0.15f64).ceil()` is negative zero, and
`-0.0 * 10.0` is still negative zero — is normalised by the `+ 0.0`
below, which is the one arithmetic identity that is *not* a no-op in
IEEE 754: `-0.0 + 0.0 == 0.0`.

Bounded by [`MAX_LINES`] so a degenerate ladder is a frame that draws
slightly wrong rather than a frame that never finishes.

### `fn is_major`

Compared against a tenth of a minor step rather than exactly, because
`major` and `minor` both arrive from a division — by the scale factor
and by [`minor_divisions`] — so `value / self.major` lands a few ulps
either side of an integer even for exact multiples. An exact remainder
test drops those, visible as a ruler that stops labelling halfway
along.

[`Self::steps`] keeps the error at that floor by multiplying an
integer index. A walk that accumulated `value += minor` would outgrow
this tolerance after a few hundred ticks.

### `enum Axis`

`pub(super)` because [`super::grid`] speaks the second reading; see below.

One enum for both, and the two readings are stated where each is used:
[`ticks`] takes the axis of the **quantity being measured** (the top ruler
measures canvas *x*), while [`grid_axis`] takes the axis the lines are
**spaced along** (vertical lines are spaced along *x*). They coincide,
which is why one enum serves; naming it for one of the two would make the
other read backwards.

### `fn point`

Legal in **either** space, because the screen ⟷ canvas map is
separable: `to_page` and `to_screen` compute x from x and y from y, so
the component this axis does not care about cannot affect the one it
does. That is what lets every position in this module be produced by
handing a point to [`PageMapping`] rather than by a hand-rolled
`origin + value * zoom` — which would compile, run, and drift the
instant a page's drawn rect and its nominal zoom disagreed by a
rounding, as they do on the frame a fit is settling. `canvas::mapping`
exists precisely so there is no second place that divides by the zoom.

### `fn draw`

# Where the zero is, and why it is the page's top-left

The origin is the **current page's own top-left corner in canvas space** —
the same origin `canvas::mapping` calls canvas space and the same one the
`canvas-pointer` trace reports as `page=`. Three consequences, all wanted:

1. **The ruler and the pointer readout agree.** They are the same number in
   the same frame; a ruler with its own origin would be a second coordinate
   system with nothing to say which one a value came from.
2. **Y increases downward**, which is what Acrobat, InDesign, Illustrator
   and every layout tool do, and which needs no flip — so the classic
   silent Y-up/Y-down defect `mapping`'s header warns about cannot occur
   here, because there is no conversion to get backwards.
3. **Under a continuous mode the numbers restart at each sheet**, because
   the zero follows `view.page_index`. That is right for the same reason
   the grid is per page: each sheet is its own drawing with its own
   coordinate system, and a ruler that kept counting through a 36-sheet set
   would be measuring the scroll rather than the drawing.

The alternative — PDF user space, Y-up from the un-rotated CropBox corner —
is what an annotation `/Rect` is written in, and is deliberately *not* what
is shown: it disagrees with the pointer trace, it flips under `/Rotate`,
and it is a frame the operator never sees anywhere in this application.
