"""Author `output-preview.pdf`: one Letter page for Output Preview's overprint
line, soft proof and ink-coverage warnings.

    python fixtures/output-preview.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/output_preview_*.rs`.

- A page group whose blending space is DeviceCMYK, so the separation preview
  composites in ink.
- `/OutputIntents` naming `output-preview.icc` as `/DestOutputProfile`, so the
  soft proof can offer the document's own press condition. The profile is
  byte-for-byte pdfcer's `fixtures/synthetic/icc-rgb/dest-cmyk.icc` (from
  iccce, MIT, the same author), copied in beside this script.
- A 400% patch, C = M = Y = K = 1, at page points (72, 500)-(272, 650), painted
  under `/GS0` (`/OP true /op true /OPM 1`). Every component is non-zero, so
  overprint is set but leaves nothing to the backdrop: the page reads
  "set, no effect". The patch is over every total area coverage limit the
  panel offers.
- A rich black patch, C = 0.6, K = 1, M = Y = 0, at (340, 500)-(540, 650),
  painted with overprint off.
- A plain black patch, K = 1 alone, at (340, 300)-(540, 450): black but not
  rich, and 100% coverage, so neither warning flags it.
- A 40% cyan patch at (72, 300)-(272, 450): the control that flags nothing.
"""
import io
import os

HERE = os.path.dirname(os.path.abspath(__file__))
DST = os.path.join(HERE, 'output-preview.pdf')
ICC = os.path.join(HERE, 'output-preview.icc')

icc = io.open(ICC, 'rb').read()
content = '\n'.join(
    [
        '2 w 36 36 540 720 re S',
        'q /GS0 gs 1 1 1 1 k 72 500 200 150 re f Q',
        '0.6 0 0 1 k 340 500 200 150 re f',
        '0 0 0 1 k 340 300 200 150 re f',
        '0.4 0 0 0 k 72 300 200 150 re f',
    ]
).encode('ascii')

objects = [
    b'<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent '
    b'/S /GTS_PDFX /OutputConditionIdentifier (pdfcer test press) '
    b'/DestOutputProfile 5 0 R >>] >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Group << /S /Transparency /CS /DeviceCMYK >> '
    b'/Resources << /ExtGState << /GS0 << /OP true /op true /OPM 1 >> >> >> '
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
