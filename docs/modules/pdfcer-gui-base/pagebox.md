# `pagebox` — whether a page's crop box hides part of its sheet

`Page::crop_box` is the effective box: the engine intersects `/CropBox` with
`/MediaBox` (§14.11.2.1), so the shell never recomputes the visible area.

| Item | Contract |
|---|---|
| `crop_hides_sheet(page)` | the visible area is smaller than the paper, so growing the paper shows nothing new. `pagesize::set` counts these into `disclosure_crop_inside` |
