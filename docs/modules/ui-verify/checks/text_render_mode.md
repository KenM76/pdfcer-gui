# `ui-verify/checks/text_render_mode`

`text_render_mode` — **sweep text, choose "Outlined" under Properties › Drawn
as, and the file changes.**

## What it guards

The Drawn as combo lists the eight PDF text render modes. Each entry raises
`Action::TextStyle` with `StyleChange::RenderMode`, which reaches the engine's
`format_text` through `FormatRequest::render_mode`. A combo that draws but
whose pick raises nothing looks identical in every unit test.

## How it drives

1. It opens the text fixture from `fixture::text_point_target` (`--pdf` and
   `--doc-point` are ignored) with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.
2. It arms the text-sweep tool (`T`) and drags 60 pt along the baseline, then
   requires a `canvas-text-selection` line with `chars` > 0.
3. It finds the `properties.text.render` region, scrolling the Properties panel
   up to six notches.
4. It clicks the combo, then the popup entry `properties.text.render.1`. Each
   entry publishes its own region, `properties.text.render.<mode>`, so the
   popup can be aimed at.
5. It polls for `text-style-applied` or `text-style-declined` (20 s ceiling).

It passes when `text-style-applied` reads `change=render-mode` with
`applied` > 0 and a `format-text` line follows. A decline, a missing summary,
a different `change`, or `applied=0` fails with its own diagnosis.

## Falsification

With the entry's `chosen = Some(mode)` removed, the check fails: no summary
line follows the pick.
