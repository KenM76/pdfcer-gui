# `ui-verify/checks/tool_row`

`the_text_tool_types_on_one_click` and `the_points_tool_shows_points_on_one_click`
— **the operator's own two gestures**, driven.

# What these are for


> *"How do I select and edit end points on the canvas? How do I edit text
> when on the canvas? I get a box and the I cursor, but I can't type
> anything. How do I make new text when I click on the canvas and expect to
> edit there? Same problem as the previous. How do I get to see the end
> points of an object and select them to drag and move? This doesn't work
> either."*

And then the diagnosis, which was correct and is the reason both of these
checks exist:

> *"The selector should be predictable like other programs. It seems a lot of
> ideas are getting invented instead of just using the … most common method
> expected."*

Both features **existed**. Reaching them was invented:


Neither ritual is discoverable and neither resembles any other program. The
fix was to make the **tool the rung**: press `T`, click, type; press `A`,
click, see the points. That is Illustrator, Inkscape, Figma, CorelDRAW and
Word, and it is what these two checks assert.

# Why the assertion is "ONE click"

Because the count is the feature. A check that armed the tool from the
ribbon, clicked, and asserted a caret would pass on the **old** build too —
the old build could do all of that, it just needed four steps to get there.
So each check performs exactly one press of one key and exactly one click,
and asserts the outcome. Anything that needs a second click fails.

And the key is pressed as a **bare letter through the OS**, not as a
command dispatched by name. `V`/`A`/`T`/`H` being bare is the whole
convention being adopted, and a bare letter is the one chord shape that can
be broken by a stray focus — `canvas::keys` gates every keystroke on
`text_edit_focused()`, which is `DEFECTS.md` D1's guard, and D1 is this
project's canonical example of a keyboard rule that was right in the test
harness and wrong in the running window.

## Item notes

### `const ANCHORS_EVENT`

**Written even when `total=0`, since 2026-08-29** — and the two checks
below are the reason. `overlay::draw_anchors` used to return before this line
when there was nothing to draw, so *"this object has no points"* and *"the
draw never ran"* were the same trace: nothing. Both checks have a `total == 0`
SKIP arm written for the first case, and neither could reach it.

### `const DECLINED_EVENT`

The suffix is load-bearing twice over. It keeps `last(ANCHORS_EVENT)` from
ever returning one of these — which reads `total=`, and these carry no
`total` — and it is the convention `tools/gates/check-trace-names.py`
enforces against `vector_edit`'s funnel labels, for exactly that failure.

### `const AIM_REASONS`

This list is the whole difference between the sweep of 2026-08-29 and an
honest one. Four checks read anchors on that run — these two plus
`multi_node` and `bezier_handle` — all four aimed at the same
`--doc-point 0,1140,62` on `SW41177.pdf`, all four saw no `canvas-anchors`
line, and they split two-and-two on what that meant: two SKIPPED saying *"the
point named a text run or an image"* and two FAILED naming specific lines of
`painting::draw_anchors`. The SKIPs were right — at that point
`the_text_tool_types_on_one_click` passes with `text-edit-caret run=426`, so
the aim **is** a text run, and a text run has no anchors — and the two
failures were reports about the aim wearing the clothes of reports about the
code.

⇒ With the reason in the trace, no check has to guess. `not-entered` stays a
failure, because it means the click did not reach the rung and that is the
program's job; everything here means the driver pointed somewhere the feature
has nothing to say about.

### `const ORIGIN_TOLERANCE_PT`

Generous on purpose, and the generosity has a source: the driver clicks a
**whole screen pixel**, and at the fit zoom this shell opens at, one screen
pixel is a little under two PDF points on a D-size sheet. Rounding the
window point to an integer, converting back through the renderer's inverse
transform, and comparing in page space therefore cannot be exact.

⚠ It is a bound on *rounding*, not a bound on *correctness*. Six points is
far below any wrong answer this check is built to catch — a page corner, the
previous caret's run, the page centre — every one of which is hundreds of
points away. If a future build lands inside six points and is still wrong,
widening this is the wrong repair; read the anchor out of the trace instead.

### `const BLANK_AIM_PT`

⚠ **This is a guess about the fixture and it is labelled as one** — which is
exactly what the code this replaces failed to do. On a CAD sheet the very
corner of the media box is outside the drawn border, so it is usually blank;
but "usually" is not an assertion, which is why a run landing here SKIPs
with the run number rather than failing, and why the check reports the point
it used in every outcome.

### `fn parse_origin`

Returns `None` rather than a default on anything unparseable, because the
caller must be able to tell *"the origin is in the wrong place"* from *"the
trace format moved and this check is now reading noise"*. A default would
merge them, and the second dressed as the first is a false defect report.
