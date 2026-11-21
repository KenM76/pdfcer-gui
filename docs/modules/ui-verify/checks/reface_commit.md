# `ui-verify/checks/reface_commit`

Keys typed into a run whose font lacks them go into the draft, commit set in
the nearest face that has them through the engine's fallback, and come out
again with one Undo. Two checks, one drive:

| Check | Fixture | Typing | Committed | `operators=` |
|---|---|---|---|---|
| `a_key_the_font_lacks_is_set_in_the_nearest_face` | `subset-font-floor.pdf` (`ABC`, one `Tj`) | `End`, `qz` | `ABCqz` | `1` |
| `a_change_across_show_operators_is_set_in_the_nearest_face` | `subset-glyph-per-show.pdf` (`ABC`, one `Tj` per glyph in one `BT … ET`) | `Home`, `q`, `End`, `z` | `qABCz` | `3` |

Both fixtures use the same embedded subset (`A`, `B`, `C` only) at the same
place. Off-desktop window, `ScriptedPointer` only,
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so they run under `--no-input`.

## Steps

1. Click the run, type as in the table, `Escape` (commits the changed draft).
2. Re-click the run and read the caret's `len`, then `Escape` (unchanged, so
   abandoned).
3. `Ctrl+Z`, then read the caret's `len` again the same way.

## Judgement

- No `text-edit-key-refused` line.
- `text-edit-reface-planned` names `characters=U+0071,U+007A` and
  `route=engine`: the keystroke confirmed the face with the engine's
  fallback.
- The last `text-edit-shaped` for the committed draft's length, before the
  commit, has `shaped=1`: the draft was laid out in the run's place with the
  keys in the fallback face, not in the stand-in editor box.
- `text-edit-fallback` names `characters=U+0071,U+007A` and the case's
  `operators=`: the engine set both keys in the fallback face, across three
  show operators in the spanning case.
- `text-edit-reface-readback reads=1` and the reopened run has the committed
  length.
- Exactly one `undo-applied`, and the run reopens at `len=3`.

## Falsification

Dropping `with_fallback` from `app::actions::reface::try_commit` fails both
checks on the missing `text-edit-fallback` line.
