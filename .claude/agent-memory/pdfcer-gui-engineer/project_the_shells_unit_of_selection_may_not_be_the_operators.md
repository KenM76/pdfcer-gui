---
name: the-shells-unit-of-selection-may-not-be-the-operators
description: A PDF path object routinely holds 1,000+ unrelated subpaths on Ken's drawings, so picking "the object" is not picking the thing under the cursor
metadata:
  type: project
---

A **PDF path object is not a drawn entity.** On Ken's own drawings one object
routinely holds hundreds or thousands of unrelated subpaths spread across the
whole sheet.

Measured 2026-09-03 with `pdfcer object-list` on
`D:/Dev/pdfTests/SW41177/SW41177.pdf` page 1: three objects carry **4,405**,
**4,972** and **6,681** anchors; the largest holds **1,194 subpaths across a
550 × 500 pt region** — half the sheet. `ncored-benchmark-cad-drawing.pdf` page 1
peaks at 4,660.

**Why:** this caused `OPERATOR_REQUESTS.md` O105. The radius/diameter tool
contributed *every anchor of the object under the cursor* to its circle fit, so
one click on a hole produced a circle hundreds of points across, and Ken reported
it as *"selecting a point sometimes makes a big circle"*. The number was already
recorded in this repository — decision 028's Objects-panel note — thirty lines
below the function that produced it.

**How to apply:** any feature whose unit is *"the thing the operator pointed at"*
must pick a **point** or a **subpath**, never an object, unless it has a stated
reason. `ObjectModelProvider::subpath_anchors`, `subpath_bounds_canvas` and
`subpath_count` exist for exactly this. Before shipping such a feature run
`pdfcer object-list <his drawing> --page 1` and look at the largest `anchors=`
and `subpaths=`; if the answer is in the thousands, an object-scoped pick is a
defect waiting for a report. Related:
[[feedback_the_canvas_is_the_primary_surface_never_a_panel]].
