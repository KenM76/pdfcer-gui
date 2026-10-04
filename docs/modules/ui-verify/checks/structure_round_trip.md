# `ui-verify/checks/structure_round_trip`

`hand_edits_compile_back_as_an_appended_update`: the QDF round trip on File ▸
Export, in one off-screen launch driven only through `ScriptedPointer`.

## Fixture

A copy of `fixtures/a1-titleblock.pdf`: one page, 45 objects, its embedded
fonts Flate-compressed, its page content showing `(Construction drawing)` in
two content objects. `PDFCER_DIAG_INVOKE=mode.review` so the Pages tab is
shown. `PDFCER_DIAG_SAVE_PATH` is a `;` queue (export, compiled, stale) and
`PDFCER_DIAG_OPEN_PATH` answers the export path twice.

## Steps

1. **Export.** `export-structure` traced; the fixture contains
   `/FlateDecode` and the export does not; the export shows
   `(Construction drawing)` in plain text.
2. **Hand edit.** The check replaces the first occurrence with
   `(Hand-edited drawings)`, the same length, in place.
3. **Compile.** `import-structure` must read `modified=1 added=0 removed=0`
   with `streams_matched` above 0, and the save picker's traced title must
   carry those counts. The compiled file must begin with the fixture's bytes
   and contain the new string.
4. **Stale base.** Pages ▸ Rotate right (`rotate-pages` traced), then Compile
   the same copy again: `import-structure-refused reason=stale-base`, and the
   third save path is not written.

## Falsification

- Disabling the memo comparison in `app::actions::structure::compile`: step 4
  fails, the stale copy is written.
- `writer::save_full` in place of `save_incremental`: step 3 fails, the
  compiled copy no longer begins with the original's bytes.
