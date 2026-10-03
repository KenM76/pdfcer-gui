# -*- coding: utf-8 -*-
"""Build fixtures/two-ruled-tables.pdf.

Two pages, each holding one ruled table of three columns by three rows: a
word header and two rows of a word, a whole number and a three-decimal
number. It is the document on which `export_tables_honours_its_choices`
exercises the Export tables window's page range, sheet grouping and number
reading, each of which changes what its default export writes:

- all pages give two tables, one sheet each;
- every Mass value (1.234, 2.500, ...) reads differently in the US and in
  Europe, so the engine's default keeps all four as text and counts them
  ambiguous; read as US or European numbers they are numbers.

Offsets are computed here, so the file can be edited and regenerated.
"""
import io
import os

NL = chr(10).encode('ascii')

XS = [20, 110, 200, 280]
YS = [170, 140, 110, 80]
PAGES = [
    [['Item', 'Qty', 'Mass'], ['Bolt', '12', '1.234'], ['Nut', '40', '2.500']],
    [['Item', 'Qty', 'Mass'], ['Washer', '7', '3.125'], ['Pin', '9', '4.750']],
]


def content(rows):
    lines = ['0.8 w 0 G']
    for y in YS:
        lines.append('%d %d m %d %d l S' % (XS[0], y, XS[-1], y))
    for x in XS:
        lines.append('%d %d m %d %d l S' % (x, YS[0], x, YS[-1]))
    for r, row in enumerate(rows):
        for c, word in enumerate(row):
            lines.append('BT /Helv 10 Tf %d %d Td (%s) Tj ET' % (XS[c] + 6, YS[r + 1] + 10, word))
    return NL.join(l.encode('ascii') for l in lines)


def stream(data):
    return ('<< /Length %d >>' % len(data)).encode('ascii') + NL + b'stream' + NL + data + NL + b'endstream'


objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R >>',
    2: b'<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] '
       b'/Resources << /Font << /Helv 7 0 R >> >> /Contents 5 0 R >>',
    4: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] '
       b'/Resources << /Font << /Helv 7 0 R >> >> /Contents 6 0 R >>',
    5: stream(content(PAGES[0])),
    6: stream(content(PAGES[1])),
    7: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
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
with open(os.path.join(here, 'two-ruled-tables.pdf'), 'wb') as f:
    f.write(out.getvalue())
