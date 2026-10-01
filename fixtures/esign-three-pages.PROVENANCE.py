# -*- coding: utf-8 -*-
"""Build fixtures/esign-three-pages.pdf -- one empty signature box on each of three tall pages.

## Why this fixture exists

O269: the signing strip's *Next* button must bring each signature box still to
sign into view, in document order, skipping the ones already signed. That is
only measurable when the boxes cannot all be on screen at once, so each sits
near the foot of its own tall page.

## What it carries

| page | field | box (points) |
|---|---|---|
| 1 | `FirstSig` | 220 x 18 at y=60 |
| 2 | `SecondSig` | 220 x 18 at y=60 |
| 3 | `ThirdSig` | 220 x 18 at y=60 |

Pages are 420 x 1400 points. Every field is `/FT /Sig`, unsigned, with no `/V`.
Each widget carries an `/AP` holding only a thin grey border (the on-canvas
census admits only widgets it can draw). Page content is black captions only,
so blue ink inside a box can only be a signature.

Run with any Python 3; it writes the PDF beside itself.
"""
import io
import os

NL = chr(10).encode('ascii')
_objects = []


def emit(body):
    _objects.append(body)
    return len(_objects)


def ref(num):
    return '%d 0 R' % num


def stream(dict_head, body):
    return (
        (dict_head % len(body)).encode('ascii') + NL
        + b'stream' + NL + body + NL + b'endstream'
    )


CATALOG = emit(b'PLACEHOLDER')
PAGES = emit(b'PLACEHOLDER')
HELV = emit(
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
    b'/Encoding /WinAnsiEncoding >>'
)

PAGE_W, PAGE_H = 420, 1400
LEFT, BOX_W, BOX_H, BOX_Y = 160, 220, 18, 60
NAMES = ['FirstSig', 'SecondSig', 'ThirdSig']


def border(w, h):
    body = ('q 0.5 G 0.5 w 0.25 0.25 %.1f %.1f re S Q' % (w - 0.5, h - 0.5)
            ).encode('ascii')
    head = ('<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] '
            '/Resources << >> /Length %%d >>' % (w, h))
    return stream(head, body)


fields = []
pages = []
for index, name in enumerate(NAMES):
    page = emit(b'PLACEHOLDER')
    contents = emit(b'PLACEHOLDER')
    ap = emit(border(BOX_W, BOX_H))
    widget = emit((
        '<< /Type /Annot /Subtype /Widget /FT /Sig /T (%s) '
        '/TU (Sign here) /Rect [%d %d %d %d] /F 4 /P %s '
        '/MK << /BC [0 0 0] >> /BS << /W 1 /S /S >> /AP << /N %s >> >>'
        % (name, LEFT, BOX_Y, LEFT + BOX_W, BOX_Y + BOX_H, ref(page), ref(ap))
    ).encode('ascii'))
    fields.append(widget)
    pages.append(page)
    ops = [
        'BT /Helv 13 Tf 20 %d Td (Page %d of 3) Tj ET' % (PAGE_H - 40, index + 1),
        'BT /Helv 10 Tf 20 %d Td (Signature %d) Tj ET' % (BOX_Y + 4, index + 1),
    ]
    _objects[contents - 1] = stream(
        '<< /Length %d >>', (chr(10).join(ops)).encode('ascii'))
    _objects[page - 1] = (
        '<< /Type /Page /Parent %s /MediaBox [0 0 %d %d] '
        '/Resources << /Font << /Helv %s >> >> /Contents %s /Annots [%s] >>'
        % (ref(PAGES), PAGE_W, PAGE_H, ref(HELV), ref(contents), ref(widget))
    ).encode('ascii')

_objects[CATALOG - 1] = (
    '<< /Type /Catalog /Pages %s /AcroForm << /Fields [%s] '
    '/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv %s >> >> >> >>'
    % (ref(PAGES), ' '.join(ref(n) for n in fields), ref(HELV))
).encode('ascii')
_objects[PAGES - 1] = (
    '<< /Type /Pages /Kids [%s] /Count %d >>'
    % (' '.join(ref(p) for p in pages), len(pages))
).encode('ascii')

out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL
offsets = {}
for i, body in enumerate(_objects, start=1):
    assert body != b'PLACEHOLDER', 'object %d was never filled in' % i
    offsets[i] = len(out)
    out += ('%d 0 obj' % i).encode('ascii') + NL + body + NL + b'endobj' + NL
startxref = len(out)
count = len(_objects) + 1
out += b'xref' + NL + ('0 %d' % count).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for i in range(1, count):
    out += ('%010d 00000 n ' % offsets[i]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root %s >>' % (count, ref(CATALOG))).encode('ascii') + NL
out += b'startxref' + NL + ('%d' % startxref).encode('ascii') + NL + b'%%EOF' + NL

dest = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    'esign-three-pages.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes, %d objects, %d fields'
      % (dest, len(out), len(_objects), len(fields)))
