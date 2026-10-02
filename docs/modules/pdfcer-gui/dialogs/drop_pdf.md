# `dialogs/drop_pdf` — what a dropped PDF is for

A PDF dropped on the window while a document is open is ambiguous in a way a
picture is not: the operator may want to read it, to add its sheets to this
set, or to put one of its sheets on this page as artwork (a title block, a
logo, a detail). Every editor in the class asks; this window asks once.

## When it opens

`app::dropped::documents` opens it when the drop holds **exactly one PDF and
nothing else** and a document is open. Several PDFs, or a PDF dropped with
pictures or text files, open as before: a question about one file of many has
no good wording. With no document open there is nothing to insert into, so
the file opens.

## What it offers

| answer | offered when | raises |
|---|---|---|
| **Open it** — the default, drawn as the accent button, taken by Enter | always | `Action::Open`, as File ▸ Open |
| Insert its *N* pages after page *p* | the mode changes page content (`edit_content`) | `PageAction::InsertPagesFromFile` with every page, `InsertPosition::After(current)` — the Insert pages window's defaults |
| Place its first page here, as artwork | the mode authors markup (`author_markup`) and the drop point has a page | `Action::CommitTextAnnot` with a `CustomStamp` naming page 0 of the file, through `customstamp::place` |
| Cancel | always | nothing |

An answer the mode does not offer is absent, not greyed. When neither insert
nor place is offered (Read), there is nothing to ask and the drop opens.

## Reading the file once

`DropPdfDialog::read` loads the file once for its page count and first page's
crop box. A file that will not load or has no pages yields `None`, and the
drop opens it so the open path's own refusal reaches the operator in its usual
words.

## Placement

The rectangle is the first page's crop box at natural size, centred on the
drop point and kept on the page by `clippaste::rect_at`, exactly as a dropped
picture lands. A full sheet dropped on a sheet of the same size therefore
covers it; a small one lands where it was dropped. The engine verb
(`place_page_artwork`) maps the crop box onto the rectangle, so the aspect
ratio is preserved.

## Trace

- `drop-pdf-asked insert=<bool> place=<bool>` when the window opens.
- `drop-pdf-chosen choice=open|insert|place n=<pages>`, with
  `page= llx= lly= urx= ury=` for place.

Regions: `drop-pdf.body`, `drop-pdf.insert`, `drop-pdf.place`, and the host
footer's `dialog.buttons.accept` / `dialog.buttons.cancel`.
