# -*- coding: utf-8 -*-
"""Build fixtures/password-field.pdf: one page, one text field with the
Password flag and no stored value, carrying an /AP so the canvas can draw it.

Shape follows three-text-fields.PROVENANCE.py.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, y, ap_ref, page_ref):
    # /Ff 8192 is bit 14, Password (ISO 32000 §12.7.4.3). No /V: a password
    # field stores nothing.
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /Ff 8192 /T (%s) /TU (%s) '
        '/Rect [70 %d 260 %d] /F 4 /P %d 0 R /DA (/Helv 10 Tf 0 g) '
        '/AP << /N %d 0 R >> >>' % (name, name, y, y + 25, page_ref, ap_ref)
    ).encode('ascii')


def appearance(value):
    body = ('/Tx BMC q BT /Helv 10 Tf 0 g 2 7 Td (%s) Tj ET Q EMC' % value).encode('ascii')
    head = (
        '<< /Type /XObject /Subtype /Form /BBox [0 0 190 25] '
        '/Resources << /Font << /Helv 7 0 R >> >> /Length %d >>' % len(body)
    ).encode('ascii')
    return head + NL + b'stream' + NL + body + NL + b'endstream'


CONTENT = (
    b'BT /Helv 10 Tf 20 232 Td (PIN:) Tj ET' +
    b''
)

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] '
       b'/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 7 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 260] '
       b'/Resources << /Font << /Helv 7 0 R >> >> /Contents 4 0 R '
       b'/Annots [5 0 R] >>',
    4: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
    5: widget('Pin', 225, 6, 3),
    6: appearance(''),
    7: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
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

dst = 'fixtures/password-field.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
