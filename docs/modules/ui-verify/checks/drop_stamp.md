# `a_dropped_picture_stamps_in_review`

**Defect it guards:** a picture dropped on a page in Review is refused,
although Review adds comments and a picture stamp is one.

## What it drives

Off-screen, scripted pointer `drop` step, on the shared paste fixture
(`os_image_paste::launch`), with a 48×24 PNG the harness encodes and
`fixtures/vector-art.svg`.

1. Ctrl+2 (Review). Drop the PNG at `osp::FIRST`.
2. Drop the SVG there.
3. Ctrl+1 (Read). Drop the PNG.

## Oracles

| step | requires |
|---|---|
| 1 | one new `image-dropped … as=stamp`, then `picture-stamp-placed kind=image` with a non-zero `id` |
| 2 | the same with `kind=svg` |
| 3 | no new `image-dropped`, and a new `drop-declined` |

## Falsification

Making `actions::picture::stamp` return before its edit fails step 1 on the
missing `picture-stamp-placed`.
