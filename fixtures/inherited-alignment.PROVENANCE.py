# -*- coding: utf-8 -*-
"""Build fixtures/inherited-alignment.pdf.

fixtures/three-text-fields.pdf with one change: the /AcroForm states /Q 1
(centred) and no field states its own /Q. Every field therefore resolves to
centred by inheritance, which is the state where "Left" written on the field
and "nothing written on the field" are different assertions that look alike.

Built by running the sibling generator's source with the /AcroForm and the
output path replaced, so the two files differ only in that entry.
"""
import io

src = io.open('fixtures/three-text-fields.PROVENANCE.py', encoding='utf-8').read()
old_form = "b'/DA (/Helv 0 Tf 0 g) /DR"
new_form = "b'/Q 1 /DA (/Helv 0 Tf 0 g) /DR"
old_dst = "dst = 'fixtures/three-text-fields.pdf'"
new_dst = "dst = 'fixtures/inherited-alignment.pdf'"
assert src.count(old_form) == 1, 'the /AcroForm line moved in the sibling generator'
assert src.count(old_dst) == 1, 'the output path moved in the sibling generator'
exec(src.replace(old_form, new_form).replace(old_dst, new_dst))
