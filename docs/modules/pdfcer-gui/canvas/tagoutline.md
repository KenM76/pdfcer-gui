# `canvas::tagoutline` — the outline of the picked tag's content

A selection affordance, not content marking: the theme's selection ink, a
2 px stroke just outside each box, on every drawn page the picked element
spans, each mapped through its own page. It is the cursor's kind of mark —
it exists only while something is picked in the Tags panel and is never
saved. Nothing is drawn once the document is edited, because the boxes were
read against the page as it was. Region `canvas.tag-outline.<page>`,
1-based, per box drawn.
