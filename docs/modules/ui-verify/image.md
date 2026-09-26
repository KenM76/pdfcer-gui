# `ui-verify/image`

## Item notes

### `fn temp_path`

Uniqueness comes from the process id plus a monotonic counter, which is
enough for a harness that never runs two conversions concurrently and does
not warrant a uuid dependency.
