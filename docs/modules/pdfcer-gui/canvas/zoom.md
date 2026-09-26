# `canvas::zoom` — the anchor rule, decided once, and the five paths that route through it

## The rule

> **A zoom holds one page point still, and that point is where the operator
> is looking: the pointer when it is over the canvas, the centre of the
> viewport when it is not. A zoom that *frames* something — a selection, a
> marquee'd region — holds that thing's centre still, at the centre of the
> viewport.**

It lives in [`anchor_point`] and in nothing else, and every zoom in the
product goes through it:

| path | anchor | how it gets here |
|---|---|---|
| Ctrl+wheel | the pointer | [`arm_anchor`], from `canvas::show` |
| Ctrl+Plus / Ctrl+Minus | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
| Ctrl+0 (actual size) | pointer, else viewport centre | [`arm_for_actions`], or [`zoom_step`] |
| Zoom to selection | the selection's centre → viewport centre | [`zoom_to_selection`] |
| Marquee zoom to region | the region's centre → viewport centre | [`zoom_to_rect`] |

`FEATURES.md` records the discrete commands as deferred *"so the rule is
decided once for all four"*. This module is that decision, and it is
deliberately a *rule about the operator's attention* rather than a rule
about arithmetic:

* **The pointer wins when it is over the canvas.** It is the only evidence
  the application has about what the operator is looking at, it is what the
  wheel path already honours (measured under 0.01 px of drift), and a
  keyboard zoom that ignored a pointer resting on the detail being
  inspected would behave differently from the wheel for no reason the
  operator could see.
* **The viewport centre when it is not.** A Ctrl+Plus pressed with the
  pointer parked over the Objects panel, off-window, or in the ribbon must
  not zoom about a point outside the page. The centre of what is on screen
  is the only other honest candidate; the page's **top-left** — today's
  behaviour, and the defect — is not a candidate at all, because it is
  where nobody is looking.
* **Framing is the same solve with a different target.** "Zoom to this
  rect" is not a different feature from "zoom about this point"; it is
  *put this point at the centre* instead of *put this point back where it
  was*. [`crate::canvas::geometry::offset_holding_anchor_at`] is the half
  they share.

## Why the anchor is a CANVAS-space point and not a screen position

Because a screen position stops naming the thing it named the instant the
zoom lands — which is the same reason [`crate::canvas::selection`] holds
identity rather than coordinates. `frac`, the fraction of the page's drawn
size that [`crate::app::state::ZoomAnchor`] carries, is
`canvas_point / extent` and therefore **independent of the zoom**: it can
be computed before the new zoom is known, which is exactly the situation
every one of these commands is in.

## The two-frame handshake, and the gate that makes it work for commands

`ZoomAnchor`'s own docs explain why it spans two frames: *"the new zoom is
not known when the wheel is seen … recording the inputs and solving later
avoids predicting a clamp we do not control."* The wheel is seen **during**
`canvas::show`, so the anchor it records is always consumed on the frame
after, by which time the action has been applied and the page's real drawn
size is known.

A **command** is not seen during `show`. Keyboard chords are dispatched at
step 1a of the frame, the ribbon at 1b, the status bar at 1b² — all *before*
the canvas draws — while the zoom action they raise is applied at step 3,
*after*. An anchor armed at step 1a and consumed unconditionally at the top
of `show` would therefore be spent on a frame that still shows the old zoom:
`display_after == display_before`, the solve is a no-op, and the anchor is
gone before the zoom it was for ever lands. That is precisely how a
plausible implementation of this feature ships doing nothing.

So consumption is gated by [`anchor_step`]: **an anchor is solved only on a
frame whose drawn page size differs from the size recorded when it was
armed** — i.e. only once the zoom has actually landed. One frame of grace is
allowed for the action to be applied, and an anchor still unspent after that
is *dropped* rather than held, because a zoom that never changed the page
size (Ctrl+Plus already at the raster ceiling) has nothing to re-anchor, and
an anchor left pending would be spent much later on an unrelated layout
change — a page step, a window resize under a fit mode — as a visible jump.

