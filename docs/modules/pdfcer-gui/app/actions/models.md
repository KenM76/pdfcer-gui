# `app::actions::models` — place a 3D model on a page; save, view and re-picture one

Every verb is reached through `AttachmentAction`: those that open a native
file dialog must run in the apply arm, never in a layout pass.

## `insert(doc, page)` — Edit ▸ Insert ▸ 3D model…

1. `files::pick_model_source` (U3D/PRC filter; `PDFCER_DIAG_MODEL_PATH`
   answers it in driven runs). Dismissed → `model-insert-cancelled`.
2. Read the file. Failure → `model-insert-unreadable`, a note.
3. `ThreeDSpec::new(centred(crop_box), data)`. The engine sniffs the format
   and refuses empty data, unrecognised data and STEP (a PDF carries only U3D
   or PRC); the refusal is named in a note, STEP with the advice to convert.
   → `model-insert-refused`.
4. `model-insert-requested page= format= bytes=`, then the vector funnel
   under the label `add-3d` — one undo entry; a refusal traces
   `add-3d-refused`.
5. Notes: what was placed and what the page picture is (`poster_note`); plus
   a version warning when the document's declared PDF version is below the
   one the format needs.

`centred` is a 4:3 box, half the page width (capped by its height), centred
on the crop box.

## `save(doc, artwork)` — the Save model… button

The row is re-listed and must still be present, else nothing is written
(`model-save-declined`). The extension comes from the sniffed format, then the
declared one, else `bin`. The suggested name is `<stem> - page N model.<ext>`
beside the document. Writes the bytes exactly as stored; `model-saved bytes=
ext= contradicts=`. Notes: where it went, and a mismatch or unrecognised
warning when the data disagrees with what the document declares.

## `save_mesh(doc, artwork)` — the Save as mesh… button (feature `3d`)

Re-lists and extracts as `save` does, then asks for a target
(`files::pick_mesh_target`, `PDFCER_DIAG_MESH_SAVE_PATH`); an `.obj` ending
writes OBJ, anything else binary STL, from `assemble`'s meshes. Traces
`mesh-saved obj= bytes= meshes= triangles= skipped= placed=`,
`mesh-save-refused`, `mesh-save-declined`, `mesh-save-cancelled`,
`mesh-save-failed`. The receipt says whether parts were placed. Without the
feature the button is not drawn.

Driven by `ui-verify` check `a_3d_model_is_placed_listed_and_saved_back`.

## `assemble(data)` — decode, place and colour a PRC model (feature `3d`)

A thin mapping of `pdfcer_3d::assemble`, the routine the engine's own mesh
export, renderer and default poster use, so the viewer cannot disagree with
the page picture. `Assembled::colours` is parallel to `meshes` (empty when
the tree could not be read; `placed` is then false and each mesh sits where
its file stores it); `uncoloured()` counts the meshes drawn grey. `skipped`
is wires + markups + compressed meshes left out. `Unassembled` maps
`AssembleError`: `NotPrc`; `CompressedOnly { count }` and `NoTriangles` to
`Empty`; anything else to `Unreadable` with the engine's sentence.

The page picture's status line (`poster_note`) counts the poster's
`uncoloured_meshes` the same way; its trace is
`model-insert-poster drawn= reason= uncoloured=`.

## `load_view(doc, artwork)` — the View… button (feature `3d`)

Re-lists and extracts as `save` does, then `assemble`s; the error is the
sentence `apply` records as a note (with `model-view-declined page=`). On
success `apply` opens `dialogs::model3d`.

Driven by `ui-verify` check `a_3d_model_turns_under_the_pointer`.

## `set_poster(doc, artwork, picture)` — a new page picture

`picture` is image file bytes. The row is re-listed and must still be present
with an annotation of its own, and the bytes must import
(`image_import::import`); otherwise nothing changes and a sentence is noted
(`model-poster-declined page= reason=moved|picture`). Then
`model-poster-requested page= annot= w= h=` and the vector funnel under the
label `set-3d-poster`: `EditSession::set_3d_poster` fits the picture inside
the annotation's rectangle, preserving its shape, and rewrites only `/AP /N`;
the model is untouched. One undo entry.

Reached two ways: the viewer's *Use this view on the page*
(`SetModelPoster`, the samples the viewer drew, built with
`ImportedImage::from_rgba8`), and *Picture…* on the
Attachments row (`PickModelPoster` → `pick_poster`, which asks
`files::pick_image_source` for a file; `PDFCER_DIAG_IMAGE_PATH` answers it in
driven runs; `model-poster-cancelled` when dismissed, and
`model-poster-declined reason=unreadable kind=` with a note when the file
cannot be read).

`model-poster-requested … image=` says how the picture reached the engine:
`pixels` from the viewer, `file` from a picked file.

Driven by `ui-verify` checks `the_3d_viewers_view_becomes_the_page_picture`
and `a_picture_file_becomes_a_3d_models_page_picture`.

## `save_picture(doc, artwork, png)` — the viewer's Save picture…

Asks `files::pick_picture_target` (PNG filter; `PDFCER_DIAG_PICTURE_SAVE_PATH`
answers it in driven runs) with `<document> - page N model.png` suggested,
and writes the bytes the viewer drew. Traces `model-picture-saved bytes=`,
`model-picture-cancelled`, `model-picture-failed kind=`; the note says where
it went. The document is not changed and no undo entry is made.

The three saves share `suggested(doc, artwork, extension)`.

Driven by `ui-verify` check `the_3d_viewer_saves_its_view_as_a_picture`.
