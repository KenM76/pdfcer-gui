# `canvas::clipimage` — **the copied selection, as a picture other programs
# can paste**

## What this closes

The operator (`OPERATOR_REQUESTS.md` **O71**):

> *"In read mode the regular pointer should also allow us to select images
> so we can copy and paste them as well as text outside of the pdfcergui."*

**Outside** is the requirement. `canvas::clipboard` carries a rich internal
clip — an `ObjectClip`, a `MarkupSpec`, structure a bitmap cannot express —
which is the right payload for pdfcer→pdfcer work and meaningless to Word.
This module is the other half of the same copy: a picture any program can
paste.

## ★★★ Where the pixels come from, and why not the page

From **the clip's own one-page PDF**, not from a crop of the rendered page.

`ObjectClip::to_pdf` returns a standalone document whose `/MediaBox` is
exactly the selection's bounding box, with the content translated to the
origin, and it exists for exactly this consumer: a host shell putting a
selection on the operating system's clipboard. Rendering that gives:

| | clip PDF | crop of the page |
|---|---|---|
| resolution | **chosen** — the page is only as big as the selection, so 4× costs nothing | capped by what the whole sheet can be rendered at |
| contents | exactly what was selected | everything overlapping the box |
| correctness | the engine's own compositor, colour spaces, masks | the same, but with neighbours |

The second row is the deciding one. On a CAD sheet almost every object's
bounding box overlaps a dozen others, so a crop would paste a picture of the
neighbourhood and the operator would have to explain to themselves why.

⇒ A **snapshot** tool — Acrobat's, where the rectangle IS the request — is a
different feature and would rightly crop the page. It is not this one.

## ★★ Why it composites onto white

`CF_DIB` at 32 bits has no alpha channel consumers agree about
(`native_window::clipboard`'s header has the detail). Some read the fourth
byte, most ignore it, and one that ignores it renders a
composited-on-**black** picture — a black rectangle with a drawing in it,
which is what "pasting a PDF selection" looks like when this is got wrong.

White rather than transparent is also what the operator sees on screen: the
page is white, so the picture that arrives in Word matches the one they
copied. A checkerboard would be more honest about the alpha and less honest
about the document.

## The size cap, and what happens at it

A selection can be one glyph or a whole drawing, so the scale is chosen to
give a useful picture in both cases and then clamped so a careless
select-all cannot ask for a gigabyte. At the cap the picture is smaller than
ideal and still correct; nothing is cropped, because a cropped clipboard
picture is a wrong one and a small one is merely a small one.
