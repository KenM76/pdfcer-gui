# -*- coding: utf-8 -*-
"""Build fixtures/page-boxes.pdf.

Three 200 x 100 pages, each with one black square, whose page boxes resolve
three ways under ISO 32000-2 14.11.2.1:

- page 1 writes no CropBox, TrimBox, BleedBox or ArtBox (all defaulted; the
  control);
- page 2 writes /CropBox [-20 -20 220 120], past the MediaBox on every side,
  so it is clipped to the MediaBox;
- page 3 writes /TrimBox [300 300 400 400], wholly off the sheet, so it
  cannot be used and the default (the crop box) stands in.

Offsets are computed here, so the file can be edited and regenerated.
"""
import io

NL = chr(10).encode('ascii')
CONTENT = b'0 g 20 20 60 60 re f'

PAGE = (b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] %s'
        b'/Resources << >> /Contents 6 0 R >>')
objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R >>',
    2: b'<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>',
    3: PAGE % b'',
    4: PAGE % b'/CropBox [-20 -20 220 120] ',
    5: PAGE % b'/TrimBox [300 300 400 400] ',
    6: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
}

out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL
offsets = {}
for num in sorted(objects):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL + objects[num] + NL + b'endobj' + NL
startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % (len(objects) + 1)).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in sorted(objects):
    out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % (len(objects) + 1)).encode('ascii') + NL
out += b'startxref' + NL + ('%d' % startxref).encode('ascii') + NL + b'%%EOF' + NL

dst = 'fixtures/page-boxes.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
