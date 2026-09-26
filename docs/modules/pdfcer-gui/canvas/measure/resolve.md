# `canvas::measure::resolve` — one derivation of *"where would this click
land, and on what"*

The seam against [`super`]: everything here answers a question **about the
pointer**, and everything there is about the tools and their state.

## Why this is one function and not two

[`Resolved`] carries the whole answer — the snapped point, the candidate
that produced it, and the entity under the pointer — because the indicator
the operator aims at and the point the next click commits must be *the same
value*, not two derivations that agree by construction.

⚠ Two derivations fail invisibly. Resolve the marker against a raw screen
position while the click uses a converted canvas one and the two disagree by
the scroll origin over the zoom — **zero at the top-left of an unscrolled
page at 100 %**, growing from there. That reads as *"sometimes it is fine"*,
and no unit test can see it, because both functions are individually
correct.

## Neither reader may require stored state

`MeasureState` is not written to `egui::Memory` until the operator has
clicked once — [`super::load`] builds a default and only the click paths
store it. A reader that bails on the empty case therefore switches the whole
hover affordance on **after the first pick of a gesture**, leaving the snap
marker and the entity highlight dead in exactly the moment they do their
work: *"the measuring tools don't give me any indication of what is being
selected"*.

⇒ Both this function and the paint site fall back to a value built from the
armed kind rather than declining.

A read must not write. Persisting from here would make moving the pointer an
edit to shared state, and arming is the only thing that should decide what
is armed.

## Item notes

### `fn the_master_toggle_off_returns_the_raw_point`

The master toggle is the operator's, and a tool that snapped anyway
would be applying an inference they had switched off — which is rule 4's
definition of sneaky rather than fuzzy.

### `fn a_caller_with_no_stored_state_gets_the_shipped_defaults`

The vertex drag's case, and the reason [`snap_point`] exists rather than
the caller doing `read(ctx)?`. [`load`] persists nothing and [`read`]
answers `None` until a measure tool has been *clicked*, so a `?` here
would mean **snapping switches on only after you have used a different
tool** — which is not a smaller version of the feature; it is the exact
defect `resolve_hover`'s own header records, which shipped and was
reported as *"the measuring tools don't give me any indication of what
is being selected"*.

Asserted through the master toggle rather than through a candidate,
because what is being proved is *which state was used*: a build that
declined on empty memory answers the raw point with `None`, and so does
a build that found nothing nearby. The distinguishable fact is that the
fallback carries `snap_master: true`.

### `fn the_snap_radius_is_wider_than_the_selection_radius_at_every_zoom`

Both are screen-pixel constants divided by the same zoom, so the
relation is scale-invariant — and an assertion on a relation alone
passes for a build whose magnitudes have both gone wrong together, so
the magnitudes are checked too.

### `fn snapped`

This is what puts [`crate::canvas::snap`]'s primitives on the pick path. A
pick taken at the raw pointer position is, on a CAD sheet, the difference
between a dimension that measures a line and one that measures *near* a
line — and the second is worse than no dimension, because it is wrong by an
amount nobody can see.

# The four gates, in order, and what each is for

1. **The master toggle and the Alt override**, through
   [`snap::snap_query_enabled`]. Alt is the operator saying *"not this
   time"* — it is what makes the offer refusable, and it is why the catch
   radius can afford to be generous ([`PageMapping::snap_tolerance`]).
2. **A decomposition must exist.** No model, no candidates, raw point. This
   is a real case rather than a defensive one: the model is built only when
   something needs it, and a measure click is one of the things that asks.
3. **The query**, `pdfcer_core::vector::snap_candidates` — the engine's, not
   ours. `SnapConfig::new` leaves intersections off and axes on, which is
   the shipped default; the grid is `None` because the canvas grid is a
   *view* aid drawn in page space and snapping to it would be snapping to
   something the document does not contain.
4. **The Tab cycle**, through [`snap::active_snap_candidate`], which is what
   lets the operator choose between an endpoint and a midpoint that are
   within a few pixels of each other.

Returns the raw point unchanged when any gate declines, with `None` for the
candidate — so a caller never has to distinguish "snapping is off" from
"nothing was near", because neither changes what it does next.

### `fn snap_point`

The perimeter's vertex drag is the caller. It routes here rather than
snapping by its own rules because **a predicate with two claimants must
exist exactly once**: *"where would this land if it snapped"* is one
question, and the operator's answer to it — the master toggle, the Alt
override, the tolerance, the Tab cycle — is one set of settings. A drag with
its own snap would honour a different "Snap to content" switch from the tool
beside it.

# The gap this closes, in the operator's terms

`ui-conventions/drag-moves.md` D6 requires a snap to announce its target
while the drag is live. A vertex drag that does not snap at all, while the
tool that placed that vertex does, is worse than a silent one: the tool
teaches the operator that corners land exactly on lines, then withdraws it
for the one gesture whose entire purpose is correcting a corner that landed
wrong.

# It builds a `MeasureState` rather than requiring one

[`super::load`] persists nothing and [`super::read`] answers `None` until
the operator has clicked a measure tool at least once. A vertex drag can
happen in a session where no measure tool was ever armed, so requiring
stored state would mean *snapping switches on only after you have used a
different tool* — the defect [`resolve_hover`]'s own body records.

The fallback carries `snap_master: true`, the shipped default, and a
`snap_cycle` of 0. Tab-cycling between two nearby candidates is therefore
**not** offered during a vertex drag; that is a decision rather than an
oversight, because Tab during a drag is not a gesture any program in the
class defines.

### `struct Resolved`

The indicator and the click read *this same value*, which is the whole
reason it exists as a type rather than as two calls to [`snapped`]. A
preview that re-ran the query would be a second derivation of the same
answer, and the two would agree right up until they did not — the operator
aiming at a marker drawn over an endpoint and committing a point somewhere
else. `pdfce_FeatureRequests/README.md` rule 4 is explicit that a
pre-commit affordance must describe *what is about to happen*; one
derivation is how that stays true rather than being maintained.

### `fn resolve_hover`

Called from `canvas::interact` **before** it drops the provider, which is
the constraint that shaped this API: the draw happens after the drop, so the
query cannot happen there.
`canvas_pos` is **CANVAS** space, not screen space, and the name says so
because getting it wrong is invisible.

# ⚠ Why the name is load-bearing

Hand this an unconverted `screen_pos` and the **click** path still commits
the right place — it converts through `Pick::canvas_point` — while only the
**preview** resolves its candidate near a different part of the page. A
wrong answer beside a right one, offset by the scroll origin over the zoom,
so it is zero at the top-left of an unscrolled page at 100 % and grows from
there. The operator sees it before any test does:

> *"when I click on measure it on the drawing the crosshairs click the
> right place under them, but the preview of what is being selected is
> offset from the crosshairs instead of being underneath them."*

`Pos2` cannot carry its own space, so the only defences available are the
parameter's **name** and this paragraph. `canvas::mapping`'s header is the
standing argument for why these conversions live in one place; the call
sites are the other half of it.

### `fn frame`

The constraint that shapes it: the page decomposition is borrowed **only
here** and dropped before anything is painted, so this query cannot happen
at paint time — and it must not be repeated. See this module's header for
what two derivations of one answer cost.

One value out, not two. The circular pick set is a list of POINTS
(`pick::CircularPick`) which the preview reads straight out of
`egui::Memory` and projects itself, so it needs no decomposition and no
channel back through three call sites; only the hover needs the borrow.

`kind` is `None` for every non-measure tool, and then this costs one
`Option` check and runs no query at all: panning a 129,758-object drawing
decomposes nothing.
