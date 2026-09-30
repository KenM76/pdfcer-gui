# `ui-verify/checks/page_tabs_chooser`

`page_tabs_chooser` — **a page's tab order is chosen in the Tab-order section,
written to the page, refused where the file's version cannot hold it, and
removable again.**

## What it guards

Each page block in the Forms panel's Tab-order section carries a "Tab order:"
combo (`forms.tab_order.tabs.<page>`, options `….<absent|R|C|S|A|W>`). A
change dispatches `FieldAction::SetPageTabs`, which calls
`EditSession::set_page_tabs` through the edit funnel. `/A` and `/W` are PDF 2.0
values; the engine refuses them on an older file and the refusal reaches the
status line through the funnel.

## How it drives

On the engine's `forms/demo-form.pdf` (PDF 1.7, one page, no `/Tabs`), with
the window maximised, Edit mode, the Forms pane enlarged and the Tab-order
section opened:

1. Chooses Columns. Requires `page-tabs-set page=0 before=absent after=C` and
   the listing's `forms-tab-page page=0` line to read `tabs=page:C`.
2. Chooses "This list (PDF 2.0)". Requires `set-page-tabs-refused`, no
   `page-tabs-set` after it, and the page still `page:C`.
3. Chooses Not stated. Requires `tabs=absent`.

## Not covered

- Rows, Tag structure and "Fields in this list, then the rest".
- A PDF 2.0 file, where `/A` and `/W` are accepted.
- An inherited `/Tabs` (the combo shows it as Not stated).

## Falsification

Passing `PageTabs::Row` to `set_page_tabs` in place of the chosen value fails
step 1: the listing reads `tabs=page:R`.
