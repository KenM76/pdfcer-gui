---
name: a-falsification-proves-the-check-not-each-assertion
description: A red falsification run proves the check as a whole discriminates; it says nothing about whether each assertion inside it can be reached
metadata:
  type: feedback
---

**A falsification proves a check AS A WHOLE discriminates. It says nothing
about whether each assertion inside it can be reached.** After planting a
defect and watching the check go red, ask of every `if` in it: *what input
reaches this line?*

**Why:** O185's driven check had three assertions. The one its own module
header called load-bearing — the cross-run comparison that is the only thing
ruling out "Cancel and Keep are the same button" — **could never fire**, because
a per-run guard earlier in the file already implied it. Two falsification runs
had both gone red and neither exposed it: the other two assertions caught both
planted builds and reported them well. Being red is evidence about the check,
not about its parts.

**How to apply:** when a check has more than one assertion, falsify it once per
assertion, each plant aimed at only that one — or read the guards above them
and prove by hand which inputs survive to each line. And when an assertion turns
out to be unreachable, the two cases are not the same defect:

- **Unreachable because of ORDERING** — an earlier guard or an earlier
  assertion already implies it. That is a defect. Reorder so the strongest
  claim is tested first, where it can still fail.
- **Unreachable because the DOMAIN is currently too small** — the value it
  guards against has only two possible tokens today. That is a legitimate guard
  against the domain widening, and it must be **labelled in-comment as one**, or
  the next reader counts it as evidence the check does not actually provide.

Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_an_assertion_both_outcomes_satisfy_is_not_a_measurement_of_which_one_shipped]],
[[feedback_a_long_green_check_can_be_aiming_at_nothing]].
