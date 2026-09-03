---
name: smoke-launch-offscreen-when-the-desktop-is-blocked
description: PDFCER_DIAG_VIEWPORT gives a real laid-out off-screen window, so a launch-and-read-the-trace check costs the operator nothing even when ui-verify is blocked
metadata:
  type: feedback
---

**When the go-ahead for `ui-verify` has not been given, a smoke launch is still
available and should still be done.**

```bash
PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT=-4000,-4000,1400,900 \
  timeout 12 ./target/release/pdfcer-gui.exe <fixture.pdf> > smoke.txt 2>&1
```

`with_position([-4000,-4000]) + with_active(false)` gives a window that is
**genuinely laid out and genuinely invisible** — panels allocate their real
sizes, regions publish their real rects — and it does not steal focus. It
injects no input, so it does not touch the pointer or the keyboard.

**Why:** [[ui-verify-competes-for-the-machine]] blocks *executing the harness*,
which is input injection plus window raising. A bare offscreen launch is
neither. Skipping it means shipping with no evidence at all that the binary
starts, which is strictly worse than shipping with partial evidence.

**How to apply:**

- Run it after every release build, and grep the trace for the regions the
  change was supposed to produce. On 2026-08-20 it proved the new document tab
  strip was composed on a real frame with 108 × 22 pt of clickable extent — the
  whole *"registered but never drawn / drawn at zero height / clipped out of its
  pane"* family, which this project keeps finding, ruled out in one command.
- **Report exactly what it does and does not establish.** It says a surface is
  drawn and where. It says nothing about any gesture. Write both sentences; the
  second is the one that keeps [[r1-drive-the-binary]] intact.
- It never substitutes for a driven check. Queue those as usual.
