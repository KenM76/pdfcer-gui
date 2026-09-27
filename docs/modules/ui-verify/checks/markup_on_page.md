# `ui-verify/checks/markup_on_page`

`markup_draws_into_page_content`: O246, drawing a Review shape into the page's own content.

## What it drives

1. Review mode, Markup tab. It clicks `markup.style.target`, the pen's
   **On page** switch, and requires the next `markup-pen` line to read
   `on_page=true`.
2. It arms the rectangle and drags it on page 1.
3. It passes only if no `add-markup` (annotation) line followed, and
   `markup-as-content-applied` reports a non-empty `objects=S..E` range.

## What it tells apart

- **No switch:** fails at step 1 with the declared markup regions. This is
  how the published build fails.
- **Switch ignored at commit:** the annotation line appears.
- **Engine refusal:** `add-markup-as-content` ran but nothing was applied. The
  message says so.
- **An empty commit:** a zero-width object range.

The object range is the engine's own report (`MarkupContentOutcome::objects`),
so the check does not depend on how many objects a rectangle becomes.
