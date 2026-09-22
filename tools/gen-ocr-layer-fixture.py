#!/usr/bin/env python3
"""Generate ``fixtures/ocr-layer.pdf`` -- a page that already carries an OCR
text layer, readable without the recogniser and without the model weights.

    python tools/gen-ocr-layer-fixture.py

Deterministic: same bytes every run, no timestamps, no object ids drawn from
anything but position in the list.

WHAT THIS FIXTURE IS FOR
========================

The shell needs to read, paint and edit an OCR text layer -- text shown at
rendering mode 3, invisible, sitting over a picture of the words it spells.
Until now nothing in ``fixtures/`` had one.

``synthetic-image-only.pdf`` is the closest thing and it is the opposite case:
a page with NO text at all, built so the recogniser has something to recognise.
Producing an OCR'd document from it costs the ~12 MB ``ocrs`` weights, which
are not in this repository, several seconds of CPU, and a result that changes
whenever the model does. That is the right fixture for testing the recogniser
and the wrong one for testing everything downstream of it.

So this file is the downstream half: the sandwich as the engine writes it,
frozen, cheap, and byte-identical on every machine.

WHAT IT IS NOT
--------------

It is not a scan, and its "picture" is an eight-by-eight checkerboard rather
than an image of the words the invisible layer spells. Nothing here measures
recognition quality, glyph placement accuracy, or whether an OCR layer lines up
with the ink beneath it. A check that wants those needs a real scan.

THE SHAPE, AND WHY IT MATCHES THE ENGINE BYTE FOR BYTE
======================================================

``pdfcer_core::ocr::layer::build_layer_content`` appends a *separate* content
stream to the page's ``/Contents`` array and binds a collision-free font under
``pick_font_name``, which yields ``/pdfceF1`` on a page whose ``/Font`` has no
such key. The stream it writes is::

    q
    BT
    3 Tr
    /pdfceF1 <size> Tf
    <tz> Tz
    1 0 0 1 <x> <y> Tm
    (<word>) Tj
    ... one Tf/Tz/Tm/Tj group per word ...
    ET
    Q

Stream B below is that, verbatim in shape. Three properties of it are load
bearing and each is copied deliberately:

* the ``q``/``Q`` wrapper, because ``Tf``/``Tr``/``Tz`` are graphics state and
  must not leak into the streams that follow (PDF 32000-1 s.8.4.2);
* one ``Tf`` and one ``Tz`` per word rather than hoisted, because adjacent OCR
  words almost never share a size;
* a ``Tz`` that is not 100 on one word, because the engine sets horizontal
  scale to make the glyphs span the recognised box, and a reader that drops
  ``Tz`` gets the extent wrong without getting anything visibly wrong. It is
  the same word as one of the unscaled ones, for the reason written beside
  ``SCALE_CONTROL_WORD``.

THE CONTROLS
============

A fixture has to distinguish "the build classified the right runs" from "the
build classified everything the same way and happened to agree". Three
controls, each defeating a different wrong build:

1. **A visible run on the same page** -- ``VISIBLE CONTROL STAMP`` in stream A,
   at the default rendering mode 0. A build that reported every glyph invisible
   satisfies every assertion about the OCR layer and fails here.

2. **A hidden run in a stream that is not the OCR layer** -- the first run of
   stream C. A build that decided invisibility by asking which content stream a
   glyph came from, or by looking for ``3 Tr`` anywhere in a stream and
   condemning the whole of it, gets this one wrong.

3. **A visible run AFTER a hidden one, inside the same ``BT``/``ET``** -- the
   second run of stream C, reached by ``0 Tr``. A build that treats rendering
   mode as latching, or that scans forward from the first ``3 Tr`` to ``ET``,
   reports this run invisible when it is not.

Controls 2 and 3 are the ones that matter for the overlay: the selector is
``ExtractedGlyph::invisible``, which the engine computes from the ambient text
state, and the failure mode being guarded is a shell that reimplements that
judgement from the content stream and gets the ladder wrong.

WHAT IS DELIBERATELY ABSENT
===========================

A stream that sets ``3 Tr`` and never restores it, leaving the streams after it
invisible. That is a real defect real producers ship, and it is a torture case
for the *extractor*, not for the overlay: on such a page the correct answer is
"all of it is invisible", which is the answer a broken build gives too. Putting
it here would make this fixture ambiguous about what passing means. It belongs
in its own fixture with its own argument.

THE PICTURE
===========

Eight by eight, ``/DeviceGray``, eight bits, uncompressed, sixty-four bytes,
scaled to the full 612x792 sheet so each cell is 76.5 x 99 points.

Pure black and pure white, not two greys, because the slider this fixture
exists to verify has to be decidable from a screenshot at three positions: at
0.0 the raster is drawn as saved, at 0.5 it is tinted, and at 1.0 it is not
drawn at all. Two mid-greys make "tinted" and "blanked" look alike; black and
white make them not.

The top half is the checkerboard and the bottom half is flat white, so that the
invisible OCR words (high on the page, over the pattern, where scanned words
would be) and the visible controls (low on the page, over white, where they
stay legible in a screenshot) each sit on the background that suits them.
"""

