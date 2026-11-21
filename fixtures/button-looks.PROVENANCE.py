# -*- coding: utf-8 -*-
"""Build fixtures/button-looks.pdf.

Two button widgets whose /AP carries a rollover (/R) and a down (/D) look
beside the normal one (ISO 32000-1 12.5.5 Table 168), each a flat fill so a
screenshot can tell them apart:

  Go    push button, /H /P   /N grey   /R blue    /D red
  Tick  check box, /AS /Off  /N {Off: grey, Yes: grey}
                             /R {Off: blue}  /D {Off: red, Yes: red}

`ui-verify` `a_buttons_rollover_and_down_looks_are_drawn` hovers and presses
each and reads the colour. Written from scratch here; rights-clean. Offsets
are computed.
"""
import io

NL = chr(10).encode('ascii')

W, H = 160, 60


def fill(r, g, b):
    body = ('%g %g %g rg 0 0 %d %d re f' % (r, g, b, W, H)).encode('ascii')
    return (b'<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] /Length %d >>'
            % (W, H, len(body)) + NL + b'stream' + NL + body + NL + b'endstream')


GREY, BLUE, RED = (0.8, 0.8, 0.8), (0.1, 0.2, 0.9), (0.9, 0.1, 0.1)

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R 5 0 R] >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
       b'/Annots [4 0 R 5 0 R] >>',
    4: b'<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 65536 /T (Go) '
       b'/Rect [72 600 232 660] /F 4 /P 3 0 R /H /P /MK << /CA (Go) >> '
       b'/AP << /N 10 0 R /R 11 0 R /D 12 0 R >> >>',
    5: b'<< /Type /Annot /Subtype /Widget /FT /Btn /T (Tick) /V /Off /AS /Off '
       b'/Rect [72 450 232 510] /F 4 /P 3 0 R '
       b'/AP << /N << /Off 13 0 R /Yes 14 0 R >> /R << /Off 15 0 R >> '
       b'/D << /Off 16 0 R /Yes 17 0 R >> >> >>',
    10: fill(*GREY),
    11: fill(*BLUE),
    12: fill(*RED),
    13: fill(*GREY),
    14: fill(*GREY),
    15: fill(*BLUE),
    16: fill(*RED),
    17: fill(*RED),
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

size = max(objects) + 1
startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % size).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in range(1, size):
    if num in offsets:
        out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
    else:
        out += b'0000000000 65535 f ' + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % size).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/button-looks.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
