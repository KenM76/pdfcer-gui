# `ui-verify/checks/off_sheet`

`a_view_carried_off_the_sheet_comes_back` — O186 stage one, made
falsifiable.

# The report


> *"the canvas will just stop zooming in"*

and, separately, that the page can disappear entirely with no way back.

# What is wrong — the theory this check was written against, and what it measured


## The theory: an unclamped deep anchor

Above the deep threshold — `longest_page_pt × zoom` over
`viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT` — the view's position stops
being an `f32` `egui` scroll offset and becomes a
`viewer::deep::DeepAnchor`: an `f64` page point plus the screen pixel it
sits under. The `egui` scroll area clamps its offset to the content it
scrolls; **nothing clamped the anchor.** So an anchor whose page point is
*off the sheet* can be magnified until the sheet itself is thousands of
viewports away, and the canvas draws nothing. That was real; it is fixed by
`canvas::deep::confine`; and this check's passing run traced the clamp
firing on `y` nine times on the way up to 107,105,325 %.

## The measurement: his blank frame is two orders of magnitude below that

Driven against the pre-fix binary. The canvas went blank after **thirty**
notches at a peak zoom of **3,981 %**, with `tier=scroll` on every frame,
the deep tier never reached, and `confine` never called. This A1 sheet's
`f64` hand-over is at a zoom of about **440** — that is **44,000 %**.

So the clamp on the anchor cannot be what the operator hit, and **a fix
confined to it would have left his own reproduction untouched while every
instrument in this crate went green.**

The cause is one term in `canvas::geometry::pasteboard`, documented in full
at `MIN_SHEET_ON_SCREEN` rather than restated here. In short: the pasteboard
was **exactly one viewport**, which makes the two extremes of
`geometry::visible_origin_range` the placements at which viewport and strip
touch with *zero* overlap — at both ends, on both axes, **at every zoom**.
Narrowing the range by a sliver fixes it, and because every other bound in
that module is derived from `pasteboard`, one subtraction covers the shallow
tier the operator hit, the deep tier's new clamp, **and** a scroll bar
dragged to its stop.

The scroll-bar route deserves a sentence of its own: it was blank too,
reachable at any zoom with no deep tier involved and no pointer off the
sheet, and nobody had ever reported it — because a scroll bar sitting at its
own stop does not feel like a defect.

That state was **terminal**, and the reason it was terminal is worth
stating because it is not obvious from the symptom:

* `canvas::present` returns early when no page was drawn, *above* every
  input handler in the function. Only the movers inside
  `canvas::deep::track` run, because `viewpos::position` is called higher
  up.
* Those movers are a pan and a plain wheel. `smooth_scroll_delta` is zero
  while Ctrl is held, so the deep pan route cannot zoom out at all.
* `DeepAnchor::panned(delta, zoom)` moves the page point by `delta / zoom`.
  At zoom 540 one plain wheel notch is about 0.09 pt. Escaping by panning
  would take roughly seven and a half thousand rolls.

**That terminality belongs to the deep tier, and the operator's blank
frame was not in the deep tier** — so on his own reproduction the escape
hatch was never the thing standing between him and a page. Measured: from
the shallow-tier blank at 3,981 %, eighty Ctrl+wheel notches out drew a page
again at 10 %. The hatch works. It simply was not the broken half, and the
argument above describes a state he had to climb two orders of magnitude
further to reach.

# What this check drives, and why it is shaped like this

Aiming Ctrl+wheel at a point **above the sheet's top edge** is the whole
reproduction. `DeepAnchor::zoomed_about(at, from_zoom)` re-states the anchor
as *"the page point that was under `at` at the old zoom, pinned to `at`"* —
so every notch re-seeds the anchor from whatever is under the pointer, and
if that is blank pasteboard above the sheet the anchor's page y is
**negative** and grows in screen magnitude with the zoom.

`DocPoint` y is PDF y-up; `DeepAnchor.page` y is y-down from the page's
top-left. A point *above* the sheet is therefore a **high** `DocPoint` y and
a **negative** anchor page y. That sign flip is what made the measured
anchor `(1199.50, −0.54)` read as "almost on the page" when it was in fact
half a point off the top of it, with a zoom of several hundred multiplying
that half point into a view that had left the sheet.

