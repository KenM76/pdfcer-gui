# -*- coding: utf-8 -*-
"""Build fixtures/rich-field.pdf.

One rich-text field (/Ff bit 26) whose stored rich value holds a bold run:

  Note   /Ff RichText, /DA /Helv 14   rect [72 600 372 640]
         /V (Old bold)  /RV (<body><p>Old <b>bold</b></p></body>)
         /DS (font: 14pt Helvetica)

The widget carries an empty /AP /N so the canvas offers it for filling.
`ui-verify` `a_rich_text_field_fills_on_the_page` types into it and expects a
rich fill that reports the bold it replaced. Written from scratch here;
rights-clean. Offsets are computed.
"""
import io

NL = chr(10).encode('ascii')


def widget(name, rect, extra, da, ap_ref):
    return (
        '<< /Type /Annot /Subtype /Widget /FT /Tx /T (%s) /Rect [%s] /F 4 /P 3 0 R '
        '/DA (%s) %s /AP << /N %d 0 R >> >>' % (name, rect, da, extra, ap_ref)
    ).encode('ascii')


def blank(w, h):
    body = b'/Tx BMC EMC'
    head = ('<< /Type /XObject /Subtype /Form /BBox [0 0 %d %d] /Length %d >>'
            % (w, h, len(body))).encode('ascii')
    return head + NL + b'stream' + NL + body + NL + b'endstream'


objects = {
    1: b'<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [4 0 R] '
       b'/DA (/Helv 0 Tf 0 g) /DR << /Font << /Helv 6 0 R >> >> >> >>',
    2: b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    3: b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Annots [4 0 R] >>',
    4: widget('Note', '72 600 372 640',
              '/Ff 33554432 /V (Old bold) /RV (<body><p>Old <b>bold</b></p></body>) '
              '/DS (font: 14pt Helvetica)', '/Helv 14 Tf 0 g', 5),
    5: blank(300, 40),
    6: b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica '
        b'/Encoding /WinAnsiEncoding >>',
}

out = bytearray()
out += b'%PDF-1.7' + NL
out += b'%' + bytes(bytearray([0xE2, 0xE3, 0xCF, 0xD3])) + NL
offsets = {}
for num in sorted(objects):
    offsets[num] = len(out)
    out += ('%d 0 obj' % num).encode('ascii') + NL
    out += objects[num] + NL
    out += b'endobj' + NL
size = max(objects) + 1
startxref = len(out)
out += ('xref' + chr(10) + '0 %d' % size).encode('ascii') + NL
out += b'0000000000 65535 f ' + NL
for num in range(1, size):
    out += ('%010d 00000 n ' % offsets[num]).encode('ascii') + NL
out += b'trailer' + NL
out += ('<< /Size %d /Root 1 0 R >>' % size).encode('ascii') + NL
out += b'startxref' + NL
out += ('%d' % startxref).encode('ascii') + NL
out += b'%%EOF' + NL

dst = 'fixtures/rich-field.pdf'
io.open(dst, 'wb').write(bytes(out))
print('wrote', dst, len(out), 'bytes')
