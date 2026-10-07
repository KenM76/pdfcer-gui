# `ui-verify/checks/model_tree`

`the_3d_viewer_lists_the_model_tree` — the 3D viewer lists the model's tree
as the engine reads it (`PrcFile::model_tree`), in a part list drawn inside
the window.

# What it drives

The engine corpus's `fixtures/synthetic/prc/assembly.prc` (a root placing one
part twice), placed from the Edit tab and opened with *View…*
(`model_inferences::open_viewer`), under `--no-input` with a
`ScriptedPointer`.

1. `model-view-parts` reads `nodes=3 depths=0|1|1 names=-|-|-`.
2. The `model3d.parts` region is declared inside the window.

No corpus PRC names, hides or suppresses a part (G135), so those fields are
not asserted.

# Falsified

Dropping the panel's draw call fails step 2. Joining the depths with `,` fails
step 1.
