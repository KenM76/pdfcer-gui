# -*- coding: utf-8 -*-
"""Build fixtures/skewed-scans.pdf: four letter pages, each one greyscale
150 dpi "scan" of twenty-four ruled text lines, drawn with Pillow.

Page 1 is tilted 2.0 degrees counter-clockwise, page 2 1.5 degrees
clockwise, page 3 is level, and page 4 is tilted like page 1 but also draws
one line of real text, so File > Straighten scans with its default skips it.
Expected: pages 1 and 2 straightened, page 3 already level, page 4 skipped,
and the two corrections folded into one undo entry. Run from the repository
root; needs Pillow.
"""
import io
import zlib

from PIL import Image, ImageDraw

W, H = 1275, 1650  # 8.5 x 11 in at 150 dpi
TILTS = [2.0, -1.5, 0.0, 2.0]
FONT = b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>'


def scan(degrees):
    img = Image.new('L', (W, H), 255)
    draw = ImageDraw.Draw(img)
    for row in range(24):
        y = 160 + row * 54
        x = 140
        for word in range(9):
            length = 40 + (row * 37 + word * 53) % 70
            draw.rectangle([x, y, x + length, y + 14], fill=0)
            x += length + 22
    return img.rotate(degrees, resample=Image.BICUBIC, fillcolor=255)


pages = len(TILTS)
# Objects: 1 catalog, 2 pages, 3 font, then per page: page, image, content.
objs = [b'<< /Type /Catalog /Pages 2 0 R >>',
        b'<< /Type /Pages /Kids [%s] /Count %d >>'
        % (b' '.join(b'%d 0 R' % (4 + 3 * i) for i in range(pages)), pages),
        FONT]
for i, tilt in enumerate(TILTS):
    page, image, content = 4 + 3 * i, 5 + 3 * i, 6 + 3 * i
    objs.append(b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << '
                b'/XObject << /Im0 %d 0 R >> /Font << /F1 3 0 R >> >> /Contents %d 0 R >>'
                % (image, content))
    pixels = zlib.compress(scan(tilt).tobytes(), 9)
    objs.append((b'<< /Type /XObject /Subtype /Image /Width %d /Height %d /ColorSpace '
                 b'/DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length %d >>'
                 % (W, H, len(pixels)), pixels))
    draw = b'q 612 0 0 792 0 0 cm /Im0 Do Q'
    if i == 3:
        draw += b'\nBT /F1 10 Tf 72 30 Td (Typed footer on a scanned page) Tj ET'
    objs.append((b'<< /Length %d >>' % len(draw), draw))

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objs, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n)
    if isinstance(body, tuple):
        out.write(body[0] + b'\nstream\n' + body[1] + b'\nendstream')
    else:
        out.write(body)
    out.write(b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1))
for o in offsets:
    out.write(b'%010d 00000 n \n' % o)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objs) + 1, xref))
io.open('fixtures/skewed-scans.pdf', 'wb').write(out.getvalue())
print('wrote fixtures/skewed-scans.pdf', out.tell(), 'bytes')
