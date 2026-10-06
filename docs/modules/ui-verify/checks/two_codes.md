# `ui-verify/checks/two_codes`

`a_letter_drawn_two_ways_is_named` — an edit to text holding a letter its
font draws two ways is refused for that letter and said as such, and text in
the same font without it still edits.

The engine rewrites a pinned show operator whole, so typing `B` after an `A`
the font maps from two codes re-encodes the `A` too and is refused with
`RInvTrigger::Ambiguous` naming `A`. The shell tells that apart from the
operator having typed the letter by counting it in the draft
(`RefusedCharacter::added`): fewer or the same in the replacement as in the
original means the text already held it.

# What it drives

`fixtures/two-codes-one-letter.pdf` (ignores `--pdf`): `A` from (72, 600) and
`B` from (72, 540), 48 pt, one font whose character map gives `A` codes 1
and 2.

1. Caret into the `A` at (80, 615), End, type `B`, Escape. Owed:
   `edit-text-classified ... character='A' ... said=TextHoldsTwoGlyphsFor`, no
   `edit-text page=` line, and the last `refused-char` line reads
   `character='A'` with `two_ways=1` (the Properties offer says the font draws
   the letter two ways rather than that it lacks it).
2. Caret into the `B` at (80, 555), End, type `AB`, Escape. Owed: an
   `edit-text page=` line whose disclosures hold `pdfcer cannot write ‘A’`
   and not `has no ‘A’`. The engine sets the `A` in Helvetica; the shell's
   sentence must not claim the font lacks a letter it has twice.

# Falsified

- With `refused_char_kind` returning `TwoGlyphsFor` for a held letter, step 1
  fails (`said=FontHasTwoGlyphsFor`, which tells him he typed an `A`); the
  other two findings stay silent.
- Against the build before this check, all three findings fire: the held
  letter is said as typed, the offer line has no `two_ways`, and the commit
  says `This text's font has no ‘A’`.

# Driven off-screen

Scripted pointer and keys, window at `-4200,-4200`; it runs under
`--no-input`.
