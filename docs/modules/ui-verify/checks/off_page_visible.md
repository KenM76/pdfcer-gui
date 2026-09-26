# `ui-verify/checks/off_page_visible`

`an_object_off_the_page_is_actually_drawn` — **O23's "see" half, with a
screenshot as its oracle.**

# The report


> *"also objects should still be reachable even if they are off the page."*

and, three weeks later and after the *reach* half had shipped:

> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

Two verbs in one sentence, and they were built separately because they
break separately:

| half | check | what it proves |
|---|---|---|
| **reach** | `off_page_press` | a press in the grey becomes a gesture, so the object can be banded, selected and dragged |
| **see** | **this file** | the object is *painted*, so the operator knows it is there at all |

`off_page_press`'s own header states the boundary verbatim — *"it does not
claim the operator can SEE the object"* — and that sentence is what this
check discharges.

# Why a screenshot, and why nothing else would do

`D:/dev/rag/egui/` carries the rule this obeys: **layout and clipping
defects have exactly one oracle, and it is a rendered screenshot.** The
whole defect here is a *pixmap size* — `pdfcer_render::render_page` sizes
its pixmap to the `/CropBox`, so nothing culls the off-page square; there
are simply no pixels out there to put it in. Every layer above that is
innocent and reports success:

* the decomposer lists the square (`render::offpage` asserts the content
  union includes it);
* the hit test finds it (`off_page_press` asserts a band takes it);
* the render worker returns a pixmap with no error;
* the canvas paints that pixmap at the right rectangle.

⇒ A trace-only check would have been green for the entire three weeks the
operator could not see his object. So this one counts **ink in the
capture**, and the trace line below is the *corroboration*, not the
assertion.

# The two assertions, and why neither is sufficient alone

1. **`canvas-halo tier=halo`** — the shell decided to widen the raster, and
   the box it widened to reaches left of x = 0. Without this, a passing
   pixel test could be measuring the wrong square, a stale capture, or a
   window that happens to have something dark at that address.
2. **Ink where the off-page square is, and paper where it is not** — the
   pixels. Without this, `tier=halo` would say only that the shell *asked*
   for a bigger raster; the engine could still have clipped it, the texture
   could be placed at the wrong rectangle, or the image could be drawn
   under the backdrop.

The **pair** is the point, and the second half of the pair is the control
this suite has learned to insist on: *"a uniform failure at every rung of a
sweep is about the probe."* A patch of the halo that is inside the widened
raster but outside the square must come back as **paper**. If both patches
read dark, the probe is aimed at something other than the page — a panel, a
shadow, the desktop — and this check says so rather than reporting a pass.

# The fixture

`fixtures/off-page-object.pdf`, shared with both sibling checks, because two
fixtures for one property is two chances for one of them to stop having it.
A 200 × 200 page, no `/CropBox`, two black filled rectangles:

```text
0 0 0 rg
40 40 60 60 re f          <- A, on the page
-160 100 120 40 re f      <- B, ENTIRELY left of the media box
```

So the content union is x −160…200, y 0…200 and the halo box must be that.
B's centre is `(−100, 120)`; the paper control is `(−100, 40)`, which is
60 pt below B, still 100 pt left of the sheet, and therefore inside the
widened raster and outside every mark in the file.

# Why `view.zoom_actual` and not fit-page

The same reason `off_page_press` gives, and it was measured there: fit-page
on a 200 × 200 fixture in a maximised window puts the sheet at roughly
3.8 px per point, so x = −100 lands about 381 px left of the sheet where
only ~243 px of viewport exists, and the conversion refuses — correctly —
and the check SKIPS **silently, because a SKIP is not red**. At 100 % the
sheet is ~200 px wide in a ~1250 px viewport and the whole halo fits with
room to spare. 100 % is also a property of the DOCUMENT rather than of the
window, so this check's geometry no longer varies with the screen it runs
on.

# Every way this reports SKIP

No binary, no diagnostic channel, no `canvas-viewport` region, not enough
grey on screen to reach x = −100, or a capture that could not be taken.
**Not** "the halo never engaged" and **not** "there was no ink" — those are
failures, and they are the two this check exists to find.

## ⚠ And one deliberate non-skip: `known=false`

`canvas-halo known=` reports whether the page had been decomposed when the
tier was decided. `OpenDoc::content_bounds_if_known` peeks and never
builds — a build costs 469 ms on the operator's own drawing and the canvas
runs every frame — so on the first frame of any document the answer is
`false` and there is no halo. `render::settle` builds it immediately after,
so by the time this check reads anything it must be `true`.

⇒ `known=false` in the **last** line is therefore a real failure with a
precise cause (the settle-time build is not running), not a timing wobble,
and it is reported as one.

## Item notes

### `const INVOKE`

`mode.edit` is named FIRST, and it is not decoration. Since
2026-09-11 the display of off-sheet content is a per-mode preference and
**Read ships with it OFF** — the operator's request: *"by default, read
doesn't show off page items, review and edit do show off page items."*
This check's whole subject is off the sheet, so without an explicit mode it
would run in whatever mode the shell opens in, find nothing, and report a
defect that is a correctly-implemented setting.

Edit rather than Review because that is the mode this check's gestures
belong in anyway, and because a mode named explicitly cannot drift when a
later session changes which mode the shell opens in.

### `const PATCH_PT`

The square is 120 × 40 pt, so ±8 pt about its centre stays 52 pt clear of
its left and right edges and 12 pt clear of its top and bottom. The paper
patch is the same size so that the two counts are directly comparable — a
ratio between differently sized samples is a number nobody can read.

### `const INK`

The fixture fills with `0 0 0 rg`, so true ink is `#000000`; the slack is
for the capture's colour management and for antialiasing at the patch's
edge, which cannot reach the middle 16 × 16 pt of a 120 × 40 pt rectangle.

### `const INK_FRACTION`

Deliberately not 1.0: the patch is converted through the window frame's
scale and rounded, so its outermost row can land a pixel outside the square
on a fractional-DPI display. Deliberately not 0.1 either — a tenth of a
patch is what a scroll bar or a tooltip edge could contribute.

### `fn ink_fraction`

Returns `None` when the patch has no area in the capture at all — which is
a finding, never "no ink": `WindowFrame::logical_to_capture_pixels` clamps
to the capture, so a zero-area result means the address is off screen.
