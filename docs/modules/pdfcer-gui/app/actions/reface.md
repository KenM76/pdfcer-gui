# `app::actions::reface` — committing characters the run's font lacks

`try_commit` runs when `Action::CommitTextEdit` carries a `Reface`
(`textcommit::commit_text_edit` calls it first). It answers `false`, leaving
the commit to the ordinary path, when the run now takes every character
(`repertoire::refused_now`).

## The engine's fallback

Inside one `vector_edit`, the edit is asked of the engine with the ordinary
`Plan` plus `EditOptions::with_fallback(fallbackface::named(face))` (G078):
the engine writes the characters in the face with a `Tf` switch at the run's
own size, as one undo entry. It does so for a match in one text object,
however many `Tj`/`TJ` operators the change spans. On success the engine's
disclosures are kept, `set_in` is appended, and `not_embedded` too when the
face is a standard-14 resource the edit added, which a reader substitutes.

A refusal is classified by `record_edit_text_refusal` and shown in the
engine's words. Two shapes are refused by the engine and have no other route
here: a match joined across `ET` (two text objects), which the engine refuses
for any edit, fallback or not, and a match anchored on a `'` or `"` operator,
whose own refusal offers the rewrite-to-`Tj` workaround.

## Trace

- `text-edit-fallback page= run= characters=U+.. face= source=page|standard14|embedded operators=`
  — the engine set those characters in the fallback face; `face` is its
  `/BaseFont`, `operators` its `EditReport::operators_spanned`.
- `text-edit-reface-readback page= run= reads=0|1` — whether the page's
  extracted text now contains the replacement.
