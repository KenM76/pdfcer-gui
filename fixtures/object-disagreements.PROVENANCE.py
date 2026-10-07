# -*- coding: utf-8 -*-
"""Build fixtures/object-disagreements.pdf.

One page whose object list and picture disagree in each of the three ways
`pdfcer_core::vector::DecomposeDiagnostics` counts, plus one ordinary path as
the control:

- a black square (ordinary; listed and visible);
- a square filled under an /ExtGState with /ca 0 (listed, invisible:
  `paths_invisible_by_alpha`);
- a square filled in a /Separation spot colour (listed, colour undecoded:
  `paths_with_undecoded_colour`);
- an axial gradient painted with `sh` (visible, not listed:
  `shadings_unmodelled`).

Offsets are computed here, so the file can be edited and regenerated.
"""
import io

NL = chr(10).encode('ascii')

CONTENT = NL.join([
    b'0 g 20 20 60 60 re f',
    b'q /Clear gs 1 0 0 rg 100 20 60 60 re f Q',
    b'q /Spot cs 1 scn 180 20 60 60 re f Q',
    b'q 260 20 60 60 re W n /Ramp sh Q',
])

objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 340 100] '
       b'/Resources << /ExtGState << /Clear << /Type /ExtGState /ca 0 /CA 0 >> >> '
       b'/ColorSpace << /Spot [/Separation /PANTONE#20185#20C /DeviceCMYK 5 0 R] >> '
       b'/Shading << /Ramp 6 0 R >> >> /Contents 4 0 R >>',
    4: ('<< /Length %d >>' % len(CONTENT)).encode('ascii') + NL + b'stream' + NL
       + CONTENT + NL + b'endstream',
    5: b'<< /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [0 1 0.8 0] /N 1 >>',
    6: b'<< /ShadingType 2 /ColorSpace /DeviceRGB /Coords [260 0 320 0] '
       b'/Function << /FunctionType 2 /Domain [0 1] /C0 [0 0 1] /C1 [1 1 0] /N 1 >> '
       b'/Extend [true true] >>',
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

dst = 'fixtures/object-disagreements.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
