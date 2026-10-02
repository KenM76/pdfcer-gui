# `ui-verify/checks/selection_gestures`

`the_text_tool_selects_as_a_word_processor_does` — with the text tool on a
page's own text, a double click selects the word, a triple click the line,
Shift+Down on the last line selects to its end without leaving it, a drag that
starts on text selects instead of drawing a new text box, and a click on
rotated text puts the caret where it landed.

The window is off the desktop and driven by `ScriptedPointer` only, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so the check runs under `--no-input`.
The oracle is the `text-select` trace line (`from=F to=T n=N`, or
`none caret=C`), the draft's selection in characters.

## First launch: `fixtures/word-fragmented-lines.pdf`

Line 1 is `Date Premises Required____ `, 27 characters at y=700 from x=72;
line 2 is at y=670.

1. `dclick` inside `Date` (PDF 80, 704). Expect `from=0 to=4`.
2. `tclick` at the same point. Expect `from=0 to=27`.
3. `Escape`; click there, `Home`, `Shift+ArrowDown`. Expect `from=0 to=27`,
   and no second `text-edit-caret` line since the click (the draft stayed on
   its run).
4. `Escape`; drag from inside `Date` to PDF (110, 674) on line 2. Expect no new
   `text-box-open` line, and `to=27`: a sweep that leaves the line on the
   next line's side takes it to its end (`hit::Layout::swept_index_at`).

## Second launch: `fixtures/rotated-text.pdf`

Run 1 is `UPWARD`, Helvetica 12 at `0 1 -1 0 100 300 Tm`. `U` and `P` advance
16.668 pt together, so PDF (97, 318.7) is two points into `W` on the
ascender side. Expect `none caret=2`.

## Falsification

Each fix, reverted alone, turns the check red on its own step:

- `keys::pointer` deciding a multi-click by `press_origin` (cleared on the
  release frame egui reports the click on): steps 1 and 2.
- `edits::vertical` without the Shift extension past the last line: step 3.
- `interact`'s `TextBox` arm without `sweep::over_text`: step 4 opens a box.
- `keys::pointer` sweeping with `index_at` alone: step 4 stops at `to=6`.
- `place::caret_index_at` comparing x alone: the rotated click lands at 0.
