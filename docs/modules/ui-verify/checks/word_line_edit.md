# `ui-verify/checks/word_line_edit`

`a_line_written_in_pieces_edits` — typing into a line a word processor wrote
as several text objects previews in the line's own face and commits.

The fixture is `fixtures/word-fragmented-lines.pdf`: each line is one marked
content sequence holding several `q … BT … ET Q` fragments, one of them a
trailing space-only object. The engine matches a whole-line request across
those objects when the line is in one font, and previews only the part it
rewrites; `canvas::textedit::splice` places that part back into the line. A
line in two fonts refuses the whole-line request, and the narrowed request
(`canvas::textedit::tier`) reaches only the piece that changed.

The window is placed off the desktop and driven only through
`ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the
Edit Text tool, so the check runs under `--no-input`.

## Steps

For the one-font first line (`tier=Line`), then the two-font third line
(`tier=Narrowed`):

1. **Open a caret and type.** A click on the line, `End`, then `_`. The last
   `text-edit-shaped` line must read `shaped=1` and the tier: the preview is
   the engine's own layout, not the stand-in face.
2. **Commit.** `Escape` commits. The edit's `edit-text-left-edge` line must
   read `committed=yes`; on the narrowed tier its `edit-text-narrowed
   for=commit` line must read `landed=1`, and on the line tier there is none.

## Falsification

With the engine-trim splice removed from `shaped::shape_any`, the first line's
preview falls back unshaped (`reason=unpaired`) and the check fails at step 1.
With `Plan::narrowed` left `None`, the third line's edit is refused and the
check fails at step 1 on `tier=Narrowed`.
