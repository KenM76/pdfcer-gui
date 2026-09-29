# -*- coding: utf-8 -*-
"""Build fixtures/tagged-report.pdf.

One tagged page: an H1, a paragraph, a one-item list, an element of the
non-standard type `Blurb`, and a table with a THead row (Name, Qty), one body
row and a total cell spanning both columns, plus an artifact page number. No
rules are drawn, so table detection finds nothing and only the tags can
yield the table.

The driven check `export_word_follows_the_tags` exports it to Word and
requires the trace to say the tree was followed, with one table and one
heading. The page and tree are the engine's own `export_structure` test file.

Offsets are computed here, so the file can be edited and regenerated.
"""
import io
import os

NL = chr(10).encode('ascii')

LINES = [
    '/H1 <</MCID 0>> BDC BT /F1 18 Tf 72 720 Td (Quarterly Report) Tj ET EMC',
    '/P <</MCID 1>> BDC BT /F1 11 Tf 72 690 Td (Sales rose in every region.) Tj ET EMC',
    '/Lbl <</MCID 2>> BDC BT /F1 11 Tf 72 665 Td (1.) Tj ET EMC',
    '/LBody <</MCID 3>> BDC BT /F1 11 Tf 90 665 Td (First item) Tj ET EMC',
    '/Blurb <</MCID 4>> BDC BT /F1 11 Tf 72 640 Td (A styled note) Tj ET EMC',
    '/TH <</MCID 5>> BDC BT /F1 11 Tf 72 600 Td (Name) Tj ET EMC',
    '/TH <</MCID 6>> BDC BT /F1 11 Tf 250 600 Td (Qty) Tj ET EMC',
    '/TD <</MCID 7>> BDC BT /F1 11 Tf 72 580 Td (Widget) Tj ET EMC',
    '/TD <</MCID 8>> BDC BT /F1 11 Tf 250 580 Td (12) Tj ET EMC',
    '/TD <</MCID 9>> BDC BT /F1 11 Tf 72 560 Td (Total 12) Tj ET EMC',
    '/Artifact BMC BT /F1 9 Tf 300 40 Td (Page 1) Tj ET EMC',
]
CONTENT = NL.join(l.encode('ascii') for l in LINES)

BODIES = [
    '<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 6 0 R /MarkInfo << /Marked true >> >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R '
    '/Resources << /Font << /F1 5 0 R >> >> /StructParents 0 >>',
    None,
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
    '<< /Type /StructTreeRoot /K 7 0 R >>',
    '<< /S /Document /P 6 0 R /K [8 0 R 9 0 R 10 0 R 14 0 R 15 0 R] >>',
    '<< /S /H1 /P 7 0 R /Pg 3 0 R /K 0 >>',
    '<< /S /P /P 7 0 R /Pg 3 0 R /K 1 >>',
    '<< /S /L /P 7 0 R /K 11 0 R >>',
    '<< /S /LI /P 10 0 R /K [12 0 R 13 0 R] >>',
    '<< /S /Lbl /P 11 0 R /Pg 3 0 R /K 2 >>',
    '<< /S /LBody /P 11 0 R /Pg 3 0 R /K 3 >>',
    '<< /S /Blurb /P 7 0 R /Pg 3 0 R /K 4 >>',
    '<< /S /Table /P 7 0 R /K [16 0 R 20 0 R] >>',
    '<< /S /THead /P 15 0 R /K 17 0 R >>',
    '<< /S /TR /P 16 0 R /K [18 0 R 19 0 R] >>',
    '<< /S /TH /P 17 0 R /Pg 3 0 R /K 5 >>',
    '<< /S /TH /P 17 0 R /Pg 3 0 R /K 6 >>',
    '<< /S /TBody /P 15 0 R /K [21 0 R 24 0 R] >>',
    '<< /S /TR /P 20 0 R /K [22 0 R 23 0 R] >>',
    '<< /S /TD /P 21 0 R /Pg 3 0 R /K 7 >>',
    '<< /S /TD /P 21 0 R /Pg 3 0 R /K 8 >>',
    '<< /S /TR /P 20 0 R /K 25 0 R >>',
    '<< /S /TD /P 24 0 R /Pg 3 0 R /A << /O /Table /ColSpan 2 >> /K 9 >>',
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
with open(os.path.join(here, 'tagged-report.pdf'), 'wb') as f:
    f.write(out.getvalue())
