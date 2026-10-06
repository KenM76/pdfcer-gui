"""Author `print-shop.pdf`: one Letter page whose render trips three of the
engine's DIVERGENCE counters and carries detail too small to see at fit.

    python fixtures/print-shop.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/print_shop.rs`.

- `/Nowhere sh` names a shading the page's resources do not hold, so the
  engine refuses it and counts `shading.refused` (`shadings_refused`).
- `/Nowhere cs` names a colour space the resources do not hold, so it counts
  `color.spaces_unresolved` (`cs_unresolved`).
- `/Pattern cs /Nowhere scn` selects a pattern the resources do not hold, so
  the fill paints nothing and counts `color.patterns_unpainted`.
- 400 copies of a 10 x 10 form XObject drawn at 0.02 scale: each is 0.2 pt
  square, under `pdfcer_render::interpret::SUBPIXEL_CULL_PX` (half a device
  pixel) at any fit zoom below 2.5 px/pt, so with subpixel culling on every one
  is skipped and counted in `subpixel_culled`; with it off none is.
- Two overlapping process-colour patches, cyan then magenta, the magenta one
  painted under `/GS0` (`/OP true /op true /OPM 1`) in a page group whose
  blending space is DeviceCMYK. The engine composites that in ink, so the
  overlap holds C = M = 100 % rather than knocking the cyan out: the ink probe's
  calibration point, near page point (400, 152).
- A visible frame, so the page is not blank.
"""
import io

DST = 'fixtures/print-shop.pdf'

tiny = []
for row in range(20):
    for col in range(20):
        x = 100 + col * 20
        y = 300 + row * 20
        tiny.append(f'q 0.02 0 0 0.02 {x} {y} cm /Fm0 Do Q')
content = '\n'.join(
    [
        '2 w 36 36 540 720 re S',
        '/Nowhere sh',
        '/Nowhere cs 0.5 sc 72 100 144 72 re f',
        '/Pattern cs /Nowhere scn 72 200 144 72 re f',
        '1 0 0 0 k 300 100 144 72 re f q /GS0 gs 0 1 0 0 k 340 120 144 72 re f Q',
        *tiny,
    ]
).encode('ascii')
form = b'0 0 10 10 re f'

objects = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Group << /S /Transparency /CS /DeviceCMYK >> '
    b'/Resources << /XObject << /Fm0 5 0 R >> '
    b'/ExtGState << /GS0 << /OP true /op true /OPM 1 >> >> >> /Contents 4 0 R >>',
    b'<< /Length %d >>\nstream\n' % len(content) + content + b'\nendstream',
    b'<< /Type /XObject /Subtype /Form /BBox [0 0 10 10] /Length %d >>\nstream\n'
    % len(form) + form + b'\nendstream',
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
