---
name: a-lesson-in-a-docstring-is-not-an-instrument
description: When a finding generalises, the next sibling gets written the same way anyway — build the sweep in the same session, don't write the paragraph
metadata:
  type: feedback
---

When a defect turns out to be a **class** rather than an instance, writing the
generalisation down does not stop the next instance. Build the thing that looks
for the class, in the same session, or it recurs.

**Why:** the index-vs-working-tree defect (see
[[feedback_a_gate_whose_input_set_comes_from_git_measures_the_index]]) was written
four times in this repository. The first instance,
`tools/check-suite-name-absent.py`, **recorded the correct generalisation in its
own docstring** — *"a gate whose input set is 'what is already committed' cannot
see the commit you are about to make"* — and it was written **before instances 2,
3 and 4 happened**. Instance 2 was repaired by excluding one directory. Instance 3
was the same gate again, and cost a release pre-flight that declared 41 of 41
green on a tree the packager failed thirty minutes later. Instance 4,
`check-doc-markup`, had **never once fired** in its whole life and was found only
because an audit went looking.

The prose was correct, specific, and adjacent to the code. It propagated to
nobody, because **nothing swept for the pattern.** One ~470-line script closed
the class permanently; three paragraphs had not closed it in a week.

**How to apply:**

- The trigger is the **second** occurrence, not the fifth. A second instance is
  the announcement that there is a mechanism, and the mechanism is the finding —
  repairing only the instance buys about a day.
- Ask *what would I grep for to find the next one?* If that question has an
  answer, it is a script, and writing the script is the work. If it has no answer,
  the generalisation is not yet sharp enough to be worth writing down either.
- The instrument must not carry the defect it hunts. The gate for this class walks
  the tree with `os.walk` and never asks git anything, for exactly that reason.
- Pair it with an escape hatch that states a reason, not a name: a marker like
  `gate-input-scope-exempt: <reason>` keeps the legitimate cases legible and
  distinguishes them from the ones nobody thought about. A hard-coded exclusion
  list is how instance 2 got "fixed" — see
  [[feedback_an_unevidenced_excuse_is_worse_than_silence]].
- Register it in the runner in the same commit. A checker named in every document
  and registered in no runner runs when somebody remembers — see
  [[feedback_a_checker_named_in_every_document_and_registered_in_no_runner]].
- Then falsify it. This class falsifies in one step: plant the violation in an
  **untracked** file, which a gate with the hole cannot see at all.