## Where the per-frame geometry comes from

An entry point called from the command dispatcher has no `Ui`, no page
rect, no viewport and no scroll offset. [`CanvasFrame`] is the canvas's own
record of those, written at the end of every `show` and read here. It is
*last drawn frame's* geometry, which is the correct "before" state in both
call orders — before `show`, the layout has not changed since; after `show`
but before the actions are applied, it **is** this frame's geometry.

## Item notes

### `fn trace_outcome`

Not de-duplicated: two identical zoom commands are two events, and a gate
that silenced the second would make a harness unable to tell a command that
ran twice from one that ran once.

### `fn framing_puts_the_regions_centre_in_the_middle_of_the_viewport`

Asserted as the outcome — where the anchored point ends up on screen —
rather than as an offset, so it checks the framing rather than the code
agreeing with itself.

### `fn a_region_at_the_page_corner_can_be_centred_because_the_pasteboard_is_there`

It used to assert the opposite: that framing a region hard against the
page's top-left saturated at offset zero, *"there is no page to the
left of or above the origin to scroll to"*, and the operator simply saw
it off-centre. That was true when the scroll content was the page and
nothing else.

**O23 made it false on purpose.** The operator asked for exactly this:

> *"I should also be able to move the view of the corner of the page to
> the center of the screen, or even all the way vertically to the
> opposite corner if I want to."*

The pasteboard is a viewport of slack on every side, so there IS
somewhere above and to the left to scroll to, and the solve's negative
page-local offset is a legitimate position rather than an over-range
one to be truncated.

It stayed asserting saturation for a while after the pasteboard
landed, because `geometry::zoom_anchor_offset` was still clamping to
the page's own range — which is `OPERATOR_REQUESTS.md` O24e, the zoom
that threw away whatever the operator had panned to. A test that pins
last year's constraint is how a stale clamp survives a feature
designed to remove it.

### `fn the_discrete_zooms_are_recognised_and_the_wheel_is_not`

The predicate [`arm_for_actions`] funnels on. A zoom action missing
from it is a command that silently keeps the old top-left anchoring —
which is the defect Phase 3.1 exists to close, reappearing one variant
at a time.

### `fn an_empty_selection_offers_no_bounds_to_frame`

The wiring above turns this `None` into [`ZoomOutcome::NoBounds`] and
raises no action; what is pinned here is that the `None` is real, i.e.
that the decline is reachable rather than a branch nothing can enter.

### `fn a_plan_past_the_ceiling_carries_the_clamped_scale_as_well_as_the_asked_one`

The second is what reaches `Action::ZoomTo`, so the status bar's
readout states the truth on the same frame — see
[`ZoomOutcome::ceiling_changed_the_answer`] on why that *is* the
ceiling's report rather than a substitute for one.

### `const MIN_REGION_EXTENT`

A marquee can be dragged three pixels, and a selected horizontal rule has a
real bounding box that is *exactly zero* high (see
[`crate::canvas::overlay::visible_outline_rect`], which grows outlines for
the same reason). Fitting either literally asks for an unbounded scale,
which the ceiling then clamps to something that shows the operator a
featureless field of ink. Growing the region to a minimum first makes the
answer *"as close as this page can go, framed on what you pointed at"*
rather than *"as close as this page can go, framed on nothing"*.

Applied symmetrically about the region's own centre, so the thing the
operator aimed at stays in the middle of what they get.

### `struct CanvasFrame`

`Copy` and small, for the same reason [`PageMapping`] is: it is a fact
about one frame, and anything that outlived a frame would be a mapping for
a page rect that has since moved.

### `fn last_frame`

