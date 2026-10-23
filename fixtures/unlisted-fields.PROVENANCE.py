# -*- coding: utf-8 -*-
"""Build fixtures/unlisted-fields.pdf: one page, three text fields drawn as
merged field-widgets (each carries its own /T, /FT, /V and /DA), only ONE of
which is listed in /AcroForm /Fields.

Producers that append widgets without updating /Fields write exactly this. An
insert carries the listed field and leaves the other two as orphans that keep
their own name, which `EditSession::adopt_widget` registers losslessly: the
subject of `adopt_widget_puts_a_form_control_back`.

Run from the repository root: python fixtures/unlisted-fields.PROVENANCE.py
"""
import io

dst = 'fixtures/unlisted-fields.pdf'


def widget(name, value, y):
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /T (%s) /V (%s) '
        '/DA (/Helv 12 Tf 0 g) /F 4 /Rect [72 %d 300 %d] /P 3 0 R >>'
        % (name, value, y, y + 24)
    )


objs = [
    '<< /Type /Catalog /Pages 2 0 R /AcroForm 4 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] '
    '/Resources << /Font << /Helv 8 0 R >> >> /Contents 9 0 R '
    '/Annots [5 0 R 6 0 R 7 0 R] >>',
    '<< /Fields [5 0 R] /DA (/Helv 12 Tf 0 g) /DR << /Font << /Helv 8 0 R >> >> >>',
    widget('Listed', 'in Fields', 600),
    widget('Unlisted_A', 'not in Fields', 550),
    widget('Unlisted_B', 'not in Fields', 500),
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>',
]
content = 'BT /Helv 14 Tf 72 680 Td (Three fields, one listed) Tj ET'
objs.append('<< /Length %d >>\nstream\n%s\nendstream' % (len(content), content))

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for i, body in enumerate(objs, 1):
    offsets.append(out.tell())
    out.write(('%d 0 obj\n%s\nendobj\n' % (i, body)).encode('latin-1'))
xref = out.tell()
out.write(('xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1)).encode())
for off in offsets:
    out.write(('%010d 00000 n \n' % off).encode())
out.write(('trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n'
           % (len(objs) + 1, xref)).encode())
io.open(dst, 'wb').write(out.getvalue())
print('wrote', dst, len(out.getvalue()), 'bytes')
