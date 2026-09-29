# -*- coding: utf-8 -*-
"""Build fixtures/inline-dr-font.pdf: one filled text field whose form default
resources hold the font INLINE (`/DR << /Font << /Helv << ... >> >> >>`).

That is the shape Acrobat draws blank (requests G068 and G070) and the one
Edit > Forms > Repair fonts moves into an object of its own. Everything else is
fixtures/text-field-with-appearance.pdf. Run from the repository root.
"""
import io

HELV = b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>'
objs = [
    b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] /DA (/Helv 0 Tf 0 g) '
    b'/DR << /Font << /Helv ' + HELV + b' >> >> >> >>',
    b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /Font << /Helv 7 0 R '
    b'>> >> /Contents 4 0 R /Annots [5 0 R] >>',
    None,  # content stream
    b'<< /Type /Annot /Subtype /Widget /FT /Tx /T (FullName) /TU (Full name) '
    b'/Rect [70 160 260 185] /F 4 /P 3 0 R /DA (/Helv 10 Tf 0 g) /V (Ada) /AP << /N 6 0 R >> >>',
    None,  # appearance stream
    HELV,
]
streams = {
    4: (b'<< /Length %d >>', b'BT /Helv 10 Tf 20 170 Td (Name:) Tj ET'),
    6: (b'<< /Type /XObject /Subtype /Form /BBox [0 0 190 25] /Resources << /Font << /Helv 7 0 R '
        b'>> >> /Length %d >>', b'/Tx BMC q BT /Helv 10 Tf 0 g 2 7 Td (Ada) Tj ET Q EMC'),
}
out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objs, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n)
    if body is None:
        head, data = streams[n]
        out.write(head % len(data) + b'\nstream\n' + data + b'\nendstream')
    else:
        out.write(body)
    out.write(b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1))
for o in offsets:
    out.write(b'%010d 00000 n \n' % o)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objs) + 1, xref))
io.open('fixtures/inline-dr-font.pdf', 'wb').write(out.getvalue())
print('wrote fixtures/inline-dr-font.pdf', out.tell(), 'bytes')
