"""Author `prepress-mix.pdf`: one Letter page with one case for each File >
Print prepress command.

    python fixtures/prepress-mix.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/prepress.rs`.

- A 0.05 pt stroke from (72, 700) to (540, 700): a hairline under the 0.25 pt
  default, so Fix hairlines widens exactly one line.
- A grey square at (72, 400)-(272, 600) painted under `/GS0` (`/ca 0.5`), over
  a black square at (172, 300)-(372, 500): the page's one transparent area,
  so Flatten transparency has one page to flatten.
- A DeviceRGB red square at (340, 100)-(540, 250): the page's one RGB paint,
  so Convert colours converts it and Preflight's `rgb-colour` rule finds it.
- `/OutputIntents` naming `output-preview.icc` (byte-for-byte the CMYK
  profile beside `output-preview.PROVENANCE.py`) as `/DestOutputProfile`, so
  Convert colours can offer the document's own output intent.
"""
import io
import os

HERE = os.path.dirname(os.path.abspath(__file__))
DST = os.path.join(HERE, 'prepress-mix.pdf')
ICC = os.path.join(HERE, 'output-preview.icc')

icc = io.open(ICC, 'rb').read()
content = '\n'.join(
    [
        '0 0 0 1 K 0.05 w 72 700 m 540 700 l S',
        '0 0 0 1 k 172 300 200 200 re f',
        'q /GS0 gs 0 0 0 0.5 k 72 400 200 200 re f Q',
        '1 0 0 rg 340 100 200 150 re f',
    ]
).encode('ascii')

objects = [
    b'<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent '
    b'/S /GTS_PDFX /OutputConditionIdentifier (pdfcer test press) '
    b'/DestOutputProfile 5 0 R >>] >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Resources << /ExtGState << /GS0 << /ca 0.5 /CA 0.5 >> >> >> '
    b'/Contents 4 0 R >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    b'<< /N 4 /Length %d >>\nstream\n' % len(icc) + icc + b'\nendstream',
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
