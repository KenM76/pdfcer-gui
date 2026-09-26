# `canvas::ocrlayer` — the invisible text a scan carries, drawn

A scanned page is a picture. When it has been through OCR it is a picture
**plus** a layer of real text, positioned over the marks it was recognised
from and drawn in text rendering mode 3 — present, searchable, copyable,
and never visible. This module draws it.

`OPERATOR_REQUESTS.md` O226 asks for the page and that layer to be
comparable through a slider: at one end the scan as it is, at the other end
the text alone, and everything between.

## Two layers, two positions in the order

| slider | the page raster | the text |
|---|---|---|
| 0.0 | full opacity | not drawn |
| 0.5 | half | half |
| 1.0 | not drawn | full opacity |

The raster's half is **not a re-render**. Nothing here asks for a pixmap at
a different opacity; [`draw_veil`] paints paper over the texture already on
screen. Re-rasterizing per slider position would put a render request
behind a drag, and `OPERATOR_REQUESTS.md` O24 is this shell's record of
what that costs.

The text's half is **vector, laid out every frame** by [`draw_text`]. It is
never baked into a texture, because it must stay crisp at every zoom and
because a cached overlay is a second rendering path for content that has
exactly one.

The two sit at different places in `painting`'s layer order — the veil
under the grid, because it is about the *paper*; the text above the grid
and below the find wash, because it is page content and a search answer
outranks it. `painting`'s own table carries the argument.

## R8b — this does not mark the canvas

The rule forbids styling **applied content** as provisional. Nothing here
is applied content and nothing here is provisional: this draws text the
file already holds, in a mode the operator turned on and can turn off, and
with the mode off the module paints nothing at all. The one-line test
answers no — a screenshot of the page with `view.ocr_overlay` at `None` is
a screenshot of the saved document.

The colour is the operator's (`OPERATOR_REQUESTS.md` O229) precisely
because this is a *reading instrument* rather than a rendering of the page:
the text has to be told apart from the scan underneath it, and which colour
does that depends on the scan.

## Why there is no cap and nothing is skipped

`canvas::chunks` bounds its work with `MAX_CHUNK_BOXES` and draws nothing
past it. That is right for a *selection* — the operator can select less —
and wrong here, because this is a **view** and the pages it exists for are
exactly the dense ones. A cap would blank the feature on its own subject.

The cost is bounded instead, in two ways that omit nothing visible:

* **Runs outside the clip are not laid out.** They are not on screen, so
  nothing is lost by not drawing them.
* **A run too small to resolve into glyphs is drawn as a filled box** at
  the run's own rectangle — see [`MIN_FONT_PX`]. That is not an omission
  and not a placeholder: it is the same content at a scale where letters
  do not survive, and it says *there is recognised text here* truthfully.
  It also costs no layout, which is what makes a whole dense sheet at
  fit-page zoom affordable.

So this module owes no "some were not shown" disclosure, because there is
no state in which it does not show one.

## One page

[`crate::app::state::OpenDoc::page_text`] caches the **current page only**.
Under a continuous strip the overlay therefore draws on that page and not
on its neighbours. The drawing is still routed through that page's own
[`crate::canvas::strip::PageView::map`] rather than the acting page's —
they are the same page today, and using the acting map would be the exact
failure `painting`'s find-wash comment records as the one most likely to
ship silently, waiting for the day the cache learns a second page.
