# `pdfcer-gui/app/save/outcome`

## Item notes

### `fn fmt`

`check-ui-strings.sh`'s exclusion 3 permits a `Display` impl to carry
text that is not in the catalog **because it is diagnostic**, and states
in the same breath that this "is not permission to route UI text through
an error type". Nothing here reaches an operator: the bar's sentence is
`crate::text::status::save_copy_failed`.
