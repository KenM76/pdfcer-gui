---
name: later-feature-captures-an-earlier-gesture
description: A new feature that intercepts a gesture (a click widening a run to its paragraph) silently broke an earlier feature's driven check; three releases shipped it. Re-drive the neighbours' checks, not just the new one.
metadata:
  type: feedback
---

**When a feature changes what a gesture opens, re-drive every check that uses
that gesture, not only the new feature's own check.**

**Why:** O288 item 9 made a click on any line of a multi-line paragraph open
the whole paragraph (`textedit::promote::widen`). The engine's v0.80 OCR
writes recognised words into paragraph blocks, so a click on an OCR word now
opened a block draft and lost O286 item 5's own-font, layer-coloured preview.
`an_ocr_word_is_previewed_in_its_own_font_in_the_layer_colour` went red, and
v0.5.0-dev.20261008.1, .2 and .3 shipped that way. The gates were green the
whole time: driven checks are not in `run-all.sh`, and the per-commit
falsify scripts drove only the new feature's checks. It was found only
because item 15's regression list happened to include that neighbour.

**How to apply:** when a change touches a gesture's routing (place, widen,
promote, a click handler), grep `tools/ui-verify/src/checks` for the trace
lines that gesture emits (`text-edit-shaped`, `text-edit-caret`, ...) and put
every check that waits on them into the regression set of the falsify script.
Run the control (the last published exe) alongside, so a red neighbour is
attributed to the right commit.
