# -*- coding: utf-8 -*-
"""Build fixtures/stacked-labels.pdf -- three lines in one text object.

## Why this fixture exists

`a_text_object_splits_into_lines` needs one text object holding several
lines, placed the way CAD exporters place them: one absolute `Tm`, then a
relative `Td` step per line. Split into lines must cut it into three objects.

## What it carries

One Helvetica text object, size 12:

| y   | positioned by | text |
|-----|---------------|------|
| 700 | `Tm`          | `DRAWN BY KM` |
| 680 | `0 -20 Td`    | `CHECKED BY JS` |
| 660 | `0 -20 Td`    | `SCALE 1:50` |

Run with any Python 3; it writes the PDF beside itself.
"""
import io
import os

NL = chr(10).encode('ascii')

content = chr(10).join([
    'BT /F1 12 Tf 1 0 0 1 72 700 Tm',
    '(DRAWN BY KM) Tj',
    '0 -20 Td (CHECKED BY JS) Tj',
    '0 -20 Td (SCALE 1:50) Tj',
    'ET',
]).encode('latin-1')

objects = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
    ('<< /Length %d >>' % len(content)).encode('ascii') + NL + b'stream' + NL
    + content + NL + b'endstream',
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
    b'/Encoding /WinAnsiEncoding >>',
]

out = bytearray(b'%PDF-1.7' + NL + b'%' + bytes([0xE2, 0xE3, 0xCF, 0xD3]) + NL)
offsets = []
for i, body in enumerate(objects, start=1):
    offsets.append(len(out))
    out += ('%d 0 obj' % i).encode('ascii') + NL + body + NL + b'endobj' + NL
startxref = len(out)
out += b'xref' + NL + ('0 %d' % (len(objects) + 1)).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for off in offsets:
    out += ('%010d 00000 n ' % off).encode('ascii') + NL
out += ('trailer' + chr(10) + '<< /Size %d /Root 1 0 R >>' % (len(objects) + 1)
        ).encode('ascii') + NL
out += b'startxref' + NL + ('%d' % startxref).encode('ascii') + NL + b'%%EOF' + NL

dest = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    'stacked-labels.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes' % (dest, len(out)))
