# -*- coding: utf-8 -*-
"""Build fixtures/rich-comment.pdf.

One page with two sticky notes (`/Text`) whose bodies carry rich text
(`/RC`, ISO 32000-1 12.7.3.4) beside the plain `/Contents`:

- note A: `/RC` as a text STRING, with bold, a red span, and a `/DS` of
  12 pt Helvetica;
- note B: `/RC` as a text STREAM (the other form 12.7.3.4 permits), italic.

The driven check `a_formatted_comment_says_it_is_formatted` requires the
Comments panel to say both notes are formatted and name what each holds.

Offsets are computed here, so the file can be edited and regenerated.
"""

import io
import os

NL = chr(10).encode('ascii')

BODY = ('<?xml version="1.0"?><body xmlns="http://www.w3.org/1999/xhtml" '
        'xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/" '
        'xfa:APIVersion="Acrobat:11.0.0" xfa:spec="2.0.2">%s</body>')
RC_A = BODY % '<p>Check the <b>weld</b> <span style="color:#FF0000">size</span></p>'
RC_B = BODY % '<p><i>Approved as drawn</i></p>'


def pdf_string(s):
    bs = chr(92)
    return '(' + s.replace(bs, bs + bs).replace('(', bs + '(').replace(')', bs + ')') + ')'


BODIES = [
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R '
    '/Resources << >> /Annots [5 0 R 6 0 R] >>',
    ('stream', b''),
    '<< /Type /Annot /Subtype /Text /Rect [100 650 120 670] /Name /Comment '
    '/T (Reviewer) /Contents (Check the weld size) /RC ' + pdf_string(RC_A) +
    ' /DS (font: 12pt Helvetica) /C [1 1 0] >>',
    '<< /Type /Annot /Subtype /Text /Rect [100 550 120 570] /Name /Comment '
    '/T (Reviewer) /Contents (Approved as drawn) /RC 7 0 R /C [1 1 0] >>',
    ('stream', RC_B.encode('ascii')),
]

out = io.BytesIO()
out.write(b'%PDF-1.7' + NL)
offsets = []
for i, body in enumerate(BODIES):
    offsets.append(out.tell())
    if isinstance(body, tuple):
        payload = body[1]
        data = ('<< /Length %d >>' % len(payload)).encode('ascii') + NL + b'stream' + NL \
            + payload + NL + b'endstream'
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
with open(os.path.join(here, 'rich-comment.pdf'), 'wb') as f:
    f.write(out.getvalue())
