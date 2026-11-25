# `ui-verify/checks/info_entries`

`a_custom_info_entry_reaches_info_and_xmp`, on `fixtures/drawing-sheets.pdf`
(four 420×297 sheets; `/Info` holds Title and Author only, and the catalog
carries an XMP packet with `dc:title`), launched with
`mode.review,file.document_properties`.

1. The panel's opening `info-entries-now` must not already list `DrawingNo`
   — otherwise the add proves nothing.
2. *Other entries* ▸ name `DrawingNo`, value `S-200`, **Add entry**.
3. `Producer` / `me`, **Add entry** — a name the engine reserves.
4. File ▸ Save a copy to `PDFCER_DIAG_SAVE_PATH`.
5. A second process opens the saved copy with the same panel.

## PASS

| step | must hold |
|---|---|
| 2 | `info-entry-set key=DrawingNo info_changed=1 xmp=updated:…`, then `set-info-entry` (the funnel), then `info-entries-now` listing `DrawingNo:5` |
| 3 | `set-info-entry-refused`, and no `set-info-entry` after the press |
| 4 | the saved bytes hold `/DrawingNo` followed by `(S-200)`, and `<pdfx:DrawingNo>S-200</pdfx:DrawingNo>` |
| 5 | the reopened panel's `info-entries-now` lists `DrawingNo:5` |

The trace carries keys and value **lengths**, never values: a value is the
operator's document metadata. The saved bytes are the value's witness.

## Why this fixture

A file with no XMP packet would make step 2 trace `xmp=no-packet` and step 4's
XMP half impossible, so the fixture carries one. The value is ASCII so the
engine writes a literal string the byte scan can find; a non-ASCII value is
written as UTF-16 and would need decoding to match.
