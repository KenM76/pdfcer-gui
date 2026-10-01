# `ui-verify/checks/word_line_edit`

`a_line_written_in_pieces_edits` — typing into a line a word processor wrote
as several text objects previews in the line's own face and commits.

The fixture is `fixtures/word-fragmented-lines.pdf`: each line is one marked
content sequence holding several `q … BT … ET Q` fragments, one of them a
trailing space-only object. The request for the whole line matches nothing on
such a line, so both the preview and the commit depend on the narrowed request
(`canvas::textedit::tier`).

The window is placed off the desktop and driven only through
`ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the
Edit Text tool, so the check runs under `--no-input`.

## Steps

1. **Open a caret and type.** A click inside `Required` on the first line,
   `End`, then `_`. The last `text-edit-shaped` line must read `shaped=1` and
   `tier=Narrowed`: the preview is the engine's own layout of the touched
   operator, not the stand-in face.
2. **Commit.** `Escape` commits. The last `edit-text-narrowed for=commit` line
   must read `landed=1`, and the last `edit-text-left-edge` line
   `committed=yes`.

## Falsification

With the narrowed tier disabled (`Plan::narrowed` left `None`), the preview
falls back unshaped and the check fails at step 1; the commit is refused as a
split.
