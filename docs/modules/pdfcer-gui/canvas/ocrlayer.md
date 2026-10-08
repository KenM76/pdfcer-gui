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

The text's half is drawn by [`draw_text`] in each word's own font through
`canvas::ocrink`, which lays the page out with the text editor's preview, so
the layer and an edit of it are one drawing. A run `ocrink` has not laid out
yet, or cannot, is drawn by the stand-in here: the run's text in the interface
font, fitted to its box, laid out every frame.

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

## The run being edited is left to the editor

While a text edit is open on an invisible run and `textedit::shaped` has laid
it out, that preview draws the run in its own font, in this layer's colour
(`colour32`) and at its opacity, and this module skips the run
(`edited_run`): drawing both would show the old and the new text on top of
each other. The layer reads the provenance-bearing extraction
(`OpenDoc::provenance_page_text`), whose run indices the editor uses, so the
run is matched by index. When the preview has fallen back to the stand-in
box, nothing is skipped. `ocr-layer-held runs=` traces how many runs were left
to the editor.

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

### `fn draw_run`

The size is taken from the box's **height** and then corrected by a
measurement of the laid-out width, which is at most two layouts and usually
one. Taking it from the glyph metrics instead would mean reproducing the
text matrix here; taking it from the width alone would make a two-word run
and a twenty-word run in equal boxes render at wildly different sizes.

### `fn the_slider_stops_are_no_paint_and_full_paint`

A build that inverted the slider satisfies neither: it would paint the
veil at the left stop, which is the position that means *show me the
scan*.

### `fn every_size_is_a_multiple_of_the_quantum`

The property the atlas argument rests on, asserted over a walk rather
than at two chosen points: a rounding that worked at 12.3 and failed
near the clamps would pass the test above.

### `const MIN_FONT_PX`

Below it a run is drawn as a filled box instead — see the header. The value
is where a proportional face stops resolving into distinguishable letters
on a 96 dpi display; under it the glyphs are a smudge that costs a layout
and reads as noise, and a solid bar reads as *text, too small*, which is
what is true.

### `const MAX_FONT_PX`

⚠ A ceiling on the **font atlas**, not on the design. A run's box grows
without bound as the operator zooms, and egui rasterizes a glyph per
(face, size): asking for a 4,000 pt face once is a multi-megabyte atlas
upload in the middle of a zoom gesture.

The visible consequence is that past roughly this size the overlay text
stops growing with the page while the scan under it keeps growing. That is
a real divergence and it is stated here rather than hidden: it begins at a
zoom where one run fills the window, which is far past any zoom at which
two layers are being compared.

### `const FONT_SIZE_QUANTUM_PX`

Not cosmetic. Every distinct size is a separate set of rasterized glyphs
in egui's atlas, and a page of OCR runs has as many distinct box heights as
it has runs. Rounding collapses a sheet's worth of near-identical sizes
onto a few dozen shared ones, so the atlas holds a face-sized set rather
than a page-sized one, and a zoom re-uses what the last frame uploaded.

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

`clip` culls: a run whose screen rectangle misses it is never laid out.

The colour is read from the painter's own context rather than passed in, so
the `NOT A THEME COLOUR:` argument stays beside the value it is about and
`painting` does not have to carry a colour it makes no decision on.
