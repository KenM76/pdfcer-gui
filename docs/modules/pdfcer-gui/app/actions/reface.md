# `app::actions::reface` — committing characters the run's font lacks

`try_commit` runs when `Action::CommitTextEdit` carries a `Reface`
(`textcommit::commit_text_edit` calls it first). It answers `false`, leaving
the commit to the ordinary path, when the run now takes every character
(`repertoire::refused_now`).

## The engine's fallback first

Inside one `vector_edit`, the edit is first asked of the engine with
`EditOptions::with_fallback(fallbackface::named(face))` (G078): the engine
splits the show operator around the characters with a `Tf` switch to the face
at the run's own size, as one undo entry. It does so only when the match lies
in one `Tj`/`TJ`. On success the engine's disclosures are kept, `set_in` is
appended, and `not_embedded` too when the face is a standard-14 resource the
edit added, which a reader substitutes.

When the engine refuses and the line can be tokenised, the placeholder
gesture below runs in the same closure, so the funnel records no decline for
the first attempt. When it cannot be tokenised, the engine's refusal is
classified by `record_edit_text_refusal` and shown.

## The placeholder gesture

For a match across several show operators:

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

- `text-edit-fallback page= run= characters=U+.. face= source=page|standard14|embedded`
  — the engine set those characters in the fallback face; `face` is its
  `/BaseFont`.

- `text-edit-reface-declined page= run=` — tokenising failed.
- `text-edit-reface-committed page= segments= steps=` — every step landed.
- `text-edit-reface-readback page= run= reads=0|1` — whether the page's
  extracted text now contains the replacement.
