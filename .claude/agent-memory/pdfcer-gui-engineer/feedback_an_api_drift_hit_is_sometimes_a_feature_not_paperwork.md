---
name: an-api-drift-hit-is-sometimes-a-feature-not-paperwork
description: check-engine-api-drift going red on a new engine item may be a feature report; the exemption line is the one response that guarantees nobody asks why the engine moved
metadata:
  type: feedback
---

When `check-engine-api-drift` goes red on a new public engine item, **ask what
the item MEASURES and what it is measured AGAINST before writing an exemption.**
The gate's contract is only *"this repository names it somewhere"*, so an
exemption discharges it in fifteen seconds — and destroys the report.

**Why:** 2026-09-10. The gate went red on
`pdfcer_render::Diagnostics::annotations_icon_painted`. Exempting it was
defensible on its face: the counter records a duty pdfcer *discharged*, not an
inference it made, so R8b rule 4 asks nothing of it directly. But it is
incremented in the same arm that has just written `annotations_without_ap`, so
the painted set is a **subset** of the appearance-less set — and the difference
answers a question no surface in this shell could previously ask: *how many
annotations on this page is the operator being shown nothing for?*

It became the tenth Render-notes finding the same afternoon. The defect
underneath it had **no symptom**: an appearance-less sticky note rendered as
clean paper, which is exactly what an unannotated drawing looks like, so Ken
could never have reported it. **A gate found a capability gap that no human
could have.**

**How to apply:** on any api-drift red — read the engine's doc comment for the
new item first, then grep the engine for every site that touches it. Two
questions decide it: *is this item positioned relative to one we already
consume?* and *does the pair answer a question our surfaces cannot?* Only after
both are "no" is an exemption the right answer, and it still gets its argument
written down. Related: [[a-backlog-row-is-a-record-not-evidence]],
[[a-gate-whose-evidence-you-write-is-blind-to-what-you-forget-to-write]],
[[update-engine-before-every-build]].

★ Two corollaries from the same afternoon.
1. **When an engine deliberately hands out the PARTS instead of the SUM, the
   sum belongs at the consumer with its argument written down.** The engine
   refused to fold these two: *"folding them together would make one of the two
   numbers a lie"* — the map is a fact about the FILE, the counter a fact about
   what the OPERATOR SAW. That refusal is what makes the consumer-side
   subtraction correct rather than a workaround.
2. **Use `saturating_sub`, not `-`, across a pinned engine boundary.** The
   subset relation is the *engine's* invariant, reaching here across a pin that
   moves several times a week. An arithmetic overflow in a status bar is not an
   acceptable way to learn it changed.
