# `ui-verify/checks/run_width`

`run_width` — **select one line of text, type a width under Properties › Fit
to width, and the file changes.**

## What it guards

The Fit to width field (`panels::properties::runwidth`) raises
`StyleChange::RunWidth` when it loses focus with a changed value. The apply arm
calls `EditSession::set_text_run_width` and traces
`text-run-width … width=… detail=applied|<refusal>`. A field that draws but
commits nothing looks identical in every unit test.

## How it drives

1. It opens `fixtures/inherited-runs.pdf` (shared with `move_line_of_text`)
   with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.
2. It arms the Points tool (`A`) and clicks the second line at (128, 664). That
   line is one run that states its own position. It requires the ladder at
   `Part`.
3. It finds `properties.text.run-width`, scrolling the Properties body up to six
   notches.
4. It clicks the field, presses Ctrl+A, types `150` and presses Enter.

It passes when the apply arm's line (the `text-run-width` line carrying
`detail=`) reads `detail=applied` and `width=150`. The funnel's own success line
shares that head, but it has no `detail=`.

## Falsification

With the field's commit threshold raised so it never fires, the check fails:
no `text-run-width` line follows.
