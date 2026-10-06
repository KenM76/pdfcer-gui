"""Author `repeated-word.pdf`: one Letter page whose single show operator reads
`M10 x M10`, Helvetica 12 pt, baseline (72, 700).

    python fixtures/repeated-word.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/repeated_word.rs`: a restyle of the
second `M10` must restyle that copy and not the first. Its glyphs start at
x = 108 (`M10 x ` is 3001/1000 em in Helvetica's widths, 36.0 pt at 12 pt).
"""
import io

DST = 'fixtures/repeated-word.pdf'

content = b'BT /F1 12 Tf 72 700 Td (M10 x M10) Tj ET'
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
