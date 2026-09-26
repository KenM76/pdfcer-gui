# `ui-verify/input/observed`

## Item notes

### `struct ButtonHeld`

Nothing is stored in it; its whole job is its `Drop`. Holding the release
there rather than writing it at the end of the happy path is what makes the
verb safe to `?` through and safe to panic through.

### `fn drag_observed`

The gesture is [`Self::drag`]'s, up to the point where that verb
releases: raise, confirm both endpoints are uncovered, press at `from`,
walk to `to`. Then the pointer rests at `to` for [`OBSERVE_DWELL`],
nudged one pixel between ticks so a build that repaints only on input
still runs the frame being photographed, and `observe` is called.

The release happens after `observe` returns — or unwinds — and it
happens **at the destination**, so the drag completes normally and the
caller may go on to assert that the move landed. A check written this
way measures the affordance and the outcome in one gesture, which is
the only way to know the two describe the same drag.

# Errors

As [`Self::drag`], plus whatever `observe` returns.
