# `ui-verify/checks/markup_place`

# `a_markup_is_nudged_duplicated_restacked_and_typed_to_size`

The fixture is `fixtures/annots-with-everything.pdf`, built by
`tools/gen-annots-with-everything-fixture.py`. It is one 595 × 842 page whose
`/Annots` is `[5 0 R 6 0 R 8 0 R]`: a Square at `[120 560 320 700]`, a Text
note and a FreeText box.

The check is driven with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`, so
the Properties tab is in front. `PDFCER_DIAG_SAVE_PATH` answers the save
picker. Duplicate does not touch the clipboard, so no clipboard folder is
needed.

1. Click (220, 630) and require `annot-select subtype=Square`.
2. Press ArrowRight. Require `annot-nudge` and `move-annotation-applied`.
   The moved object's number is the original.
3. Press `Ctrl+D`. Require `selection-duplicate` and `paste-objects-applied
   annots=1`. The duplicate lands 10 pt right and 10 pt down
   (`PASTE_OFFSET_PT`), on top.
4. Click (220, 630) again. The point is inside both squares, so the click
   selects the duplicate, which is on top.
5. Press Markup ▸ Arrange ▸ Send to back. Require
   `arrange-annotation-applied`.
6. Type 150 into `properties.annotgeometry.width` and press its Apply.
   Require `resize-annotation-applied` on an object other than the original.
7. Press File ▸ Save a copy and read the saved bytes with
   `checks::savedannots`. Require exactly the original and one other
   Square. The other Square must be first in `/Annots`. The original must
   be at `[121 560 321 700]`. The duplicate must be at `[131 550 281 690]`.

The original and the duplicate are told apart by object number, not by
their position in `/Annots`. Position is what step 5 changes, so a check
keyed on it blames the wrong square when the re-stack fails.

## Falsified

- With `STEP_PT` set to 0, the check fails at step 7: the original is still
  at `[120 560 320 700]`.
- With Send to back mapped to Backward, the check fails at step 7: the
  duplicate is third in `/Annots`. Mapped to Front, it fails at step 5,
  because the engine declines with `already-there`.
