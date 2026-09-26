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
