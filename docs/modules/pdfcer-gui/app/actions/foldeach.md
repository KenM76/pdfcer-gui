# `pdfcer-gui/app/actions/foldeach`

## Item notes

### `fn within_undo_depth`

One undo step folds at most `MAX_UNDO_DEPTH` commands; past that the earliest
fall outside the step, so one Undo would take back only part of the gesture.
A gesture that knows its command count up front is refused before anything is
pushed. The refusal records `Declined::TooManyForOneUndo` (traced
`undo-depth-declined changes= limit=`), so the status line gives the reason;
the funnel's generic "That change was refused" appears only when no verb
recorded one. The engine's own `SelectionTooLargeForOneUndo` text is worded
for a cut and goes to the trace only. Driven by
`an_align_too_large_for_one_undo_is_refused`.

### `fn each`

`OnRefusal::Stop` is for verbs whose later items depend on the earlier ones
(a delete or paste in order); `Continue` is for independent items (a mixed
move), where one refusal should not strand the rest.

### `fn fold_or_say`

A visual line is however many show operators its producer wrote it as, so
one Delete is `pieces` engine commands. `coalesce_last` answers
`false` **only** when the undo stack was shorter than `pieces`; every change
is applied and only the grouping failed, so the contract's instruction is to
disclose and neither retry nor ignore.

The de-duplication is not cosmetic. A per-piece verb can say the same
sentence for every piece, and a line of nine pieces would otherwise report
the same fact nine times — which
reads as nine separate things having happened. First-seen order is kept, so
the sentences still arrive in the order the engine produced them.

