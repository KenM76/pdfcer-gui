# `pagebox` — the visible area a conforming reader shows

`Page::crop_box` is `/CropBox` as written. §14.11.2.1 makes the visible area
its intersection with `/MediaBox`; the engine does not intersect (`G059`) and
the renderer frames by `crop_box`, so a sheet shrunk under its crop box drew
at its old size while every other reader showed the new one.

| Item | Contract |
|---|---|
| `visible(crop, media)` | `crop ∩ media`; `media` when they do not overlap |
| `clip_crop_to_media(pages)` | replaces each `crop_box` with `visible`; returns how many changed. Called by `OpenDoc::assemble` and `app::actions::pages::resync`, the two places a page list is adopted |
| `crop_hides_sheet(page)` | the visible area is smaller than the paper, so growing the paper shows nothing new. `pagesize::set` counts these into `disclosure_crop_inside` |

Delete the module when `G059` lands: `Page::crop_box` will then already be
the visible area.
