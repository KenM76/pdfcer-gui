# -*- coding: utf-8 -*-
"""Build fixtures/form-text-runs.pdf.

tools/gen-form-xobject-fixture.py with a different form: one 400 x 300 page
whose only content places a form at (40, 40), and the form holds one text
object of two lines, Helvetica 14 pt:

    line 0  "Two" then " runs", two show operators on one baseline at form
            (20, 60), page (60, 100); the second inherits its origin, so the
            pair can be merged
    line 1  "Second line", one show operator 24 pt below, page (60, 76),
            positioned by its own Td, so the object can be cut before it and
            the run can be fitted to a width

Built by running the sibling generator's source with the form stream, the
form's resources and the output name replaced, so the two files differ only
there.
"""
import io

src = io.open('tools/gen-form-xobject-fixture.py', encoding='utf-8').read()
swaps = [
    (
        "12 w\n20 110 m 300 110 l S\n160 20 m 160 200 l S\n10 w\n40 40 m 280 180 l S\n",
        "0 g\nBT /F1 14 Tf 20 60 Td (Two) Tj ( runs) Tj 0 -24 Td (Second line) Tj ET\n",
    ),
    (
        "/Resources << >> /Length",
        "/Resources << /Font << /F1 << /Type /Font /Subtype /Type1 "
        "/BaseFont /Helvetica >> >> >> /Length",
    ),
    ('"form-xobject.pdf"', '"form-text-runs.pdf"'),
]
for old, new in swaps:
    assert src.count(old) == 1, 'the sibling generator moved: ' + old
    src = src.replace(old, new)
exec(src)
