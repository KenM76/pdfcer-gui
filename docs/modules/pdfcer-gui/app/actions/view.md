# `pdfcer-gui/app/actions/view`

## Item notes

### `fn apply`

The caller has already matched the variant set; the `_` arm is unreachable
and says so rather than silently doing nothing — the same rule
`app::dispatch::format` states for its own guarded fall-through.
