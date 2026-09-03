"""Generate `pdfcer-gui.ico` — the application icon, drawn from geometry.

WHY THE ICON IS A SCRIPT AND NOT A BINARY
=========================================

Operator request, 2026-08-18: *"make and add a pdf icon to the exe so it shows
as the icon when I associate it with pdfs."*

A `.ico` checked in as an opaque blob is art nobody can review, adjust or
re-derive: a reader who wants the badge two pixels lower has to open a paint
program and hope. Checked in as the script that draws it, every choice is
readable, every size is regenerated from one source of truth, and a change is a
diff rather than a replacement.

That also matters for **provenance**. `crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`
records the ribbon glyph set as the operator's own art, and `catalog.rs` states
the standing rule that follows from it: *"a new glyph is not a build session's
to add."* This icon is a different artefact — an application icon, asked for
directly — but the same care applies, and a generator makes its authorship
plain: it was written by a build session, it is geometry rather than drawing,
and the operator can replace it with real art at any time by dropping in a
`.ico` and deleting this file. See `crates/pdfcer-gui/assets/PROVENANCE.md`.

WHAT IT DRAWS
=============

The universal file-type idiom, which is what the request is actually about —
this icon's job is to be recognised in Explorer as *"the thing that opens
PDFs"*:

  * a portrait page with a folded top-right corner, in near-white on a thin
    grey outline, so it reads as a document at 16 px;
  * a red badge across the lower half carrying **PDF** over **CE** — the
    product's own name, split the way it is spelled. The second line was
    added on the operator's instruction, 2026-08-18: *"just add CE below
    PDF in the same red box."*

Red because that is the colour every PDF file-type icon has used for twenty
years and the one an operator's eye is trained on. It is deliberately NOT
Adobe's brand red and carries none of their marks: a red rectangle with three
letters in it is the generic convention, used by Chrome, Firefox, Preview,
Okular, SumatraPDF and every file manager.

Deliberately NOT pdfcer's theme accent. An application icon is not themed — it
sits in Explorer, on a taskbar and in a file-association dialog, none of which
know what theme the application is set to, and all of which show it beside
other applications' icons rather than beside pdfcer's own surfaces.

HOW THE LETTERS ARE DRAWN WITHOUT A FONT
========================================

There is no font renderer here and pulling one in for five letters would be
absurd. P, D, F and E are a vertical stem plus, at most, one bowl and two
bars; C is an ellipse ring with its right flank opened. Each is defined as a
handful of primitives in a unit square, and scaled.

The two lines share ONE cap height and are CENTRED rather than justified, so
`PDF` and `CE` have the same stroke weight and read as one lockup. Stretching
each line to the badge's width — which is what the single-line version did —
would give the two-letter line half again the stride of the three-letter one.

The result is a geometric sans that is legible at 16 px, which is the only
requirement. It is not a typeface and does not need to be.

ANTI-ALIASING
=============

Everything is rendered at `SUPERSAMPLE`x the target and box-filtered down. That
is the whole of the anti-aliasing: no coverage maths, no analytic edges, just
enough samples that a diagonal fold and a letter bowl come out smooth. At 256 px
with 4x supersampling that is a 1024x1024 evaluation, which takes a second or
two in pure Python and happens once, by hand, when the art changes.

THE `.ico` CONTAINER
====================

An ICO is a 6-byte header, then one 16-byte directory entry per image, then the
images. Each image may be a BMP or — since Windows Vista — a **PNG**, which is
what this writes: PNG is smaller, has real alpha, and avoids the BMP-in-ICO
trap where the height field must be doubled to account for a mask that modern
Windows ignores anyway.

Sizes: 16, 24, 32, 48, 64, 128, 256. Explorer picks per view mode; 16 and 32
are the ones an operator actually sees most (list view and the taskbar), and
256 is what the extra-large view and the file-association dialog use.

USAGE
=====

    python tools/make-icon.py

Writes `crates/pdfcer-gui/assets/pdfcer-gui.ico`, the raw window icon beside it,
and, for review,
`evidence/app-icon.png` — a strip of every size at 1:1 so the small ones can be
checked rather than assumed. **Look at the strip.** An icon that is fine at
256 px and mud at 16 px is the normal failure, and it is invisible from the
source.
"""

