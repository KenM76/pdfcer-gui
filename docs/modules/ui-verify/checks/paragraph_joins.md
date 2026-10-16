# `ui-verify/checks/paragraph_joins`

`edge_keys_open_the_paragraph`, on `fixtures/paragraph.pdf` (612×792), and
`enter_in_a_recognised_line_is_declined`, on `fixtures/ocr-layer.pdf`.

## Drive

`edge_keys_open_the_paragraph`, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.paragraphs,edit.text`: Paragraphs off, so a
click opens one line and only the key can open the paragraph.

1. Click (120, 668), line 2. Home, Backspace.
2. Ctrl+Enter.
3. Click (120, 668) again, End, paste `x` newline `y`, Escape.

`enter_in_a_recognised_line_is_declined`, with
`mode.edit,view.ocr_layer,edit.text`: click (85, 704) on `SITE`, End, Enter,
Escape.

## Oracle

- Step 1 traces `text-edit-promoted`, and the next `text-edit-typing` length
  is one less than its `len=`: the line break before line 2 removed.
- Step 2 traces `edit-block-text-applied`.
- Step 3 traces `text-edit-promoted` and `text-edit-paste … joined=0`.
- The recognised Enter traces `text-edit-enter-declined … reason=recognised-word`
  and no `text-edit-promoted`.

## Falsified

- `open_paragraph` returning at once → FAIL at step 1: no `text-edit-promoted`.
- Only the paste's call removed → FAIL at step 3: `joined=1`.
- `promote.rs` without the recognised guard → FAIL: Enter traced
  `text-edit-promoted … source=inferred` on the recognised paragraph.

## What it does not prove

Delete at a line's end (the same call, on the other edge), and that the
written paragraph's text reads back joined.
