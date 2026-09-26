# `ui-verify/input/observed`

## Item notes

### `struct ButtonHeld`

Nothing is stored in it; its whole job is its `Drop`. Holding the release
there rather than writing it at the end of the happy path is what makes the
verb safe to `?` through and safe to panic through.
