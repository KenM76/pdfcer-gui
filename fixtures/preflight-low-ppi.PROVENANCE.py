"""Author `preflight-low-ppi.pdf`: Letter pages with known digital-printing
findings on the first two, for the Preflight panel.

    python fixtures/preflight-low-ppi.PROVENANCE.py

Deterministic: no timestamps, no producer string.

Driven by `tools/ui-verify/src/checks/preflight_panel.rs`.

- `/OutputIntents` naming `output-preview.icc` (a CMYK press profile, copied
  from pdfcer's `fixtures/synthetic/icc-rgb/dest-cmyk.icc`) as
  `/DestOutputProfile`, so the rgb-colour rule's suggested fixup (convert to
  the output intent) has a destination and applies.
- No `/TrimBox`: the no-trim-box rule fires on both pages.
- Page 1: one DeviceRGB red fill, (72, 400)-(272, 600). An rgb-colour finding.
- Page 2: page object 0 is a DeviceRGB blue fill, (72, 600)-(272, 700); page
  object 1 is a 4 x 4 DeviceGray image drawn into a 300 pt square at
  (72, 200), about 1 ppi, under the profile's 150 ppi floor. The image-low-ppi
  finding targets PageObject(1) on page 2 -- index 1, not 0, so a selection
  that ignores the index is caught.
- No text, so no font finding.
- Pages 3 to 242 (`FILLER` pages): a TrimBox and one shared, Flate-compressed
  content stream of `MARKS` small DeviceCMYK cyan fills, so they add no finding. They exist to make the run outlast
  several frames, so the check can see the window draw while it is going.
"""
import io
import os
import zlib

FILLER = 240
MARKS = 2000

HERE = os.path.dirname(os.path.abspath(__file__))
DST = os.path.join(HERE, 'preflight-low-ppi.pdf')
ICC = os.path.join(HERE, 'output-preview.icc')

icc = io.open(ICC, 'rb').read()
page1 = b'1 0 0 rg 72 400 200 200 re f'
page2 = b'0 0 1 rg 72 600 200 100 re f\nq 300 0 0 300 72 200 cm /Im0 Do Q'
pixels = bytes([0, 255, 0, 255, 255, 0, 255, 0, 0, 255, 0, 255, 255, 0, 255, 0])


def stream(dictionary, data):
    return b'<< ' + dictionary + b' /Length %d >>\nstream\n' % len(data) + data + b'\nendstream'


# Objects 9 (the filler content) and 10.. (the filler pages).
kids = b''.join(b' %d 0 R' % (10 + i) for i in range(FILLER))

objects = [
    b'<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent '
    b'/S /GTS_PDFX /OutputConditionIdentifier (pdfcer test press) '
    b'/DestOutputProfile 5 0 R >>] >>',
    b'<< /Type /Pages /Kids [3 0 R 6 0 R' + kids + b'] /Count %d >>' % (2 + FILLER),
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> '
    b'/Contents 4 0 R >>',
    stream(b'', page1),
    stream(b'/N 4', icc),
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    b'/Resources << /XObject << /Im0 8 0 R >> >> /Contents 7 0 R >>',
    stream(b'', page2),
    stream(b'/Type /XObject /Subtype /Image /Width 4 /Height 4 '
           b'/ColorSpace /DeviceGray /BitsPerComponent 8', pixels),
    stream(b'/Filter /FlateDecode', zlib.compress(b'1 0 0 0 k\n' + b''.join(
        b'%d %d 2 2 re f\n' % (72 + i % 400, 72 + i // 400 * 4) for i in range(MARKS)), 9)),
]
objects += [b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
            b'/TrimBox [0 0 612 792] /Resources << >> /Contents 9 0 R >>'] * FILLER

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