`None` is a real state and every entry point declines on it rather than
guessing: before the first frame there is no viewport, no page rect and no
offset, and a zoom described against invented geometry would move the view
to somewhere the operator did not ask for.

### `fn anchor_point`

`pointer` is the pointer's latest screen position, if it has one. It is
honoured when it lies inside the canvas viewport; otherwise the viewport's
own centre is used. See the module docs for why those two, and why the
page's top-left is not a third option.

Note what is *not* consulted: the zoom. The result is a canvas coordinate,
which is the same number before and after the step — that is what allows an
anchor to be described before the new zoom is known.

### `fn frac_of`

`canvas_point / extent`, and therefore **zoom-independent**: `display` is
`extent × zoom` and a canvas point projects to `canvas_point × zoom` inside
it, so the ratio cancels. Dividing by the *drawn size* instead would give
the same number by a longer route and would need a zoom to do it, which is
the argument for computing it here.

A degenerate extent yields `0.5` on that axis — the middle of the page —
rather than a NaN that would reach a scroll offset. `viewer::clamp_zoom`'s
discipline: fail to a finite, harmless value.

### `fn place_centred`

# How one struct expresses both

[`crate::canvas::geometry::zoom_anchor_offset`] reads its "before" fields
only through [`crate::canvas::geometry::anchor_screen_pos`], i.e. only as
the single quantity *"where was the anchor on screen"*. So placing the
anchor somewhere else is a matter of stating that quantity directly, and
`offset_before` is solved backwards from it with
[`crate::canvas::geometry::offset_holding_anchor_at`] — the exact inverse,
pinned by `placing_an_anchor_and_measuring_it_are_exact_inverses`.

The field is therefore truthful in the only sense the solver uses it: it is
*the offset at which the anchor would have been sitting in the middle of the
view*. It is not this frame's scroll offset, and it is not claimed to be.
The alternative — a second `Option` field on `OpenDoc` and a second consume
path in `show` — would be two mechanisms for one two-frame handshake, and
the second one would be the one that gets the clamp gate wrong.

### `fn anchor_step`

`waited` is whether this same anchor already saw one frame in which nothing
had changed. One frame of grace and no more: an action raised at step 1a is
applied at step 3 of the same frame, so a zoom that is going to land has
landed by the *next* frame's `show`, and anything still pending after that
is a zoom that did not happen.

### `fn consume_anchor`

The whole of `canvas::show`'s zoom-anchor wiring, so that the gate, the
solve and the one-frame grace counter cannot be re-derived differently at
the call site.

### `fn action`

`ActualSize` is [`Action::ZoomTo`] and **not** `Fit(FitMode::None)`:
that distinction was a live defect (see `Action::ZoomTo`'s docs — a
control whose label promised 100 % and whose behaviour pinned 73 %),
and it is restated here rather than re-derived because this is now a
second place that has to know it.

### `fn arm_anchor`

The primitive. Call it immediately before pushing a zoom action from
anywhere that is not the canvas — a keyboard chord, a ribbon button, the
status bar's ± — and the next frame's `show` will keep the anchored point
still. Calling it and then raising *no* zoom costs one frame of grace and
a `Drop`; it cannot move the view on its own.

Does nothing before the canvas has drawn (no geometry to describe an anchor
against), which is also the state in which no zoom command can be reached.

### `fn is_discrete_zoom`

[`Action::ZoomBy`] is deliberately absent: it is the *wheel*, which arms
its own anchor inside `canvas::show` from the pointer it can see, and
which arrives as a stream of steps rather than as a command.

### `fn arm_for_actions`

Called once per frame at the action funnel — immediately before the
actions are applied — it covers **every** surface that can raise a zoom in
one call: the keyboard chords collected at step 1a, the manifest chords,
the ribbon at 1b, the status bar's ± at 1b², and a canvas context menu.
The alternative is arming at each of those five sites, which is the shape
the defect already took: `Action::ZoomIn` is raised from three places today
and not one of them anchors.

