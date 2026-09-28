# `dialogs::page_crop` — the visible area of the picked sheets

`pages.crop`, Pages ▸ Transform ▸ Crop…. Opened by `app::dispatch::pages` over
`page_operands` (picked sheets, else the current one). Commits
`PageAction::SetCropBox`, whose body is `app::actions::pagesize::crop`
(`EditSession::set_crop_boxes`, one undo entry).

## Contract

| Item | Contract |
|---|---|
| Margins | Whole millimetres, ≥ 0, one per edge **as the operator sees the sheet**. Stored in page space as `[left, right, bottom, top]` of the unrotated page; `Edge::pdf_index(rotate)` maps a screen edge onto it. Seeded from the first sheet's `media − crop`. |
| Commit | All zero → `CropBoxEdit::Reset` (no crop box written). Otherwise `CropBoxEdit::Set(media inset by the margins)`, page space, points. |
| Mixed pick | Sheets that differ in media box (to `PaperSize::CLASSIFY_TOLERANCE`) or rotation get no margin boxes: one set of margins would crop them differently. Only *Show whole sheet* (Reset) is offered. |
| Nothing left | Margins leaving ≤ 1 pt either way: the refusal line replaces the visible-size summary and the Crop button is absent (R9), so `EditError::CropBoxEmpty` is not reachable from here. |
| Summary | Visible width × height in mm, swapped for `/Rotate` 90 and 270 so it reads as the screen does. |

## Why a margin and not a rectangle

The operator crops *off* an edge ("lose the 10 mm border"); a rectangle in
page coordinates would ask him for the origin and the `/Rotate` convention.
Margins seen on screen are the Acrobat Crop Pages shape.

## Trace and regions

`page-crop-opened sheets uniform rotate margins_mm`, `page-crop-commit n edit
llx lly urx ury`, and from the action `page-crop-applied n overhang explicit
inherited_removed base_kept absent`. Regions: `page-crop.body`,
`page-crop.margin.{left,right,top,bottom}`, `page-crop.whole`,
`page-crop.apply`.
