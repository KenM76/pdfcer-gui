# -*- coding: utf-8 -*-
"""Build fixtures/tagged-paragraph.pdf.

One tagged page: an H1 and a paragraph, each one marked-content sequence of
one text-show operator, with the page's /ParentTree entry listing both
elements by MCID, which the engine reads to find the elements a page's text
belongs to (`text_edit::decoration::tagged`, ISO 32000-1 14.7.4.4).

The driven check
`underlining_part_of_a_tagged_paragraph_says_it_is_not_recorded` underlines
the word "rose" in the paragraph and requires the engine's sentence that a
decoration covering part of a <P> element is not recorded.

Offsets are computed here, so the file can be edited and regenerated.
"""

import io
import os

NL = chr(10).encode('ascii')

LINES = [
    '/H1 <</MCID 0>> BDC BT /F1 18 Tf 72 720 Td (Quarterly Report) Tj ET EMC',
    '/P <</MCID 1>> BDC BT /F1 11 Tf 72 690 Td (Sales rose in every region.) Tj ET EMC',
]
CONTENT = NL.join(l.encode('ascii') for l in LINES)

BODIES = [
    '<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 6 0 R /MarkInfo << /Marked true >> >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R '
    '/Resources << /Font << /F1 5 0 R >> >> /StructParents 0 >>',
    None,
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
    '<< /Type /StructTreeRoot /K 7 0 R /ParentTree << /Nums [0 [8 0 R 9 0 R]] >> '
    '/ParentTreeNextKey 1 >>',
    '<< /S /Document /P 6 0 R /K [8 0 R 9 0 R] >>',
    '<< /S /H1 /P 7 0 R /Pg 3 0 R /K 0 >>',
    '<< /S /P /P 7 0 R /Pg 3 0 R /K 1 >>',
]

out = io.BytesIO()
out.write(b'%PDF-1.7' + NL)
offsets = []
for i, body in enumerate(BODIES):
    offsets.append(out.tell())
    if body is None:
        data = ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL \
            + CONTENT + NL + b'endstream'
    else:
        data = body.encode('ascii')
    out.write(('%d 0 obj' % (i + 1)).encode('ascii') + NL + data + NL + b'endobj' + NL)
xref = out.tell()
count = len(BODIES) + 1
out.write(('xref' + chr(10) + '0 %d' % count + chr(10)).encode('ascii'))
out.write(b'0000000000 65535 f ' + NL)
for off in offsets:
    out.write(('%010d 00000 n ' % off).encode('ascii') + NL)
out.write(('trailer' + chr(10) + '<< /Size %d /Root 1 0 R >>' % count + chr(10)).encode('ascii'))
out.write(('startxref' + chr(10) + '%d' % xref + chr(10)).encode('ascii') + b'%%EOF' + NL)

here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, 'tagged-paragraph.pdf'), 'wb') as f:
    f.write(out.getvalue())
