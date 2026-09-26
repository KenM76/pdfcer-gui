# `pdfcer-gui/app/actions/forms/paste`

## Item notes

### `fn widget_index_after_paste`

Re-read from the document rather than derived from the outcome's
`widget_ids`, because the two address spaces differ: the outcome names
`ObjId`s and `SelectedField` wants an **index within the field**. Reading
the field back is the only thing that knows both.

Zero when the field cannot be found, which is unreachable on the success
path and is a defensible index rather than a panic if it ever is not.
