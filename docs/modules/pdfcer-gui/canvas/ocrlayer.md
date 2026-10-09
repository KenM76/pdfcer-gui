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

The text's half is drawn by [`draw_text`] through `canvas::ocrink`, which
renders the page's invisible text with the engine's renderer in the layer's
colour, so the layer and an edit of it are one drawing.

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

## Nothing is skipped

This is a **view**, and the pages it exists for are the dense ones, so it has
no cap: the engine renders every invisible run in the visible region, at the
screen's density. A run too small to resolve into letters renders as the
renderer renders any text that small. This module therefore owes no "some were
not shown" disclosure.

## The run being edited is left to the editor

While a text edit is open on an invisible run and `textedit::shaped` has laid
it out, that preview draws the run in its own font, in this layer's colour
(`colour32`) and at its opacity, and the layer's raster is drawn with that
run's rectangle cut out (`edited_rect`): drawing both would show the old and
the new text on top of each other. The rectangle comes from the
provenance-bearing extraction (`OpenDoc::provenance_page_text`), whose run
indices the editor uses. When the preview has fallen back to the stand-in box,
nothing is cut out. `ocr-layer-held runs=` traces whether a run is left to the
editor.

## One page

[`crate::app::state::OpenDoc::page_text`] caches the **current page only**.
Under a continuous strip the overlay therefore draws on that page and not
on its neighbours. The drawing is still routed through that page's own
[`crate::canvas::strip::PageView::map`] rather than the acting page's —
they are the same page today, and using the acting map would be the exact
failure `painting`'s find-wash comment records as the one most likely to
ship silently, waiting for the day the cache learns a second page.

## Item notes

### `const COLOUR_KEY`

Two homes, exactly as `canvas::chunks` has: the live answer here, because
the painter reaches it with a [`egui::Context`] and nothing else, and the
persisted answer on [`crate::app::prefs::Prefs`], because O229 asks for the
setting to be remembered. `crate::app::frame` mirrors the second into the
first once a frame, in that direction only.

### `fn paper`

The same value and the same argument as `canvas::shapes`' — PDF has no page
background, and `render_page` composited this raster onto an opaque white
backdrop per §11.4.7. Fading the raster towards anything else would fade it
towards a colour the renderer never used, and the page would change hue on
its way to blank.

### `fn the_slider_stops_are_no_paint_and_full_paint`

A build that inverted the slider satisfies neither: it would paint the
veil at the left stop, which is the position that means *show me the
scan*.

### `const DEFAULT_COLOUR`

Chosen to be a colour a **scan is unlikely to contain**. The overlay's
whole job is to be told apart from the marks under it, and a scanned
drawing is black, grey and — on a CAD sheet — often blue or red. Magenta is
in none of those families, so the default works before anybody has thought
about it, which is what a default is for.

### `fn sync`

The guard is not an optimisation. An unconditional write every frame would
make the value impossible to change from anywhere else, which is how a
mirror becomes an overwrite — `canvas::chunks::sync` carries the same note.

### `fn is_ocr_run`

`ExtractedGlyph::invisible` is the whole selector: the engine sets it for
text rendering modes 3 and 7, which is what an OCR producer writes and what
a page's own lettering never is.

`any`, not `all`, and the difference is a silent omission. A producer
that flips the rendering mode mid-run leaves a run with both kinds of
glyph. Taking it draws some already-visible letters a second time, which
the operator can see and dismiss. Refusing it hides recognised text with
nothing on screen to say so — and *nothing on screen* is the failure mode
this whole feature exists to end.

### `fn painted_fraction`

A **non-finite input paints nothing**, where
[`crate::viewer::normalise_ocr_overlay`] answers the same corruption with
the default position. The two are not inconsistent: that one answers *where
did the operator leave the slider*, and a lost preference should land
somewhere useful; this one answers *how opaque is this stroke*, and a
number nobody can account for must not end up drawn over the document.

Which is exactly why this is public and `app::status::ocrlayer` reads
it rather than the raw field. A disclosure that quoted the other normaliser
would report 65 % on the one input where the painter draws nothing — a
sentence describing a blend that is not on screen, produced by two
functions that each behave correctly.

### `fn text_alpha`

Equal to [`veil_alpha`] by construction rather than by coincidence: the two
halves of one slider are one number, and the left stop has to draw *no*
text rather than faint text for the same reason the right stop has to blank
the scan.

### `fn draw_veil`

Called before the grid: this is about the sheet, and everything drawn on
the sheet has to win.

Only over pages that actually have a raster this frame, and only over
[`PageView::paint_rect`] — the rectangle the texture is a picture of. A
page still waiting on its pixmap is left alone rather than veiled, because
veiling nothing would put a white rectangle over whatever the strip is
showing in its place.

### `fn draw_text`

The colour is read from the painter's own context rather than passed in, so
the `NOT A THEME COLOUR:` argument stays beside the value it is about and
`painting` does not have to carry a colour it makes no decision on.
