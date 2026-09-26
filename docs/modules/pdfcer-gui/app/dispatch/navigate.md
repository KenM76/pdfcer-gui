# `pdfcer-gui/app/dispatch/navigate`

## Item notes

### `fn decline`

Both, always, because they answer different people. The trace is read by
somebody debugging a machine they are not sitting at; the status sentence is
read by the operator who just pressed a key and saw nothing happen. Either
one alone has been a defect in this project — a silent decline, and a
sentence with no way to tell which decline it was.

### `fn handles`

A `matches!` over literals rather than a prefix test, for the reason every
`handles` in this directory gives: a prefix would silently claim the next
`view.*` command somebody adds, and the failure would be a command that
reaches this file's `match` and falls out of it doing nothing.

### `fn dispatch`

Note the signature: **no `actions`**. Every control here changes how the
next gesture is READ and none of them changes the document, so there is
nothing to push. An arm that needed one would be a control that does not
belong in this row.
