# `a_gltf_model_is_converted_and_placed`

Edit ▸ 3D model with a glTF file: a one-quad model (two triangles, buffer embedded as a data URI) written into the run folder answers the picker through `PDFCER_DIAG_MODEL_PATH`.

## Oracles

1. `model-insert-gltf triangles=2`: the engine (`pdfcer_3d::read_gltf` then `write_prc`) converted it and reported the mesh.
2. `add-3d` and `models-section count=1`: it was placed and the panel lists it.
3. Save model writes bytes starting `PRC`: the document embeds the converted PRC, not the glTF text.

## What it cannot see

Whether the model looks right in a 3D viewer; that is `model_view_picture`.
