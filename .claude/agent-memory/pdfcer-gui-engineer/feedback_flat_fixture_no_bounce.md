---
name: flat-fixture-no-bounce
description: A bounce/indirect-light oracle on coplanar geometry measures zero by physics, not by bug; visualise the shader accumulator before blaming wiring
metadata:
  type: feedback
---

A bounce check on `coloured.prc` (three coplanar squares) failed "bounce arrived, picture unchanged" with every piece of wiring verified. Coplanar faces exchange no light, so the bounce was correctly black. The check now drives `floor-box.prc`.

**Why:** hours went into GL-state probes before the fixture's geometry was questioned; a constant injected into a saturated channel also "showed nothing".

**How to apply:** for any lighting/indirect oracle, first ask whether the fixture's faces can see each other. Debug a silent shader term by outputting its accumulator (`rgb = s*0.25`), then branching its return on the normaliser — see the egui RAG entry. Related: [[feedback_oracle_needs]], [[feedback_assertion_both]].
