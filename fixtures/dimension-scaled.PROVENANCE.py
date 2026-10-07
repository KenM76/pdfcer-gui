# -*- coding: utf-8 -*-
"""Build fixtures/dimension-scaled.pdf.

One 400 x 300 page carrying one linear ce dimension, 200 pt long, from
(100, 150) to (300, 150), in the default group, calibrated so that 200 pt
reads 1000 mm. The page itself is blank: the ce dimension is the only thing
drawn.

The base page is written here; the ce dimension and the scale are authored by
the engine's own command line (`pdfcer dimension-add`, `pdfcer group-set-scale`),
so the sidecar and the baked appearance are the engine's, not a copy of them.

Run from the repository root:  python fixtures/dimension-scaled.PROVENANCE.py
"""
import os
import subprocess
import tempfile

PDFCER = os.environ.get('PDFCER_EXE', 'D:/Dev/pdfcer/target/release/pdfcer.exe')
OUT = 'fixtures/dimension-scaled.pdf'


def blank_page() -> bytes:
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Contents 4 0 R "
        b"/Resources << >> >>",
        b"<< /Length 0 >>\nstream\n\nendstream",
    ]
    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for i, body in enumerate(objs, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)
    for off in offsets:
        out += b"%010d 00000 n \n" % off
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (
        len(objs) + 1,
        xref,
    )
    return bytes(out)


with tempfile.TemporaryDirectory() as tmp:
    base = os.path.join(tmp, 'base.pdf')
    dimmed = os.path.join(tmp, 'dimmed.pdf')
    with open(base, 'wb') as f:
        f.write(blank_page())
    subprocess.run([PDFCER, 'dimension-add', '--points', '100,150 300,150',
                    '--offset', '30', '-o', dimmed, base], check=True)
    subprocess.run([PDFCER, 'group-set-scale', '--real-length', '1000mm',
                    '--drawn', '200', '--mode', 'full', '-o', OUT, dimmed], check=True)
print('wrote', OUT)
