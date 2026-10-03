---
name: read-passing-trace
description: Read the trace of a PASSING check — Insert's receipt said "after page 2" for an insert after page 1 while the index oracle passed
metadata:
  type: feedback
---

After a check passes, skim the trace lines it keyed on, including the fields it does not assert. The P10 drive passed on `insert-pages page=1 n=1`. The same line's `disclosures=` said "Inserted 1 page after page 2.", which was a long-standing off-by-one in Insert from file: the caller passed the landing index as the page the insert follows.

**Why:** an oracle asserts what its author thought of. The operator reads the sentence, not the index, so a check can be green while he is shown the wrong page number.

**How to apply:** on every new check's first PASS, read the full matched lines. If a sentence the operator sees is on the line, assert it (`TraceLine::raw` holds the whole line, because `get` splits on spaces). Related: [[feedback_assertion_both]], [[feedback_check_input_chosen]].
