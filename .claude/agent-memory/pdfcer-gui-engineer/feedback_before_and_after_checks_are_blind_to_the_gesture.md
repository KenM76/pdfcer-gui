---
name: before-and-after-checks-are-blind-to-the-gesture
description: A roster of checks that assert the state before a press and after a release cannot see anything that happens between them — the whole feedback interval is unasserted.
metadata:
  type: feedback
---

**A driven check that builds a selection, drags, and asserts what the release
committed has asserted nothing about the interval the operator is actually
looking at.** Write a check that reads the *in-flight* frame.

**Why:** the chunk-move rows — multi-select, the rubber-band, the plural verb,
the single undo entry — were every one of them green on a build that drew
**nothing whatsoever** for the length of the drag. The defect sat exactly in
the gap between the two things the checks sample: the set that was built, and
the move that was committed. Nothing in the roster looked at the frames in
between, so a whole class of feedback defect was invisible to 246 checks.

**How to apply:** whenever a feature's value IS the feedback during a gesture —
a ghost, a rubber-band, a snap indicator, a resize outline, a drag preview —
the check must sample mid-gesture. Two ways, both already in this repo:
`Driver::drag_via(from, via, dwell, to, modifier)` holds the button down at an
intermediate point long enough for the app to paint and trace, and the harness
can photograph a gesture that exists only between a press and a release. Prefer
a trace line emitted from **inside the painter's own loop** (a count of strokes
actually drawn) over a screenshot where a count will do — a build that computes
the right thing and draws none then reports zero.

⚠ The same blindness applies to the *reason* a preview is withheld. Give the
withholding arm its own trace value rather than silence, or the absence reads
as "nothing happened" instead of "this build decided not to".

Related: [[feedback_unit_tests_that_call_the_verb_cannot_see_the_chain_in_front_of_it]]
— the same shape one layer down, and
[[feedback_an_absence_reported_by_a_check_is_usually_the_panel_going_quiet]].
