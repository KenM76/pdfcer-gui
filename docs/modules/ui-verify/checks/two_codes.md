# `ui-verify/checks/two_codes`

`a_letter_drawn_two_ways_is_named`: text holding a letter its font draws two
ways still edits around it. Typing that letter is said as *drawn two ways*,
never as the font lacking it.

`EditSession::edit_text` keeps a held letter's own code when it re-encodes a
show operator, so an edit beside an `A` the font maps from two codes commits.
`RunRepertoire::ambiguous` names the letters the font draws two ways, so a
typed `A` is held as `two_ways` in the keystroke notice and the commit's
re-face sentence (`text::reface::set_in`) says so.

# What it drives

`fixtures/two-codes-one-letter.pdf` (ignores `--pdf`): `A` from (72, 600) and
`B` from (72, 540), 48 pt, one font whose character map gives `A` codes 1
and 2.

1. Caret into the `A` at (80, 615), End, type `B`, Escape, Ctrl+S. Owed: an
   `edit-text page=` line, no `edit-text-refused`, a `save-in-place
   outcome=ok`, and the appended revision showing codes 1 then 3, as
   `<00010003>` or `(\000\001\000\003)`, whichever spelling the engine writes.
2. Caret into the `B` at (80, 555), End, type `AB`, Escape. Owed:
   `text-edit-refused-keys ... two_ways=1`, then an `edit-text page=` line
   whose disclosures hold `draws ‘A’ two different ways` and neither
   `has no ‘A’` nor `cannot write ‘A’`.

# Falsified

- With both `two_ways` filters in `canvas::textedit::repertoire` answering
  false, step 2 fails on both findings (no `two_ways=1`; the commit says
  `cannot write ‘A’`). Step 1 stays green, since it measures the engine's carry.
- A needle that accepted only the hex spelling failed step 1 against an
  engine that saved the literal string, which is why both spellings are owed.

# Driven off-screen

Scripted pointer and keys, window at `-4200,-4200`; it runs under
`--no-input`.
