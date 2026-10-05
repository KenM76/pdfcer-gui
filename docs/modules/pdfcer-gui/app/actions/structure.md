# `app::actions::structure` — Export for hand editing… and Compile hand edits…

The PDF-internals round trip qpdf calls QDF, on `pdfcer_core::editable`. Both
commands sit on File ▸ Export. The export writes a new file; the compile
changes the open document as one undo entry, and Save writes it.

## Export (`WriteAction::Structure`)

1. `editable::export(&session)`: the session itself, unsaved edits included.
   Every object top-level, every stream decoded with its `/Filter` dropped, a
   classic xref, and a `%PdfcerExportBase sha256:` header line carrying
   `editable::fingerprint` of the session. `EditableError::Encrypted` is
   refused with its own sentence: a plaintext copy of an encrypted file is a
   decryption.
2. Pick a path (default `<stem>.qdf.pdf`); refuse the open document's own path.
3. Write it. The receipt counts the session's live objects
   (`EditableSource::object_ids`).

## Compile (`WriteAction::CompileStructure { edited }`)

The edited file is picked in `dispatch::exchange` before the action, for
`file.import_form_data`'s reason.

1. Refuse an edited file that does not parse.
2. **Stale-base guard** (below).
3. `EditSession::import_editable` through `apply::vector_edit` (label
   `compile-hand-edits`), so undo, the epoch and the page resync are the
   funnel's. The diff is taken against the session's current state; streams
   compare by decoded content, so a stream the export decompressed is not
   counted as changed. The engine records nothing when nothing changed.
4. The receipt is the funnel's disclosure list: the counts, the streams
   matched only after decoding, and for an unrecorded base the sentence that
   it could not be checked. An empty report says *nothing changed*: it is the
   one outcome the operator cannot otherwise see.
5. A refusal (`DocumentEncrypted` under a permission that forbids modifying
   contents, `CertificationForbidsChange`, or anything else in the engine's
   words) is worded as a note.

Save then writes an incremental update by default: the original's bytes
unchanged and only the changed, added and deleted objects appended, so a
signature over anything not edited stays valid.

## The stale-base guard

The compile diffs against the session as it is now. A copy exported before a
later edit (or from another document) differs by that edit too, and
compiling it would undo it, deleting objects added since.

| `editable::recorded_base(edited)` | Outcome |
|---|---|
| equals `editable::fingerprint(session)` | applied (`base=matches`) |
| differs | refused, `stale-base`; nothing changes |
| absent | applied (`base=unrecorded`), and the receipt says it could not be checked |

The fingerprint hashes content, not layout, so a save, a reopen or a later
run does not make a current export stale.

## Trace

| Line | When |
|---|---|
| `export-structure objects= bytes= path=` | export written |
| `export-structure-refused reason=encrypted\|overwrite` | export refused |
| `export-structure-cancelled` | picker dismissed |
| `compile-hand-edits page=0 n=0 epoch= disclosures=` | the funnel's line for the edit |
| `import-structure modified= added= removed= unchanged= streams_matched= base=` | applied |
| `import-structure-refused reason=stale-base\|unchanged\|encrypted\|certified\|engine` | not applied |
| `import-structure-failed reason=unreadable detail=` | edited copy does not parse |
| `structure-failed reason=export\|write-file detail=` | failed |

Verified by `ui-verify` check `hand_edits_compile_back_as_an_appended_update`.
