---
name: a-measured-note-that-names-the-wrong-axis-is-wrong-by-the-aspect-ratio
description: A note quoting "the sheet is N pt tall" is a claim about WHICH axis — pick the wrong one on a landscape sheet and every threshold derived from it is off by the aspect ratio, while still looking measured
metadata:
  type: feedback
---

**A dimension in a note is a claim about an axis. Name the axis, and say which
of the sheet's two numbers you used.**

**Why:** 2026-09-13, `zoom_out_keeps_place.rs`'s `MAX_CLIMB` note. It said the
A1 fixture is "1683.8 pt tall", derived a threshold of ~623 from it, and
concluded the check needed 33 notches. **1683.8 is the A1 sheet's SHORT side;
the long side is 2383.9**, and the check climbs along the long one. The real
threshold is ~440, crossed at notch **39**, at 46,479 %. Every number in the
note was wrong by the aspect ratio — about 1.42× — and the note read as
authoritative precisely *because* it quoted a measured figure to one decimal
place. A wrong number with a decimal point is more persuasive than a vague one.

This is Rule 15's shape applied to geometry rather than vocabulary: **a bare
"dimension" is ambiguous, and so is a bare "tall"**. On a landscape CAD sheet
"tall" and "the big number" are different numbers, and the one you want is
whichever axis the code under test actually travels along.

**How to apply:** in any note, test comment or constant justification that
derives a bound from a page size:

- Write **both** numbers and say which one the code uses:
  *"A1 landscape, 2383.9 × 1683.8 pt; this bound is on the 2383.9 axis because
  the climb is horizontal."*
- Re-derive the bound from the trace, not from the note — `grep` the actual
  fixture (`grep -a -o 'MediaBox[^]]*\]' file.pdf`) rather than recalling it.
- Treat any surviving bare "tall"/"wide"/"long" near a magnitude as unmeasured
  until re-checked. ★ The tell: a constant with generous headroom that has
  never once been approached — the headroom is hiding an error in the
  derivation, not proving the bound is safe.

Related: [[a-measured-limit-belongs-to-a-revision-not-a-design]] and
[[walk-the-series-never-pick-endpoints]] — walking the series is what found
notch 39 and disproved notch 33.
