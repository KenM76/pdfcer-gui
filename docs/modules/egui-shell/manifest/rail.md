# `egui-shell/manifest/rail`

## Item notes

### `struct Rail`

A newtype rather than a bare `Vec` for [`super::Trailing`]'s reason — the
region gets a name a doc comment can hang an argument on, and a
present-but-empty rail is treated exactly as an absent one so that an
operator customization which removed the last group reclaims the strip
instead of leaving a 52 pt column of nothing.

### `fn is_empty`

True for a rail with no groups **and** for one whose every group is
empty: a caption with no entries under it is the placeholder R9
forbids, and a strip of nothing but captions is that defect repeated.
