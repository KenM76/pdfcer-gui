---
name: feedback_relayed_gate_verdict
description: A gate verdict relayed by another agent may be read from a different, older log; commit only on your own run's RESULT line
metadata:
  type: feedback
---

Commit only on the RESULT line of the gate log your own run is writing, never on a verdict someone relays.

**Why:** On 2026-10-03 the coordinator reported "84 passed, RESULT: PASS" for O284 item 4 from `target/m3d_gates.txt`. That file was dated 30 September, and the real run was still at `check-region-names`. Committing on it would have committed ungated work. The coordinator later confirmed the log was stale.

**How to apply:** When told a run finished, `ls -la` the cited file for its mtime, and read your own `$S/fullgates_<tag>.txt` before committing. Related: [[feedback_skip_red_check_stop]].
