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

### `struct ShowPointsDrawsAnObjectsPointsWithoutDescending`

# Why this check exists, and it is not "one more toggle"

`view.show_points` was registered, drawn on View ▸ Display and **inert for
the whole life of the project**, behind a reason that said *"there is
nothing for it to show — this build draws no anchor mark at any rung"*. That
was true on 2026-08-15 and false four days later. Re-derived on 2026-08-28
as one of six stale blockers in eleven.

**The first wiring of it was ALSO inert, and no test could have caught
that.** The toggle was added as a disjunct to `draw_anchors`' rung guard —
which is correct — and the function then fell out two lines later on
`entered_object()`, which answers `None` at the Object rung *by
construction*, because "entered" means the operator descended. So the
control switched on, the trace said `view-chrome ShowPoints on=true`, and
not one anchor was drawn.

Every assertion the toggle had passed: it registered, it rendered pressed,
it reached `ViewState`. **What none of them asked was whether anything
changed on screen** — which is R1's whole subject, and the reason this file
gets a third member rather than the toggle getting a unit test.

# The oracle, and why it is a COMPARISON

`canvas-anchors total=… unselected_drawn=…` must be **absent** before the
toggle and **present** after it, on the same click at the same point. A
check asserting only the second half would pass on a build where anchors
draw at the Object rung unconditionally — which is a different program, and
a noisier one.

### `struct AClickOnBlankPaperStartsNewText`

> *"How do I make new text when I click on the canvas and expect to edit
> there? Same problem as the previous."* — 2026-08-19

One text tool, two outcomes: click **in** text and the caret lands in that
run; click on **blank paper** and a fresh run starts where the pointer is.
[`TheTextToolTypesOnOneClick`] asserts the first. This asserts the second,
which is a different code path — `place::resolve_run` returns
`Refusal::NoRun` and `place::click`'s fall-through arm converts it into an
`Anchor::Origin` at the click point.

# Why this is a separate check, and it is a story about instruments

It used to be six lines at the end of `drive_text`, and those six lines let
`O142` sit undetected for **two days**. They read
`trace.last("text-edit-became-add")` and, when it was absent, printed *"the
point named an existing run, which is a fact about this fixture rather than
about the feature"*. Nothing measured that. **The absent branch could not
fail and the excuse was invented**, so a completely dead feature and a
badly-aimed click produced the same green line.

And the feature *was* dead. `EditableTextModel::hit_test` had no distance
bound: asked about a point with nothing near it, it returned the nearest
line of text at **any** distance — measured at the time as a click 100,000
points to the right of a 612-point page still landing in a run, and *"nothing
here"* answered zero times across two documents and thirty probes. The
fall-through arm was therefore unreachable, and this check's ancestor
reported success-or-shrug the whole time.


# The three outcomes, all evidenced, none guessed

Every one is read from lines emitted **after** an anchor taken immediately
before the blank click, via [`crate::trace::Trace::last_after`] — never
`last()` over the whole capture, which would return the *first* click's
caret and report it as this click's answer. That fossil is the exact failure
`last_after`'s own doc comment was written to prevent.

| what the trace says after the blank click | verdict |
|---|---|
| `text-edit-caret … origin=X,Y` | **PASS** — a fresh run started |
| `text-edit-caret … run=N` | **SKIP** — the aim landed in real text |
| nothing at all | **FAIL** — the click reached no caret code at all |

The SKIP arm is the old excuse **with its evidence attached**: it can
only be reached by a trace line that names the run it hit, so it is a
measurement of the aim rather than a story about it. That is the whole
difference, and it is why the arm is allowed to exist at all.

And `origin=` is checked against **where the pointer actually went**,
not merely observed to exist. `place.rs`'s own trace comment sets this
standard — *"a trace line must carry the number a wrong build would get
wrong"* — and an origin anchor that ignored the click and used, say, the
page corner or the previous caret would satisfy a bare presence test while
putting the operator's text somewhere he did not point. The tolerance is
[`ORIGIN_TOLERANCE_PT`].

# All three arms were FALSIFIED before this check was believed


| planted | outcome | proves |
|---|---|---|
| nothing — the real thing | **PASS**, `origin=18.5,18.8` for a click asked at (20, 20) | the feature, and that the tolerance is doing rounding and not hiding a mistake |
| `BLANK_AIM_PT` moved onto known text at (1140, 62) | **SKIP**, quoting `run=426` | the excuse arm now carries the evidence the old one invented |
| `VK_A` pressed instead of `VK_T`, so no caret code runs at all | **FAIL** | the check can go red, which is the whole point of splitting it out |

⚠ One arm is *not* independently reachable and that is deliberate: a
`BLANK_AIM_PT` outside the page box is refused by `CanvasMapping` before any
click is sent (*"document point (-260, -260) is outside the 1584x1224 pt page
box"*), so the harness's own geometry guard fires first. That is correct —
it means the FAIL arm can only be reached by the application failing — but
it does mean the off-page road to it is closed, and the `VK_A` plant above is
what proves the arm at all.
