# -*- coding: utf-8 -*-
"""Build `fixtures/transparency-cmyk.pdf` — an A4 page that declares a
**subtractive transparency group** and then actually uses transparency, so the
renderer allocates a colorant (CMYK) compositing buffer for it.

## Why this fixture exists

`tools/ui-verify/src/checks/blend_space.rs` —
`BlendSpaceFallbackIsDisclosed`, the driven assertion for the operator's report
of 2026-08-26 (*"seems I get different results depending on Zoom level … up to
474% they are mismatched, but at 579% they match"*) — **SKIPPED on the full
sweep of 2026-09-11**, with this message:

    the zoom passed the 183% crossing and the renderer never engaged the CMYK
    compositing buffer at all — no `raster-blend-space` line reports
    `cmyk_buffer=true`. That is a fact about the FIXTURE: a page with no
    transparency composites nothing … SKIPPED rather than failed. Point --pdf
    at a document that uses transparency.

The check's own verdict table makes the SKIP correct and deliberate:

    | `cmyk_buffer` seen true? | `refused` seen? | verdict |
    | no                       | no              | SKIP — nothing to measure  |
    | yes                      | no              | PASS — the ink survived    |
    | yes                      | yes + disclosed | PASS — fallback declared   |
    | yes                      | yes, undisclosed| FAIL — the reported defect |

★ That SKIP is not a nuisance: an earlier revision of this check **reported
FAIL for a line-work drawing that has no transparency anywhere on it**
(`SW41177.pdf`, run of 2026-08-26), saying *"the page's colours have changed
and nothing on screen says so"* about a page whose colours had not changed.
The three-way table is the repair. But the repair left the check with nothing
in `fixtures/` that can put it in row 2 or row 3 — ⇒ this file.

## ★★★ THE TRIGGER IS THE DECLARED GROUP COLOUR SPACE, NOT "TRANSPARENCY"

This is the part that is easy to get wrong, and the reason this header is long.
Measured in the engine on 2026-09-11, against the pin in `Cargo.lock`:

* `pdfcer-render/src/lib.rs:649` — **the switch**:
  `let mut cmyk = if page_space.is_subtractive() { CmykBuffer::new(..) } else { None }`.
  Its own comment: *"A colorant buffer is engaged ONLY for a page whose group
  declares a subtractive blending space."*
* `pdfcer-render/src/lib.rs:804` — `diagnostics.cmyk_buffer_engaged = true` is
  set **only inside `if let Some(buffer) = cmyk`**. That flag is what the
  shell's `raster-blend-space cmyk_buffer=…` trace line reports, and it is what
  the check reads.
* `page_space` comes from `interpret::page_blend_space(..)`
  (`pdfcer-core/src/.../interpret.rs:2042`), which answers a page whose dict
  **declares** `/Group << /S /Transparency /CS … >>` from ISO 32000-1
  Table 147 **unconditionally** — no setting reaches that branch. Only an
  *undeclared* page falls through to the `PageBlendSpaceSource` policy
  (`DeviceNative` → Additive, `OutputIntentIfSubtractive`, `OutputIntentAlways`).

⇒ **A page that declares `/Group << /S /Transparency /CS /DeviceCMYK >>`
engages the colorant buffer whether or not one drop of alpha is ever painted,
and a page full of alpha that declares nothing does not.** Naming the fixture
after "transparency" alone would therefore be a name that does not say what the
file is for; it is named `transparency-cmyk` for that reason.

⚠ **A consequence worth stating, because a future session will hit it.** If
this fixture ever stops engaging the buffer, the first thing to check is not
the alpha — it is whether `/Group` is still on the **page** dictionary. A
`/Group` on a Form XObject is a *group* XObject and is a different clause
entirely; it does not choose the page's compositing space.

## Why it paints real transparency anyway

Because the declaration alone would make a fixture that is a lie in the
direction that matters. The check exists because the operator was comparing
**shading boxes** against a reference and they disagreed at some zooms and
agreed at others. What moved was the colour of composited patches. A fixture
that engages the buffer and then paints nothing into it composites nothing, so:

* the `to_srgb_over_white` collapse has no non-trivial pixel to convert,
* `bridged_pixels` / `groups_approximated` stay at zero, and
* the eventual *colour* assertion this check's header defers to the engine —
  *"it does not assert that the colours are right on either side of the
  ceiling"* — would have no subject when someone comes to write it.

So the page carries, in DeviceCMYK throughout:

| what | why it is here |
|---|---|
| four flat process patches (C, M, Y, K at 100 %) | a control: pure single-colorant fills, where an sRGB round trip is most visible |
| three **overlapping** pairs under `/BM /Multiply`, `/ca 0.55` | the actual composite — the operator's "boxes"; multiply is the blend mode a CAD/print workflow reaches for and is not a no-op in either space |
| an **axial shading** (`sh`) from rich black to paper | ★ the gradient. The 2026-08-26 measurement found that excluding non-flat cells "removed **the gradients**, which are the thing the operator is looking at." A fixture with only flat patches cannot reproduce what he reported. |
| a knocked-out white rule crossing the patches | a hard edge through the composite, so a resampling artefact and a compositing-space shift do not look alike |

`/ca` **and** `/CA` are both set (0.55), so both fill and stroke alpha are
non-trivial — a fixture that only set one would pass a renderer that had
dropped the other.

## Page size: A4, deliberately

The check computes the zoom at which the whole-page raster passes the engine's
`MAX_CMYK_BUFFER_BYTES` ceiling (256 MiB ÷ 20 B/px = **13,421,772 px**) from
the page's own geometry, so any size works. A4 is chosen because **every number
written in the check's header is an A4 number** — *"on an A4 page that ceiling
is crossed at 534 %… buffer used at scale 5.33 (13,394,232 px), refused at 5.34
(13,444,992 px)"*. A fixture of some other size would leave that header
describing a run nobody performs, which is the slow way a document goes stale.

The climb is well inside the check's `MAX_BATCHES = 40` at four notches a
batch.

## What is deliberately NOT here

**No `/OutputIntent`.** It would work — `OutputIntentIfSubtractive` would reach
the same buffer — but it would make the fixture depend on a **setting**
(`PageBlendSpaceSource`), so a future default change would silently turn this
fixture back into the SKIP it was written to end. The declared group is the
only route that no policy can reach.

**No spot colour / `/Separation`.** That is a distinct engine capability with
its own counters (`cmyk_unbridged_images`, `groups_approximated`) and it wants
a fixture whose whole subject is separations, so a failure there is not
attributed to this one.

**No ICC-based `/CS`.** `/DeviceCMYK` is subtractive by inspection; an
`/ICCBased` stream with `/N 4` is subtractive only if the reader looks at `/N`.
Both are worth testing and mixing them here would make one SKIP explain two
possible causes.

## Rebuilding

    python fixtures/transparency-cmyk.PROVENANCE.py

Offsets are computed, so edits are safe.
"""

