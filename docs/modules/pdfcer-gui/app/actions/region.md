# `app::actions::region` — applying `Action::Region`

Every arm runs through `apply::vector_edit`, so each is one undo step with
the funnel's disclosure.

- **Redact** — `EditSession::add_redaction` with one quad. Nothing is
  removed until the redactions are applied.
- **RedactOutside** — `EditSession::mark_redaction_outside` once per page,
  all in one undo step. `EditError::RedactOutsideNothing` (the keep covers
  everything on the page) is counted, not failed; any other error aborts the
  step. The note reports marked, nothing-outside and skipped-for-size pages.
- **Delete** — the ordinary vector delete; its own notes follow the region's
  count line.
