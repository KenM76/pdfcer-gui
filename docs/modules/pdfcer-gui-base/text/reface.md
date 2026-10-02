# `text::reface` — sentences for a mixed-face commit

- `set_in(chars, face)` — the disclosure after a granted commit: the line's
  font has none of these characters, so they were set in `face`, the nearest
  font that has them.
- `undo_split(steps)` — said when the engine would not fold the gesture into
  one undo entry, so Undo takes `steps` presses.
- `stopped(why, face, detail)` — a commit that was not made: which step
  stopped (`Line`, `Face`, `Chars`), the engine's reason, and "Nothing was
  changed."
