# `ui-verify/checks/ocr_live_search`

`recognised_text_is_searchable_before_saving` and
`stopped_recognition_is_searchable_before_saving` — File ▸ Recognise text…
puts its words into the open document: Find in the same session, with nothing
saved, finds them where it found none before.

# What it drives

`fixtures/synthetic-image-only.pdf` run to the end, or
`fixtures/synthetic-image-only-8pages.pdf` stopped once two pages are done
(ignores `--pdf`). `ocr_engine = ocrs` is seeded; a packaged build is needed
so the models sit beside the binary. Off the desktop under `--no-input` with a
`ScriptedPointer`. `PDFCER_DIAG_INVOKE=file.document_properties` opens
Document properties at launch.

1. A `security-notes` line: Document properties drew.
2. Ctrl+F, `e`, Enter: `find hits=0`. A `find-refused` line fails the check.
3. Read mode, File tab, `ribbon.item.file.ocr` (through the collapsed
   Recognise group when the item is not declared), `ocr-run`.
4. Stopped variant: `ocr-stop` once `ocr-progress attempted>=2`. A run that
   finished first is a SKIP: nothing was stopped.
5. `ocr-recognised`, then the edit funnel's `ocr-layer`. An
   `ocr-layer-refused` instead fails it.
6. Escape, then Find again: `hits>0`.

# Why Document properties is open

Every edit reaches the session through `Arc::get_mut`, which refuses while any
other `Arc` **or `Weak`** to it exists. A panel cache keeping a
`Weak<EditSession>` to recognise its document leaves every later edit, Find
and recognition refused as `session-borrowed` while the cache lives, in the
open document only; a saved and reopened file is unaffected, which is why
`ocr_scripted`'s checks cannot see it. Security notes caches per document, so
caches name a document by `OpenDoc::serial`, never by a `Weak`.

Falsification: a build whose Security notes cache holds a `Weak` fails both
variants at step 2 with `find-refused reason=session-borrowed`.
