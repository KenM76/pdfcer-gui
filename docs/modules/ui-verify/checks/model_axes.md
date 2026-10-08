# `a_3d_model_opens_from_the_page_and_turns_about_the_chosen_up`

**Defect it guards.** The operator asked to open a 3D model by clicking it on
the page, and to choose which of its axes is up, front and so on
(`OPERATOR_REQUESTS.md` O289 item 12). Ways it goes wrong: a click in Read
selects text or does nothing, the up choice is drawn but the named views keep
turning about z, or a changed up leaves the picture where it was.

**Fixture.** The engine corpus's `four-pages.pdf` with `assembly.prc` placed
from Edit ▸ 3D model (centred on page 1), driven off the desktop with the
scripted pointer.

**Steps.**

1. Place the model; require `model-insert-requested`.
2. Switch to Read and click page 1's centre: require `model-page-click` and
   `model-view-opened`.
3. Press Front: the last `model-view-rendered` `dir=` is +y (z up).
4. Open *Up* and pick +Y (`model3d.up.1`): the Front view is re-applied and
   looks along -z.
5. Press Top: it looks along -y.

**Falsified** in two ways:

- Removing the `model_hit` arm from `canvas::clicking::click` fails step 2: the
  click in Read opens no viewer.
- Making `Upright::with_up` ignore the axis picked fails step 4: Front still
  looks along +y.

**What it does not prove.** The Front-look choice is the same `axis_choice`
and is covered by `orbit`'s and `axes`' unit tests, not driven. Review takes
the same click arm as Read.
