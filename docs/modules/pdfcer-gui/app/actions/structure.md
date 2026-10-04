# `app::actions::structure` — Export for hand editing… and Compile hand edits…

The PDF-internals round trip qpdf calls QDF, on `pdfcer_core::editable`. Both
commands sit on File ▸ Export, read the document, and write a new file. The
open document is never changed.

## Export (`WriteAction::Structure`)

1. Serialize the session as Save would (unsaved edits included) and reparse
   it: `editable::export` takes a `Document`, and the session has none to
   lend.
2. `editable::export`: every object top-level, every stream decoded with its
   `/Filter` dropped, a classic xref. `EditableError::Encrypted` is refused
   with its own sentence: a plaintext copy of an encrypted file is a
   decryption.
3. Pick a path (default `<stem>.qdf.pdf`); refuse the open document's own path.
4. Write, and record the export in the stale-base memo.

## Compile (`WriteAction::CompileStructure { edited }`)

The edited file is picked in `dispatch::exchange` before the action, for
`file.import_form_data`'s reason.

1. Serialize and reparse the document as above: this is the *original* the
   diff is taken against.
2. Refuse an encrypted original; refuse an edited file that does not parse.
3. **Stale-base guard** (below).
4. `editable::import` diffs the two. Streams compare by decoded content, so a
   stream the export decompressed is not counted as changed. An empty report
   is refused with *nothing changed*: it is the one failure the operator
   cannot otherwise see.
5. Refuse under an enforced certification
   (`signature::census(..).forbids_structural_change()`), the refusal an
   `EditSession` edit would get.
6. Pick a path, with the change counts in the picker's title (default
   `<stem>-compiled.pdf`); refuse the open document's path and the edited
   copy's.
7. `writer::save_incremental(original, dirty, save options)`: the original's
   bytes unchanged, then one update holding only the changed, added and
   deleted objects, so a signature over anything not edited stays valid.

## The stale-base guard

`import` diffs against whatever it is handed. A copy exported before a later
edit (or from another document) differs from the document by that edit too,
and compiling it would silently undo it, deleting objects added since. Each
export records, in a process-wide map keyed by the canonical path written, a
hash of the bytes it was taken from. A compile of a recorded path whose
current bytes hash differently is refused. A copy exported in an earlier run
is not recorded and compiles as it stands; the counts in the picker title are
its disclosure. Engine request: a source fingerprint carried in the export
would make this guard work across runs.

## Trace

| Line | When |
|---|---|
| `export-structure objects= bytes= path=` | export written |
| `export-structure-refused reason=encrypted\|overwrite` | export refused |
| `export-structure-cancelled` | picker dismissed |
| `import-structure modified= added= removed= unchanged= streams_matched= path=` | compiled copy written |
| `import-structure-refused reason=encrypted\|stale-base\|unchanged\|certified\|overwrite` | compile refused |
| `import-structure-failed reason=unreadable detail=` | edited copy does not parse |
| `import-structure-cancelled` | picker dismissed |
| `structure-failed reason=serialize\|reopen\|export\|write\|write-file detail=` | failed |

Verified by `ui-verify` check `hand_edits_compile_back_as_an_appended_update`.
