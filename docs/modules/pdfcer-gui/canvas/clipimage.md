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

## Where the pixels come from, and why not the page

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

## Why it composites onto white

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

## Item notes

### `const TARGET_EDGE_PX`

1,600 is chosen against where these end up: pasted into an email, a
report or a chat message, then usually scaled down. It is generous enough
that a screen-sized paste is not visibly resampled and small enough that the
clipboard payload for an ordinary selection stays in single-digit megabytes.

### `const MAX_EDGE_PX`

A separate number from the target, deliberately: the target is a *quality*
choice and this is a *safety* one. A selection 4,000 pt wide would ask for a
25× scale to hit the target on its short edge, and this is what stops it.

### `const MIN_SCALE`

Below 1.0 the picture would be smaller than the selection is in points,
which is never what somebody copying an object wants — they can always
scale it down where they paste it, and cannot scale it up.

### `fn scale_for`

`None` for a degenerate clip — the engine substitutes a 1 pt page for a
zero-extent selection and discloses it, and a 1 pt page rendered at any
scale is not a picture worth putting on a clipboard.

### `fn on_white`

# Why premultiplied is the input

Because that is `tiny_skia`'s contract and therefore `pdfcer-render`'s: the
pixmap data is premultiplied RGBA8 and is handed over unchanged, which is
the same buffer and the same contract `render::worker` consumes. Treating it
as straight alpha would double-darken every edge, which reads as a picture
with a dirty outline rather than as a bug.

With premultiplication, compositing over white is one subtraction per
channel: `out = src + white × (1 − a)`, and `src` already carries its own
alpha factor.

### `fn the_scale_fills_the_target_and_stops_at_the_ceiling`

The two ends, asserted as magnitudes rather than as relations. A
relational assertion — "the scale is bigger for a smaller clip" — is
satisfied by any absurdity in the right direction, so it stays green
while the numbers are wrong.

### `fn a_degenerate_clip_produces_no_picture`

The engine substitutes a 1 pt page for a zero-extent selection and says
so; a 1 pt page is not a picture, and putting one on the clipboard would
replace whatever the operator had there with a dot.

### `fn half_transparent_red_becomes_pale_red_not_dark_red`

A 50 %-alpha red pixel is `(128, 0, 0, 128)` premultiplied. Over white
it must come out `(255, 127, 127)` — a pale red. Treating the input as
straight alpha would give `(191, 127, 127)`, a *darker* pale red, and
the difference is exactly the "dirty edges" symptom that makes a pasted
picture look subtly wrong without looking broken.

### `fn publish`

Returns the picture's size in pixels when it reached the clipboard, and
`None` when it did not — a caller should treat `None` as *"the internal
clip is there, the picture is not"* and say nothing to the operator about
it, because the copy they asked for did happen.

`text` is not decoration. `egui-winit` only produces a paste event when
the OS clipboard holds non-empty text, so writing a picture alone would stop
`Ctrl+V` arriving in this application at all. The two travel together, in
one clipboard transaction, and `native_window::clipboard`'s header carries
the measurement.
