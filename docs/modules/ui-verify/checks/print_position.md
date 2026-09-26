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
