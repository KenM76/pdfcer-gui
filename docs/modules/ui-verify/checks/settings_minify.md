# `ui-verify/checks/settings_minify`

`every_minify_filter_is_offered` — **Settings ▸ Images offers every
`MinifyFilter` the engine has (point sampling, bilinear, area averaging), and
Save writes the one pressed to the settings file.**

## What it drives

Two launches on `fixtures/checker-image.pdf` with
`PDFCER_DIAG_INVOKE=file.settings`, sharing one sandbox, through the
scripted pointer so it runs under `--no-input`. Each opens the Images page,
rolls until all three `settings.images.minify.<word>` regions are declared,
presses one, presses Save and closes.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | `settings.heading.images` is declared | the page is not reachable |
| 2 | `settings.images.minify.{point_sample,smooth,area_average}` are all declared | an engine filter is not offered; area averaging is the engine default, so without it the default is unselectable |
| 3 | after pressing `smooth` and Save, the file says `image_minify = smooth` | the radio does not reach the draft, or Save does not write it |
| 4 | after pressing `area_average` and Save, the file says `image_minify = area_average` | as 3, for area averaging |

Step 3 runs first so step 4 changes a written value: area averaging is the
engine default and a file holding it could be left from any earlier run.
