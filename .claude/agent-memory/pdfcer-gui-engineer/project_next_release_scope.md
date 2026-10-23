---
name: next-release-scope
description: Next release = after the next pdfcer engine tag is folded in and its new features supported; O323/O324/O325 work carries on meanwhile
metadata:
  type: project
---

2026-10-23, Ken, twice:
1. "Release after that work is done." — the Forms tab with Style (O324), menus/dialogs/right-click for every v0.92.0 size-reduction feature (O325), overprint + Output Preview (O323).
2. Later the same day: "carry on. once the new pdfcer engine is released finish up any work to support any new features then make a release."

**Why:** he wants the next release to carry the next engine and the GUI for whatever it adds, plus the O323–O325 work he asked for.

**How to apply:** keep building O325 (O323 and O324 are done) while waiting. When an engine tag after v0.92.0 lands: move the pin, build GUI support for its new features (each driven and falsified), finish O325 stages that are close, then release without asking (clean tree, gates, smoke, GitHub + OneDrive, no prerelease). If an O325 stage is far from done at that point, ship it in the next release and say so in the report rather than hold this one. Delete this memory once the release ships.
