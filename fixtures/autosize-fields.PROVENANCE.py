# -*- coding: utf-8 -*-
"""Build fixtures/autosize-fields.pdf.

fixtures/three-text-fields.pdf with two changes: every field's /DA asks for
an automatic text size (/Helv 0 Tf), so any verb that redraws a field picks
a size (the engine's LayoutDisclosure::applied_autosize), and the /AcroForm
states /NeedAppearances true, so the Forms panel's Redraw values has every
field to redraw.

Built by running the sibling generator's source with those entries and the
output path replaced, so the two files differ only there.
"""
import io

src = io.open('fixtures/three-text-fields.PROVENANCE.py', encoding='utf-8').read()
swaps = [
    ("/DA (/Helv 10 Tf 0 g)", "/DA (/Helv 0 Tf 0 g)"),
    ("b'/DA (/Helv 0 Tf 0 g) /DR", "b'/NeedAppearances true /DA (/Helv 0 Tf 0 g) /DR"),
    ("dst = 'fixtures/three-text-fields.pdf'", "dst = 'fixtures/autosize-fields.pdf'"),
]
for old, new in swaps:
    assert src.count(old) == 1, 'the sibling generator moved: ' + old
    src = src.replace(old, new)
exec(src)
