---
name: feedback_defined_not_on_page
description: A byte-scan of an appended save must read the page's /Annots; a deleted annotation's original body is still defined in the file
metadata:
  type: feedback
---

A saved-file oracle that counts "the last definition of each object" counts deleted objects. An appended (incremental) save writes the delete as an xref free entry plus a page with a shorter `/Annots`; the original `n 0 obj` body stays in the file's first section. The link_quads check read 2 links after a delete that had worked, because object 5's original body survived.

**Why:** being defined is not being on the page. The check was about to blame the delete for a defect in its own scan.

**How to apply:** for any check that byte-scans a saved PDF for annotations or page objects, filter to the object numbers that the last definition of a page lists (`/Annots`, `/Kids`). The unit test should also plant a stale body. Related: [[feedback_driven_failure_claim]], [[feedback_oracle_needs]].
