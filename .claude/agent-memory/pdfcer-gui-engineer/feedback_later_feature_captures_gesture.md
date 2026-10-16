---
name: later-feature-captures-an-earlier-gesture
description: A new feature that intercepts a gesture (a click widening a run to its paragraph) silently broke an earlier feature's driven check; three releases shipped it; a second victim, the refused-edit offer, too. Re-drive the neighbours' checks, not just the new one.
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

**A second victim of the same widening:** `a_refused_edit_offers_its_workaround`
(O285 item 6). Typing in `quote-operator.pdf`'s line now widens to a block
rewrite, which the engine accepts, so the `'`-operator refusal its offer hangs
on never happens. It was found by the O293 neighbour sweep on 2026-10-14 and
was confirmed red on the published build as the control.

**A third occurrence, same victim (2026-10-16):** G203 widened a click on
recognised text to the recognised paragraph. That paragraph has no engine
layout preview, so the OCR word's in-place preview was lost again. Two OCR
preview checks went red, and v0.5.0-dev.20261016.1 shipped that way because
the release was made before the neighbour sweep. The fix: a feature with no
preview never takes the click; it opens on an explicit command (*This
paragraph*). When a release is rushed ahead of testing, drive the OCR-preview
neighbours FIRST, since this gesture has been captured three times.