import io

# A4 portrait, in points. See the header on why this size specifically.
W, H = 595.276, 841.89

# ---------------------------------------------------------------------------
# The content stream.
#
# Everything is authored in `/DeviceCMYK` with the `k` (fill) and `K` (stroke)
# operators, so no colour on this page needs a conversion *before* the page
# group is composited. That keeps the fixture's subject the COMPOSITING space
# rather than an operand conversion that would happen either way.
# ---------------------------------------------------------------------------

def cmyk_fill(c, m, y, k):
    return b"%.3f %.3f %.3f %.3f k\n" % (c, m, y, k)


def rect(x, y, w, h):
    return b"%.2f %.2f %.2f %.2f re f\n" % (x, y, w, h)


parts = []

# -- 1. Four flat process patches across the top. The control row: no alpha,
#       no blend mode, one colorant each. If these shift between the two
#       compositing paths, the shift is in the collapse and not in a blend.
parts.append(b"q\n")
for i, ink in enumerate(
    [(1.0, 0.0, 0.0, 0.0), (0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 1.0, 0.0), (0.0, 0.0, 0.0, 1.0)]
):
    x = 60 + i * 120
    parts.append(cmyk_fill(*ink))
    parts.append(rect(x, H - 180, 100, 100))
parts.append(b"Q\n")

# -- 2. Three overlapping pairs under /GSalpha (/BM /Multiply, ca=CA=0.55).
#       ★ THE ACTUAL COMPOSITE. Each pair is two squares offset by half their
#       width, so every pair shows three regions: left ink alone, right ink
#       alone, and the multiplied overlap. Three regions rather than two is
#       what lets a measurement tell "the blend moved" from "both inks moved".
PAIRS = [
    ((0.90, 0.10, 0.00, 0.00), (0.00, 0.85, 0.90, 0.00)),  # cyan  x red
    ((0.05, 0.95, 0.05, 0.00), (0.80, 0.00, 0.90, 0.00)),  # magenta x green
    ((0.00, 0.10, 0.95, 0.00), (0.75, 0.60, 0.00, 0.10)),  # yellow x blue
]
for i, (a, b) in enumerate(PAIRS):
    y = H - 360 - i * 150
    parts.append(b"q\n/GSalpha gs\n")
    parts.append(cmyk_fill(*a))
    parts.append(rect(60, y, 160, 120))
    parts.append(cmyk_fill(*b))
    parts.append(rect(140, y, 160, 120))
    parts.append(b"Q\n")

