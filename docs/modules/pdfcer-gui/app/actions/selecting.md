# `pdfcer-gui/app/actions/selecting`

## Item notes

### `fn apply_action`

It takes `&mut OpenDoc` and nothing else — no `PdfcerApp`, no
`&mut Vec<Action>`, no preferences. That narrow signature is the module's
header made mechanical: an action that could reach anything else would be
one that could change something, and none of these can.
