# `ui-verify/checks/structure_round_trip`

`hand_edits_compile_back_as_an_appended_update`: the QDF round trip on File ▸
Export, in one off-screen launch driven only through `ScriptedPointer`.

## Fixture

A copy of `fixtures/a1-titleblock.pdf`: one page, 45 objects, its embedded
fonts Flate-compressed, its page content showing `(Construction drawing)` in
two content objects. `PDFCER_DIAG_INVOKE=mode.review` so the Pages tab is
shown. `PDFCER_DIAG_SAVE_PATH` answers the export path and
`PDFCER_DIAG_OPEN_PATH` is a `;` queue: the export twice, then the
unrecorded copy.

## Steps

1. **Export.** `export-structure` traced; the fixture contains
   `/FlateDecode` and the export does not; the export carries
   `%PdfcerExportBase sha256:` and shows `(Construction drawing)` in plain text.
2. **Hand edit.** The check replaces the first occurrence with
   `(Hand-edited drawings)`, the same length, in place, and writes a second
   copy whose marker reads `%PdfcerExportBasX`, the same length.
3. **Compile.** `import-structure` must read `modified=1 added=0 removed=0`,
   `streams_matched` above 0, `base=matches`, and the `compile-hand-edits`
   receipt must not say the copy could not be checked.
4. **Save.** Ctrl+S through the scripted key: `save-in-place outcome=ok`, and
   the copy now begins with the fixture's bytes, is longer, and contains the
   new string.
5. **Stale base.** Pages ▸ Rotate right (`rotate-pages` traced), then Compile
   the same copy again: `import-structure-refused reason=stale-base`, and no
   second `import-structure` line.
6. **Unrecorded.** Compile the copy with the broken marker: an
   `import-structure` line with `modified=1` (the rotation undone) and
   `base=unrecorded`, and the receipt must carry `unrecorded_base`'s
   *could not check it*.

## Falsification

- The `recorded_base` comparison in `app::actions::structure::compile`
  made never true: step 5 fails, the stale copy is applied.
- The receipt's `unrecorded_base` sentence dropped: step 6 fails, the
  receipt does not say the copy could not be checked.
