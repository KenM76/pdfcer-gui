---
name: feedback-drive-only-affected
description: Between releases drive only the checks touched by what changed since the last release tag, never the full 95-minute sweep by default
metadata:
  type: feedback
---

Drive only the ui-verify checks that what changed since the last release can
affect. The full sweep is not the default before a release (Ken, 2026-10-16,
O305: "minimize your tests to only what was changed and would be affected
between releases, and save this as a rule going forward").

**Why:** the full sweep is ~95 minutes of heavy CPU on the machine he is using,
and it had just intruded on his desktop and keyboard.

**How to apply:** before a release, select checks from `git diff <last
release tag>..HEAD` (the selector in tools/ui-verify — see DEVELOPING.md), drive
those with `--no-input`, and say in the release report which were driven and
why. Run the full sweep only when the change is framework-wide (dock, ribbon,
launch, harness) or he asks. Gates (run-all.sh) still run on every commit.
Related: [[feedback_ui_verify_competes]], [[feedback_smoke_launch_before]].
