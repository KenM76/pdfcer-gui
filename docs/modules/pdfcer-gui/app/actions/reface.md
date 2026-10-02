# `app::actions::reface` — committing characters the run's font lacks

`try_commit` runs when `Action::CommitTextEdit` carries a `Reface`
(`textcommit::commit_text_edit` calls it first). It answers `false`, leaving
the commit to the ordinary path, when the run now takes every character
(`repertoire::refused_now`) or the line cannot be tokenised.

## The gesture

Inside one `vector_edit`:

1. `edit_text` the tokenised line through the ordinary `Plan`, so the line is
   written exactly as any other edit (pins, narrowing, disposition).
2. For each `(token, stretch)`, last first so earlier tokens keep their
   offsets: `format_text` the token to the face, then `edit_text`
   find-and-replace the token with the stretch.
3. `coalesce_last(steps, EditText)`, so one Undo takes the whole gesture.
   Refused → the `undo_split` note.

A checkpoint is taken before step 1. A step that fails rolls the session
back to it (`EditSession::rollback`), so the document, Undo and Redo are as
they were, and returns `Stopped`, whose sentence the funnel shows; undo
history the rollback could not keep is added to it (`history_lost`). A
gesture longer than the undo bound cannot be rolled back and is undone step
by step, which leaves its steps on Redo.

The engine's disclosures are de-duplicated (each `edit_text` repeats the
save/relayout notes) and `set_in` is appended.

## Trace

- `text-edit-reface-declined page= run=` — tokenising failed.
- `text-edit-reface-committed page= segments= steps=` — every step landed.
- `text-edit-reface-readback page= run= reads=0|1` — whether the page's
  extracted text now contains the replacement.