Two guards, both of which matter:

* **an anchor already pending is left alone.** The framing verbs
  ([`zoom_to_rect`], [`zoom_to_selection`]) raise `Action::ZoomTo` *and*
  arm a centring anchor of their own; overwriting it here with a
  hold-the-pointer anchor would turn every marquee zoom back into a zoom
  about the cursor, which is the one thing a marquee zoom must not be.
  The same guard keeps a wheel anchor armed at the end of the previous
  frame intact;
* **nothing is armed when no zoom is present**, so this is free on the
  overwhelming majority of frames — one `matches!` per action, over a list
  that is almost always empty.

### `fn zoom_step`

Arms the anchor and raises the action, in that order, so a caller cannot do
one and forget the other — which is the shape the defect took: the actions
were raised from three surfaces and none of them anchored.

### `fn wheel_step`

Does nothing when this frame carries no `zoom_delta`, which is every frame
but the ones the operator is actually turning the wheel on. That early
return is why the caller can be a bare `if hovered` with no second test.

# Why this is a function rather than a block in `present`

It has **two** callers, and it had to before either of them could be
trusted: the ordinary one in `canvas::present`, and the escape hatch in
`canvas::escape` that runs on a frame where nothing was drawn.
`OPERATOR_REQUESTS.md` **O186** is the reason the second exists — a view
carried off the sheet publishes `canvas-unavailable reason=nothing-visible`
and `present` returns **above** its own input handling, so the one gesture
that would have got the operator out was unreachable.

Inlining it at the second site would be the **fourth** spelling of the
zoom rule in this crate's history, and the first three drifted: the wheel
built its own [`ZoomAnchor`] from the pointer position while the discrete
commands went through [`arm_anchor`], and *"the rule is decided once for all
four"* is what fixed it. A rescue path that zoomed *without* arming the
anchor would zoom about the viewport's top-left, which on a blank canvas
means the operator claws his way out and arrives somewhere else again.

Takes the [`Context`] and not a `Ui`, so the escape hatch can call it on a
frame where no `Ui` for the canvas *content* exists — which is the very
condition the hatch is for. Nothing in here needs a `Ui`: the wheel delta and
the pointer position are both context-wide input, not widget state.

### `enum ZoomOutcome`

`#[must_use]` because the *declining* variants are the whole point: a
caller that drops this on the floor has silently turned "there is nothing
to zoom to" into "the command did nothing", which is the difference between
a control that declines and a control that looks broken.

### `fn ceiling_changed_the_answer`

# How the ceiling reports itself, and why this follows rather than
invents

The ceiling already has a self-report and it is deliberately quiet:
`viewer`'s header states that *"rather than let the operator zoom into
an error message, `max_zoom_for_page` lowers the ceiling per page and
`ViewState` clamps against it — the zoom buttons simply stop"*, and the
status bar's readout then shows the scale that was actually pinned.
The report is **the number on the status bar being the truth**.

A framing zoom must not break that contract by claiming a fit it did
not get, so it does two things and neither is a new mechanism:

1. it raises [`Action::ZoomTo`] carrying the **clamped** scale, so the
   status readout states the real answer on the same frame;
2. it still frames the region *centred*, at whatever scale it got —
   because the offset is solved on the following frame from the page's
   real drawn size, the framing is correct even when the scale was not
   granted. The operator gets "as close as this page can go, centred on
   what you asked for", which is the honest partial answer.

## The surface now exists, and this is deliberately NOT wired to it

This sentence used to read *"this predicate is what a caller with a
notice surface would key on to say so in words. There is no such
surface in this shell yet."* Both halves are now out of date: the
status bar words declines (`crate::app::status::decline`, 2026-08-14),
and the operator's ruling was that **the clamped region zoom must not
be worded through it.**

The reason is the two numbered points above, taken seriously. A clamped
framing zoom is **a partial grant, not a decline**:

