# -*- coding: utf-8 -*-
"""Build fixtures/retype-seam.pdf -- a word drawn in two fonts.

## Why this fixture exists

The engine's exact text surgery refuses an edit to a word whose letters are
drawn by show operators in different fonts, and offers the Retype workaround:
remove the operators and set the new text at the first one's origin, size and
colour, in its own font where that encodes the text and otherwise in a face
the replacement-face ladder picks from the installed fonts. The check
`a_letter_no_page_font_has_is_typed_in_an_installed_face` needs that word.

## What it carries

One text object at y=700, 12 pt: `Hel` in Helvetica, `lo` in Times-Roman,
then ` world` in Helvetica. Neither font is embedded, and neither encodes the
Greek capital omega.

Run with any Python 3; it writes the PDF beside itself.
"""
import io
import os

NL = chr(10).encode('ascii')

content = chr(10).join([
    'BT /F1 12 Tf 72 700 Td',
    '(Hel) Tj /F2 12 Tf (lo) Tj /F1 12 Tf ( world) Tj',
    'ET',
]).encode('latin-1')

objects = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> /Contents 4 0 R >>',
    ('<< /Length %d >>' % len(content)).encode('ascii') + NL + b'stream' + NL
    + content + NL + b'endstream',
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
    b'/Encoding /WinAnsiEncoding >>',
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman '
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
                    'retype-seam.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes' % (dest, len(out)))
