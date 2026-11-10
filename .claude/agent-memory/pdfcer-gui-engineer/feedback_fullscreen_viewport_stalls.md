---
name: fullscreen-viewport-stalls-in-sweep
description: In a long sweep the 3D viewer's full-screen viewport can stop emitting frames, so a later click lands nowhere; solo it passes
metadata:
  type: feedback
---

`the_model_viewer_zooms_at_the_pointer_and_fills_the_screen` FAILed only in the 545-check sweep on 2026-11-10: after going full screen, viewport 8750 traced no frames at all (only root frames) until the check's `gone`, so the Fit click was delivered to a viewport that never painted. Its `viewport-inner` briefly read the real monitor size (3424x1401), not the off-screen 1600x648.

**Why:** a sweep-only failure needs a named mechanism before it is waved through ([[feedback_passes_alone_trace]]).
**How to apply:** if it recurs, grep the trace for frames in the model viewport after `model-view-full-screen`; no frames = this stall, not a Fit regression. Worth a harness wait-for-frame guard if it repeats.
