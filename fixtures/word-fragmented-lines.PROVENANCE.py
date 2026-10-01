# -*- coding: utf-8 -*-
"""Build fixtures/word-fragmented-lines.pdf -- three lines written the way a
word processor exports them.

## Why this fixture exists

A Word-exported form draws one visual line as several text objects. Editing
such a line must reach only the operators the change touches; the request for
the whole line matches nothing, because the engine matches show operators
inside one text object only. The checks that drive narrowing need that shape
with no personal data.

## What it carries

As Word writes them: each line is one `/P <</MCID n>> BDC … EMC`, and inside it
every fragment is its own `q <clip> W* n BT /Fx 12 Tf 1 0 0 1 x y Tm [(...)] TJ
ET Q`, with the next fragment's x at the previous one's end from the
standard-14 AFM widths, so the extractor joins each line into one run.

| line | fragments | what it is for |
|---|---|---|
| y=700 | `Date Premises Required____`, ` ` | a trailing space-only object |
| y=670 | `(   ) Common`, `-`, `Law`, ` ` | a word split across three objects |
| y=640 | `Applicant`, right quote + `s` (Times-Roman), ` Name____`, ` ` | a curly-apostrophe run in a second font |

Run with any Python 3; it writes the PDF beside itself.
"""
import io
import os

NL = chr(10).encode('ascii')

# /Widths in 1000ths of an em, from the Helvetica and Times-Roman AFMs.
HELV = {' ': 278, '(': 333, ')': 333, '-': 333, '_': 556, 'A': 667, 'C': 722,
        'D': 722, 'L': 556, 'N': 722, 'P': 667, 'R': 722, 'a': 556, 'c': 500,
        'd': 556, 'e': 556, 'i': 222, 'l': 222, 'm': 833, 'n': 556, 'o': 556,
        'p': 556, 'q': 556, 'r': 333, 's': 500, 't': 278, 'u': 556, 'w': 722}
TIMES = {'\x92': 333, 's': 389}
FONTS = {'F1': HELV, 'F2': TIMES}
SIZE = 12

LINES = [
    (700, [('F1', 'Date Premises Required____'), ('F1', ' ')]),
    (670, [('F1', '(   ) Common'), ('F1', '-'), ('F1', 'Law'), ('F1', ' ')]),
    (640, [('F1', 'Applicant'), ('F2', '\x92s'), ('F1', ' Name____'),
           ('F1', ' ')]),
]


def escape(text):
    bs = chr(92)
    for c in (bs, '(', ')'):
        text = text.replace(c, bs + c)
    return text


ops = []
for mcid, (y, fragments) in enumerate(LINES):
    ops.append('/P <</MCID %d>> BDC' % mcid)
    x = 72.0
    for font, text in fragments:
        ops.append('q 0 0 612 792 re W* n BT /%s %d Tf 1 0 0 1 %.3f %d Tm 0 g '
                   '[(%s)] TJ ET Q' % (font, SIZE, x, y, escape(text)))
        x += sum(FONTS[font][c] for c in text) / 1000.0 * SIZE
    ops.append('EMC')
content = chr(10).join(ops).encode('latin-1')

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
                    'word-fragmented-lines.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes, %d lines' % (dest, len(out), len(LINES)))
