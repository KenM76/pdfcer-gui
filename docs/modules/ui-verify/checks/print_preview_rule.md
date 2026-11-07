# `ui-verify/checks/print_preview_rule`

`print_preview_follows_the_paper_rule` — Print obeys paper's annotation flags and
the file's `Print` layer usage, not the screen's rule.

The preview and the spool take their options from one builder,
`dialogs::print::commit::render_options`, so the preview's pixels stand for both.
The check reads the share of the preview's raster that is not white
(`print-preview-render ink= of=`) on one 200-point page built at run time:

| Square | Share of page | On paper? | On screen? |
|---|---|---|---|
| Page content | 2.25% | yes | yes |
| Annotation, `F 36` (`NoView`, `Print`) | 2.25% | yes | no |
| Annotation, `F 0` (`Print` clear) | 9% | no | yes |
| Content in a layer with `/Usage /Print /PrintState /OFF` | 16% | no | yes |

The annotations are `/Watermark`, which paints under every scope but fields-only,
so the scope the dialog remembers does not decide them. The right share is 4.5%,
and each mistake lands somewhere of its own: 13.5% is the `Print`-clear annotation,
20.5% is the layer, 2.25% is the paper-only annotation missing. The failure sentence
names which.

A layer the operator hid on screen is covered by a unit test of `render_options`.
