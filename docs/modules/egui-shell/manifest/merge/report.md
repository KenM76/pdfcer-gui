# `egui-shell/manifest/merge/report`

## Item notes

### `struct Skip`

A structured value rather than a message, deliberately. The shell has
no business deciding how another application words a note to its
operator, and an application that wants to offer "remove this stale
entry from your file" needs the id, not a sentence containing it.

[`std::fmt::Display`] is provided for diagnostics — a log line, a
failing test, `tools/ui-verify` — and is **not** operator-visible copy.

### `struct MergeReport`

Returned by value so the caller must deal with it: returning the rejects
alongside the result makes them a value that has to be handled rather
than a side effect that can be forgotten.
