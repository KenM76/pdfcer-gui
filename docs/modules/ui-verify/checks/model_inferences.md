# `ui-verify/checks/model_inferences`

`the_3d_viewer_says_the_assembly_coloured_it` — a 3D model whose parts take
their colour from its assembly's entity references opens in the viewer with
that counted and said (`t::view_overridden`).

# What it drives

The engine corpus's `fixtures/synthetic/prc/overridden.prc`, placed from the
Edit tab as `model_colours` places its model, under `--no-input` with a
`ScriptedPointer`.

1. Edit ▸ 3D model inserts it; `models.view` is declared beside it.
2. *View…* clicked: `model-view-opened … overridden=N` with N at least 1.

The best-fit count (`best-fit=`, `t::mesh_best_fit`) is traced but not driven:
the corpus holds no PRC whose mesh only a best-fit search rebuilds;
`compressed.prc` is refused outright (`model-view-declined`).

# Falsified

Plumbing `overridden: 0` into `actions::models::assemble` in place of the
engine's count fails step 2.
