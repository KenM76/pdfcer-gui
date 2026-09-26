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
