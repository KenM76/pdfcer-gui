# -*- coding: utf-8 -*-
"""Build fixtures/quote-operator.pdf -- a line drawn with the ' operator.

## Why this fixture exists

The engine's exact text surgery refuses to edit a run drawn by the `'` show
operator and offers a workaround: rewrite it as the `T*` and `Tj` ISO 32000-1
section 9.4.3 Table 109 defines it to be. The check that drives the
workaround offer (`a_refused_edit_offers_its_workaround`) needs a run that is
refused that way and nothing else.

## What it carries

One Helvetica text object, leading 20:

| y   | operator | text |
|-----|----------|------|
| 700 | `Tj`     | `Plain line` |
| 680 | `'`      | `Quoted line` |

Run with any Python 3; it writes the PDF beside itself.
"""
import io
import os

NL = chr(10).encode('ascii')

content = chr(10).join([
    'BT /F1 12 Tf 20 TL 72 700 Td',
    '(Plain line) Tj',
    "(Quoted line) '",
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
                    'quote-operator.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes' % (dest, len(out)))
