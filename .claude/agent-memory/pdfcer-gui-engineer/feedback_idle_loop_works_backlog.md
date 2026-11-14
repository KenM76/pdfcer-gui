---
name: feedback-idle-loop-works-backlog
description: An idle request/engine loop tick is not idle — work the ENGINE_BACKLOG "wanted" rows; a pin release whose feature sits unwired should ship WITH the wiring
metadata:
  type: feedback
---

Between loop ticks with no new request and no new engine tag, build the
`ENGINE_BACKLOG.md` **wanted** rows (78 on 2026-11-14) instead of reporting
"nothing new". When an engine release's improvement sits behind a switch the
shell does not set, wire the switch as part of that release, not "later".

**Why:** Ken, 2026-11-14, after a v0.151.0 release note said the vector-shadings
switch was "already on the GUI's to-do list": *"What is on the to-do list? Why
isn't everything just getting done before you release?"* Eight idle ticks had
reported "nothing new" while the print-route switches sat wanted since v0.146.0.

**How to apply:** a loop tick checks the folder and the tags first (a new tag
still releases promptly), then picks the next wanted row and builds it to the
usual bar (driven check, gates). Prefer rows an imminent release would
otherwise ship unconsumed. Related: [[feedback_never_defer_external]],
[[feedback_engine_feature_unconsumed]].
