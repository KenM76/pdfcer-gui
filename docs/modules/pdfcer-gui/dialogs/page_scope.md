# `dialogs::page_scope` — which pages a run covers

One group, drawn by Recognise text and by Remove OCR text, so the two windows
answer "which pages" with the same four choices and the same range parser.

## The choices

| Scope | Pages |
|---|---|
| All pages (default) | every page |
| This page | the page shown when the window opened |
| Selected pages (n) | the thumbnail rail's selection at open; offered only when it is not empty (R9) |
| Pages: | a typed range, parsed by `dialogs::print::tabs::parse_page_range` — the print dialog's parser, so every range field in the program accepts the same text |

`PageScope::pages(count)` resolves the answer to zero-based ascending
indices, or `None` when it names no page. The caller greys its run button on
`None`: the operator is mid-way through typing a range, which is R9's
*temporarily* unavailable case.

## Captured at open

The current page and the rail's selection are both read once, when the
window opens. The operator can page the document or work the rail while the
window is up; a run that read either live would act on a set he had stopped
thinking about, and the label would change under him as he read it. A picked
page deleted since is dropped by `Scope::pages`, never handed on as an index
past the end.

## Typing is the choice

Focusing or editing the range field selects the range radio; nobody types a
range and expects the run to ignore it.

## Trace and regions

- The group's rect under the caller's region name, and each radio's under
  `<region>.all`, `<region>.current`, `<region>.picked`.
- `<tag> pages= first= last=` on every change of the resolved list — not per
  frame; a line per frame is a haystack.

Recognise text passes `ocr-scope` for both; Remove OCR text passes
`remove-ocr.scope` and `remove-ocr-scope`.
