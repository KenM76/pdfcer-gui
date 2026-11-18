# `ui-verify/checks/insert_ce_dimensions`

`inserted_pages_say_what_came_of_their_ce_dimensions`: in Edit mode on a copy
of the corpus's `four-pages.pdf`, a copy of `fixtures/dimension-scaled.pdf`
(one page, one ce dimension in the default group, calibrated) is dropped on
the page and Insert is chosen.

PASS needs the funnel's `insert-pages` line to carry `1 ce dimension came
across still editable` (`text::cedimcopy::copied` over
`InsertOutcome::ce_dimensions_carried`) and not `came across as plain lines`.

## Falsified

- The build before the receipt said anything of ce dimensions → FAIL: its
  `insert-pages` line reads only `Inserted 1 page after page 1.`

## What it does not prove

- The group half of the sentence. This fixture's group is the source's
  default and the destination has none; the engine adopts it as the
  destination's default and counts it in no `ce_dimension_groups_*` field
  (request G298). Carried, joined and renamed groups are unit-tested in
  `text::cedimcopy`.
- Extract, split, Merge files and Pages ▸ Insert…; they call the same
  sentence through `CeDimCopy`'s `From` impls, and `extract_ce_dimensions`
  drives extract's trace fields.
- The plain-lines half (a newer sidecar); no fixture carries one.
