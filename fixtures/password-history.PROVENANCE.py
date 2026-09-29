# -*- coding: utf-8 -*-
"""Build fixtures/password-history.pdf: password-field.pdf's one Password
field, saved twice. Version 1 stores /V (s3cret); version 2 is an incremental
update that rewrites the field without /V. A reader sees an empty field; the
old value is still in the bytes, which is what Remove old passwords... finds.

Shape follows password-field.PROVENANCE.py.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, y, ap_ref, page_ref, value):
    # /Ff 8192 is bit 14, Password (ISO 32000 §12.7.4.3).
    v = '' if value is None else '/V (%s) ' % value
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /Ff 8192 /T (%s) /TU (%s) %s'
        '/Rect [70 %d 260 %d] /F 4 /P %d 0 R /DA (/Helv 10 Tf 0 g) '
        '/AP << /N %d 0 R >> >>' % (name, name, v, y, y + 25, page_ref, ap_ref)
    ).encode('ascii')


def appearance(value):
    body = ('/Tx BMC q BT /Helv 10 Tf 0 g 2 7 Td (%s) Tj ET Q EMC' % value).encode('ascii')
    head = (
        '<< /Type /XObject /Subtype /Form /BBox [0 0 190 25] '
        '/Resources << /Font << /Helv 7 0 R >> >> /Length %d >>' % len(body)
    ).encode('ascii')
    return head + NL + b'stream' + NL + body + NL + b'endstream'


CONTENT = b'BT /Helv 10 Tf 20 232 Td (PIN:) Tj ET'

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] '
       b'/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 7 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 260] '
       b'/Resources << /Font << /Helv 7 0 R >> >> /Contents 4 0 R '
       b'/Annots [5 0 R] >>',
    4: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
    5: widget('Pin', 225, 6, 3, 's3cret'),
    6: appearance('******'),
    7: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
        b'/Encoding /WinAnsiEncoding >>',
}


def xref_section(out, offsets, first_line):
    out += b'xref' + NL
    for start, nums in first_line:
        out += ('%d %d' % (start, len(nums))).encode('ascii') + NL
        for num in nums:
            if num == 0:
                out += b'0000000000 65535 f ' + NL
            else:
                out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL


out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL

offsets = {}
for num in sorted(objects):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL
    out += objects[num] + NL
    out += b'endobj' + NL

first_xref = len(out)
xref_section(out, offsets, [(0, [0] + sorted(objects))])
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % (len(objects) + 1)).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % first_xref).encode('ascii') + NL
out += b'%%EOF' + NL

# Version 2: the field without /V and an empty appearance, appended.
update = {5: widget('Pin', 225, 6, 3, None), 6: appearance('')}
for num in sorted(update):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL
    out += update[num] + NL
    out += b'endobj' + NL

second_xref = len(out)
xref_section(out, offsets, [(0, [0]), (5, [5, 6])])
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R /Prev %d >>' % (len(objects) + 1, first_xref)).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % second_xref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/password-history.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
