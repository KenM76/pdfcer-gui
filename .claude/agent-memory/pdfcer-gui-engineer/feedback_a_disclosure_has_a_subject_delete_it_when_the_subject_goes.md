---
name: a-disclosure-has-a-subject-delete-it-when-the-subject-goes
description: When a fix removes the visible consequence a disclosure explained, the disclosure becomes a lie with a citation attached — delete it, and record why it must not come back.
metadata:
  type: feedback
---

**A disclosure has a subject. When the subject goes, the disclosure is a lie
with a citation attached — delete it, and write down why it must not be
restored.**

**Why:** 2026-09-07. `text::rotating::rect_grew` told the operator *"the dashed
box around this mark is now larger, because a box that is square to the page has
to be bigger to hold something turned at an angle. The mark itself is exactly
the size it was."* It was correct, well argued, cited to ISO 32000-1 §12.5.2,
and commissioned by the engine team by name — **while the selection outline was
drawn from `/Rect`**. The outline is now drawn at the mark's own angle, so no box
swells, and the sentence described something that does not happen. A status row
that keeps explaining a phenomenon the operator no longer sees trains him to
ignore the row, and the next real warning is spent.

★ The sharper half: there **was** still a growth to explain — the engine defect
where a mark rotated twice really does get bigger. Restoring the sentence for
*that* would be disclosing a **defect** as though it were correct behaviour,
which is exactly how the old GUI accumulated the red flags Ken asked to be rid
of. The deleted function's site carries that warning explicitly.

**How to apply:** every time a fix removes a visible consequence, grep for the
disclosure that explained it. Delete the sentence and its tests, and leave a
comment at the site with the old text and the reason — a test that vanishes from
a file looks like coverage nobody wrote, and a deleted disclosure with no record
gets re-added by the next person who meets the same physics. Related:
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[delete-the-workaround-when-the-cause-is-removed]].
