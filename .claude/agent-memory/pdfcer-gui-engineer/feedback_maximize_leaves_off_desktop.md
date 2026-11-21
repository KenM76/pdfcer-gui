---
name: feedback-maximize-leaves-off-desktop
description: Under --no-input, a check calling Session::maximize put its window onto Ken's monitor and an OS capture photographed his desktop; maximize now grows the window in place off-desktop
metadata:
  type: feedback
---

An off-screen run is only off-screen until something maximises the window. `sys::maximize_window` moves the window onto a monitor. Under `--no-input` that window then sits over Ken's work, and `capture::window_to_png` photographs whatever that monitor shows: once, his 3D viewer. The result was a FAIL that read like a product defect, with the control patch at 0.000.

**Why:** the off-desktop guard covered the launch placement only. `maximize` bypassed it.

**How to apply:**
- `Session::maximize` resizes in place when `OFF_DESKTOP` is set.
- Any check judging pixels takes `ScriptedPointer::screenshot`, never an OS capture.
- If a check whose window shows Ken's desktop goes red, suspect the capture before the product.
- 220 of 231 sweep SKIPs were OS-input checks. Their conversion is the coverage lever. See [[feedback-skip-red-check-stop]].
