# `dialogs::page_crop` — a page box of the chosen sheets

`pages.crop`, Pages ▸ Transform ▸ Crop…, opened by `app::dispatch::pages` over
`page_operands` (picked sheets, else the current one); and
`pages.crop_to_region`, opened by `PageCropDialog::open_at` with the margins
seeded from the marquee region. Commits `PageAction::SetPageBoxes`, whose body
is `app::actions::pagesize::crop` (`EditSession::set_page_boxes`, one undo
entry, one `PageBoxEdit` per page for the chosen `PageBox`).

## Contract

| Item | Contract |
|---|---|
| Box | Crop, Bleed, Trim or Art (`page_crop::boxes`). Choosing one reseeds the margins from that box on the first operand, as the engine resolved it (`Page::{crop,bleed,trim,art}_box`), so a page with no trim box starts from its crop box. Remove white margins is offered with Crop only: it crops. |
| Margins | Whole millimetres, ≥ 0, one per edge **as the operator sees the sheet**. Converted to page space per page with `Edge::pdf_index(rotate)` against that page's own `/Rotate`. Seeded from the first operand's `media − crop`, or from the region's rect for Crop to region. |
| Scope | This page / the picked pages, All, Odd (1, 3, 5…), Even (2, 4, 6…), or a typed range; the page count shows under it. An empty or unparsable range offers no Crop button (R9). |
| Per page | Each page gets the margins inset from **its own** media box, so a mixed-size or mixed-rotation document crops each sheet the same distance in from the edges as he sees them. |
| Commit | Crop with all zero → `CropBoxEdit::Reset` for every page in scope. Bleed, trim or art at zero → `Set(sheet)`: *Set to zero* makes the box the paper. Their **Remove** button commits `Reset` at once, so the box defaults to the crop box. Otherwise `CropBoxEdit::Set(rect)` per page. |
| Same margin on every edge | While ticked, editing one margin sets all four; ticking it copies the left margin to the others. |
| Back to the marked region | Present when opened by Crop to region; puts the margins back to the region's. Applies to whichever box is chosen. |
| Picture | Beside the margins: the first page in scope's sheet as seen (after `/Rotate`) with the box the margins describe outlined. The window's own diagram — the page view is not marked (R8b). |
| Nothing left | Margins leaving ≤ 1 pt on any page in scope: the refusal line replaces the summary and the Crop button is absent, so `EditError::PageBoxEmpty` is not reachable from here; `EditError::PageBoxRepeated` cannot arise because the scope lists each page once. Either refusal still reaches the status line verbatim through `vector_edit`. |
| Disclosure | The note says crop hides content outside the box and does not remove it, and names Redact outside the region as the way to remove it. |

## Why margins and not a rectangle

The operator crops *off* an edge ("lose the 10 mm border"); a rectangle in
page coordinates would ask him for the origin and the `/Rotate` convention.
Margins seen on screen are the Acrobat Crop Pages shape, and they carry across
sheets of other sizes where one rectangle would not.

## Remove white margins

Below the summary: **Keep around the ink** (whole mm, 0–500, default 0) and
the **Remove white margins** button. Pressing it closes the window and raises
`PageAction::RemoveWhiteMargins` on the same page scope as Crop; the margin
boxes above play no part. The measuring is
[`app::actions::whitemargins`](../app/actions/whitemargins.md). The group is
absent while the scope names no page.

## Trace and regions

`page-crop-opened sheets rotate margins_mm`, `page-crop-box box margins_mm`
(comma-separated, on choosing a box), `page-crop-commit n scope box set reset
margins_mm llx lly urx ury` (the rect is the first page's; `box` is
`crop|bleed|trim|art`), and from the action `page-crop-applied n box overhang
explicit inherited_removed base_kept absent` (`box` is the page-dictionary key,
e.g. `TrimBox`). Regions: `page-crop.body`, `page-crop.margin.{left,right,top,bottom}`,
`page-crop.scope.{operands,all,odd,even,range}`, `page-crop.range`,
`page-crop.whole`, `page-crop.apply`, `page-crop.white.keep`, `page-crop.white`,
`page-crop.box.{crop,bleed,trim,art}`, `page-crop.zero`, `page-crop.remove`,
`page-crop.equal`, `page-crop.region`, `page-crop.picture`. Driven:
`crop_window_sets_the_trim_box`.
