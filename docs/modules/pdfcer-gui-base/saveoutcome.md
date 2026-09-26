# `pdfcer-gui-base/saveoutcome`

## Item notes

### `fn fmt`

`check-ui-strings.sh`'s exclusion 3 permits a `Display` impl to carry
text that is not in the catalog **because it is diagnostic**, and states
in the same breath that this "is not permission to route UI text through
an error type". Nothing here reaches an operator: the bar's sentence is
`crate::text::status::save_copy_failed`.

### `enum Written`

It exists for the staged-redaction route, and it is an enum rather
than `(SaveReport, Option<RedactionReport>)` because the two writers do not
both run: `EditSession::save_applying_redaction` produces no
[`SaveReport`] at all — it returns bytes and a
[`pdfcer_core::redact::RedactionReport`] — so a struct with both would have
to carry a fabricated one, and every field of a fabricated `SaveReport`
(`bytes_appended`, `byte_identical`, `promoted`) is a claim about a save
that did not happen in that shape.

The trace lines differ for the same reason and that is the point. A reader
of a trace must be able to tell a save that appended a revision from one
that rewrote the whole document with content removed, and the two events
have no fields in common worth pretending they share.

### `enum SaveError`

Two variants rather than a `String`, on `crate::app::lifecycle`'s rule that
a branch is made on **structured error data, never by inspecting a message**
— and because the two are genuinely different facts about different
subsystems. Neither is worded to the operator separately today (the bar
carries one sentence for both; see §5), and keeping them apart is what makes
wording them separately a copy decision later rather than a re-plumbing.
