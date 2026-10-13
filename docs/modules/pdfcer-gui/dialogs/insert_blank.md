# `dialogs::insert_blank` — how many blank sheets, what size, where

The window behind Pages ▸ Insert ▸ **Insert blank…** (`pages.insert_blank`).
It asks three things and raises one `PageAction::InsertBlankPages`:

| control | values | default |
|---|---|---|
| Size | *Same as page N — W × H mm*, then every `PaperSize::ALL` entry | Same as the page it was opened on |
| Portrait / Landscape | shown for a standard size only | Portrait |
| How many | 1 to 500 | 1 |
| Where | after page N, before page N, at the start, at the end (`dialogs::insert_where`) | After the current page |

Acrobat's *Insert Pages ▸ Blank Page* asks the same: size, count, and where.

## Same as page N

The default sheet is the page the dialog opened on, **as it displays**: its
crop box, with width and height swapped when `/Rotate` is 90 or 270
(`displayed_size`). An operator adding a sheet to a landscape drawing gets a
landscape sheet without choosing one. The orientation pair is hidden for this
choice because the shape is the page's own.

The current page is frozen at open, for the reason `dialogs::insert_pages`
gives: "after page 7" must not become "after page 9" because something else
moved the view while the window was open.

## How the sheets are made

The engine has no blank-page verb. `app::actions::insertruns::insert_blank`
builds a one-page blank document of the chosen size
(`pdfcer_gui_base::blank::document_sized`) and inserts its page `count`
times through the same `insert_from_view` funnel every page insert uses, so
the receipt, the landing on the first new sheet and the single undo step are
the same as Insert from file's. See `app/actions/insertruns.md` for why a
repeated source page needs one engine call per copy.

## Region names, for `ui-verify`

`insert-blank.body`, `insert-blank.size` (the closed combo),
`insert-blank.size.item.N` (`0` is *Same as page N*, then `PaperSize::ALL`
from `1`), `insert-blank.landscape`, `insert-blank.count`,
`insert-blank.where.0..3`, `insert-blank.insert`. The press traces
`insert-blank-dialog choice= landscape= n= w_pt= h_pt= position=`.

## Gating

Edit mode only: `app::dispatch` declines the command with
`reason=mode-cannot-edit-content` elsewhere, and the item is greyed on
`doc.pages` with no document open.
