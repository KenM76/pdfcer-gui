---
name: a-fixture-twin-page-answers-a-whole-file-judge
description: annots-align.pdf repeats page 1's annotations (same /NM, same /Rect) on page 2, so a saved-file judge reading every page finds a "survivor" the cut never touched; scope saved-file judges to the page the gesture acted on
metadata:
  type: feedback
---

Scope a saved-file judgement to the page the gesture acted on: `savedannots::listed_on_page(bytes, n)`, not `listed_annots`.

**Why:** `fixtures/annots-align.pdf` carries the same SQ/CI/LK annotations (same `/NM`, same `/Rect`) on page 2. A Ctrl+X that correctly removed page 1's three was judged "still lists `/NM (SQ)`" — page 2's copy answered. I then misread the failure as "Ctrl+D selects the duplicates" and rewrote the judge around that convention; the trace (`annot-delete-gates id=5/7/9`, the originals) and the saved file's two `/Annots` arrays disproved it. Two wrong turns before opening the artifact.

**How to apply:** when a saved-file assertion fails, dump the saved file's per-page `/Annots` and object bodies first (pass `--out` to an absolute folder: the default relative out lands in the launch sandbox and is deleted). Only then decide whether the app or the judge is wrong. Related: [[a-value-cannot-identify-its-producer]], [[defined-is-not-on-the-page]].
