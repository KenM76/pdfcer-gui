# `ctrl_b_bolds_the_word_at_the_caret`, `the_ribbon_aligns_the_paragraph_at_the_caret`

The word-processor character and paragraph controls inside a text edit
(O273, O271). Both run on `fixtures/paragraph.pdf`: six lines of Helvetica
12 pt from (72, 700), 16 pt apart. Both launch with `mode.edit,edit.text`
invoked, off the desktop, through the scripted pointer.

## Bold

A click at (112, 703), inside `drawing`, opens a caret. The check raises the
Format tab so Bold is drawn, then presses Ctrl+B.

It asserts:

- no `text-style-declined` line;
- a `text-span-style-applied` line with `applied` not `0` and
  `reselected=true`: the word alone was restyled and stays selected;
- the shell's `ribbon-item-selected id=format.bold selected=1`, read under the
  shell's own prefix (`egui-shell-diag`), not the application's.

The ribbon draws only the active tab's items, so without the tab click the
pressed event is never emitted and the check fails on a working build.

## Align

A click at (120, 671), in the third line, opens a caret. The check clicks the
Format tab, expands the Font group if it is collapsed, and clicks Align Right.

It asserts a `text-align-applied` line with `blocks=1/1`, and a
`reflow-block-applied` line carrying `alignment=Some(Right)`.

## What makes them fail

- Bold: dropping `Key::B` from the style arm of `canvas::textedit::edits`.
  The global chord yields while a draft is open, so nothing restyles.
- Align: removing `format.align_*` from `app::dispatch::textformat::handles`;
  the press then reaches `command-unimplemented`.
