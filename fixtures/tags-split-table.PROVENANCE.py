# -*- coding: utf-8 -*-
"""Build fixtures/tags-split-table.pdf.

Two tagged pages holding one 2x2 table: row 1 (cells A1, B1) on page 1 and
row 2 (cells A2, B2) on page 2. No rules are drawn, so only the tags say it
is a table. The second row's last cell also lists the Table element among
its kids, a cycle the reader must step over.

The driven check `the_tags_panel_outlines_the_picked_element` picks a cell, a
row and the table, and requires one box in the cell's place, one box over the
row, and one box on each page for the table.

Cell text, in default user space (page 612 x 792, Helvetica 12):
  A1 at (72, 700), B1 at (300, 700)   on page 1
  A2 at (72, 700), B2 at (300, 700)   on page 2

Offsets are computed here, so the file can be edited and regenerated.
"""
import io
import os

NL = chr(10).encode('ascii')


def content(mcid0, left, right):
    lines = [
        '/TD <</MCID %d>> BDC BT /F1 12 Tf 72 700 Td (%s) Tj ET EMC' % (mcid0, left),
        '/TD <</MCID %d>> BDC BT /F1 12 Tf 300 700 Td (%s) Tj ET EMC' % (mcid0 + 1, right),
    ]
    return NL.join(l.encode('ascii') for l in lines)


PAGE1 = content(0, 'A1', 'B1')
PAGE2 = content(0, 'A2', 'B2')

BODIES = [
    # 1
    '<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 8 0 R /MarkInfo << /Marked true >> >>',
    # 2
    '<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>',
    # 3, 4
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 5 0 R '
    '/Resources << /Font << /F1 7 0 R >> >> /StructParents 0 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 6 0 R '
    '/Resources << /Font << /F1 7 0 R >> >> /StructParents 1 >>',
    # 5, 6
    ('stream', PAGE1),
    ('stream', PAGE2),
    # 7
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
    # 8
    '<< /Type /StructTreeRoot /K 9 0 R >>',
    # 9
    '<< /S /Document /P 8 0 R /K 10 0 R >>',
    # 10
    '<< /S /Table /P 9 0 R /K [11 0 R 14 0 R] >>',
    # 11 row 1
    '<< /S /TR /P 10 0 R /K [12 0 R 13 0 R] >>',
    '<< /S /TD /P 11 0 R /Pg 3 0 R /K 0 >>',
    '<< /S /TD /P 11 0 R /Pg 3 0 R /K 1 >>',
    # 14 row 2
    '<< /S /TR /P 10 0 R /K [15 0 R 16 0 R] >>',
    '<< /S /TD /P 14 0 R /Pg 4 0 R /K 0 >>',
    # 16: its kids include the Table again, a cycle.
    '<< /S /TD /P 14 0 R /Pg 4 0 R /K [1 10 0 R] >>',
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
with open(os.path.join(here, 'tags-split-table.pdf'), 'wb') as f:
    f.write(out.getvalue())
