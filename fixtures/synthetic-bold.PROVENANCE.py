"""Author `synthetic-bold.pdf`: one Letter page with one line, `heavy text`,
in Helvetica 12 pt from (72, 700), made bold by a stroke rather than by its
face: render mode 2 (fill, then stroke) with a 0.264 pt line width, the
stroke-to-size ratio pdfcer itself writes for a synthesized bold.

    python fixtures/synthetic-bold.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/synthetic_bold.rs`.
"""
import io

DST = 'fixtures/synthetic-bold.pdf'

content = b'BT /F1 12 Tf 2 Tr 0.264 w 72 700 Td (heavy text) Tj ET'
objects = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
]

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objects, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n + body + b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objects) + 1))
for off in offsets:
    out.write(b'%010d 00000 n \n' % off)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objects) + 1, xref))
io.open(DST, 'wb').write(out.getvalue())
print('wrote', DST, len(out.getvalue()), 'bytes')
