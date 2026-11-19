---
name: feedback-build-on-untagged-engine
description: Wire a shipped engine reply against engine main at once; never wait for the tag to start — the tag gates only the release pin
metadata:
  type: feedback
---

When the engine ships a reply (`reply_*_SHIPPED.md`) before tagging, move Cargo.lock to its main
(`cargo update -p pdfcer-3d`, or the crate concerned) and build, wire and drive against it now.
The tag gates only the committed pin and the release; if the tagged engine differs, rebuild then.

**Why:** Ken, 2026-11-19: "what's the harm in working on things before the engine is tagged? …
you might find bugs to send back to pdfcer before it finishes tagging. please remember to do this
from now on." I had written the G305–G307 wiring and then sat waiting on the tag.

**How to apply:** start on a shipped reply immediately. Bugs found go back as requests while the
engine can still fix them before the tag. Commit the work; the pin-move commit (lock, API snapshot,
licences, FEATURES/RESUME pin lines) and the release still follow the tag ([[project-pin-moves-approved]]).
