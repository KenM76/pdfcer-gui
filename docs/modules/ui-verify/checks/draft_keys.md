# `ui-verify/checks/draft_keys`

`the_draft_keys_do_what_a_word_processor_does` — the keys a word processor
gives a caret work inside a text draft.

The fixture is `fixtures/word-fragmented-lines.pdf`, copied to the check's
output folder so `Ctrl+S` saves the copy. The window is placed off the desktop
and driven only through `ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,
edit.text` arming the Edit Text tool, so the check runs under `--no-input`.
Every character typed is one the line's subset font holds, so the sieve keeps
all of them.

## Steps

A click on the first line, then `End`.

1. **Undo and redo.** Type `ad`, then `e`: the draft grows by 3. `Ctrl+Z`
   returns it to its starting length with `text-edit-history owner=draft`, as
   both typed runs are one undo entry; `Ctrl+Y` restores them.
2. **Word delete.** `Ctrl+Backspace` removes more than one character.
3. **Tab and paste.** `Tab` adds the `spaces=` its `text-edit-tab` line names
   and raises `text-edit-note note=tab-as-spaces`. A pasted `a\nd` adds three
   characters, traces `text-edit-paste joined=1` and raises
   `note=lines-joined`; the status bar publishes `status-group:draft-note`.
4. **Arrows and Enter.** Two Shift+Left trace `text-select ... n=2`; Left
   traces `text-select none`; Enter on the line declines
   (`text-edit-enter-declined`) and the length stays.
5. **Select and save.** `Ctrl+A` traces `text-select from=0 to=<len>`.
   `Ctrl+S` traces `text-edit-save in_place=1`, the edit commits
   (`edit-text-left-edge committed=yes`) and the file saves
   (`save-in-place outcome=ok`).

## Falsification

With `edits::Keys::step` sending every undo to the document, step 1 fails:
the draft is committed and closed on the first `Ctrl+Z`.