import pathlib

FIXTURES = pathlib.Path(__file__).resolve().parent.parent / "fixtures"
OUT = FIXTURES / "ocr-layer.pdf"

PAGE_WIDTH = 612
PAGE_HEIGHT = 792

IMAGE_EDGE = 8

# The OCR layer, as (text, size, horizontal-scale, x, baseline-y).
#
# Two lines of a title block, plus a third line that exists only as the
# horizontal-scale control.
#
# The engine emits a per-word ``Tz`` so the glyphs span the recognised box, and
# a reader that drops it gets every OCR run's extent wrong without getting
# anything visibly wrong. The control for that has to be the SAME WORD at two
# scales: comparing two different words confounds the scale with the letters,
# because `I` is narrow and `W` is not. So `SCANNED` appears twice, identical
# in text and size, differing only in ``Tz``, and the ratio between the two
# runs' advances is a number with one cause.
SCALE_CONTROL_WORD = "SCANNED"
SCALE_CONTROL_TZ = 86.4

OCR_WORDS = [
    ("SITE", 14.0, 100.0, 72.0, 700.0),
    ("PLAN", 14.0, 100.0, 112.0, 700.0),
    ("REVISION", 14.0, 100.0, 160.0, 700.0),
    ("C", 14.0, 100.0, 252.0, 700.0),
    ("SCANNED", 14.0, 100.0, 72.0, 676.0),
    ("DRAWING", 14.0, 100.0, 145.0, 676.0),
    ("SHEET", 14.0, 100.0, 225.0, 676.0),
    ("2", 14.0, 100.0, 272.0, 676.0),
    (SCALE_CONTROL_WORD, 14.0, SCALE_CONTROL_TZ, 72.0, 652.0),
]

# Control 1: a visible run on the same page, in the picture's own stream.
STAMP = "VISIBLE CONTROL STAMP"
STAMP_SIZE = 18.0
STAMP_AT = (72.0, 120.0)

# Controls 2 and 3: the rendering-mode ladder, both runs inside one BT/ET.
LADDER_HIDDEN = "HIDDEN RUN IN A VISIBLE STREAM"
LADDER_VISIBLE = "VISIBLE RUN AFTER A HIDDEN ONE"
LADDER_SIZE = 10.0
LADDER_HIDDEN_AT = (72.0, 200.0)
LADDER_VISIBLE_AT = (72.0, 180.0)


def checkerboard() -> bytes:
    """Eight by eight grey samples: checkered top half, flat white bottom."""
    rows = []
    for row in range(IMAGE_EDGE):
        for col in range(IMAGE_EDGE):
            if row >= IMAGE_EDGE // 2:
                rows.append(0xFF)
            else:
                rows.append(0xFF if (row + col) % 2 == 0 else 0x00)
    return bytes(rows)


def number(value: float) -> str:
    """Shortest exact decimal, so the stream has no trailing-zero noise."""
    text = f"{value:.4f}".rstrip("0").rstrip(".")
    return text if text else "0"


def picture_and_stamp() -> str:
    """Stream A: the page image, then the visible control run at mode 0."""
    return (
        f"q {PAGE_WIDTH} 0 0 {PAGE_HEIGHT} 0 0 cm /Im0 Do Q\n"
        "BT\n"
        f"/F1 {number(STAMP_SIZE)} Tf\n"
        f"1 0 0 1 {number(STAMP_AT[0])} {number(STAMP_AT[1])} Tm\n"
        f"({STAMP}) Tj\n"
        "ET\n"
    )