import struct
import zlib
from pathlib import Path

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent.parent
ICO_PATH = ROOT / "crates" / "pdfcer-gui" / "assets" / "pdfcer-gui.ico"
STRIP_PATH = ROOT / "evidence" / "app-icon.png"
# The WINDOW icon, as raw RGBA for `eframe::IconData`.
#
# ★ Why a second artefact when the .ico already holds every size.
#
# They are consumed by different things. The .ico is a Windows RESOURCE,
# read by Explorer, the shell and the file-association dialog from the
# executable's PE image — the operator's actual request. The window icon is
# read by winit at run time, for the title bar and the taskbar button, and it
# wants straight RGBA with no container at all. Decoding the .ico at start-up
# to get there would mean a PNG decoder in the binary for one 64-pixel image.
#
# 64 rather than 256: the taskbar asks for 32 or 48 on every current Windows
# and scales down cleanly, while 256 would be a quarter of a megabyte of
# `include_bytes!` for pixels nothing displays.
WINDOW_ICON_PATH = ROOT / "crates" / "pdfcer-gui" / "assets" / "window-icon-64.rgba"
WINDOW_ICON_SIZE = 64

SIZES = [16, 24, 32, 48, 64, 128, 256]

# Samples per axis. 4 is enough for a fold and a letter bowl; 8 doubles the
# runtime for a difference nobody can see.
SUPERSAMPLE = 4

# ---------------------------------------------------------------------------
# Palette — an application icon, so NOT theme colours. See the module docstring.
# ---------------------------------------------------------------------------

PAGE = (250, 250, 249)  # near-white, not pure: pure white loses its own edge
PAGE_EDGE = (176, 182, 189)  # a cool grey outline so the page reads on white
FOLD = (214, 220, 226)  # the turned-back corner, darker than the page
FOLD_EDGE = (176, 182, 189)
BADGE = (198, 40, 40)  # the generic PDF red — see the docstring on Adobe
BADGE_TEXT = (255, 255, 255)

# ---------------------------------------------------------------------------
# Geometry, in a 0..1 unit square. Everything below is expressed here so a
# change is a number rather than a redraw.
# ---------------------------------------------------------------------------

MARGIN_X = 0.14  # left/right space around the page
MARGIN_TOP = 0.06
MARGIN_BOTTOM = 0.06
FOLD_SIZE = 0.30  # the folded corner's leg length, as a fraction of the page
BADGE_TOP = 0.48  # where the red band starts, down the page
BADGE_BOTTOM = 0.92
BADGE_INSET = -0.06  # negative: the badge overhangs the page, as most do
CORNER_R = 0.03  # page corner rounding


