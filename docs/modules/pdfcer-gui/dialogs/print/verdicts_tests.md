# `pdfcer-gui/dialogs/print/verdicts_tests`

## Item notes

### `fn placed`

The offsets and scale are shared, which matters: two plans built from this
helper with the same `clipped` value are `PartialEq`, and the verdict cache
is keyed partly on exactly that. A test that wanted two *different*
placements has to change one of these numbers on purpose — see
[`a_verdict_does_not_survive_a_change_of_placement`].

### `fn job_of`

The page indices are deliberately not `0, 1, 2`. `PagePlan::index` names
a **document** page and the plan list is the job's *sequence*; a cache that
confused the two would pass every test built on a whole-document forward
job and fail on the first custom range an operator typed.

### `fn a_job_nobody_has_previewed_reports_the_plain_geometric_count`

Every clipped sheet is unexamined, nothing is subtracted, and the claim is
[`ClipClaim::Geometric`] — which produces exactly the sentence the button
carried before O113. That is deliberate rather than incidental: with no
evidence at all there is nothing to correct the count *with*, and hedging a
statement that is exactly true would soften pdfcer's whole divergence from
Acrobat (which clips silently) in the one state where there is nothing to
soften it with.

The variant is asserted, not just the number. `Geometric(2)` and
`AtMost(2)` put different sentences on the button and would be
indistinguishable if only the count were checked.

### `fn a_single_sheet_examined_and_found_blank_removes_the_warning_entirely`

A 1:1 CAD drawing whose overhang is empty paper. The placement reports a
clip — that is a true geometric fact and `Job::clipped()` still says 1 —
but the one sheet has been looked at and nothing is printed out there, so
the count is zero and the button reads plain **Print**.

### `fn blank_inked_and_unexamined_sheets_produce_a_ceiling`

Five sheets clipped: one examined and blank, one examined and inked, three
never looked at. The count is `5 − 1 = 4`, which is `known_inked (1) +
unexamined (3)`, and it is a **ceiling**: the true figure is somewhere in
`1..=4`. So the claim is [`ClipClaim::AtMost`] and the sentence hedges.

### `fn examining_every_clipped_sheet_makes_the_count_a_measurement`

Three clipped, two found blank, one found inked. Nothing is left
unresolved, so the number is not a bound — it is the number of sheets that
really will lose something, and the sentence drops the hedge.

### `fn a_sheet_that_would_not_render_stays_counted`

[`Overhang::Unknown`] is what `preview::lost_regions` returns when the page
would not render and the whole band was hatched as the honest fallback. It
must leave the sheet in the count — a failed render is not allowed to
switch a warning off, which is the same rule the hatch itself follows.

### `fn both_copies_of_one_examined_sheet_are_subtracted`

An uncollated two-copy job sends the same document page twice. The two
plans carry identical placements by construction, so the ink test would
return the identical answer for both: the fact is about *this page under
this placement*, not about a position in the send order.

### `fn a_verdict_does_not_survive_a_change_of_rendering_settings`

The verdict is a claim about pixels, and this is the field that decides
them. `PreviewKey` carries the whole `Settings` for the reason its own docs
give; the verdict cache carries it for the stronger reason that a stale
verdict *removes* a warning.

### `fn a_verdict_does_not_survive_a_change_of_annotation_scope`

Turning markup on can put a comment out in the border — which is the
difference between a blank overhang and a lost annotation, and is exactly
the case where a stale "blank" would be most expensive.

### `fn a_verdict_does_not_survive_a_change_of_printable_area`

A different printer, a different paper, or an orientation that re-plans the
geometry moves the boundary the band is measured from. Same pixels,
different question.

### `fn a_verdict_does_not_survive_a_change_of_placement`

`PreviewKey` deliberately omits the placement: it scales the drawn
rectangle and changes not one pixel of the raster, so the texture cache is
right to keep its bitmap across a switch from Fit to 100 %. The *verdict*
is not: the band moves, and a page whose overhang was empty paper at one
scale can have a title block in it at another.

This is why the entry key is strictly stronger than `PreviewKey` rather
than equal to it.

### `fn a_verdict_does_not_survive_the_page_being_resized`

The band is computed as a fraction of the page, so the same placement over
a page of a different size is a different band. This is the safe direction
of an inherited hole: `PreviewKey` carries no edit generation, so a page
resized by an unsaved edit does not invalidate the *texture* — but it does
invalidate the verdict, which drops the sheet back to unexamined and puts
the number **up**.

### `fn a_verdict_is_filed_under_the_document_page_and_not_the_send_position`

The job here sends document page 7 first. If the cache filed the verdict
under the loop position instead, this claim would come back uncorrected —
and would look exactly like a preview that had never been opened.

### `fn the_claim_decision_table_holds_in_every_arm`

The bucket counts go in and the claim comes out, with no `Job` in the way.
This is the one place the *rule* is asserted rather than an instance of it,
and the ordering of the arms is what it pins: `Measured` is tested before
`Geometric` so that a job whose every clipped sheet was examined and found
inked reports the stronger measured sentence, even though the two counts
coincide there.

### `fn each_claim_state_is_distinguishable_in_the_trace`

The trace is the only headless evidence of which state a frame was in:
`Geometric(2)` and `AtMost(2)` are the same number and a different truth,
and a driven check reading only the count could not tell a working
correction from a cache that silently never matched.
