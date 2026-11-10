# `dialogs::redact_outside` — Redact outside the region

Marks everything outside the snapshot box for removal, on the pages chosen.

## Wording

The warning is the operator's text verbatim: *"This permanently removes all
text, images, vector graphics and annotations OUTSIDE the selected area on N
pages. Only the selected area remains."* N is the number of pages that will
actually be marked, not the number chosen.

## Pages

`PageScope`, defaulting to the region's own page. The region's rect is used
only on pages whose MediaBox is the region page's size within 0.5 pt, and is
moved by each page's MediaBox origin so it keeps its place on the paper. A
page of another size is left alone and counted in the window and on the
status line; one page's rect is never laid on a sheet it was not drawn on.

## Preview

While the window is open the canvas hatches the outside of the box
(`preview_active`). The hatch is a pre-commit affordance, like the box
itself: it is drawn only while choosing and never over applied content. It
is keyed on a frame stamp, so a window dropped without being drawn (its
document closed) stops the hatching on the next frame.

The window closes itself if the box goes away.
