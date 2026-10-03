# `ui-verify/checks/reface_commit`

`a_key_the_font_lacks_is_set_in_the_nearest_face` — keys typed into a run whose
font lacks them go into the draft, commit set in the nearest face that has
them, and come out again with one Undo.

Fixture `fixtures/subset-font-floor.pdf` (one run, `ABC`, a subset font with
only those glyphs); off-desktop window, `ScriptedPointer` only,
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so it runs under `--no-input`.

## Steps

1. Click the run, `End`, type `qz`, `Escape` (commits the changed draft).
2. Re-click the run and read the caret's `len`, then `Escape` (unchanged, so
   abandoned).
3. `Ctrl+Z`, then read the caret's `len` again the same way.

## Judgement

- No `text-edit-key-refused` line.
- `text-edit-reface-planned` names `characters=U+0071,U+007A` and
  `route=engine`: the keystroke confirmed the face with the engine's
  fallback, not by the placeholder requirement.
- The last `text-edit-shaped` for the five-character draft has `shaped=1`:
  the draft was laid out in the run's place with the keys in the fallback
  face, not in the stand-in editor box.
- `text-edit-fallback` names `characters=U+0071,U+007A` (the engine's
  fallback took the single-operator run).
- `text-edit-reface-readback reads=1` and the reopened run is `len=5`.
- Exactly one `undo-applied`, and the run reopens at `len=3`.

## Falsification

Making `engine_takes` answer `false` fails on `route=placeholders`.
Dropping `with_fallback` from the preview (`shaped::refresh`) fails on
`shaped=0`. Dropping it from `try_commit` fails on the missing fallback line
(the placeholder route commits instead). Skipping
`coalesce_last` fails the one-Undo judgement.
