# -*- coding: utf-8 -*-
"""Build fixtures/off-page-blank-overhang.pdf: one 200 x 200 page, two 4 x 1
DeviceGray images each crossing an edge.

- Im1 spans x 150..250 across the RIGHT edge. Its two samples beyond x = 200
  are 255, paper, which is the state a clean leaves a picture in. The engine's
  scan decodes them and counts the picture in `PageScan::inkless_overhang`
  rather than listing it.
- Im2 spans x -50..50 across the LEFT edge, all samples 0, so it has ink
  beyond the edge and is listed as partial.

Expected census: objects=1, blank_overhangs=1. Run from the repository root.
"""
import io

def image(samples):
    return (b'<< /Type /XObject /Subtype /Image /Width 4 /Height 1 /ColorSpace /DeviceGray '
            b'/BitsPerComponent 8 /Length 4 >>\nstream\n' + bytes(samples) + b'\nendstream')

content = b'q 100 0 0 20 150 90 cm /Im1 Do Q\nq 100 0 0 20 -50 40 cm /Im2 Do Q'
objs = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] '
    b'/Resources << /XObject << /Im1 5 0 R /Im2 6 0 R >> >> /Contents 4 0 R >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    image([0, 0, 255, 255]),
    image([0, 0, 0, 0]),
]
out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objs, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n + body + b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1))
for o in offsets:
    out.write(b'%010d 00000 n \n' % o)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objs) + 1, xref))
io.open('fixtures/off-page-blank-overhang.pdf', 'wb').write(out.getvalue())
print('wrote fixtures/off-page-blank-overhang.pdf', out.tell(), 'bytes')
