# -*- coding: utf-8 -*-
"""Build fixtures/autosize-field.pdf from text-field-with-appearance.pdf.

The only change is the WIDGET's own /DA: `/Helv 10 Tf` -> `/Helv  0 Tf`.

Two spaces, deliberately: the replacement is BYTE-FOR-BYTE THE SAME LENGTH as
what it replaces, so every offset in the xref table stays valid and the fixture
needs no rebuild. PDF's tokeniser treats a run of whitespace as one separator,
so `/Helv  0 Tf` and `/Helv 0 Tf` are the same operand list.

Only the widget's DA is touched. The AcroForm default already says `0 Tf`; the
widget was overriding it, which is why the shipped fixture never exercised
auto-size at all.
"""
import io

src = 'fixtures/text-field-with-appearance.pdf'
dst = 'fixtures/autosize-field.pdf'

d = io.open(src, 'rb').read()

old = b'/DA (/Helv 10 Tf 0 g) /V (Ada)'
new = b'/DA (/Helv  0 Tf 0 g) /V (Ada)'
assert len(old) == len(new), 'the replacement must not move any byte offset'
assert d.count(old) == 1, 'expected exactly one widget /DA'

out = d.replace(old, new, 1)
assert len(out) == len(d), 'length changed; the xref would be invalid'

io.open(dst, 'wb').write(out)
print('wrote', dst, len(out), 'bytes')
