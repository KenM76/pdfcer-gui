# The Escape ladder, enumerated

[`super::super::escape`] — the function, not the file next door — is an
ordered list of `if let` arms, and the only way to hold an ordered list in
place is to enumerate it: every rung needs a case that **reaches** it and a
case that proves the rung above **did not swallow** it. That is why twelve
tests assert one short function, and why they are their own file rather
than a section of [`super`].

## The rungs, in the order Escape meets them

| Rung | Reached by | Protected from above by |
|---|---|---|
| a live drag spends the press | `an_escape_spent_on_a_drag_leaves_the_rung_alone` | — it is the top |
| a markup tool is put down | `escape_retires_the_markup_tool_before_the_region_zoom` | `an_escape_spent_on_a_markup_drag_leaves_the_tool_armed` |
| an armed region zoom retires | `escape_retires_an_armed_region_zoom_before_it_touches_the_ladder` | `an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone` |
| the selection ascends a rung | `escape_ascends_a_rung_and_raises_no_action` | `escape_reaches_the_ladder_again_once_nothing_is_armed` |

The in-progress constructions — a guide drag, a circle fit, a vertex run —
each get a pair of their own, because each is abandoned *before* the rung
below it and a second press then reaches that rung.

## ★ Not to be confused with `canvas::escape`

That module is the keyboard route **out of a canvas that drew nothing**,
and has no rungs. This one is the Escape key's precedence over the canvas's
claimants on a frame that drew normally. The two never interact; the shared
word is the key's name.

## What every case here passes, and why

`targets: None` with `model_attempted: true` — no decomposition, and the
frame asked for one — which is what lets these run without opening a file.
`page: None`, which is honest: no assertion here presses an arrow, and
[`keys::Keys::page`] exists for the nudge alone. `PickFilter::all()`, which
is what a shell that has never touched the filter hands over. Those
statements are true **of this file**; [`super`]'s header no longer claims
them of the Delete side, where a Tab case and an arrow case both exist.
