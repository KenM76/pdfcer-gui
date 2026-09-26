# `pdfcer-gui/app/actions/chrome`

## Item notes

### `enum ViewChrome`

An enum rather than one action variant per toggle — see that variant's own
docs — and it lives here rather than in `canvas` because it is the *operand
of an
action*, and `shell::commands` (which maps ids to it) must not have to
reach into the canvas to name one.

### `const ALL`

Iterated by the tests that assert each has a command and each command
has a `selected:` condition — the same both-directions check
`PageDisplay::ALL` exists for, and for the same reason: a toggle added
to the enum with no registration would draw nothing, and nothing else
in the suite would notice.

### `fn write`

The pair with [`Self::read`], so the enum's mapping onto
[`crate::viewer::ViewState`]'s own fields is stated exactly twice, in
adjacent functions, instead of once per consumer.
