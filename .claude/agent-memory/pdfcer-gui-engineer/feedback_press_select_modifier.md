---
name: press-select-modifier
description: A select-on-press step must decline on EVERY modifier the click treats as a toggle, or the click toggles its own pick back out
metadata:
  type: feedback
---

When a step selects on button-DOWN and the click on button-UP toggles, both
must read the same modifier set. `canvas::presspick::at_press` declined on
Shift only while the click toggled on Shift OR Ctrl, so Ctrl+click on an
unselected object replaced the selection at press and then removed it at
release: `sel=0`. Nobody reported it; it surfaced the first time a check
gathered a selection with Ctrl.

**Why:** two readers of one gesture, each with its own copy of "is this
additive?", drift apart silently. The only witness is a driven check using
the modifier the other reader forgot.

**How to apply:** when adding a modifier to a click path, grep every other
step of the same gesture (press, drag start, click) for its own modifier
read. Drive with each modifier, not just the one the original test used.
See [[feedback_proxy_condition]].
