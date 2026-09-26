# `ui-verify/checks/print_position`

**The page can be moved on the paper, and the hatch follows it to all four
edges** — operator request O208, driven against the real binary.

The request, verbatim:

> can we add a control to our print preview screen so that when we are
> printing at a scale that will lose content we have the option to drag the
> drawing to a new position on the print page? That way we can choose what
> gets cropped. we could also have an option to reset position, center,
> center horizontally, or center vertically. This can be remembered for
> each page. Also the hash lines we use to show what won't be printed
> should have a line for each edge of the page.

# Why this check has to exist, and why the unit tests are not enough

`dialogs::print::position` has fifteen unit tests and they cover the
arithmetic completely: centre is not reset, centring is idempotent, a
fitting page dragged off the near edge clips, the engine still starts an
oversized page flush at the corner. Every one of them calls the verb
directly.

None of them can see the chain in FRONT of the verb, which is where this
feature can fail in at least five ways that all leave the tests green:

* the drag is classified as a **pan** — same mouse button, same rectangle,
  and the only thing separating the two gestures is where the press landed;
* the displacement is applied to the placement after somebody has already
  read it, so the preview moves and the print does not, or the reverse;
* the delta is not divided by the preview scale, so the page crawls;
* the sign is flipped on one axis, which looks like a working feature until
  the operator tries to recover content off the bottom of the sheet;
* the position is keyed on the plan position rather than the document page
  index, so it is remembered against the wrong page as soon as the job is a
  custom range or an odd/even subset.

# What it asserts

| # | Gesture | Property |
|---|---|---|
| 1 | open the dialog, choose **Actual size** | the page starts unmoved: `pos=0.00,0.00 moved=0` |
| 2 | drag inside the page, up and to the left | `grab=page` on some frame, `pos=` negative on both axes, and larger in paper points than the pointer moved in screen points |
| 3 | — | `edges=` gains BOTH near edges: `l` and `t` are now set |
| 4 | **Centre** | `pos=` changed and `moved=1` |
| 5 | **Centre horizontally**, then **Centre vertically** | `pos=` does not move — centring twice lands in the same place |
| 6 | **Reset all pages** | `pos=0.00,0.00 moved=0` |
| 7 | **Centre**, then **Reset position** | back to zero again, by the per-page route |

Assertion 3 is the second clause of the request and it is the one a
screenshot cannot settle: three hatched bands and two hatched bands are the
same picture to anything that is not a human looking for the difference.
`edges=` is a four-character word in the band order left, right, top,
bottom — `.r.b` for a page flush at the corner, `lrtb` for one dragged off
all four — so the check reads *which* edges rather than how many.

# What it deliberately does NOT do

**It never presses the commit button**, and no future edit may make it do
so. Same rule and same reason as `print_dialog`, `print_paper` and
`print_clip_claim`: that button is the one control in the application that
consumes paper and cannot be undone, and a harness able to start a print
job will eventually start one by accident.

It also never asserts the per-edge crop sentence. That disclosure is a
label, a label's text is not in the trace, and `position::cropped`'s unit
tests own the numbers in it.

# The fixture

A sheet bigger than the machine's printable area, so that **Actual size**
crops and the four-edge half has something to be about:
`fixtures/a1-titleblock.pdf`. Assertions 1, 2 and 4–7 hold on any fixture —
a page that fits can be dragged off the paper too — and assertion 3 states
which edges the page hangs over after being dragged up and left, which is
the near two whatever the sheet's size.

## Item notes

### `const TAB_POSITION`

A button, deliberately: it does not consume a wheel notch, so the notch
reaches the scrolling body behind it. A `DragValue` — and the Position group
has two — would have eaten the notch and changed the number it was sitting
on, which is a silent edit to the very geometry being measured.

It is also the only anchor that can work. The scale radios, which this
check used to scroll over, are on a different tab: once Position is open
they are not drawn at all, so an anchor there would be absent exactly when
it was needed and the check would read "the group drew nothing".

### `const DRAG_PT`

Bounded above by the preview canvas, which a measured run reported as
**340x438 logical points** on this machine's default window: the gesture has
to start and finish inside the page, the page at Actual size fills the
canvas, and so the canvas's short side is the ceiling. 240 pt was tried
first and skipped every run for want of room.

Bounded below by [`EGUI_MAX_CLICK_DIST`]: the driver walks the pointer in
[`DRAG_STEPS`] equal steps, and the prediction below is only the truth while
one of those steps is longer than that distance. `travel_per_step_clears_egui`
holds the two numbers against each other.

### `const EGUI_MAX_CLICK_DIST`

A press becomes a drag on the first frame whose travel from the press origin
exceeds this, and that frame's `drag_delta()` carries its entire step. So a
driver whose step is longer than this loses nothing to the decision, and one
whose step is shorter loses every step before the crossing. The prediction
below is the whole travel, which assumes the first case — hence the test.

### `const MOVE_BAND`

The prediction is the whole pointer travel, so the floor is for steps the
application misses under load — two of the eight — and the ceiling is for
rounding. Deliberately too narrow to be satisfied by a delta applied twice
(a ratio near 2) or by one never divided by the preview scale (a ratio near
the scale itself, about 0.4). The sharp assertion about magnitude is not this
band — it is [`assert_magnified`].

### `const ROOM_FOR_THE_DRAG`

The drag runs from the page's centre outward, so the requirement is that
half the smaller side exceeds the travel — a factor of two — plus a margin
for the pointer landing a pixel off.

### `fn scrolled_into_view`

The Position group fills a scrolling options column, so on a short window
its lower controls are genuinely off screen and their regions are genuinely
absent — `ui_rect_visible` is what publishes them and it refuses below 60 %
visible. An absent region here is therefore a scroll position, not a missing
control, and a check that read it as the latter would report a defect in a
button that is drawn every time the dialog opens.

### `fn assert_magnified`

This is the assertion a missing `/ scale` fails, and it fails it by a factor
rather than by a percentage — which the wide [`MOVE_BAND`] cannot promise,
because at a typical fit scale of 0.4 a missing division lands inside the
band.
