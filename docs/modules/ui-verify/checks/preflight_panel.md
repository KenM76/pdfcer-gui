# `the_preflight_panel_runs_jumps_and_fixes`

**Owns:** the Preflight panel (O458): the run, the jump, the fix.

**Drive.** `preflight-low-ppi.pdf` (`fixtures/preflight-low-ppi.PROVENANCE.py`),
launched with `mode.edit,view.preflight`: a CMYK output intent, no TrimBox,
an RGB fill on page 1, and on page 2 a 4×4 image drawn 300 pt square (about
1 ppi), page object 1. Pages 3 to 242 carry a TrimBox and 2,000 CMYK
marks each and no finding: they make the run outlast many frames (about
0.7 s), so `frames=` can be non-zero. Press Run (digital-printing), then the row the
`preflight-rows` listing names `image-low-ppi@2:object`, then the first
rule's Fix (or Fix all when no rule offers one alone).

**Oracle.**

- `preflight-finished` with `findings>0` and `cancelled=0`.
- `frames>0` on that line: the UI drew while the worker ran.
- `preflight-picked selected=1`, then `selection-set page=1 object=1`
  (0-based page): the canvas's own selection holds the image.
- Fix is pressed after the pick, so the Preflight tab must still be in
  front: a pick that let Properties cover it would leave the Fix region
  retired and the check red.
- `preflight-fixed` with `steps>0`.

**Not covered.** The rows' pixels; Cancel; the stale line after an edit;
Copy report's clipboard contents.
