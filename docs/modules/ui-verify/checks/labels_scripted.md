# `ui-verify/checks/labels_scripted`

`page_labels_without_the_mouse`: Pages ▸ Stamp ▸ Number pages… on
`fixtures/four-pages.pdf`, off the desktop and driven only through
`ScriptedPointer`, so it runs under `--no-input`.

## Steps

1. `ribbon.mode.review`, `ribbon.tab.pages`, then `ribbon.item.pages.labels`
   (through the collapsed Stamp group when the band is narrow). The first
   `labels-opened` must carry `ranges=0 first=0 last=3`: nothing picked means
   the whole document.
2. `labels.style.1` (i, ii, iii), then `labels.apply`. `labels-commit` must
   carry `style=LowerRoman last=3`, and the edit funnel must write
   `page-labels-set`.
3. Reopened, `labels-opened` must carry `ranges=1`: the label cache refreshed
   with the edit epoch. *Remove all labels* must be drawn; its absence is
   reported as a failure, not a harness error.
4. `labels.clear`: the funnel writes `page-labels-cleared`, and the window
   opened after it reads `ranges=0`.

## Falsification

Keying the label cache on "built once" instead of the edit epoch turns step 3
red: the reopened window reads `ranges=0` and offers no Remove button.

## What it does not cover

- Pixels: the captions and the page box text.
- Typing a label into the page box (its resolution is unit-tested in
  `app::status::page_box`).
- Prefix and start fields, and the rail-pick scope.
