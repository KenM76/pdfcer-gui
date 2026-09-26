# `ui-verify/checks/render_diagnostics`

`render_diagnostics_opens_its_report` — the regression test for **the
inert control whose data was already there.**

# The defect class this exists for

`shell::commands::reach` called `tools.render_diagnostics` the least
defensible entry on its allow-list, and the reason was not that the feature
was hard:

> NO RECORDED REASON for the missing arm, and the data already exists. […]
> That is an argument for moving a readout that is already being computed,
> which makes the inert control the least defensible kind — the work behind
> it is done.


# Why a unit test cannot cover it

[`crate::checks`]' rule: *"it must fail against a build where the wiring is
absent, and the wiring must be something no unit test in the workspace can
observe."*

`DialogsState`'s guards are unit-tested — no document means no dialog, a
second press does not rebuild the first. `text::diagnostics` is unit-tested
down to its units. **Every one of those passes against a build with no
dispatch arm**, because a dialog nothing opens is still a dialog that
refuses correctly. The join — a ribbon token reaching
`DialogsState::open_diagnostics`, and the resulting window being *drawn* —
is a property of two call sites, and `measure_linear`'s recorded incident is
what a missing call site looks like: four passing unit tests and a control
that did nothing visible.

# The oracle is a region the dialog declares while drawing itself

`dialogs::diagnostics` publishes `dialog:render-diagnostics` from inside its
body closure, so the region exists **only on frames where egui actually laid
the window out**. That is a stronger statement than "the state field became
`Some`", and it is stronger than a pixel diff in one specific way that
matters here: a window whose body panicked, whose scroll area collapsed to
nothing, or which was laid out entirely off-screen would still change
pixels somewhere. This asserts that the surface has a real rect.

The rect is then required to be **substantial**, for the reason
`panels::mod`'s header records: three panels in the old shell shipped with a
body, a rail entry and no control anyone could click, and passed every
verification for their whole shipped life.

# What it deliberately does not assert

**What the report says.** The findings, the duration and the raster line are
read from a live texture, so their *content* depends on the fixture and on
how far the render had got — asserting on them here would make this check a
test of `a1-titleblock.pdf` rather than of the wiring. The editorial rules
behind the list are pinned where they live
(`app::status::notes::findings`'s tests) and the wording and units in
`text::diagnostics`'s.

**That pressing it twice does nothing.** That is `DialogsState`'s
already-open guard and it has a unit test with a pointer-identity assertion,
which is a stronger check than anything a window can offer.