# -- 3. The axial shading, to the right of the pairs. See the header: the
#       2026-08-26 measurement established that excluding non-flat cells
#       removed the gradients, "which are the thing the operator is looking
#       at". A fixture of flat patches only cannot reproduce his report.
#
#       Clipped to a rectangle because `sh` paints the whole clip region.
parts.append(
    b"q\n"
    b"340 %.2f 200 420 re W n\n" % (H - 780)
    + b"/Sh0 sh\n"
    b"Q\n"
)

# -- 4. A knocked-out white rule straight across the composites. A hard edge,
#       so that a resampling artefact at a different zoom (soft, everywhere)
#       and a compositing-space shift (flat, inside the patches) cannot be
#       mistaken for one another by eye or by a box average.
parts.append(
    b"q\n0 0 0 0 K\n6 w\n40 %.2f m %.2f %.2f l S\nQ\n" % (H - 470, W - 40, H - 470)
)

CONTENT = b"".join(parts)

# ---------------------------------------------------------------------------
# Objects.
#
#   1 Catalog
#   2 Pages
#   3 Page      ★ /Group << /S /Transparency /CS /DeviceCMYK >>  -- THE TRIGGER
#   4 Contents
#   5 ExtGState  /BM /Multiply  /ca 0.55  /CA 0.55
#   6 Shading    type 2 axial, /DeviceCMYK
#   7 Function   type 2 exponential, rich black -> paper
# ---------------------------------------------------------------------------
objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    # ★★★ THE ONE LINE THIS FILE EXISTS FOR.
    #
    # `/Group` on the PAGE dictionary, `/S /Transparency`, `/CS /DeviceCMYK`.
    # `/I true` (isolated) is written explicitly: an isolated page group is
    # what §11.4.7's second formula composites over the medium's white
    # backdrop, which is the code path at `pdfcer-render/src/lib.rs:786-830`.
    # Leaving it to the default would make this fixture's path depend on a
    # reader's reading of the default rather than on the file.
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %.4f %.4f] "
    b"/Group << /S /Transparency /CS /DeviceCMYK /I true /K false >> "
    b"/Resources << /ExtGState << /GSalpha 5 0 R >> /Shading << /Sh0 6 0 R >> >> "
    b"/Contents 4 0 R >>" % (W, H),
    b"<< /Length %d >>\nstream\n" % len(CONTENT) + CONTENT + b"endstream",
    # Both `/ca` (non-stroking) and `/CA` (stroking) are set. See the header:
    # a fixture that sets one would pass a renderer that had dropped the other.
    b"<< /Type /ExtGState /BM /Multiply /ca 0.55 /CA 0.55 >>",
    # Axial shading, bottom-left to top-right of the clipped box.
    b"<< /ShadingType 2 /ColorSpace /DeviceCMYK "
    b"/Coords [340 %.2f 540 %.2f] /Function 7 0 R "
    b"/Extend [true true] >>" % (H - 780, H - 360),
    # Rich black (0.60 0.40 0.40 1.00) -> paper (0 0 0 0), linear.
    #
    # Rich black rather than /K 1 alone on purpose: a four-colorant endpoint is
    # where an sRGB round trip and a colorant composite disagree most, and a
    # gradient that ends in a single colorant would understate the effect the
    # operator reported.
    b"<< /FunctionType 2 /Domain [0 1] "
    b"/C0 [0.60 0.40 0.40 1.00] /C1 [0 0 0 0] /N 1 >>",
]

out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % i
    out += body
    out += b"\nendobj\n"

xref_at = len(out)
n = len(objects) + 1
out += b"xref\n0 %d\n" % n
out += b"0000000000 65535 f \n"
for off in offsets:
    out += b"%010d 00000 n \n" % off
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (n, xref_at)

path = "fixtures/transparency-cmyk.pdf"
io.open(path, "wb").write(bytes(out))

# The zoom at which a whole-page raster of THIS page passes the engine's
# 13,421,772 px colorant-buffer ceiling. Printed rather than asserted: it is
# the number the check climbs to, and a reader rebuilding the fixture at a
# different page size needs to see it move.
ceiling_px = 256 * 1024 * 1024 / 20
scale = (ceiling_px / (W * H)) ** 0.5
print(
    "wrote %s  %d bytes  A4  page group /CS /DeviceCMYK  ceiling crossed at %.0f%% zoom"
    % (path, len(out), scale * 100)
)
