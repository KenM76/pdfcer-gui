# `ui-verify/checks/measure_calibrate`

`measure_calibrates_by_picking_two_points` — set the scale by measuring
something on the drawing, which is the workflow a drafter actually uses.

# The gap this closes


> *"Measure tool still missing the feature where we set the scale by
> selecting two lines or points and defining what that distance
> represents."*

It was real and unusually well documented: `dialogs::scale`'s own header
said the real-length path *"is offered when a reference line has been drawn,
and drawing one is a canvas gesture (`ScalePick`) that is not yet armed by
any command"*. The **model** had been complete since the Phase 7 salvage —
`ScalePick`, `ScaleEntryFields::sync_real_length`, the back-calculation
through the engine's own `preview_group_scale` — all pure and unit-tested.
What was missing was arming, click routing, and a way back into the dialog.

# Why this needs driving

Because every part of it was already unit-tested while the feature did not
exist. `ScalePick::commit_point` has tests, `ScaleEntryFields::commit` has
tests, `ScaleDialog` has tests, and an operator still could not calibrate a
drawing — because **nothing in the workspace can observe those three being
connected**. The chain is five links and three are frame-level:

1. a ribbon press opens the Set-scale dialog;
2. a button in it raises a request and steps the window aside;
3. `app::frame` notices the request and arms `MeasureKind::Scale`;
4. two canvas clicks advance `ScalePick` to a completed reference line;
5. `app::frame` notices *that*, hands the measured length to the waiting
   dialog, and disarms the tool -- which is what brings the window back.


Steps 3 and 5 are edges read once per frame. Only a running window sees
them.

# The assertion it would be easy to leave out

The last one: **the measured length is not zero**, and is near the distance
actually clicked. Without it this passes on a build that re-opens the dialog
carrying `0.0` — which is what a broken snap, a mis-mapped coordinate or a
pick that recorded the same point twice all produce, and all three are
indistinguishable from success at every earlier step.

A scale is a number every later dimension is multiplied by. A calibration
that silently measured nothing makes every dimension on the sheet wrong in
the same direction, which is worse than a tool that plainly fails.