def _lerp(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


# ---------------------------------------------------------------------------
# Letterforms — see the docstring. Each returns True if (x, y) is inside the
# glyph, in a unit square where y runs downward.
# ---------------------------------------------------------------------------

# The weight of every stroke, as a fraction of the cap height.
#
# ★ 0.20, and the first attempt used 0.26 — which produced a P and a D with NO
# COUNTERS. The bowl of a P is drawn as an outer capsule minus an inner one
# inset by the stroke weight, so the counter's height is
# `bowl_height - 2 * STEM`; at 0.26 with a 0.54 bowl that is negative and the
# subtraction removes nothing. The letters rendered as solid blobs and read as
# "PIF" at every size.
#
# The general form is worth having: **a stroke weight and a counter are the
# same number seen from two sides.** Whenever a letterform is defined by
# insetting, the weight has an upper bound set by the SMALLEST feature it must
# leave a hole in, and exceeding it fails silently by filling the hole rather
# than by erroring.
STEM = 0.20


def _capsule(x, y, x0, x1, y0, y1):
    """A rectangle with a semicircular right end — the P and D bowl shape.

    Degenerate boxes return False rather than misbehaving: the inner capsule of
    a too-heavy letter has a negative height, and the honest answer for "is this
    point inside a shape with no area" is no.
    """
    if x1 <= x0 or y1 <= y0:
        return False
    r = (y1 - y0) / 2.0
    if x1 - x0 < r:  # too short to round; treat as a plain box
        return x0 <= x <= x1 and y0 <= y <= y1
    if x < x0 or y < y0 or y > y1:
        return False
    if x <= x1 - r:
        return True
    dx = (x - (x1 - r)) / r
    dy = (y - (y0 + r)) / r
    return dx * dx + dy * dy <= 1.0


def _bar(x, y, x0, x1, y0, y1):
    return x0 <= x <= x1 and y0 <= y <= y1


# Each glyph is (width, test). Widths differ, which is what keeps the three
# letters evenly spaced rather than evenly boxed — a monospaced P, D and F
# leaves a visible gap after the F.
def glyph_p(x, y):
    if _bar(x, y, 0.0, STEM, 0.0, 1.0):
        return True
    return _capsule(x, y, 0.0, 0.86, 0.0, 0.58) and not _capsule(
        x, y, STEM, 0.86 - STEM, STEM, 0.58 - STEM
    )


def glyph_d(x, y):
    if _bar(x, y, 0.0, STEM, 0.0, 1.0):
        return True
    return _capsule(x, y, 0.0, 0.85, 0.0, 1.0) and not _capsule(
        x, y, STEM, 0.85 - STEM, STEM, 1.0 - STEM
    )


def glyph_f(x, y):
    if _bar(x, y, 0.0, STEM, 0.0, 1.0):
        return True
    if _bar(x, y, 0.0, 0.78, 0.0, STEM):  # top arm
        return True
    # The middle arm sits ABOVE the optical centre, which is where it goes in
    # every sans-serif: placed at the true centre it reads as low.
    return _bar(x, y, 0.0, 0.62, 0.42, 0.42 + STEM)


def glyph_e(x, y):
    """`F` with a foot. Written out rather than composed from `glyph_f`, because
    the two arms differ in length and a shared helper would need both."""
    if _bar(x, y, 0.0, STEM, 0.0, 1.0):
        return True
    if _bar(x, y, 0.0, 0.78, 0.0, STEM):  # top arm
        return True
    if _bar(x, y, 0.0, 0.62, 0.42, 0.42 + STEM):  # middle arm, optically high
        return True
    return _bar(x, y, 0.0, 0.78, 1.0 - STEM, 1.0)  # foot


# `C`'s geometry. An ellipse RING with the right flank opened, rather than the
# flat-left/rounded-right capsule the P and D bowls use — a C is the mirror of
# that shape and reusing `_capsule` would have produced a backwards letter.
C_RX = 0.42
C_APERTURE = 0.30  # half-height of the opening on the right


def glyph_c(x, y):
    cx, cy = C_RX, 0.5
    rx, ry = C_RX, 0.5
    # The aperture, cut first: cheapest test, and it is what makes this a C
    # rather than an O.
    if x > cx and abs(y - cy) < C_APERTURE:
        return False
    outer = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1.0
    if not outer:
        return False
    irx, iry = rx - STEM, ry - STEM
    if irx <= 0.0 or iry <= 0.0:
        return True  # too heavy to have a counter; see STEM's note
    return ((x - cx) / irx) ** 2 + ((y - cy) / iry) ** 2 > 1.0


# The badge's two lines. `pdfcer` is PDF + ce, and the operator asked for the
# second line on 2026-08-18: *"just add CE below PDF in the same red box."*
LINE_PDF = [(0.86, glyph_p), (0.85, glyph_d), (0.78, glyph_f)]
LINE_CE = [(2 * C_RX, glyph_c), (0.78, glyph_e)]
GLYPH_GAP = 0.20

# Fractions of the badge's height. The two lines share one cap height, so
# `PDF` and `CE` have the same stroke weight and read as one lockup rather
# than as a heading and a subtitle.
LINE_PAD = 0.13
LINE_GAP = 0.11


def _text_line(u, v, box, glyphs):
    """One line of glyphs at the box's height, CENTRED, never stretched.

    ★ Centred rather than justified, which is the whole reason this helper
    exists. The previous version mapped `x` across the box so a line always
    FILLED it — fine for one line, wrong the moment there are two: `CE` is
    two glyphs where `PDF` is three, so stretching both to the same width
    would draw `CE` half again as wide-strided as `PDF` and the pair would not
    read as one word.
    """
    x0, x1, y0, y1 = box
    height = y1 - y0
    if height <= 0.0 or v < y0 or v > y1:
        return False
    # Glyph units are multiples of the cap height, so the line's drawn width
    # falls out of the same number rather than being a second measurement.
    span = (sum(w for w, _ in glyphs) + GLYPH_GAP * (len(glyphs) - 1)) * height
    at = (u - ((x0 + x1) / 2.0 - span / 2.0)) / height
    gy = (v - y0) / height
    for width, test in glyphs:
        if at < 0.0:
            return False
        if at < width:
            return test(at, gy)
        at -= width + GLYPH_GAP
    return False


def _letters(box):
    """`PDF` over `CE`, as a predicate over the badge's text box."""
    x0, x1, y0, y1 = box
    height = y1 - y0
    pad = height * LINE_PAD
    gap = height * LINE_GAP
    cap = (height - 2.0 * pad - gap) / 2.0
    top = (x0, x1, y0 + pad, y0 + pad + cap)
    bottom = (x0, x1, y1 - pad - cap, y1 - pad)

    def test(u, v):
        return _text_line(u, v, top, LINE_PDF) or _text_line(u, v, bottom, LINE_CE)

    return test


# ---------------------------------------------------------------------------
# The icon itself
# ---------------------------------------------------------------------------


def sample(u, v):
    """Colour and alpha at unit position (u, v). Returns (r, g, b, a) floats."""
    px0, px1 = MARGIN_X, 1.0 - MARGIN_X
    py0, py1 = MARGIN_TOP, 1.0 - MARGIN_BOTTOM

    # --- the badge, drawn over everything, and it overhangs the page ---------
    bx0, bx1 = px0 + BADGE_INSET, px1 - BADGE_INSET
    by0 = py0 + (py1 - py0) * BADGE_TOP
    by1 = py0 + (py1 - py0) * BADGE_BOTTOM
    if bx0 <= u <= bx1 and by0 <= v <= by1:
        # The letters occupy the middle of the badge, with breathing room.
        lx0, lx1 = bx0 + (bx1 - bx0) * 0.06, bx1 - (bx1 - bx0) * 0.06
        if lx0 <= u <= lx1 and by0 <= v <= by1:
            if _letters((lx0, lx1, by0, by1))(u, v):
                return (*BADGE_TEXT, 1.0)
        return (*BADGE, 1.0)

    if not (px0 <= u <= px1 and py0 <= v <= py1):
        return (0.0, 0.0, 0.0, 0.0)

    # --- the folded corner ---------------------------------------------------
    # The fold is the triangle cut from the top right. Above its hypotenuse is
    # outside the page; below it, near the edge, is the turned-back flap.
    fold = FOLD_SIZE * (px1 - px0)
    fx = px1 - u
    fy = v - py0
    if fx + fy < fold:
        # Outside the page entirely — this is the corner that was turned back.
        return (0.0, 0.0, 0.0, 0.0)
    if fx + fy < fold + 0.012:
        return (*FOLD_EDGE, 1.0)
    if fx < fold and fy < fold:
        return (*FOLD, 1.0)

    # --- the page, with rounded corners and an outline -----------------------
    edge = 0.012
    r = CORNER_R
    # Distance outside the rounded rectangle, for the three corners that are
    # not the fold.
    cx = min(max(u, px0 + r), px1 - r)
    cy = min(max(v, py0 + r), py1 - r)
    d = ((u - cx) ** 2 + (v - cy) ** 2) ** 0.5 - r
    if d > 0.0:
        return (0.0, 0.0, 0.0, 0.0)
    if d > -edge:
        return (*PAGE_EDGE, 1.0)
    # A very slight vertical shade so a large icon does not read as flat.
    return (*_lerp(PAGE, PAGE_EDGE, 0.10 * (v - py0) / (py1 - py0)), 1.0)


def render(size):
    """One RGBA image at `size`, supersampled."""
    big = size * SUPERSAMPLE
    out = bytearray(size * size * 4)
    inv = 1.0 / big
    n = SUPERSAMPLE * SUPERSAMPLE
    for y in range(size):
        for x in range(size):
            r = g = b = a = 0.0
            for sy in range(SUPERSAMPLE):
                v = (y * SUPERSAMPLE + sy + 0.5) * inv
                for sx in range(SUPERSAMPLE):
                    u = (x * SUPERSAMPLE + sx + 0.5) * inv
                    sr, sg, sb, sa = sample(u, v)
                    # Accumulate PREMULTIPLIED, or a transparent sample's
                    # colour (which is arbitrary) bleeds into the average and
                    # every edge picks up a dark fringe.
                    r += sr * sa
                    g += sg * sa
                    b += sb * sa
                    a += sa
            at = (y * size + x) * 4
            if a <= 0.0:
                continue
            # …and un-premultiply on the way out, because both PNG and
            # `CustomCursor::from_rgba` want straight alpha.
            out[at] = min(255, int(r / a))
            out[at + 1] = min(255, int(g / a))
            out[at + 2] = min(255, int(b / a))
            out[at + 3] = min(255, int(255.0 * a / n))
    return bytes(out)


# ---------------------------------------------------------------------------
# Encoders
# ---------------------------------------------------------------------------


def png_rgba(width, height, rgba):
    def chunk(tag, data):
        body = tag + data
        return (
            struct.pack(">I", len(data))
            + body
            + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
        )

    raw = b"".join(
        b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height)
    )
    return (
        b"\x89PNG\r\n\x1a\n"
        # Colour type 6 = truecolour with alpha.
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def ico(images):
    """`images` is [(size, png_bytes)]. Returns the .ico file."""
    # ICONDIR: reserved 0, type 1 (icon), count.
    out = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    entries, payloads = b"", b""
    for size, data in images:
        # A dimension of 256 is written as 0 — the field is one byte.
        dim = 0 if size >= 256 else size
        entries += struct.pack(
            "<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset
        )
        payloads += data
        offset += len(data)
    return out + entries + payloads


def main():
    ICO_PATH.parent.mkdir(parents=True, exist_ok=True)
    STRIP_PATH.parent.mkdir(parents=True, exist_ok=True)

    assert WINDOW_ICON_SIZE in SIZES, (
        f"the window icon is taken from the rendered set, so {WINDOW_ICON_SIZE} "
        f"must be in SIZES"
    )

    images = []
    rendered = {}
    for size in SIZES:
        rgba = render(size)
        rendered[size] = rgba
        images.append((size, png_rgba(size, size, rgba)))
        print(f"  rendered {size}x{size}")

    ICO_PATH.write_bytes(ico(images))
    print(f"wrote {ICO_PATH} — {ICO_PATH.stat().st_size} bytes, {len(SIZES)} sizes")

    # The window icon: the same art, raw, for `eframe::IconData`. Taken from
    # the rendered set rather than re-rendered, so the two can never drift.
    WINDOW_ICON_PATH.write_bytes(rendered[WINDOW_ICON_SIZE])
    print(
        f"wrote {WINDOW_ICON_PATH} — {WINDOW_ICON_SIZE}x{WINDOW_ICON_SIZE} raw RGBA, "
        f"{WINDOW_ICON_PATH.stat().st_size} bytes"
    )

    # The review strip: every size at 1:1, on a mid grey so both the near-white
    # page and the white letters are judged against something.
    gap = 8
    width = sum(SIZES) + gap * (len(SIZES) + 1)
    height = max(SIZES) + gap * 2
    strip = bytearray()
    for _ in range(width * height):
        strip += bytes((0x80, 0x80, 0x80, 0xFF))
    x = gap
    for size in SIZES:
        rgba = rendered[size]
        y0 = gap + (max(SIZES) - size)  # bottom-aligned, like a taskbar
        for y in range(size):
            for sx in range(size):
                at = (y * size + sx) * 4
                a = rgba[at + 3] / 255.0
                o = ((y0 + y) * width + (x + sx)) * 4
                for c in range(3):
                    strip[o + c] = int(rgba[at + c] * a + 0x80 * (1 - a))
        x += size + gap
    STRIP_PATH.write_bytes(png_rgba(width, height, bytes(strip)))
    print(f"wrote {STRIP_PATH} — {width}x{height}. LOOK AT IT, especially the 16.")


if __name__ == "__main__":
    main()
