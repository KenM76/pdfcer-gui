# -*- coding: utf-8 -*-
"""Build fixtures/ruled-table.pdf.

One page holding one ruled table: three columns by three rows, drawn as
stroked lines, a header row of words and two rows mixing words and numbers.
It is the smallest document on which File > Export > Tables... finds a table
by its rules, so the driven check `export_tables_without_the_mouse` can export
it as CSV, Excel and OpenDocument and read what reached disk.

The numbers are unambiguous in every locale (12, 40, 1.5 has one digit after
the point), so both workbook writers must write them as numbers.

Offsets are computed here, so the file can be edited and regenerated.
"""
import io
import os

NL = chr(10).encode('ascii')

XS = [20, 110, 200, 280]
YS = [170, 140, 110, 80]
ROWS = [
    ['Item', 'Qty', 'Mass'],
    ['Bolt', '12', '1.5'],
    ['Nut', '40', '0.25'],
]

lines = ['0.8 w 0 G']
for y in YS:
    lines.append('%d %d m %d %d l S' % (XS[0], y, XS[-1], y))
for x in XS:
    lines.append('%d %d m %d %d l S' % (x, YS[0], x, YS[-1]))
for r, row in enumerate(ROWS):
    for c, word in enumerate(row):
        lines.append('BT /Helv 10 Tf %d %d Td (%s) Tj ET' % (XS[c] + 6, YS[r + 1] + 10, word))
CONTENT = NL.join(l.encode('ascii') for l in lines)

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] '
       b'/Resources << /Font << /Helv 5 0 R >> >> /Contents 4 0 R >>',
    4: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
    5: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
}

out = io.BytesIO()
out.write(b'%PDF-1.4' + NL)
offsets = {}
for n in sorted(objects):
    offsets[n] = out.tell()
    out.write(('%d 0 obj' % n).encode('ascii') + NL + objects[n] + NL + b'endobj' + NL)
xref = out.tell()
count = len(objects) + 1
out.write(('xref' + chr(10) + '0 %d' % count + chr(10)).encode('ascii'))
out.write(b'0000000000 65535 f ' + NL)
for n in sorted(objects):
    out.write(('%010d 00000 n ' % offsets[n]).encode('ascii') + NL)
out.write(('trailer' + chr(10) + '<< /Size %d /Root 1 0 R >>' % count + chr(10)).encode('ascii'))
out.write(('startxref' + chr(10) + '%d' % xref + chr(10)).encode('ascii') + b'%%EOF' + NL)

here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, 'ruled-table.pdf'), 'wb') as f:
    f.write(out.getvalue())
