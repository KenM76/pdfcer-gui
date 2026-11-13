# -*- coding: utf-8 -*-
"""Build fixtures/page-box-sizes.pdf.

Four pages, each with one black square, for Document properties ▸ Page box
sizes. Pages that agree on all five box sizes and how each resolves share an
entry:

- pages 1 and 2: 600 x 800 sheet, /BleedBox [10 10 590 790] and
  /TrimBox [20 20 580 780] written, crop and art defaulted (one entry);
- page 3: the same sheet with no boxes written (its own entry: the control
  that a missing trim box is not merged with a written one);
- page 4: a 1000 x 700 sheet with /ArtBox [100 100 900 600].

Offsets are computed here, so the file can be edited and regenerated.
"""
import io

NL = chr(10).encode('ascii')
CONTENT = b'0 g 20 20 60 60 re f'

PAGE = (b'<< /Type /Page /Parent 2 0 R %s'
        b'/Resources << >> /Contents 7 0 R >>')
objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R >>',
    2: b'<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R 6 0 R] /Count 4 >>',
    3: PAGE % b'/MediaBox [0 0 600 800] /BleedBox [10 10 590 790] /TrimBox [20 20 580 780] ',
    4: PAGE % b'/MediaBox [0 0 600 800] /BleedBox [10 10 590 790] /TrimBox [20 20 580 780] ',
    5: PAGE % b'/MediaBox [0 0 600 800] ',
    6: PAGE % b'/MediaBox [0 0 1000 700] /ArtBox [100 100 900 600] ',
    7: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
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

dst = 'fixtures/page-box-sizes.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
