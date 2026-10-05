---
name: skip-on-missing-asset
description: A driven OCR check SKIPping with missing-files at HEAD meant target/release/models was incomplete, not that the engine was absent — check the asset folder before reading a SKIP as a capability verdict
metadata:
  type: feedback
---

A SKIP whose reason is a missing model file is first a question about the
scratch build's asset folder, not about the capability. The paddle check
SKIPped at HEAD (`missing-files`) because `target/release/models/paddle` held
only part of the add-on; hard-linking det.onnx, rec.onnx and LICENSE from the
installed slot turned it into a PASS.

**Why:** the reflex reading "paddle is unrunnable at HEAD" would have been a
false regression report in an O286 bisect whose whole subject was "OCR stopped
working".

**How to apply:** before quoting an OCR (or any add-on) SKIP, list the model
folder beside the exe under test against the installed slot. Related:
[[feedback_skip_red_check_stop]], [[feedback_launch_failure_blamed]].
