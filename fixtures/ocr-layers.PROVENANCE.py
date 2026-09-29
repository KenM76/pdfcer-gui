# -*- coding: utf-8 -*-
"""Build fixtures/ocr-layers.pdf: two pages, each with visible text and one
OCR layer in the shape pdfcer writes, plus one decoy on page 2.

A pdfcer layer is a whole `/Contents` stream opening with
`/pdfc_OCR << /Producer (pdfcer) ... >> BDC` and closing with its `EMC`
(`pdfcer_core::ocr::marker`). The decoy carries the same tag with another
`/Producer`, so File > Recognise > Remove OCR text must leave it alone:
the expected count is 2 layers on 2 pages. Run from the repository root.
"""
import io

FONT = b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>'


def layer(producer, word):
    return (b'/pdfc_OCR << /Producer (' + producer + b') /Version 1 /Engine (ocrs) >> BDC\n'
            b'q BT 3 Tr /F1 12 Tf 72 700 Td (' + word + b') Tj ET Q\nEMC')


# Objects: 1 catalog, 2 pages, 3 font, 4-5 page dicts, 6.. streams.
streams = [
    b'BT /F1 12 Tf 72 720 Td (Visible page one) Tj ET',  # 6
    layer(b'pdfcer', b'recognised one'),                  # 7
    b'BT /F1 12 Tf 72 720 Td (Visible page two) Tj ET',  # 8
    layer(b'pdfcer', b'recognised two'),                  # 9
    layer(b'someone else', b'not ours'),                  # 10
]
objs = [
    b'<< /Type /Catalog /Pages 2 0 R >>',
    b'<< /Type /Pages /Kids [4 0 R 5 0 R] /Count 2 >>',
    FONT,
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R '
    b'>> >> /Contents [6 0 R 7 0 R] >>',
    b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R '
    b'>> >> /Contents [8 0 R 9 0 R 10 0 R] >>',
] + [None] * len(streams)

out = io.BytesIO()
out.write(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
offsets = []
for n, body in enumerate(objs, start=1):
    offsets.append(out.tell())
    out.write(b'%d 0 obj\n' % n)
    if body is None:
        data = streams[n - 6]
        out.write(b'<< /Length %d >>\nstream\n' % len(data) + data + b'\nendstream')
    else:
        out.write(body)
    out.write(b'\nendobj\n')
xref = out.tell()
out.write(b'xref\n0 %d\n0000000000 65535 f \n' % (len(objs) + 1))
for o in offsets:
    out.write(b'%010d 00000 n \n' % o)
out.write(b'trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n' % (len(objs) + 1, xref))
io.open('fixtures/ocr-layers.pdf', 'wb').write(out.getvalue())
print('wrote fixtures/ocr-layers.pdf', out.tell(), 'bytes')
