# `ui-verify/checks/model_inferences`

`the_3d_viewer_says_the_assembly_coloured_it` — a 3D model whose parts take
their colour from its assembly's entity references opens in the viewer with
that counted and said (`t::view_overridden`); and
`the_3d_viewer_says_a_mesh_was_best_fit` — a mesh only a best-fit search
rebuilds opens with that counted and said (`t::mesh_best_fit`).

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

# Falsified

Plumbing `overridden: 0` into `actions::models::assemble` in place of the
engine's count fails step 2. The best-fit check pointed at `overridden.prc`
(`best-fit=0`) fails step 2.
