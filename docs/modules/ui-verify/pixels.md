# `ui-verify/pixels`

**The pixel oracle.** The half of this harness that catches D2.

## Why pixels are an oracle at all

Most defects have a state oracle: something is wrong in memory, and a test
that reads memory finds it. A whole class does not. Legibility, clipping,
occlusion and layout have exactly one honest oracle — what was actually
drawn — and no amount of asserting on the widget tree reaches them.

`DEFECTS.md` D2 is the canonical example, and it is worth reading closely
because it explains why this module is shaped the way it is. Every
collapsible section heading in the Settings dialog renders near-white on
light grey. The cause is one unset field: the theme assigns
`widgets.active.fg_stroke` a near-white and `widgets.active.weak_bg_fill`
the accent, but never assigns `widgets.active.bg_fill` — so widgets that
paint with `bg_fill` (`egui_tiles` tab buttons, `CollapsingHeader` headers)
get near-white text on a light background.

Two tests sit directly adjacent to that bug and neither could catch it:

* one checks `text` against `surface`/`panel`, and never tests the colour
  that is wrong;
* the other **asserts that the wrong colour stays light** — correctly, for
  its own stated purpose, because that colour also sits over the white page.

Both are testing the palette. The defect is in the *pairing*, and the
pairing only exists once something is drawn. So: read the picture.

## `contrast_at` — the algorithm, and why not min/max

The obvious implementation is "brightest pixel versus darkest pixel in the
region". It is wrong in both directions and would make this gate useless:

* **False pass.** One stray dark pixel — a window border clipped into the
  region, a scrollbar, a single antialiased corner of an unrelated icon —
  gives a blank region a 15:1 contrast. The check would pass on a region
  containing no text at all, which is exactly the D2 symptom.
* **False fail.** Antialiasing puts a continuum between foreground and
  background, so the extremes are unrepresentative of the glyph the reader
  actually sees.

So [`contrast_at`] works on **populations**:

1. Quantise every pixel to a 5-bits-per-channel bucket (32 768 buckets).
   Coarse enough to fuse antialiasing into its neighbours, fine enough to
   keep a real foreground and a real background apart.
2. The **background** is the most populous bucket. In any region containing
   text, most pixels are not text.
3. The **foreground** is the bucket, among those holding at least
   [`MIN_FOREGROUND_SHARE`] of the region, whose luminance is furthest from
   the background's. The share floor is what makes a single stray pixel
   unable to vote.
4. Each bucket reports its **mean colour**, not the bucket centre, so
   quantisation does not bias the measurement.
5. Contrast is the WCAG ratio `(L1 + 0.05) / (L2 + 0.05)`, from 1.0
   (identical) to 21.0 (black on white).

If no bucket clears the share floor, the region is uniform: foreground and
background come back equal and the ratio is 1.0. That is the correct answer
for a blank region, and it is why [`region_not_uniform`] exists as a
separate question — "nothing was drawn here" and "what was drawn is
invisible" are different failures with different fixes, and a check should
be able to tell the operator which one it found.

## Thresholds

WCAG 2.1 asks 4.5:1 for body text and 3:1 for large text. [`AA_LARGE`] is
this harness's default because ribbon group captions and dialog section
headings are short, styled strings where 3:1 is a defensible floor and is
not a matter of taste — it is a published standard, which is what stops a
failing check turning into an argument about whether the grey is nice.

For calibration: D2's headings measure around **1.1:1**. The threshold does
not need to be finely tuned to catch that; it needs to exist.

## Item notes

### `const MIN_FOREGROUND_SHARE`

0.5%: a 200×30 caption region is 6 000 pixels, so this is 30 pixels — about
one glyph stroke, and far more than any stray edge pixel. Raising it makes
the oracle blind to thin text; lowering it lets a scrollbar sliver vote.

### `fn is_uniform`

Two conditions, and both are needed. A gradient has many buckets and no
dominant one; a flat fill with a single antialiased pixel has two
buckets and a 99.99% dominant one. Only requiring "more than one
bucket" would call the second varied.

### `fn relative_luminance`

The gamma expansion is not decoration. A naive `(r+g+b)/3` says mid-grey
text on white has plenty of contrast; the perceptual curve says it does
not, and the reader agrees with the curve.

### `fn contrast_at`

See the module docs for the algorithm and for why it is not min/max. A
region with no pixels — off the edge of the image, or degenerate — reports
black on black at 1.0 with `sampled: 0`, and callers are expected to look
at `sampled` before reading the ratio as a verdict.

### `fn mean_luminance`

The companion to [`contrast_at`] for a different question. That one asks
*is this ink readable* and answers with a ratio between two quantised
buckets; this one asks *is this the same ink, weaker* and answers with the
mean over every pixel.

A translucent copy of a region composited over the same paper is lighter
than the original at every ink pixel and identical at every paper pixel, so
its mean rises - monotonely with the alpha, with no bucket boundary and no
threshold able to sit between the two readings. That is the property a
pre-commit affordance drawn as a tinted blit has to be measured on, and
neither [`contrast_at`] nor [`ink_run_into`] has it: both quantise, and a
tint small enough to leave every pixel in its own bucket moves neither.

`None` for a region with no pixels. A mean over nothing is not zero, and a
caller comparing two regions must be able to tell the empty case apart
from a black one.

### `fn is_text`

Three pixels, because two adjacent antialiased pixels are reachable on a
steep glyph edge and three are not — while the shortest thing an
operator would call clipped text is a lower-case x-height, which at this
project's smallest shipped size is seven.
