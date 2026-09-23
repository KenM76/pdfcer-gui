---
name: crate-split-is-standing-background-work
description: From 2026-09-23 Ken wants the pdfcer-gui crate split (DESIGNS.md "The GUI is one crate…" Stages 2–3) worked on continuously between other tasks; Stage 3 is approved
metadata:
  type: project
---

Ken, 2026-09-23: *"Yes always be working on the crate split between other
tasks."* This answers DESIGNS.md's open question — Stage 3 (cutting
`app` ↔ {canvas, text, panels, dialogs}) is approved, not his call pending.

**Why:** a leaf edit in `pdfcer-gui` costs ~28 s incremental vs 1.4 s in
`egui-shell`; the payoff is build time, and the module cycles grow while
features land (25 → 26 pairs between 2026-09-17 and 2026-09-23).

**How to apply:** whenever idle between his requests (e.g. waiting on an
engine release), pick up the next split step without asking. Measure with
`python tools/module-graph.py`. Every stage that moves files must re-point the
gates that hard-code `crates/pdfcer-gui/src` and FALSIFY each under the new
root. Wide mechanical stages run against a clean tree in one sitting. His
requests still come first.
