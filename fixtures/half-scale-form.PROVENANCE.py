# -*- coding: utf-8 -*-
"""Build fixtures/half-scale-form.pdf.

tools/gen-form-xobject-fixture.py with a different form and placement: one
400 x 300 page whose only content places a form at half scale at (40, 40),
and the form holds two leaves:

    leaf 0  a blue line, form (20,110)..(300,110), 4 units wide: on the page
            y 95, x 50..190, 2 pt wide
    leaf 1  a red filled rectangle, form (20,20) 80 x 60: on the page
            (50,50)..(90,80)

Half scale so a width typed in points must reach the form's stream doubled;
the form keeps its own (empty) /Resources so opacity can be bound there.

Built by running the sibling generator's source with the form stream, the
placement and the output name replaced, so the two files differ only there.
"""
import io

src = io.open('tools/gen-form-xobject-fixture.py', encoding='utf-8').read()
swaps = [
    (
        "0 0 0 RG\n12 w\n20 110 m 300 110 l S\n160 20 m 160 200 l S\n10 w\n40 40 m 280 180 l S\n",
        "0 0 1 RG\n4 w\n20 110 m 300 110 l S\n1 0 0 rg\n20 20 80 60 re f\n",
    ),
    ('b"q 1 0 0 1 %d %d cm /Fm0 Do Q', 'b"q 0.5 0 0 0.5 %d %d cm /Fm0 Do Q'),
    ('"form-xobject.pdf"', '"half-scale-form.pdf"'),
]
for old, new in swaps:
    assert src.count(old) == 1, 'the sibling generator moved: ' + old
    src = src.replace(old, new)
exec(src)
