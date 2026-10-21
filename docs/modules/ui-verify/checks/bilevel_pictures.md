# `ui-verify/checks/bilevel_pictures`

`bilevel_pictures_store_as_ccitt_and_jbig2` — **Make pictures smaller stores a
picture as black and white through CCITT Group 4 and through JBIG2, each
reaching the engine as the encoder chosen.**

## What it drives

Opens `fixtures/two-photos.pdf` (two 400×400 grey noise pictures, PDF 1.7, so
JBIG2's 1.4 floor is met) in Edit mode. For picture A with `ccitt`, then
picture B with `jbig2`:

1. Click the picture; press `ribbon.item.format.shrink_images`; require
   `shrink-images-opened`.
2. Press `compact.pictures.colour`, then `compact.pictures.colour.bilevel`.
3. Press `compact.pictures.format`, then `compact.pictures.format.<encoder>`.
4. Press `shrink-images.commit`; require
   `images-shrunk changed=1 <encoder>=1 skipped=0`.

## Why the counts prove the encoder

`<encoder>=` is counted from the engine's plan (`Tally::exact`), one per
picture rewritten with that `ImageEncode`. A dialog that dropped the choice
stores the picture as it was (`ccitt=0`); one that sent the encoder without
Black and white gets the picture skipped as not bilevel (`skipped=1`). Neither
can produce the assertion.

## What a failure means

| arm | failure |
|---|---|
| opened | the Format tab route is missing |
| press | a list or a choice was never drawn: the colour list, Black and white, or the encoder |
| shrunk | Apply did not reach the engine, or the plan stored the picture another way |

## `smallest_exact_picture_tries_every_storage`

The same route on picture A with `compact.pictures.format.smallest`
(`ImageEncode::Auto`); it requires `images-shrunk changed=1 tried=1 skipped=0`.
`tried=` counts pictures whose `ImageChange::candidates` is non-empty, which the
engine fills only under `encode=auto`. A dialog that sent any single encoder
instead writes the picture with `tried=0`.

## `jbig2_raises_an_old_documents_version`

The JBIG2 route on picture A of `two-photos-pdf13.pdf`, the same page
declaring PDF 1.3, older than the 1.4 JBIG2 needs. It requires
`images-shrunk changed=1 jbig2=1 skipped=0` and then `raised=1.4`, read from
`OptimizePlan::version_raise`. An engine refusing the raise leaves the picture
(`skipped=1`, `VersionTooLow`); a shell that dropped the disclosure reads
`raised=none`. Either fails.
