---
name: a-symptom-at-one-zoom-is-rarely-one-bug
description: Ken's "and at other junctions too" is the load-bearing clause — keep hunting after the first cause reproduces, because one symptom has hidden seven causes.
metadata:
  type: feedback
---

When Ken names a magnification and then adds *"but it seems to happen at other
junctions too"*, the qualifier is the important half. Do **not** stop when the
first cause reproduces at the number he named.

**Why:** 2026-08-24, *"zoom out … repositions the page off screen in the far
bottom left corner … from around 2 million% but seems to happen at other
junctions too."* The 2,000,000 % crossing was real — and it was **one of seven**
independent faults with the identical symptom, and the least often reached.
Three of the other six fire at 30 %. Stopping after the named one would have
shipped a fix he could disprove in thirty seconds. The same pattern held in
2026-08-22's message, which carried three separate faults in one sentence.

**How to apply:** after the first cause is fixed and driven, *keep driving the
same gesture across the whole range* and read the trace for the symptom's
signature rather than for the cause you just fixed. Here the signature was "the
page point under the cursor becomes ≈(0,0)" — the page's own origin — and
grepping the position trace for it found three more causes after the first was
closed. Write each cause up as its own lettered sub-request so he can see the
count.

Related: [[feedback_kens_sentences_are_reports_not_measurements]],
[[feedback_scope_a_request_to_the_whole_expected_behaviour]].
