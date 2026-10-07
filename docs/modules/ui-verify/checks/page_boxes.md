# `document_properties_say_which_page_boxes_were_not_used_as_written`

**Defect it guards.** ISO 32000-2 §14.11.2.1 reads a crop, bleed, trim or art
box that extends past the media box as its intersection with it, and the
engine falls back to the Table 30 default for one whose intersection has no
area or that is not a rectangle. The page shown, printed and exported is then
not the one the file states, and the file is not rewritten. The engine records
the outcome per page (`Page::crop_box_resolution` and its three siblings,
`BoxResolution::{Clipped, Unusable}`); unread, nothing tells the operator his
crop box was cut.

**Fixture.** `fixtures/page-boxes.pdf`, built by `page-boxes.PROVENANCE.py`:
three 200 × 100 pages. Page 1 writes no optional box (the control). Page 2
writes `/CropBox [-20 -20 220 120]`, past the sheet on every side. Page 3
writes `/TrimBox [300 300 400 400]`, wholly off the sheet.

**Steps.** Launch on the fixture with Document properties open. The
`page-boxes` line must say `pages=3 crop_clipped=2 trim_unusable=3` and carry
no other box key; the `docprops.page-boxes` region must be published as
visible and end inside the right dock.

**What it does not prove.** The wording of each line, read in
`text::panels::docprops`; and that a bleed or art box is said, which goes
through the same loop as the trim box.
