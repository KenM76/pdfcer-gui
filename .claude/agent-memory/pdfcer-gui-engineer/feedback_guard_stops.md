---
name: a-guard-that-stops-repetition-does-not-stop-creep
description: When a measurement feeds a size, a "never ask twice for the same value" guard does not close the loop — monotonic growth makes every value a new one.
metadata:
  type: feedback
---

When a value this shell computes feeds back into something that changes that
value, **remembering the last request and refusing to repeat it is not a loop
guard.** Monotonic creep makes every request a new one, so the guard never
fires. Bound the *direction* and put a floor on how much change is worth
acting on.

**Why:** `dialogs::host::Host::fit` grows a dialog's window to fit its body. Its
first version padded the measured content by an item spacing before comparing,
so the wanted size exceeded the window every frame and each frame asked for
eight more pixels than the last. The About window opened at 560×480 and was
**1,624×746** by the time the trace was read. The once-per-size guard was
present and useless. This is R128's fit-zoom feedback loop in a second place,
and the project has now paid for it twice.

**How to apply:** any time a rendered measurement is fed back into layout, zoom,
or window geometry — three guards, not one: (1) change in one direction only,
(2) a floor below which the difference is measurement noise, (3) never repeat an
identical request. Measure **raw**: a margin added before the comparison is
indistinguishable from real overflow, which is exactly what made this a loop.

Related: [[write-the-lesson-to-the-rag-not-the-chat]] — this is filed in
`D:/dev/rag/egui/a_fit_to_content_window_resize_that_adds_a_margin_grows_forever_because_every_size_is_a_new_size.md`
with the code.