⚠ The magnitude of the initial aim is deliberately **not** load-bearing —
and the reason this paragraph originally gave for that is the sentence
that had the defect written into it. Below the deep threshold the `egui`
scroll area clamps the offset to its own content range, so however far above
the sheet the first notch aims, by the time the threshold is crossed the
page's top edge is **at most one pasteboard below the viewport** — which is
why the measured anchor was a fraction of a point off the page rather than
the eighty-odd points this check starts by aiming at.

That clause was written as a reassurance. It **is** the defect. One
pasteboard below the viewport was one whole *viewport* below it, which is
precisely the placement at which the sheet occupies zero of the canvas — so
the harmless-sounding bound the aim converges to was itself the blank frame,
and it is reached long before there is an anchor to clamp. Every ingredient
of the diagnosis was in that sentence and it was read as an all-clear.

The conclusion survives: the aim only has to be *above the sheet at all*,
and to be reachable on screen; see [`OFF_PAGE_FRACTION`] and
[`ZOOM_OUT_NOTCHES`]. But it now holds because `MIN_SHEET_ON_SCREEN` keeps
that convergence point **visible**, not because the convergence point was
ever safe.

# The three verdicts, and the order they are asked in

The order is load-bearing and is the opposite of the order the sentences
suggest:

1. **Did the canvas go blank?** `canvas-unavailable reason=nothing-visible`
   after the climb began is the defect, and it FAILS. This is asked first so
   that a build with the clamp removed goes **red**, not SKIPPED — a
   falsification run must be able to fail.
2. **Was a raster ceiling learned while it was blank?** The non-obvious
   half. The zoom ceiling is learned from what the renderer refused, and a
   blank canvas refuses everything, so a ceiling learned in this state is
   learned from the defect and then **outlives the fix**.
3. **Did the zoom keep climbing?** The operator's own sentence. A run whose
   peak zoom never passed [`ZOOM_FLOOR`] reproduces *"the canvas will just
   stop zooming in"* whether or not anything went blank.

and only then the two vacuity guards — *did the run reach the deep tier*,
and *did the clamp ever actually fire* — which SKIP rather than pass,
because a run that never drove the anchor out of range has measured nothing
about a mechanism whose only job is to catch an anchor out of range.

**The first run of this check answered the open design question it was
written to settle, and the answer was worse than the question allowed for.**

The question: clamping the anchor to the ends of
`geometry::visible_origin_range` parks the page at a placement where
viewport and sheet touch with **zero overlap** — a near-blank view. The
draft's defence was that the shallow tier permits exactly the same placement
(`egui` clamps scroll to `content − viewport`, and with a full-viewport
pasteboard the maximum offset shows a viewport of grey beyond the strip), so
it is pre-existing behaviour rather than an O186 regression, and tier
agreement is the design's whole correctness claim.

**Every clause of that defence is true and the conclusion is wrong.** The
placement is a near-blank view; the shallow tier does permit it; the tiers
do agree — and what they agree on is a blank frame. Verdict 1 fired against
the build of the day at the **shallow** tier, thirty notches in, before the
clamp existed to be exercised. That is what sent the fix to
`geometry::pasteboard` instead of to either tier.

The lesson to carry, because it generalises past this defect: *"the other
tier does the same thing"* is an argument about **blame**, not about
correctness, and a clamp whose range endpoints are themselves the failure
state will park the view on that failure and report itself as having
confined it.

# What this check does NOT cover, stated rather than implied

`canvas::escape::offer` restores two gestures above the blank-frame early
return: `paging::flip` and `zoom::wheel_step`. **Only the second of those
has any coverage here, and the first has none anywhere.**

`DESIGNS.md`'s own obligation for this stage asked for a check that
*"confirms a page-flip gesture does nothing"* on the old behaviour. That
obligation is **wrong**, and it is worth recording why rather than quietly
satisfying it: `paging::flips_pages` requires
`prefs.wheel_paging.flips()`, and `WheelPaging::Scroll` carries
`#[default]`. The flip route is off on any build nobody has configured, so
"confirm the page-flip gesture does nothing" is satisfied by a correctly
implemented preference and would have been recorded as evidence about the
clamp.

The route that distinguishes the builds is **Ctrl+wheel out**, through
`escape::offer` → `zoom::wheel_step`, which is what [`RECOVER_NOTCHES`]
drives. The paging route needs a non-default preference *and* a blank frame,
and on a fixed build the clamp prevents the blank frame — so it is only
reachable on a deliberately broken binary, and it is left uncovered and said
so here instead of being asserted vacuously.
