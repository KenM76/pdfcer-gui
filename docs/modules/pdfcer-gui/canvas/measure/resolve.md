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
