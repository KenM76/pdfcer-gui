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
- `text-edit-reface-planned` names `characters=U+0071,U+007A`.
- `text-edit-reface-committed` is present (the commit took the re-faced path).
- `text-edit-reface-readback reads=1` and the reopened run is `len=5`.
- Exactly one `undo-applied`, and the run reopens at `len=3`.

## Falsification

`reface::take` returning `None` fails on the missing committed line. Skipping
`coalesce_last` fails the one-Undo judgement.
