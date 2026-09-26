# `canvas::measure::circular` — the radius/diameter tool, and the gesture
the operator has to end

The canvas hosting for [`MeasureKind::Circular`] alone: what one click does
to the fit set, what the two endings do, and what has to be resolved out of
the decomposition so the set can be drawn. [`super`] hosts the other two
tools and everything the three share — the memory, the snap resolution, the
preview painting.

## Why this is a file of its own, and what the seam actually is

**R2** (no `.rs` file over 1,500 lines) forced a split when the tool was
armed: [`super`] reached 1,617 lines. But the line count only says *that*
something had to move; it does not say what, and `tools/gates/check-file-size.sh`
says in its own header that shaving prose to fit a threshold is the
behaviour it exists to refuse. So the question was which subject was
separable, and this one is, for a reason none of the other tools give:

> **Linear and two-line gestures end themselves. This one does not.**

A linear dimension is finished at its third click and a two-line dimension
at its second, because both are picks of a **fixed arity** — the pick
machine in [`super::pick`] knows it is done, and [`super::click`] simply
raises whatever the machine hands back. A best-fit circle has no such
number. An arc drawn as four separate polyline objects needs four picks; the
same arc drawn as one needs one; nothing in the geometry can tell pdfcer
which the operator meant. So the operator says when, and the machinery for
*saying when* — two entrances, one commit path, a predicate the ribbon reads
every frame to decide whether the control is even live — is a subject the
other two tools have nothing corresponding to.

That is the seam. Everything here answers *"when is this gesture over, and
what does ending it do?"*; everything left in [`super`] answers *"where did
that click land?"*.

## The two endings, and why there is exactly one commit path

| ending | entrance | why it exists |
|---|---|---|
| **double-click** on the canvas | [`click`], via [`super::click`]'s `double` flag | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
| **`measure.finish`** on the ribbon | [`finish`], via `app::dispatch` | discoverable without knowing the double-click, and reachable when the last picked arc sits somewhere awkward to double-click |

Both call [`commit`] and nothing else raises a circular
`Action::CommitDimension`. Two arms that each assembled a `DimensionKind`
would be two derivations of one answer: they would agree on the day they
were written, diverge at the first change to either, and **the operator
would have no way to see it** — a circle fitted from the same points looks
the same whichever code drew it.

Neither ending is an accept box floating over the canvas, which is what
decision 024 retired at the operator's instruction and what kept this tool
unarmed through Phase 7.

## This module owns no geometry either

The fit is [`pdfcer_core::dimension::fit_circle_taubin`], reached through
[`super::pick::CircularPick`]; the authored value is `pdfcer-core`'s own
`DimensionKind`. Nothing here computes a centre, a radius or a residual.
What it owns is *composition and lifetime*: which objects are in the set,
when the set becomes a dimension, and when it is emptied.

## Item notes

### `fn pending`

The single derivation behind both halves of the Finish control:
[`finishable`] asks whether to enable it and [`finish`] asks what to do when
it is pressed. Two spellings of "is there something to finish?" would
eventually disagree, and the way they would disagree is the worst available
— an enabled control that does nothing when pressed, which is precisely the
placeholder the no-placeholders invariant forbids.

Three conditions, and each rules out a state that really occurs:

1. **The radius/diameter tool is armed.** The pick set outlives disarming
   (`disarm_measure` puts the tool down; it does not discard work — see its
   docs, and Escape's two rungs), so without this the ribbon would offer
   Finish for a set the operator can no longer see being outlined.
2. **A state exists.** Nothing has been picked on this page since the tool
   was armed, so there is nothing to finish.
3. **The fit is not degenerate** — [`super::pick::CircularPick::author`]
   returns `None` for fewer than three usable points or a numerically
   singular set, and its own docs say that is when Accept must not be
   offered. One or two picked objects whose anchors lie on a line is the
   ordinary way to reach it, not a pathological one.

### `fn origin_tag`

Deliberately not the operator-facing wording: a trace field is a machine
contract a driven check matches on, and coupling it to a translatable string
would make a wording change break the harness. `crate::text` owns what the
operator reads.

### `fn a_click_adds_a_point_and_a_click_near_it_takes_that_point_out`

The whole of the pick, and both halves matter. A build that only added
would pass a test of the first click alone, and the operator's complaint
would be `OPERATOR_REQUESTS.md` O107 — *"I can't unselect things once I
have selected them"* — which is how this behaviour came to be asked for
in the first place.

The second assertion is the one that could not exist under the old
object pick: it adds a point 0.5 pt from the first, and requires the set
to SHRINK. Under an object pick there was no such distance — the unit
was the whole object — which is exactly why *"selecting more points
around a hole"* did not narrow the fit.

### `fn a_pick_with_no_snap_candidate_is_recorded_as_a_free_position`

The origin is carried rather than discarded, because a set of five free
positions and a set of five snapped nodes produce the same numbers and
are not the same evidence. The Tool panel says which; the canvas does
not, which is rule 4's disclosure boundary.

### `fn three_free_positions_on_a_raster_still_produce_a_circle`

Asserted on the fit rather than on the count, because *"the points went
in"* is not the claim — the claim is that a drawing with no vector
geometry at all can still be measured.

### `fn removing_a_point_from_the_panel_changes_the_set_the_canvas_draws`

`OPERATOR_REQUESTS.md` O107 asks for both routes, and the failure to
guard against is two pick sets: a panel that removed from its own copy
would leave the canvas drawing markers for points the fit no longer
contains, which is worse than having no panel.

### `fn the_double_click_and_the_command_author_the_same_dimension`

The property the one-commit-path design exists for, asserted the only
way that means anything: run *both* endings over identical states and
compare the actions they raise. Two arms that each built a
`DimensionKind` would agree on the day they were written, drift on the
first change to either, and the operator would have no way to see it — a
circle fitted from the same points looks the same whichever code drew
it.

### `fn finishing_empties_the_pick_set_so_it_cannot_be_committed_twice`

The failure without it is quiet and expensive: the operator presses
Finish, sees the dimension land, presses it again out of habit or
because they did not see the first, and gets two dimensions stacked
exactly on top of each other — indistinguishable on screen and two undo
steps to remove.

### `fn a_degenerate_fit_is_refused_by_both_endings`

`CircularPick::author` returns `None` for fewer than three usable points
or a numerically singular set, and its docs say that is precisely when
Finish must not be offered. Three points the operator clicked along a
straight edge is the ordinary way to reach it — not a contrived one —
and the honest response is to place nothing rather than to guess.

### `fn finish_is_offered_only_when_there_is_a_fit_and_the_tool_is_armed`

This is the condition behind a ribbon control, so each `false` row is a
control that would otherwise be live and inert. The fourth row is the
one that is easy to miss: putting the tool down does **not** discard the
pick set (Escape's two rungs, `disarm_measure`'s own docs), so without
the armed-tool check the ribbon would keep offering Finish for a set
nothing is marking any more.

### `fn asking_whether_finish_is_available_creates_no_measure_state`

[`finishable`] runs on every frame, for every document, armed or not. If
it went through `super::load` — which builds a `MeasureState` when there
is none — the ribbon merely *drawing itself* would leave a measure state
in memory for a tool nobody armed, and the next `store` would persist
it.
