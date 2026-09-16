# -*- coding: utf-8 -*-
"""Build fixtures/three-text-fields.pdf.

Modelled on text-field-with-appearance.pdf, which this project already ships,
and different from it in the one way O204's driven check needs: it carries
THREE text widgets rather than one.

Why three and not two: a ring of two cannot tell "Tab moved to the next stop"
from "Tab wrapped", so a forward press and a backward press would land on the
same field and the direction could not be asserted. Three is the smallest
number on which forward and backward differ.

Why every widget carries an /AP: `canvas::forms::boxes` puts a widget in the
on-canvas census only when it can draw it, and the shell routes an /AP-less
field to the properties panel instead. Every text field in the engine's own
form corpus is /AP-less, so none of them can be clicked on the page -- which is
the gesture the operator's report is about.

Offsets are computed here rather than written out, so the file can be edited
and regenerated without hand-counting bytes.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, y, value, ap_ref, page_ref):
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /T (%s) /TU (%s) '
        '/Rect [70 %d 260 %d] /F 4 /P %d 0 R /DA (/Helv 10 Tf 0 g) /V (%s) '
        '/AP << /N %d 0 R >> >>' % (name, name, y, y + 25, page_ref, value, ap_ref)
    ).encode('ascii')


def appearance(value):
    body = ('/Tx BMC q BT /Helv 10 Tf 0 g 2 7 Td (%s) Tj ET Q EMC' % value).encode('ascii')
    head = (
        '<< /Type /XObject /Subtype /Form /BBox [0 0 190 25] '
        '/Resources << /Font << /Helv 11 0 R >> >> /Length %d >>' % len(body)
    ).encode('ascii')
    return head + NL + b'stream' + NL + body + NL + b'endstream'


CONTENT = (
    b'BT /Helv 10 Tf 20 232 Td (One:) Tj ET' + NL +
    b'BT /Helv 10 Tf 20 182 Td (Two:) Tj ET' + NL +
    b'BT /Helv 10 Tf 20 132 Td (Three:) Tj ET'
)

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R 7 0 R 9 0 R] '
       b'/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 11 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 260] '
       b'/Resources << /Font << /Helv 11 0 R >> >> /Contents 4 0 R '
       b'/Annots [5 0 R 7 0 R 9 0 R] >>',
    4: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
    5: widget('FieldOne', 225, 'Ada', 6, 3),
    6: appearance('Ada'),
    7: widget('FieldTwo', 175, 'Bea', 8, 3),
    8: appearance('Bea'),
    9: widget('FieldThree', 125, 'Cyd', 10, 3),
    10: appearance('Cyd'),
    11: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
        b'/Encoding /WinAnsiEncoding >>',
}

out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL

offsets = {}
for num in sorted(objects):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL
    out += objects[num] + NL
    out += b'endobj' + NL

startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % (len(objects) + 1)).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in sorted(objects):
    out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % (len(objects) + 1)).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/three-text-fields.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
