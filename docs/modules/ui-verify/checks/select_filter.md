# `ui-verify/checks/select_filter`

`select_filter_changes_what_a_click_hits` — the filter is load-bearing, not
decorative.

# Why the obvious check would have been worthless

The tempting assertion is *"clicking Select opens a popup"*. That is already
a unit test, and — more to the point — **it is the claim that is also true of
every inert control.** This project shipped a checkbox wired to a field
nothing read, and on 2026-08-21 it shipped this very popup with a double
toggle that made the button do nothing at all, under 1,628 passing tests, 17
green gates and a smoke launch that confirmed the button's published rect.

So the claim worth driving is the one an operator would make: **switching a
class off changes what the next click on the same pixel selects.** Nothing
weaker distinguishes a filter that works from a filter that is drawn.

# The shape: a round trip, not a one-way assertion

| step | assertion |
|---|---|
| click the object | it selects — **the control point** |
| Select ▸ None, click the same pixel | it selects nothing |
| Select ▸ All, click the same pixel | it selects again |

The first step is what makes the second meaningful: without it, "nothing was
selected" could equally mean the click missed, the fixture has no object
there, or the mode forbids selection — three things that look identical in a
trace. The third step is what makes the second *attributable*: a build that
had simply broken selection outright would fail there, and a check that
stopped after step 2 would have called that a passing filter.

# Why None and All rather than a named class row

The popup publishes an indexed rect per class row, and aiming at one would
be a statement about **the fixture** — *"row 1 is Lines, and the object at
`--doc-point` is a path"*. Both halves can be wrong: the display order is
documented as changeable for display reasons, and what sits under a given
point depends on the file.

`None` and `All` reach a known filter state with no knowledge of either, so
a failure here means *the filter does not gate the hit test*, which is the
thing under test, rather than *the fixture changed*.

# Edit mode

Content selection needs `edit_content`, which only Edit has. Driving this in
Read would assert nothing about the filter — nothing would select in either
state, and the check would pass while measuring the mode gate.

## Item notes

### `const SELECTION_EVENT`

**The first version of this check asked whether the selection's
`ui-rect` region had been published, and it produced a confident, wrong
FAIL on a build that was working.** The `ui-rect` channel is a **change
log**: it emits when a rect moves, so the last rect of a region that has
since stopped being drawn is still sitting in the trace. Asking "did this
region ever appear" answers a question about the whole run, not about now.

`D:/dev/rag/egui/a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`
is the standing finding, and this check walked straight into it — while its
own failure message accused the application of shipping a decorative
control. **Read the trace before believing the check.**

`canvas-selection … sel=N` is the honest oracle: a count, emitted per
gesture, that a wrong build would get wrong. It is the rule `resize.rs`
states — *a trace line must carry the number a wrong build would get
wrong* — applied to the thing being read rather than to the thing being
written.

### `fn selected`

Reads the last `canvas-selection` line's `sel=` count. See
[`SELECTION_EVENT`] for why a count is the honest oracle here and why the
obvious alternative produced a confident wrong failure.

### `fn set_filter`

The popup is opened fresh each time rather than left open between steps.
It closes on a click outside itself, and the canvas clicks in steps 2 and 3
are outside it — so a version that assumed it stayed open would be reading a
popup that had already gone, and would click the canvas at the button's
coordinates instead. That failure would present as *"the filter did
nothing"*, which is the check's own failure message: it would accuse the
application of exactly the defect the harness had just committed.
