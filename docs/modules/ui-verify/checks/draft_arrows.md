# `ui-verify/checks/draft_arrows`

`arrows_in_a_text_draft_stay_with_the_caret`, O288 item 8, on
`fixtures/four-pages-unrotated.pdf` (four 612×792 pages).

## Drive

1. Edit mode, *Add text* armed (`PDFCER_DIAG_INVOKE`).
2. Ctrl+Minus three times, so more than one page is on screen and a focus
   walk has a neighbour page to reach.
3. Click (200, 700) on page 1, type `abc`.
4. Down, Down, Up, Up, Left, Left, Right, Right.
5. Type ` X`, press Enter.

## Oracle

- A `keyboard-focus` line after the click — the witness that this build
  traces focus at all, so silence afterwards means something. Missing → error.
- No `egui-shell-diag ribbon-command-invoked` after the arrows (read through
  `shell_trace`; `session.trace()` holds only `pdfcer-diag` lines). Checked
  before the Enter step's own result, because a command that opens a modal
  stops frames and the step goes unacknowledged.
- No `keyboard-focus` line after the arrows.
- The `canvas page=` index unchanged.
- The draft grew by the typed characters plus Enter's newline.

## Falsified

`hold_arrows` not called → FAIL: *Enter … also ran a ribbon command:
`ribbon-command-invoked id=file.save_as`*. Before Enter was part of the drive
the same build failed on 7 focus moves.

## What it does not prove

That Up/Down cannot flip a page by a route other than a focus walk; the page
keys proper are covered by `textedit::composing`. The arrows inside a form
field's text box (egui's own `TextEdit` locks them) are not driven.
`arrow_keys_walk_between_blocks` and `tab_moves_between_form_fields` need OS
input and were not re-run.
