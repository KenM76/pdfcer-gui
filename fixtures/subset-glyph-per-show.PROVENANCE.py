# -*- coding: utf-8 -*-
"""Build fixtures/subset-glyph-per-show.pdf -- `ABC` in a subset font, one
show operator per glyph, inside one text object.

## Why this fixture exists

`subset-font-floor.pdf` draws `ABC` with one `Tj`, so a change there always
lies in one show operator. The check
`a_change_across_show_operators_is_set_in_the_nearest_face` needs the
opposite: a run whose change spans several `Tj`s inside one `BT ... ET`, in a
font that lacks the typed keys, so the engine's fallback face
(`EditOptions::with_fallback`) has to split a match across operators.

## What it carries

The font objects of `subset-font-floor.pdf` (`SUBSET+pdfceSubsetDemo`, an
embedded TrueType subset of `A`, `B`, `C`, widths 600) copied unchanged, and
one page, 612 x 792 pt, whose content is

    BT /F0 48 Tf 72 600 Td (A) Tj (B) Tj (C) Tj ET

so the run reads `ABC` at the same place as in `subset-font-floor.pdf`, with
its centre at (115.2, 612.0).

`--across-et OUT` writes a variant to OUT instead, each glyph in its own
`BT ... ET` with its own `Td`: the shape the engine still refuses a fallback
for. It is not a fixture; it reproduces that measurement.

Run with any Python 3; it reads `subset-font-floor.pdf` beside itself.
"""
import io
import os
import re
import sys

NL = chr(10).encode('ascii')
HERE = os.path.dirname(os.path.abspath(__file__))

one_object = chr(10).join([
    'BT', '/F0 48 Tf', '72 600 Td', '(A) Tj', '(B) Tj', '(C) Tj', 'ET', ''])
across_et = chr(10).join(
    ['BT /F0 48 Tf %.1f 600 Td (%s) Tj ET' % (72 + 28.8 * i, g)
     for i, g in enumerate('ABC')] + [''])


def source_objects():
    """Objects 5 to 7 (font, descriptor, font program) of the floor fixture."""
    with io.open(os.path.join(HERE, 'subset-font-floor.pdf'), 'rb') as f:
        data = f.read()
    found = {}
    for m in re.finditer(rb'(\d+) 0 obj' + NL + rb'(.*?)' + NL + rb'endobj',
                         data, re.S):
        found[int(m.group(1))] = m.group(2)
    return [found[5], found[6], found[7]]


def build(content):
    content = content.encode('latin-1')
    objects = [
        b'<< /Type /Catalog /Pages 2 0 R >>',
        b'<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 612 792] '
        b'/Resources << /Font << /F0 5 0 R >> >> >>',
        b'<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>',
        ('<< /Length %d >>' % len(content)).encode('ascii') + NL + b'stream'
        + NL + content + NL + b'endstream',
    ] + source_objects()
    out = bytearray(b'%PDF-1.7' + NL + b'%' + bytes([0xE2, 0xE3, 0xCF, 0xD3])
                    + NL)
    offsets = []
    for i, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += ('%d 0 obj' % i).encode('ascii') + NL + body + NL + b'endobj' + NL
    startxref = len(out)
    out += b'xref' + NL + ('0 %d' % (len(objects) + 1)).encode('ascii') + NL
    out += b'0000000000 65535 f ' + NL
    for off in offsets:
        out += ('%010d 00000 n ' % off).encode('ascii') + NL
    out += ('trailer' + chr(10) + '<< /Size %d /Root 1 0 R >>'
            % (len(objects) + 1)).encode('ascii') + NL
    out += (b'startxref' + NL + ('%d' % startxref).encode('ascii') + NL
            + b'%%EOF' + NL)
    return bytes(out)


if len(sys.argv) == 3 and sys.argv[1] == '--across-et':
    dest, body = sys.argv[2], across_et
else:
    dest, body = os.path.join(HERE, 'subset-glyph-per-show.pdf'), one_object
pdf = build(body)
with io.open(dest, 'wb') as f:
    f.write(pdf)
print('wrote %s: %d bytes' % (dest, len(pdf)))
