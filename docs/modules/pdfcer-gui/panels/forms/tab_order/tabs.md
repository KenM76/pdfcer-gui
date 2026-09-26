# `pdfcer-gui/panels/forms/tab_order/tabs`

## Item notes

### `fn tabs_name`

`Dict::get` collapses a null-valued entry to `None` (§7.3.7/§7.3.9), so
`/Tabs null` reads as absent without a second check — which is right: a null
value is the standard's way of saying the key is not there.

A `/Tabs` whose value is not a name (a string, a number, an array) reads as
absent too. That is a malformation, and the honest reading of a malformed
entry is that the file has not named a tab order — inventing one from a
string that happened to say "R" would be pdfcer deciding what the file meant.
