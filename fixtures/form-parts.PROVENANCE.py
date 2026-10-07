# -*- coding: utf-8 -*-
"""Build fixtures/form-parts.pdf.

tools/gen-form-xobject-fixture.py with a different form: one 400 x 300 page
whose only content places a form at (40, 40), and the form holds three leaves
whose PARTS can be deleted:

    leaf 0  one stroked path of two subpaths: bars at form y 180 and y 160,
            x 20..140 (page y 220 and 200, x 60..180), 8 pt wide
    leaf 1  one stroked polyline of four anchors (180,40) (220,120) (260,40)
            (300,120), 8 pt wide, so deleting one anchor leaves three
    leaf 2  one text object of two lines, "First line" at form (20,60) and
            "Second line" 24 pt below it, Helvetica 14 pt

Two subpaths because a path's only subpath takes the object with it; four
anchors because the engine refuses a node delete that would leave a subpath
with fewer than two; the deleted text line is the LAST, so no later run slides.

Built by running the sibling generator's source with the form stream, the
form's resources and the output name replaced, so the two files differ only
there.
"""
import io

src = io.open('tools/gen-form-xobject-fixture.py', encoding='utf-8').read()
swaps = [
    (
        "12 w\n20 110 m 300 110 l S\n160 20 m 160 200 l S\n10 w\n40 40 m 280 180 l S\n",
        "8 w\n20 180 m 140 180 l 20 160 m 140 160 l S\n"
        "180 40 m 220 120 l 260 40 l 300 120 l S\n"
        "0 g\nBT /F1 14 Tf 20 60 Td (First line) Tj 0 -24 Td (Second line) Tj ET\n",
    ),
    (
        "/Resources << >> /Length",
        "/Resources << /Font << /F1 << /Type /Font /Subtype /Type1 "
        "/BaseFont /Helvetica >> >> >> /Length",
    ),
    ('"form-xobject.pdf"', '"form-parts.pdf"'),
]
for old, new in swaps:
    assert src.count(old) == 1, 'the sibling generator moved: ' + old
    src = src.replace(old, new)
exec(src)
