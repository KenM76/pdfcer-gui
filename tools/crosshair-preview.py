"""Render the crosshair cursor onto black, mid grey and white, as a PNG.

WHY THIS EXISTS
===============

`crates/pdfcer-gui/src/canvas/cursor.rs` builds pdfcer's own crosshair cursor
because the platform's stock one is monochrome and its colour belongs to the
operator's pointer scheme — which is how it came to be *"white, making it hard
to see"* on the operator's machine (2026-08-18).

A cursor whose entire purpose is **being visible** has exactly one oracle, and
it is not a unit test. The unit tests in that module pin the arrangement — a
dark core inside a light halo, a clear centre, a hotspot on the crossing point
— and every one of them would still pass on a glyph that was illegible in
practice, because "legible" is a fact about a rendered image on a background.

So: this writes the image, and a human looks at it. Same rule as `ui-verify`'s,
applied to a thing too small to drive.

HOW TO USE IT
=============

Two commands. The first is the real code producing the real bitmap; the second
is only a viewer.

    cargo test -p pdfcer-gui --lib preview -- --ignored --nocapture
    python tools/crosshair-preview.py %TEMP%/crosshair-32.rgba evidence/crosshair-32.png 4
    python tools/crosshair-preview.py %TEMP%/crosshair-64.rgba evidence/crosshair-64.png 2

The third argument is a nearest-neighbour magnification, so the pixels stay
square and countable. Use 4 for the 32 px bitmap (scale 1.0) and 2 for the
64 px one (scale 2.0), which puts both panels at the same size on screen and
makes the two directly comparable — the point of checking both is that the core
must stay a hairline while the halo grows.

Committed artefacts live in `evidence/`. Regenerate them if you change the
geometry; a stale one is worse than none, because it looks like verification.

WHY THE THREE BACKGROUNDS
=========================

They are the operator's own three cases, in their own words: *"if they are over
a black or white or grey object"*. Grey is the one that matters — it is where
the inverting cursors this replaces used to fail, because an inverted mid grey
is another mid grey. A two-tone glyph is legible on all three, and this picture
is the claim.

WHY NO IMAGING LIBRARY
======================

Same reasoning as `tools/ui-verify/src/png.rs`, which hand-rolls its encoder:
this is reached for when something looks wrong, and a dependency is one more
thing that can fail to install on that day. `zlib` and `struct` are Python
standard library, and a PNG is a signature plus three length-prefixed,
CRC-suffixed chunks.
"""

import struct
import sys
import zlib

# The operator's three cases, left to right.
BACKGROUNDS = [(0, 0, 0), (128, 128, 128), (255, 255, 255)]

# Pixels between panels, so the white panel's edge is visible against the page
# a reader is looking at this on.
GAP_PX = 4


def png(width, height, rgb):
    """An 8-bit truecolour PNG from packed RGB rows."""

    def chunk(tag, data):
        body = tag + data
        return (
            struct.pack(">I", len(data))
            + body
            + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
        )

    # Each scanline is prefixed with its filter type. 0 = None. Forgetting this
    # byte is the classic PNG bug and the symptom is an image that shears
    # diagonally, because every row lands one byte further along than the last.
    raw = b"".join(
        b"\x00" + rgb[y * width * 3 : (y + 1) * width * 3] for y in range(height)
    )
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def main(path, out, scale):
    # The dump is named for its own edge length, which is also the only place
    # the size is recorded — the file is raw pixels with no header.
    size = int(path.replace("\\", "/").split("/")[-1].split("-")[-1].split(".")[0])
    data = open(path, "rb").read()
    if len(data) != size * size * 4:
        raise SystemExit(
            f"{path} is {len(data)} bytes; {size}x{size} RGBA is {size * size * 4}"
        )

    panel = size * scale
    width = len(BACKGROUNDS) * panel + (len(BACKGROUNDS) - 1) * GAP_PX
    height = panel
    rgb = bytearray(width * height * 3)

    for index, bg in enumerate(BACKGROUNDS):
        x0 = index * (panel + GAP_PX)
        for y in range(height):
            for x in range(panel):
                at = ((y // scale) * size + (x // scale)) * 4
                # Straight (non-premultiplied) alpha, which is what
                # `CustomCursorImage` documents and what winit expects.
                alpha = data[at + 3] / 255.0
                out_at = (y * width + (x0 + x)) * 3
                for channel in range(3):
                    over = data[at + channel] * alpha + bg[channel] * (1.0 - alpha)
                    rgb[out_at + channel] = int(over)

    with open(out, "wb") as handle:
        handle.write(png(width, height, bytes(rgb)))
    print(f"wrote {out} — {width}x{height}, {len(BACKGROUNDS)} panels at {scale}x")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]))
