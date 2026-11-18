---
name: retargeted-check-keeps-gesture
description: A check moved to a different tool keeps the old tool's gesture; plain `declared` lookups skip once a new ribbon item pushes theirs into overflow
metadata:
  type: feedback
---

When the text box became typed-in-place, two dialog checks were re-pointed from
Text box to Sticky note with names and messages changed, but they still DRAGGED.
The sticky is click-placed (`is_dragged` false), so the drag placed nothing. That
stayed hidden because the same checks were SKIPPING: two new Markup items had pushed
Sticky/Stamp into the ribbon overflow at 1100 px, and they looked the item up with a
plain `declared`.

**Why:** a retarget edits names; the gesture is the part that encodes the tool's
contract. A SKIP hides the next defect behind it ([[feedback_skip_red_check_stop]]).
**How to apply:** when re-pointing a check at another tool, re-read its gesture
against that tool's placing rule. Ribbon items in checks go through
`declared_or_in_overflow`, never `declared`.
