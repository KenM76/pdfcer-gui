# `ui-verify/checks/models`

`a_3d_model_is_placed_listed_and_saved_back` — **Edit ▸ 3D model places a
model, the Attachments panel lists it, and Save model writes back the same
bytes.**

Off-screen, scripted pointer, no OS input. Copies the engine corpus's
four-page document and writes a 76-byte file that opens with the PRC
signature (the engine checks the signature, not the geometry). The pickers are
answered by `PDFCER_DIAG_MODEL_PATH` and `PDFCER_DIAG_ATTACHMENT_SAVE_PATH`.

Oracles: `model-insert-requested`; an `add-3d` funnel line; the panel's
`models-section count=1`; `model-saved`; the saved file equal to the source.

Falsified: saving `data[1..]` fails it ("75 bytes differs from the 76
placed"). A duplicate handler token once routed the click to the Attachments
toggle instead; the check caught it as no `model-insert-requested`.
