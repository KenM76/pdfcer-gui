# `pdfcer-gui/find/bar/unsearchable_tests`

## Item notes

### `fn a_result_that_no_longer_describes_the_bar_discloses_nothing`

Three ways a result stops describing what the bar is showing, and all
three must silence the note: the operator edits the query, the operator
changes an option, or the DOCUMENT is edited (a new epoch). The last is
the one worth having a test for — an edit does not touch the query, so a
naive implementation keeps a stale sentence on screen indefinitely while
the bar beside it has already gone blank.
