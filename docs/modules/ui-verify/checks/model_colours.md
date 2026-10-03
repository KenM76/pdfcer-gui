# `ui-verify/checks/model_colours`

`a_coloured_3d_model_draws_in_its_own_colours` — **a 3D model whose parts
carry colours draws in them, in the viewer and on the page.**

Off-screen, scripted pointer, no OS input. Places the engine corpus's
`fixtures/synthetic/prc/coloured.prc` from the ribbon (through
`model_view_window::launch_with_model_file`), then opens it with *View…*.

Oracles: on the page, the pixels the insert changed must fall in at least two
of twelve hue sectors (a sector counts when it holds at least 1/50 of the
coloured pixels; a pixel is coloured when its channel spread is at least 48);
in the viewer, an immediate viewport no screenshot reaches, the
`model-view-rendered` line must count `hues>=2` and `model-view-opened` must
count fewer parts `uncoloured` than it has.

The fixture does not exist in the engine yet (our request `G103`); until it
lands the check is SKIPPED on the missing file, which is not a pass. Its
falsification, once runnable: hand the renderer an empty colours slice.
