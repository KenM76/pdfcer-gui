# `ui-verify/checks/models`

`a_3d_model_is_placed_listed_and_saved_back` — **Edit ▸ 3D model places a
model, the Attachments panel lists it, and Save model writes back the same
bytes.**

Off-screen, scripted pointer, no OS input. Copies the engine corpus's
four-page document and places its `square.prc`, a real two-triangle PRC mesh.
The pickers are answered by `PDFCER_DIAG_MODEL_PATH`,
`PDFCER_DIAG_ATTACHMENT_SAVE_PATH` and `PDFCER_DIAG_MESH_SAVE_PATH`.

Oracles: `model-insert-requested`; an `add-3d` funnel line; the panel's
`models-section count=1`; `model-saved`; the saved file equal to the source;
`mesh-saved`; the STL's header triangle count agreeing with its length.

Falsified: saving `data[1..]` fails it ("75 bytes differs from the 76
placed"); an STL one byte short fails it ("183 bytes and is not a binary
STL"). A duplicate handler token once routed the click to the Attachments
toggle instead; the check caught it as no `model-insert-requested`.
