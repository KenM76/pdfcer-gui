# -*- coding: utf-8 -*-
"""Build fixtures/tags-figure.pdf.

One tagged page holding a paragraph and a figure. The paragraph is text,
`Caption` at (72, 700) in Helvetica 12; the figure is a filled grey square
from (100, 400) to (300, 600) with no text in it, so only painted bounds
(paths, images, shadings) can outline it.

The driven check `the_tags_panel_outlines_a_figure_with_no_text` picks the
Figure and requires one box on page 1 holding that square.
"""
import io
import os

NL = chr(10).encode('ascii')

PAGE = NL.join(l.encode('ascii') for l in [
    '/P <</MCID 0>> BDC BT /F1 12 Tf 72 700 Td (Caption) Tj ET EMC',
    '/Figure <</MCID 1>> BDC 0.6 g 100 400 200 200 re f EMC',
])

BODIES = [
    '<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 6 0 R /MarkInfo << /Marked true >> >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R '
    '/Resources << /Font << /F1 5 0 R >> >> /StructParents 0 >>',
    ('stream', PAGE),
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
    '<< /Type /StructTreeRoot /K 7 0 R >>',
    '<< /S /Document /P 6 0 R /K [8 0 R 9 0 R] >>',
    '<< /S /P /P 7 0 R /Pg 3 0 R /K 0 >>',
    '<< /S /Figure /P 7 0 R /Pg 3 0 R /Alt (A grey square) /K 1 >>',
]

out = io.BytesIO()
out.write(b'%PDF-1.7' + NL)
offsets = []
for i, body in enumerate(BODIES):
    offsets.append(out.tell())
    if isinstance(body, tuple):
        data = ('<< /Length %d >>' % len(body[1])).encode('ascii') + NL + b'stream' + NL \
            + body[1] + NL + b'endstream'
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
with open(os.path.join(here, 'tags-figure.pdf'), 'wb') as f:
    f.write(out.getvalue())
