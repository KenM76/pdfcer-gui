# `an_ime_composition_types_once_committed`

Composed input in a text edit (WORDLIKE step 9). On `fixtures/paragraph.pdf`,
launched with `mode.edit,edit.text` off the desktop through the scripted
pointer, which posts `ImeEvent`s as egui-winit would.

1. Click inside `drawing` on the first line: a caret opens, and a
   `text-ime-area` line records that the draft asked for an input method.
2. A `preedit zq` step: a `text-ime-preedit chars=2` line; the screenshot
   `ime-preedit.png` shows the composition under the line.
3. A `commit é` step, then Escape commits the edit.
4. Ctrl+F, search `é`: `hits=1`. Search `zq`: `hits=0`, so the composition
   never entered the text.

## What makes it fail

- Deleting the `ImeEvent::Commit` arm in `canvas::textedit::edits::Keys::event`:
  the commit is dropped and the search for `é` reports `hits=0`.
- Deleting the `super::ime::show` call in `paint::preview`: no `text-ime-area`
  line, and the check stops at step 1.
