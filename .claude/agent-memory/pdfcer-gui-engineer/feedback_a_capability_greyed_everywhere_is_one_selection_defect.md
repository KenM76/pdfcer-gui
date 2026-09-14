---
name: a-capability-greyed-everywhere-is-one-selection-defect
description: When several unrelated controls are all dead, suspect the ONE selection they share, not the controls — and when a click picks the wrong object, ask the engine again at tolerance zero
metadata:
  type: feedback
---

**Three of O198's four claims — "the font selector is always greyed", "bold and
italic are dead", "the properties area is uneditable even for text I just
added" — were one hit-test defect.** Every font control, every Properties text
field and every restyle verb in this shell is `enabled_when("selection.text_runs")`.
A pick that cannot produce a text selection switches all of them off at once,
and the operator correctly reports it as three separate dead surfaces.

**Why:** measured 2026-09-14 on his `SW41177.pdf`. Nine aims, one per distinct
font size on page 1. **Eight of nine clicks on text selected a path** — the same
path every time, object 5,899 of 5,903, very nearly the last thing painted. The
controls were registered, enabled by their conditions, green on the pinned
fixture, and unreachable on his file.

**The measurement that separated order from candidate set** was a headless probe
asking the engine the same question at four tolerances:

> at tolerance 0.0 the frontmost candidate is text at **9 of 9** aims;
> at tolerance 8.0, at **4 of 9**; text is in the list at 9 of 9, absent at none.

The path was never on top of his text. **It was winning on slack.** The engine
hits a path within *half its scaled line width plus the tolerance* and a text
object on *its bbox inflated by the tolerance*; a CAD label sits a few points
from its own cell rule, and the rule is painted later. Fix: in
`allowed_candidates`, when more than one candidate survives the filter, re-ask
at tolerance 0.0 and partition — exact hits in front, near misses behind, paint
order preserved within each. Nine of nine after.

**How to apply:**

* **Several unrelated controls dead together ⇒ find the one predicate they
  share, and measure THAT.** Do not audit the controls. A capability that is
  registered, tested and green on a fixture but reported as "always greyed" is
  almost never the capability.
* **A click that selects the wrong object: ask the engine at tolerance zero
  before theorising.** If the right answer is frontmost at 0.0 and not at the
  live tolerance, the defect is slack ranking, not paint order, not a missing
  candidate, and not an engine bug — it is shell-side and cheap.
* **The rule, stated so it is not re-litigated:** slack is a tie-breaker of last
  resort; it may promote a candidate over *nothing*, never over a candidate that
  needed no slack. This is what every editor in the class does, which is why
  none of them needs a modifier to click a label on a busy drawing.
* Related: [[feedback_a_uniform_failure_at_every_rung_of_a_sweep_is_about_the_probe]]
  is the opposite reading and was DISPROVEN here — eight of nine SKIPs at every
  rung of a walked series was the program, not the harness. Both are live; the
  probe that tells them apart is one that asks the same question a different
  way, headlessly.
