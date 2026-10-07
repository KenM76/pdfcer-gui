# `ui-verify/checks/model_inferences`

`the_3d_viewer_says_the_assembly_coloured_it` — a 3D model whose parts take
their colour from its assembly's entity references opens in the viewer with
that counted and said (`t::view_overridden`); and
`the_3d_viewer_says_a_mesh_was_best_fit` — a mesh only a best-fit search
rebuilds opens with that counted and said (`t::mesh_best_fit`); and
`the_3d_viewer_draws_a_texture` — a part styled by a texture picture is drawn
with it and counted (`t::view_textured`).

# What it drives

The engine corpus's `fixtures/synthetic/prc/overridden.prc`, placed from the
Edit tab as `model_colours` places its model, under `--no-input` with a
`ScriptedPointer`.

1. Edit ▸ 3D model inserts it; `models.view` is declared beside it.
2. *View…* clicked: `model-view-opened … overridden=N` with N at least 1.

`the_3d_viewer_says_a_mesh_was_best_fit` drives the same steps on
`best_fit.prc`, one compressed mesh of 28 triangles that only the best-fit
search rebuilds, and requires `best-fit=N` with N at least 1, the count the
viewer states through `t::mesh_best_fit`. The fixture is read from the engine
checkout at `D:/Dev/pdfcer`, as `overridden.prc` is.

`the_3d_viewer_draws_a_texture` drives them on `textured.prc`, a square whose
texture picture is red, green, blue and white. It requires `textured=N` with N
at least 1, and the last `model-view-rendered` line to count at least three
`hues`, which a square drawn in its one base colour cannot produce.

# Falsified

Plumbing `overridden: 0` into `actions::models::assemble` in place of the
engine's count fails step 2. The best-fit check pointed at `overridden.prc`
(`best-fit=0`) fails step 2. Rendering through `render_coloured`, which draws no
texture, fails the texture check at `hues=0`.
