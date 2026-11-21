# `text::reface` — sentences for a mixed-face commit

- `set_in(chars, two_ways, face)` — the disclosure after a granted commit: the
  line's font cannot write these characters (or draws some two ways), so they
  were set in `face`, the nearest font that has them.
- `not_embedded(face)` — the fallback face is a standard font the file does
  not embed, so another reader may substitute it.
- `undo_split(steps)` — said when the engine would not fold a gesture of
  several steps into one undo entry, so Undo takes `steps` presses (raised by
  `app::actions::replace`).
