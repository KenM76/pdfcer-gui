# `the_3d_viewer_draws_a_mesh_in_part`

**Defect it guards.** O304: a 3D model whose compressed mesh no fit rebuilds
shows nothing, or is drawn in part with no word of it, or the viewer's Mesh fit
choice does not change what is drawn.

**Fixture.** The engine corpus's `fixtures/synthetic/prc/partial.prc`: one
compressed mesh of four triangles; no fit rebuilds it, three decode before its
arrays stop fitting. Placed from the ribbon and opened with *View…*, as the
other `model_*` checks do.

**Steps.**

1. `model-view-opened` carries `fit=partial opened-in-part=1 partial=1
   partial-triangles=3/4`: the default fit draws nothing, so the viewer opens
   in part and says so.
2. Mesh fit ▸ *Continued*: `model-view-refit` carries `fit=continue
   partial-triangles=4/4`.
3. Mesh fit ▸ *Exact only*: `model-view-refit-refused fit=unique`, since it
   would leave nothing to draw.

**Falsified** by opening at the default fit only: the viewer refuses the
model, so step 1 finds no `model-view-opened` line.

**What it does not prove.** That the disclosure lines are legible; the trace
fields they are worded from are what is read.
