# `dialogs::remove_metadata` — Security ▸ Protect ▸ Remove metadata…

`OPERATOR_REQUESTS.md` O289 item 21: find a document's metadata and remove the
chosen items, or all of it.

## What it lists

Every item `EditSession::metadata_inventory` finds, grouped under the
engine's kinds in its order (`MetadataKind::ALL`): description entries, XMP,
private producer data, thumbnails, scripts, attachments, comments, layers
hidden at open, form data, earlier versions, the file identifier. Each row is
a checkbox naming the item (a description entry by its key, anything else by
the engine's location text), a short look at it and its size. A kind's own
checkbox ticks or clears all its items; *Select all* ticks every row. None
is ticked at open: the window removes, so the operator names what goes.

A partial inventory (`MetadataInventory::truncated`) is said under the intro.
A document carrying nothing says so in a sentence.

## The removal writes a copy

*Remove and save a copy…* pushes one `WriteAction::RemoveMetadata` with the
ticked ids, handled by `app::actions::remove_metadata`, which follows
`app::actions::purge_passwords`:

1. Serialize the session as Save would, so unsaved edits are included, and
   open a fresh session on those bytes.
2. `remove_metadata` with the default options: a ticked file identifier is
   regenerated, not dropped (PDF 2.0 requires one).
3. `to_full_bytes_decomposing_containers` with the Save options, whose
   `ProducerPolicy::Preserve` keeps a removed `/Producer` from coming back.
4. List the output again; a copy still listing a removed id (the identifier
   aside, which is rewritten) is not written.
5. Ask for the path, write, and post the receipt off-canvas: how many items
   went, any the engine did not remove with its reason, any no longer
   present, and the engine's disclosures.

Why a copy and not an edit of the open document: this shell's Save appends a
revision, which keeps every removed value in the one before it. A removal
that Save would undo in the file is not a removal.

A signed document is refused, as Remove old passwords refuses it: the
one-version rewrite breaks every signature.

The engine's disclosure that removal needs a full rewrite is dropped from the
receipt, because this write is that rewrite. It is matched on the verb its
text names; engine request G172 asks for a way to recognise it.

## Trace and regions

| Name | What |
|---|---|
| `remove-metadata.body` | the window body |
| `remove-metadata.kind.<kind>` | a kind's checkbox; `<kind>` is `MetadataKind::as_str` |
| `remove-metadata.item.<id>` | one item's checkbox; `<id>` is the item's id, e.g. `info/Author` |
| `remove-metadata.all` | Select all |
| `remove-metadata.commit` | Remove and save a copy |
| `remove-metadata-listed items= truncated= ids=` | at open |
| `remove-metadata-requested ids=` | at Remove |
| `remove-metadata-wrote removed= not_found= not_removed= freed= path=` | the copy is on disk |
| `remove-metadata-failed reason= detail=` | refused or failed; nothing written |

Traces name ids, never previews: the previews are the operator's own content.

Driven by `remove_metadata_writes_a_copy_without_the_items_ticked`.
