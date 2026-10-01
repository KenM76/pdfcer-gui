# `app::actions::models` — place a 3D model on a page; save an embedded one out

Two verbs, both reached through `AttachmentAction` because both open a native
file dialog and so must run in the apply arm, never in a layout pass.

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
5. Notes: what was placed and that pdfcer draws a placeholder picture (the
   engine renders no 3D scene); plus a version warning when the document's
   declared PDF version is below the one the format needs.

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

## `assemble(data)` — decode and place a PRC model (feature `3d`)

Parses the PRC (`pdfcer_3d::PrcFile`), keeps every `Tessellation::Mesh` and
every rebuilt `Compressed { mesh: Some(..) }`, and counts the rest as
skipped. Each `PrcFile::placements` entry transforms its mesh into place;
when placements yield nothing (the tree is unreadable or empty) every mesh is
kept where its file stores it and `placed` is false. `Unassembled` says why
there is nothing: not PRC, unreadable, or no triangles (naming compressed
meshes the engine could not rebuild).

## `load_view(doc, artwork)` — the View… button (feature `3d`)

Re-lists and extracts as `save` does, then `assemble`s; the error is the
sentence `apply` records as a note (with `model-view-declined page=`). On
success `apply` opens `dialogs::model3d`.

Driven by `ui-verify` check `a_3d_model_turns_under_the_pointer`.