* the region really is framed, centred, at the closest scale this page
  can go to — the operator got the honest partial answer;
* the scale that was pinned is already stated, in words, in the one
  place an operator looks for a scale: the status bar's zoom readout,
  on the same frame, because point 1 raises `Action::ZoomTo` carrying
  the clamped number. **The report is the number on the status bar
  being the truth**, and that contract is kept.

Adding a sentence beside it would word a non-event, and would train the
operator to read a decline line that fires when nothing was declined —
which is how a surface stops being read at all. Only
[`ZoomOutcome::NoBounds`] and [`ZoomOutcome::NoCanvas`] are worded; see
`crate::app::status::decline`'s header, whose `Declined` type cannot
even represent a grant.

So this predicate keeps exactly the job it has: it feeds the
`PDFCER_DIAG` line, and it is returned to the dispatcher, which reads it
and correctly says nothing.

### `fn framed_region`

Also normalises: a marquee is dragged in any of four directions, and a
rect whose `min` is not the smaller corner has a negative width that would
make [`viewer::fit_scale`] return the degenerate fallback and the zoom a
no-op.

### `struct FramingPlan`

The split is this project's standing one (`PROJECT_PLAN.md`: the
unit-testable arithmetic on one side, the wiring on the other). Everything
that can be wrong about a framing zoom — the scale, the clamp, the anchor,
a degenerate region — is decided here and asserted headlessly; the two
public verbs below add only "read the selection" and "raise the action".

### `fn plan_framing`

The scale is [`viewer::fit_scale`] under [`FitMode::Page`], which is the
*same* derivation "Fit page" uses, against the region instead of the page.
One derivation, so a region zoom and a page fit cannot disagree about what
"fits" means. `margin` is subtracted from the viewport first for the same
reason `canvas::show` subtracts it before fitting a page: fitting exactly
and then being clipped by the gap is not fitting.

### `fn zoom_to_rect`

`region` is in canvas space — the space the marquee already reports and the
space the selection's outlines are cached in — so neither caller performs a
coordinate conversion of its own.

### `fn zoom_to_selection`

# Where the bounds come from, and what happens when there are none

The selection is *identity* — page, object, subpath, node — and carries no
rectangle. Its bounds are therefore resolved the way every other consumer
resolves them: through
[`crate::canvas::selection::SelectionState::outline_union`], the union of
the outlines the selection layer has already resolved against the current
decomposition, in canvas space. That is the same value the eight resize
grips are laid out on ([`crate::canvas::overlay::grip_box`]), so **what
this command frames is exactly the box the operator can see**, which is the
only definition that cannot surprise them.

`outline_union` returns `None` in three situations that are one situation
from the operator's side — nothing is selected, the selection is on another
page, or it no longer resolves after an edit — and in all three this
declines with [`ZoomOutcome::NoBounds`] and **raises no action at all**.
It does not fall back to fit-page, and it does not zoom to the page's
origin: a command that quietly did something else when it could not do the
thing asked is worse than one that does nothing.

**The visible half of the decline belongs to the caller**, and
[`can_zoom_to_selection`] is what it binds: with no resolvable bounds the
command must render *unavailable*, which is this shell's established way of
declining visibly (`FEATURES.md`: *"a menu with nothing to offer never
opens"*). The outcome returned here is the second line of defence, for the
keyboard chord that reaches the verb without passing the condition.

### `fn arm_region_zoom`

One-shot: [`crate::canvas::show`] disarms it when the drag completes, so
the canvas returns to selecting without the operator having to leave a
mode. That matches every other marquee-zoom in the product class and it is
what keeps this from becoming a fourth thing the primary button might mean
with nothing on screen to say which.

### `fn disarm_region_zoom`

The return value is what lets Escape spend itself on exactly one thing: the
canvas ascends the selection ladder only when this reports there was
nothing armed to retire first.
