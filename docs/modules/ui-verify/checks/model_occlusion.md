# `ui-verify/checks/model_occlusion`

`insert_3d_model_occlusion_reaches_the_conversion` — **Insert 3D model's
*Darken creases from the occlusion map* switch decides `GltfOptions::occlusion`,
and the status row says which.**

## What it drives

Two launches, each placing `model_occlusion.gltf` (one unit quad, orange by
`baseColorFactor`, a 2x2 PNG checker as `occlusionTexture`, no base texture)
through the Insert 3D model window with the scripted pointer: once with the
defaults, once with `insert-model.gltf.occlusion` pressed.

## What it asserts, per launch

| # | assertion | what its failure means |
|---|---|---|
| 1 | `insert-model-placed` carries `occlusion=true` / `occlusion=false` | the switch does not reach the placement |
| 2 | `model-insert-gltf` reads `occlusion=Bake textured=1` / `occlusion=Drop textured=0` | the placement does not reach the engine's options; with no base texture, only baking makes the quad textured |
| 3 | the `add-3d` line says `occlusion texture baked into` / `occlusion texture not drawn, as asked` | the engine's disclosure does not reach the status row |
