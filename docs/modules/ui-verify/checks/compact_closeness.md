# `ui-verify/checks/compact_closeness`

`a_closeness_target_picks_each_pictures_quality` — **in the compact window,
choosing how closely JPEG pictures must match the original sends each through
the engine's quality search, and the copy counts it.**

## What it drives

Opens `fixtures/noisy-photo.pdf` (one 400×400 greyscale noise picture, stored
exactly) with `PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted`, through the
scripted pointer, so it runs under `--no-input`. After the first
`compact-recompressed` line it opens the Pictures fold and its switch, then
picks JPEG from `compact.pictures.format`, then *Close (0.95)* from
`compact.pictures.target`, waiting for a fresh `compact-recompressed` line
after each choice. A list's open choices float above the window body, so only
the list itself is scrolled into view.

## What it asserts

| after | `jpeg` | `matched + short` | what a wrong count means |
|---|---|---|---|
| JPEG | 1 | 0 | a search ran with no target asked, or JPEG never reached the plan |
| Close (0.95) | 1 | 1 | the target never reached the engine's `ImagePatch::ssim` |

Whether the search met its target depends on the picture; the check asserts
the search ran, not which way it came out. The report wording for each
outcome is `text::compact_images::closeness`, and the patch carrying the
target with the quality as its ceiling is
`compact_images::tests::a_target_rides_only_with_jpeg`.