def ocr_layer() -> str:
    """Stream B: the invisible layer, in the engine's own emission order."""
    out = ["q\n", "BT\n", "3 Tr\n"]
    for text, size, tz, x, y in OCR_WORDS:
        out.append(f"/pdfceF1 {number(size)} Tf\n")
        out.append(f"{number(tz)} Tz\n")
        out.append(f"1 0 0 1 {number(x)} {number(y)} Tm\n")
        out.append(f"({text}) Tj\n")
    out.append("ET\n")
    out.append("Q\n")
    return "".join(out)


def mode_ladder() -> str:
    """Stream C: hidden then visible, one BT/ET, mode restored by 0 Tr."""
    return (
        "q\n"
        "BT\n"
        f"/F1 {number(LADDER_SIZE)} Tf\n"
        "3 Tr\n"
        f"1 0 0 1 {number(LADDER_HIDDEN_AT[0])} {number(LADDER_HIDDEN_AT[1])} Tm\n"
        f"({LADDER_HIDDEN}) Tj\n"
        "0 Tr\n"
        f"1 0 0 1 {number(LADDER_VISIBLE_AT[0])} {number(LADDER_VISIBLE_AT[1])} Tm\n"
        f"({LADDER_VISIBLE}) Tj\n"
        "ET\n"
        "Q\n"
    )


def stream_object(extra: bytes, data: bytes) -> bytes:
    """A stream object body: the dictionary with a correct ``/Length``."""
    head = b"<< " + extra + b"/Length " + str(len(data)).encode() + b" >>"
    return head + b"\nstream\n" + data + b"\nendstream"


def build() -> bytes:
    """Assemble the nine objects and a classic cross-reference table."""
    image = checkerboard()

    objects = [
        # 1 Catalog
        b"<< /Type /Catalog /Pages 2 0 R >>",
        # 2 Pages
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        # 3 Page -- three content streams in order: picture, OCR layer, ladder
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 "
        + str(PAGE_WIDTH).encode()
        + b" "
        + str(PAGE_HEIGHT).encode()
        + b"] /Resources << /Font << /F1 5 0 R /pdfceF1 8 0 R >> "
        b"/XObject << /Im0 7 0 R >> >> /Contents [4 0 R 6 0 R 9 0 R] >>",
        # 4 Stream A
        stream_object(b"", picture_and_stamp().encode("latin-1")),
        # 5 The page's own font, used by both visible runs
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        # 6 Stream B
        stream_object(b"", ocr_layer().encode("latin-1")),
        # 7 The picture
        stream_object(
            b"/Type /XObject /Subtype /Image /Width "
            + str(IMAGE_EDGE).encode()
            + b" /Height "
            + str(IMAGE_EDGE).encode()
            + b" /ColorSpace /DeviceGray /BitsPerComponent 8 ",
            image,
        ),
        # 8 The OCR font, written exactly as standard14_font_dict writes it
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
        b"/Encoding /WinAnsiEncoding >>",
        # 9 Stream C
        stream_object(b"", mode_ladder().encode("latin-1")),
    ]

    out = bytearray(b"%PDF-1.7\n")
    offsets = [0]
    for n, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += str(n).encode() + b" 0 obj\n" + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 " + str(len(objects) + 1).encode() + b"\n"
    out += b"0000000000 65535 f \n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n \n".encode()
    out += (
        b"trailer\n<< /Size "
        + str(len(objects) + 1).encode()
        + b" /Root 1 0 R >>\nstartxref\n"
        + str(xref).encode()
        + b"\n%%EOF\n"
    )
    return bytes(out)


def report() -> str:
    """The measured counts, so the PROVENANCE file quotes them rather than
    computing them by hand."""
    ocr_glyphs = sum(len(word[0]) for word in OCR_WORDS)
    hidden = ocr_glyphs + len(LADDER_HIDDEN)
    visible = len(STAMP) + len(LADDER_VISIBLE)
    lines = [
        f"  OCR layer words        {len(OCR_WORDS)}",
        f"  OCR layer glyphs       {ocr_glyphs}",
        f"  ladder hidden glyphs   {len(LADDER_HIDDEN)}",
        f"  invisible glyphs TOTAL {hidden}",
        "",
        f"  stamp glyphs           {len(STAMP)}",
        f"  ladder visible glyphs  {len(LADDER_VISIBLE)}",
        f"  visible glyphs TOTAL   {visible}",
        "",
        f"  all glyphs             {hidden + visible}",
    ]
    return "\n".join(lines)


def main() -> None:
    data = build()
    OUT.write_bytes(data)
    print(f"wrote {OUT} ({len(data)} bytes)")
    print(report())


if __name__ == "__main__":
    main()
