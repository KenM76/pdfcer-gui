# `dialogs::remove_metadata` — Security ▸ Protect ▸ Remove metadata…

`OPERATOR_REQUESTS.md` O289 item 21: find a document's metadata and remove
several entries at once.

## What it lists

The document-information entries the engine reads and writes —
`InfoField::all()`: Title, Author, Subject, Keywords — that the document holds,
each with its decoded value and a checkbox. None is ticked at open: the window
removes, so the operator names what goes. *Select all* ticks every row.

A document holding none of them says so in a sentence rather than opening an
empty list.

## What it does not list, and says so

Dates, `/Producer`, `/Creator`, custom `/Info` keys and the catalog's XMP
`/Metadata` stream are not listed. The engine has no reader for the rest of
`/Info` and no removal for any entry but these four (`ENGINE_BACKLOG.md`
G164). The window carries one off-canvas sentence saying so, under the list,
where the operator decides.

## The removal

*Remove chosen* pushes one `Action::SetInfoField { field, value: None }` per
ticked row: the same action the Document properties panel sends when a field
is emptied, so removal is the engine's `EditSession::set_info_field` with the
key deleted, not written empty. Each entry is one Undo step — the engine
records one command per call and offers no grouping.

## Trace and regions

| Name | What |
|---|---|
| `remove-metadata.body` | the window body |
| `remove-metadata.field.<Key>` | one row's checkbox; `<Key>` is the PDF key, `Title` … `Keywords` |
| `remove-metadata.all` | Select all |
| `remove-metadata.commit` | Remove chosen |
| `remove-metadata-listed fields=` | at open: the keys the document holds, comma-joined |
| `remove-metadata-requested fields=` | at Remove: the keys ticked |

Traces name keys, never values: the values are the operator's own metadata.

Driven by `remove_metadata_takes_only_the_entries_ticked`.
