# -*- coding: utf-8 -*-
"""Build fixtures/esign-one-signed.pdf -- three signature boxes, one already hand-signed.

## Why this fixture exists

O285: a hand signature is recorded in the document, so a box signed in an
earlier session still reads signed on opening. This is the file such a session
saves: `fixtures/esign-three-boxes.pdf` with `ApplicantSig` signed.

## What it carries

| field | box (points) | what it is for |
|---|---|---|
| `ApplicantSig` | 220 x 18 | a single-text-line box, where the signature rises above the box |
| `CoApplicantSig` | 220 x 18 | a second identical box, so "the others stay tagged" is measurable |
| `WitnessSig` | 220 x 48 | a tall box, where the signature is centred inside it |

`ApplicantSig` carries a drawn stroke inside its box, wrapped in
`/pdfc_HandSig <</Field (ApplicantSig)>> BDC ... EMC`, the tag
`pdfcer_core::hand_sig` writes and reads. `CoApplicantSig` carries the same tag
around nothing, which is what deleting a signature's objects leaves behind: it
paints nothing, so that box is unsigned. `WitnessSig` is untouched.

All three are `/FT /Sig` with no `/V`: a hand signature is content, never a
certificate signature. Each widget carries an `/AP`
holding only a thin border, because `canvas::forms::boxes` admits a widget to
the on-canvas census only when it can draw it. The page content holds only
black captions to the left of each box, so ink pixels inside a box can only be
the signature.

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
PAGE = emit(b'PLACEHOLDER')
CONTENTS = emit(b'PLACEHOLDER')
HELV = emit(
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
    b'/Encoding /WinAnsiEncoding >>'
)

PAGE_W, PAGE_H = 420, 420
LEFT, BOX_W = 160, 220
BOXES = [
    ('ApplicantSig', 'Applicant', 330, 18),
    ('CoApplicantSig', 'Co-applicant', 260, 18),
    ('WitnessSig', 'Witness', 150, 48),
]


def border(w, h):
    body = ('q 0.5 G 0.5 w 0.25 0.25 %.1f %.1f re S Q' % (w - 0.5, h - 0.5)
            ).encode('ascii')
    head = ('<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] '
            '/Resources << >> /Length %%d >>' % (w, h))
    return stream(head, body)


fields = []
ops = ['BT /Helv 13 Tf 20 %d Td (Three signature boxes) Tj ET' % (PAGE_H - 40),
       '/pdfc_HandSig <</Field (ApplicantSig)>> BDC '
       '0.05 0.1 0.35 RG 1 w 170 333 m 300 343 l S EMC',
       '/pdfc_HandSig <</Field (CoApplicantSig)>> BDC EMC']
for name, caption, y, h in BOXES:
    ap = emit(border(BOX_W, h))
    num = emit((
        '<< /Type /Annot /Subtype /Widget /FT /Sig /T (%s) '
        '/TU (Sign here) /Rect [%d %d %d %d] /F 4 /P %s '
        '/MK << /BC [0 0 0] >> /BS << /W 1 /S /S >> /AP << /N %s >> >>'
        % (name, LEFT, y, LEFT + BOX_W, y + h, ref(PAGE), ref(ap))
    ).encode('ascii'))
    fields.append(num)
    ops.append('BT /Helv 10 Tf 20 %d Td (%s signature) Tj ET' % (y + 4, caption))
CONTENT = (chr(10).join(ops)).encode('ascii')

_objects[CATALOG - 1] = (
    '<< /Type /Catalog /Pages %s /AcroForm << /Fields [%s] '
    '/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv %s >> >> >> >>'
    % (ref(PAGES), ' '.join(ref(n) for n in fields), ref(HELV))
).encode('ascii')
_objects[PAGES - 1] = (
    '<< /Type /Pages /Kids [%s] /Count 1 >>' % ref(PAGE)
).encode('ascii')
_objects[PAGE - 1] = (
    '<< /Type /Page /Parent %s /MediaBox [0 0 %d %d] '
    '/Resources << /Font << /Helv %s >> >> /Contents %s /Annots [%s] >>'
    % (ref(PAGES), PAGE_W, PAGE_H, ref(HELV), ref(CONTENTS),
       ' '.join(ref(n) for n in fields))
).encode('ascii')
_objects[CONTENTS - 1] = stream('<< /Length %d >>', CONTENT)

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
                    'esign-one-signed.pdf')
with io.open(dest, 'wb') as f:
    f.write(bytes(out))
print('wrote %s: %d bytes, %d objects, %d fields'
      % (dest, len(out), len(_objects), len(fields)))
