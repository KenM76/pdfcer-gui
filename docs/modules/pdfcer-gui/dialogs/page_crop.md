# `dialogs::page_crop` — the visible area of the chosen sheets

`pages.crop`, Pages ▸ Transform ▸ Crop…, opened by `app::dispatch::pages` over
`page_operands` (picked sheets, else the current one); and
`pages.crop_to_region`, opened by `PageCropDialog::open_at` with the margins
seeded from the marquee region. Commits `PageAction::SetPageBoxes`, whose body
is `app::actions::pagesize::crop` (`EditSession::set_page_boxes`, one undo
entry, one `PageBoxEdit` per page with `PageBox::Crop`).

## Contract

| Item | Contract |
|---|---|
| Margins | Whole millimetres, ≥ 0, one per edge **as the operator sees the sheet**. Converted to page space per page with `Edge::pdf_index(rotate)` against that page's own `/Rotate`. Seeded from the first operand's `media − crop`, or from the region's rect for Crop to region. |
| Scope | This page / the picked pages, All, Odd (1, 3, 5…), Even (2, 4, 6…), or a typed range; the page count shows under it. An empty or unparsable range offers no Crop button (R9). |
| Per page | Each page gets the margins inset from **its own** media box, so a mixed-size or mixed-rotation document crops each sheet the same distance in from the edges as he sees them. |
| Commit | All zero → `CropBoxEdit::Reset` for every page in scope. Otherwise `CropBoxEdit::Set(rect)` per page. |
| Nothing left | Margins leaving ≤ 1 pt on any page in scope: the refusal line replaces the summary and the Crop button is absent, so `EditError::PageBoxEmpty` is not reachable from here; `EditError::PageBoxRepeated` cannot arise because the scope lists each page once. Either refusal still reaches the status line verbatim through `vector_edit`. |
| Disclosure | The note says crop hides content outside the box and does not remove it, and names Redact outside the region as the way to remove it. |

## Why margins and not a rectangle

The operator crops *off* an edge ("lose the 10 mm border"); a rectangle in
page coordinates would ask him for the origin and the `/Rotate` convention.
Margins seen on screen are the Acrobat Crop Pages shape, and they carry across
sheets of other sizes where one rectangle would not.

## Trace and regions

`page-crop-opened sheets rotate margins_mm`, `page-crop-commit n scope set
reset margins_mm llx lly urx ury` (the rect is the first page's), and from the
action `page-crop-applied n overhang explicit inherited_removed base_kept
absent`. Regions: `page-crop.body`, `page-crop.margin.{left,right,top,bottom}`,
`page-crop.scope.{operands,all,odd,even,range}`, `page-crop.range`,
`page-crop.whole`, `page-crop.apply`.
