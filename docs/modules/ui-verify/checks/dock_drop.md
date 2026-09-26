# `ui-verify/checks/dock_drop`

The two affordances a panel drag shows before it commits — **the drop
compass over a compartment, and the outline of the window a tear would
open** — driven through the OS and asserted against the running binary.

# Why these needed a channel before they could have a check

Both affordances are *only* a picture. The compass is a wash of the accent
colour over five quadrants with one of them stronger; the tear is a thin
outline at the pointer. Neither is a thing a trace could carry, and a
screenshot taken after the gesture cannot see either, because both exist
only while the button is down.

What a wrong build gets wrong, though, is not the colour. It is *which
compartment the pointer resolved to*, *which zone of it*, *what a release
would then do*, and *whether the thing the operator was shown is the thing
that happened*. All four are decisions, and the application now publishes
them on the `dock-drop` and `dock-tear` trace slots. These checks read
those.

# Two channels, published by different mechanisms, cross-checked

The decision arrives on the trace slot. The *painting* arrives as a
published region — `dock.<addr>.zone.<zone>` for the armed quadrant and
`dock.tear.outline` for the outline — put there by the dock's own rect
reporter. A build that resolved a compartment correctly and painted
nothing satisfies one and not the other, and so does a build that painted
the compass over the compartment next door. Asserting both is what makes
the pair mean *"the operator was shown the thing that then happened"*
rather than *"a decision was taken somewhere"*.

# Nothing here is hard-coded to a compartment

The arrangement is discovered from the regions the run itself published:
which compartments the dock drew, which tabs are in which, and which of
them has a body to aim at. `panel_tab_reorder`'s lesson, and it is not a
stylistic one — the left dock in this application draws **no tab strip at
all** when its rail is showing, so a check that named a strip would be
asserting about a surface the operator cannot see. Discovery also means
these keep asserting the same property the day the default arrangement
moves a panel, instead of turning red about the move.

# What a passing run does NOT prove

* **That the compass is legible.** These read rectangles and decisions. A
  compass painted in the background colour passes every assertion here;
  that is a pixel question and a pixel oracle is the instrument for it.
* **That every zone resolves correctly.** One drop into one compartment.
  The five-zone grammar has `dock::overlay`'s unit tests; what those
  cannot reach is whether a real pointer, driven through the OS across a
  real window, arrives at the grammar at all.

## Item notes

### `const DWELL`

The offer is resolved on the frame the pointer arrives and the release is
read on a later one; a gesture that let go on arrival would be releasing
against an offer that did not exist yet. `Driver::drag_via` spends this as
one-pixel nudges rather than one sleep, because a stationary pointer
generates no input and a build is not obliged to repaint without it.

### `const ORIGIN_TOLERANCE_PTS`

The measured residual is **0.0 pt on both axes** — `dock-tear at=[1267.0
502.0]` and `viewport-outer rect=[[1267.0 502.0] - ...]` from the same run.
The allowance is not slack for a wrong conversion: it covers only the round
trip through the window manager, which is asked for a position in logical
points and reports one back after snapping to whole physical pixels. At a
`ppp` above 1 that rounding is a fraction of a point, so one point is the
ceiling on a correct build at any scale this application runs at.

Sized deliberately far below the failure it is guarding against. A
conversion that adds the wrong window origin, or none, is wrong by the
application window's own position — hundreds of points. Anything between one
point and that is a defect nobody has met yet and should be read, not
tolerated, so widening this constant is the wrong response to it failing.

### `const SIZE_TOLERANCE_PTS`

Same measurement, same reasoning: the outline is `DEFAULT_SIZE_PTS` and the
window is built `with_inner_size` from the same value, so the two agree
exactly (320x480 measured) and the allowance covers only pixel rounding.

### `fn bar`

Derived rather than looked up, so the two cannot drift: a compartment
and its strip are built from one address by the dock, and are rebuilt
from one address here.

### `fn compartments`

A compartment's region name is `dock.<side>.<column>.<stack>` and nothing
else is: four dot-separated parts, of which the last two are numbers. The
alternative — matching on the side words — would also catch
`dock.right.0.1.tabbar` and every other suffix the dock hangs off the same
stem.

### `fn a_draggable_tab`

The first in region-name order whose rectangle lies inside a compartment
that has a tab bar — which is the condition for the tab being *visible*,
and this application has a dock side that draws none.

### `fn standing_at_release`

# Why this is the last *two* lines and not the last one

The affordance is gone the instant the button is — that is what makes it a
pre-commit affordance rather than a mark on the document — so the release
itself writes the no-offer control, and the terminal line of *every*
completed gesture is `none`. A check that read only the last line would
report a perfectly correct build as having offered nothing, which is the
assertion inverted.

So the standing offer is the line **immediately before** the terminal
control, and it counts only when the control is genuinely terminal.
Requiring adjacency is what distinguishes *the pointer was over a
compartment when it was released* from *the pointer was over one earlier in
the gesture and had left by the end* — two histories whose last offer line
is the same line.

Returns the standing offer, and whether the slot's last line is the
control. The second is reported separately because an offer still standing
after the release is its own defect: the affordance has outlived the
gesture and is now marking the page.

### `fn gesture_start`

# Why a pre-commit affordance cannot be read with `declared`

[`declared`] answers *is this region on screen now*, and honours the
`ui-rect-gone` line that retires one. Every affordance this module measures
is retired by the time the trace is read — the wash, the caret and the
outline all vanish at the release, which is what makes them affordances
rather than marks on the document — so `declared` reports each of them
absent on a build that painted all three correctly, and the failure
sentence it produces names the region in its own *"regions drawn"* list.

[`declared_since`] asks the right question, *was it published during this
gesture*, and requires an anchor so that it cannot degrade into reading a
fossil left by an earlier drag in the same run. This is that anchor: the
last region line the application wrote before the pointer moved.

### `fn control_seen`

Asserted because a change-only slot that simply stops emitting looks
exactly like a slot nothing writes to. The control line is what says the
channel is alive.
